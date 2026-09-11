// MT-706 protocol-first shutdown integration.
//
// This file is textually included by the `sessions::supervisor` module. The
// existing MT-207 `stop` implementation remains the single OS soft-stop/forced
// kill escalation path; this extension adds a bounded protocol-exit attempt in
// front of it.

const PROTOCOL_EXIT_GRACE_TIMEOUT: Duration = Duration::from_millis(1500);

impl SessionSupervisor {
    pub(crate) fn stop_with_protocol_exit(&self, session_id: &str) -> AppResult<StopSessionResult> {
        let (child, diagnostics) = {
            let inner = recover_lock(&self.inner);
            let current = current_session(&inner, session_id)?;
            if current.snapshot.state != SessionState::Running {
                return Err(AppError::new(
                    "MAME_SESSION_NOT_RUNNING",
                    "The requested MAME session is not running.",
                )
                .with_details(serde_json::json!({
                    "sessionId": session_id,
                    "state": current.snapshot.state
                })));
            }
            let child = current.child.clone().ok_or_else(|| {
                AppError::new(
                    "MAME_SESSION_CHILD_MISSING",
                    "The running MAME session has no supervised child process.",
                )
                .with_details(serde_json::json!({ "sessionId": session_id }))
            })?;
            (child, current.diagnostics.clone())
        };

        match super::control::request_session_exit(session_id) {
            Ok(()) => {
                {
                    let mut inner = recover_lock(&self.inner);
                    if let Ok(current) = current_session_mut(&mut inner, session_id) {
                        if let Some(control) = current.control.as_mut() {
                            control.begin_shutdown_close();
                        }
                    }
                }

                let status = wait_for_child(&child, PROTOCOL_EXIT_GRACE_TIMEOUT).map_err(|error| {
                    AppError::new(
                        "MAME_PROTOCOL_EXIT_WAIT_FAILED",
                        "The MAME process could not be observed after accepting clean exit.",
                    )
                    .with_details(serde_json::json!({
                        "sessionId": session_id,
                        "cause": error.to_string()
                    }))
                })?;
                if let Some(status) = status {
                    return finish_protocol_exit(self, session_id, status);
                }

                record_diagnostic_error(
                    &diagnostics,
                    format!(
                        "Protocol clean exit was acknowledged, but MAME remained active for {} ms; escalating through the MT-207 OS shutdown path.",
                        PROTOCOL_EXIT_GRACE_TIMEOUT.as_millis()
                    ),
                );
                fallback_stop_or_terminal(self, session_id)
            }
            Err(error) => {
                let already_exited = wait_for_child(&child, Duration::ZERO).map_err(|wait_error| {
                    AppError::new(
                        "MAME_PROTOCOL_EXIT_WAIT_FAILED",
                        "The MAME process could not be observed after the clean-exit request failed.",
                    )
                    .with_details(serde_json::json!({
                        "sessionId": session_id,
                        "cause": wait_error.to_string()
                    }))
                })?;
                if let Some(status) = already_exited {
                    return finish_protocol_exit(self, session_id, status);
                }

                record_diagnostic_error(
                    &diagnostics,
                    format!(
                        "Protocol clean exit unavailable or failed ({}: {}); escalating through the MT-207 OS shutdown path.",
                        error.code, error.message
                    ),
                );
                fallback_stop_or_terminal(self, session_id)
            }
        }
    }
}

fn finish_protocol_exit(
    supervisor: &SessionSupervisor,
    session_id: &str,
    status: ExitStatus,
) -> AppResult<StopSessionResult> {
    settle_capture(&supervisor.inner, session_id, &status);
    finalize_session(&supervisor.inner, session_id, status);
    let session = supervisor
        .current_session()?
        .filter(|session| session.session_id == session_id)
        .ok_or_else(|| {
            AppError::new(
                "MAME_SESSION_NOT_FOUND",
                "The requested MAME session is no longer available.",
            )
            .with_details(serde_json::json!({ "sessionId": session_id }))
        })?;

    if session.state != SessionState::Exited {
        return Err(AppError::new(
            "MAME_PROTOCOL_EXIT_ABNORMAL",
            "MAME terminated after the clean-exit request but did not report a clean process exit.",
        )
        .with_details(serde_json::json!({
            "sessionId": session_id,
            "state": session.state,
            "exitCode": session.exit_code,
            "terminationSignal": session.termination_signal
        })));
    }

    Ok(StopSessionResult {
        schema_version: 1,
        soft_stop_requested: false,
        forced_termination: session.forced_termination,
        session,
    })
}

