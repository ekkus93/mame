// MT-706 protocol-first shutdown integration.
//
// The existing MT-207 `SessionSupervisor::stop` remains the single OS
// soft-stop/forced-kill escalation path. This extension only places the
// authenticated protocol-exit attempt ahead of it, then observes the normal
// supervisor snapshot until the protocol grace deadline expires.

use std::{
    thread,
    time::{Duration, Instant},
};

use crate::errors::{AppError, AppResult};

use super::{
    control,
    supervisor::{SessionState, SessionSupervisor, StopSessionResult},
};

const PROTOCOL_EXIT_GRACE_TIMEOUT: Duration = Duration::from_millis(1500);
const PROTOCOL_EXIT_POLL_INTERVAL: Duration = Duration::from_millis(25);

impl SessionSupervisor {
    pub(crate) fn stop_with_protocol_exit(&self, session_id: &str) -> AppResult<StopSessionResult> {
        let initial = self
            .current_session()?
            .filter(|session| session.session_id == session_id)
            .ok_or_else(|| session_not_found(session_id))?;
        if initial.state != SessionState::Running {
            return Err(AppError::new(
                "MAME_SESSION_NOT_RUNNING",
                "The requested MAME session is not running.",
            )
            .with_details(serde_json::json!({
                "sessionId": session_id,
                "state": initial.state
            })));
        }

        if control::request_session_exit(session_id).is_ok() {
            let started = Instant::now();
            loop {
                let snapshot = self
                    .current_session()?
                    .filter(|session| session.session_id == session_id)
                    .ok_or_else(|| session_not_found(session_id))?;
                match snapshot.state {
                    SessionState::Exited => {
                        return Ok(StopSessionResult {
                            schema_version: 1,
                            soft_stop_requested: false,
                            forced_termination: snapshot.forced_termination,
                            session: snapshot,
                        });
                    }
                    SessionState::Crashed | SessionState::Failed => {
                        return Err(AppError::new(
                            "MAME_PROTOCOL_EXIT_ABNORMAL",
                            "MAME terminated after the clean-exit request but did not report a clean process exit.",
                        )
                        .with_details(serde_json::json!({
                            "sessionId": session_id,
                            "state": snapshot.state,
                            "exitCode": snapshot.exit_code,
                            "terminationSignal": snapshot.termination_signal
                        })));
                    }
                    SessionState::Created
                    | SessionState::Starting
                    | SessionState::Running
                    | SessionState::Stopping => {}
                }

                if started.elapsed() >= PROTOCOL_EXIT_GRACE_TIMEOUT {
                    break;
                }
                thread::sleep(PROTOCOL_EXIT_POLL_INTERVAL);
            }
        }

        // Protocol exit was unavailable/rejected/failed, or MAME acknowledged
        // it but remained alive through the grace period. Delegate unchanged to
        // MT-207 so the established OS soft-stop timeout, forced kill, and
        // forced_termination accounting remain authoritative.
        fallback_stop_or_terminal(self, session_id)
    }
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

fn session_not_found(session_id: &str) -> AppError {
    AppError::new(
        "MAME_SESSION_NOT_FOUND",
        "The requested MAME session is not available.",
    )
    .with_details(serde_json::json!({ "sessionId": session_id }))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::mame::{MameExecutableSource, MameLaunchTarget};

    use super::super::{supervisor::EffectiveLaunchConfig, supervisor::EventSink};
    use super::{SessionState, SessionSupervisor};

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