fn fallback_stop_or_terminal(
    supervisor: &SessionSupervisor,
    session_id: &str,
) -> AppResult<StopSessionResult> {
    match supervisor.stop(session_id) {
        Ok(result) => Ok(result),
        Err(error) if error.code == "MAME_SESSION_NOT_RUNNING" => {
            let session = supervisor
                .current_session()?
                .filter(|session| session.session_id == session_id);
            match session {
                Some(session) if session.state == SessionState::Exited => Ok(StopSessionResult {
                    schema_version: 1,
                    soft_stop_requested: false,
                    forced_termination: session.forced_termination,
                    session,
                }),
                _ => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod mt706_supervisor_tests {
    use std::{
        fs,
        path::PathBuf,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::mame::{MameExecutableSource, MameLaunchTarget};

    use super::{EffectiveLaunchConfig, EventSink, SessionState, SessionSupervisor};

    fn no_op_sink() -> EventSink {
        Arc::new(|_, _| Ok(()))
    }

    #[cfg(unix)]
    #[test]
    fn clean_stop_prefers_protocol_exit_before_os_soft_stop() {
        let root = unique_temp_dir("protocol-exit");
        let executable = write_protocol_exit_mame(&root, true, false);
        let supervisor = SessionSupervisor::default();
        let started = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect("fake MAME must launch");

        let stopped = supervisor
            .stop_with_protocol_exit(&started.session_id)
            .expect("protocol clean exit must complete");
        assert!(!stopped.soft_stop_requested);
        assert!(!stopped.forced_termination);
        assert_eq!(stopped.session.state, SessionState::Exited);
        assert_eq!(stopped.session.exit_code, Some(0));

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn accepted_protocol_exit_that_stalls_uses_existing_soft_then_forced_escalation() {
        let root = unique_temp_dir("protocol-exit-forced");
        let executable = write_protocol_exit_mame(&root, false, true);
        let supervisor = SessionSupervisor::default();
        let started = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect("fake MAME must launch");

        let stopped = supervisor
            .stop_with_protocol_exit(&started.session_id)
            .expect("fallback escalation must stop fake MAME");
        assert!(stopped.soft_stop_requested);
        assert!(stopped.forced_termination);
        assert_eq!(stopped.session.state, SessionState::Exited);
        assert!(stopped.session.forced_termination);
        assert!(stopped
            .session
            .diagnostic_error
            .as_deref()
            .is_some_and(|message| message.contains("Protocol clean exit was acknowledged")));

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    fn write_protocol_exit_mame(
        root: &PathBuf,
        honor_protocol_exit: bool,
        ignore_term: bool,
    ) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        fs::create_dir_all(root).expect("create fake MAME directory");
        let executable = root.join("fake protocol-exit mame");
        let honor_protocol_exit = if honor_protocol_exit { "1" } else { "0" };
        let ignore_term = if ignore_term { "1" } else { "0" };
        let script = format!(
            r#"#!/bin/sh
if [ "$1" = '-noreadconfig' ] && [ "$2" = '-version' ]; then
  printf '%s\n' '0.288 test-build'
  exit 0
fi
bootstrap=''
while [ "$#" -gt 0 ]; do
  if [ "$1" = '-autoboot_script' ]; then
    shift
    bootstrap=$1
  fi
  shift
done
token=$(sed -n 's/^local expected_token = "\(.*\)"$/\1/p' "$bootstrap")
session_id=$(sed -n 's/^local session_id = "\(.*\)"$/\1/p' "$bootstrap")
ready_json=$(printf '{{"version":1,"type":"event","sessionId":"%s","event":"ready","payload":{{"commands":["pause","resume","reset","exit"],"maxMessageBytes":16384}}}}' "$session_id")
ready_encoded=$(printf '%s' "$ready_json" | base64 | tr '+/' '-_' | tr -d '=\n')
printf '\n@@MAME_TAURI_CONTROL_V1@@%s@@%s\n' "$token" "$ready_encoded"
IFS= read -r control_line || exit 2
case "$control_line" in
  mame_tauri_control_v1*) ;;
  *) exit 3 ;;
esac
accepted_json=$(printf '{{"version":1,"type":"response","sessionId":"%s","requestId":"req-1","status":"accepted","ok":true,"result":{{}}}}' "$session_id")
accepted_encoded=$(printf '%s' "$accepted_json" | base64 | tr '+/' '-_' | tr -d '=\n')
printf '\n@@MAME_TAURI_CONTROL_V1@@%s@@%s\n' "$token" "$accepted_encoded"
if [ "{honor_protocol_exit}" = '1' ]; then
  exit 0
fi
if [ "{ignore_term}" = '1' ]; then
  trap '' TERM
else
  trap 'exit 0' TERM
fi
while :; do :; done
"#
        );
        fs::write(&executable, script).expect("write fake MAME executable");
        let mut permissions = fs::metadata(&executable)
            .expect("read fake MAME metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).expect("mark fake MAME executable");
        executable
    }

    #[cfg(unix)]
    fn unique_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "mame-tauri-mt706-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn target(machine: &str) -> MameLaunchTarget {
        MameLaunchTarget {
            machine: machine.to_owned(),
            software: None,
            project_paths: Vec::new(),
        }
    }
}
