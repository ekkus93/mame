#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::{Arc, Mutex},
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use crate::mame::{MameExecutableSource, MameLaunchTarget};

    use super::{
        recover_lock, ControlChannelState, EffectiveLaunchConfig, EventSink,
        SessionLifecycleEventV1, SessionState, SessionSupervisor, DIAGNOSTIC_TAIL_LIMIT,
    };

    fn no_op_sink() -> EventSink {
        Arc::new(|_, _| Ok(()))
    }

    fn recording_sink(events: Arc<Mutex<Vec<String>>>) -> EventSink {
        Arc::new(move |name, _event: SessionLifecycleEventV1| {
            recover_lock(&events).push(name.to_owned());
            Ok(())
        })
    }

    #[test]
    fn state_machine_rejects_terminal_transitions() {
        assert!(!SessionState::Exited.allows(SessionState::Running));
        assert!(!SessionState::Crashed.allows(SessionState::Running));
        assert!(SessionState::Running.allows(SessionState::Stopping));
        assert!(SessionState::Running.allows(SessionState::Crashed));
    }

    #[cfg(unix)]
    #[test]
    fn starts_captures_and_records_clean_exit() {
        let root = unique_temp_dir("clean-exit 日本語");
        let executable = write_fake_mame(
            &root,
            "printf '%s\\n' 'stdout-from-mame'\nprintf '%s\\n' 'stderr-from-mame' >&2\nexit 0\n",
        );
        let events = Arc::new(Mutex::new(Vec::new()));
        let supervisor = SessionSupervisor::default();

        let started = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                recording_sink(events.clone()),
            )
            .expect("fake MAME must launch");
        assert_eq!(started.state, SessionState::Running);
        assert!(started.pid.is_some());

        let exited = wait_for_terminal(&supervisor);
        assert_eq!(exited.state, SessionState::Exited);
        assert_eq!(exited.exit_code, Some(0));
        assert!(exited.stdout_tail.contains("stdout-from-mame"));
        assert!(exited.stderr_tail.contains("stderr-from-mame"));
        let events = recover_lock(&events);
        assert!(events.iter().any(|name| name == "session.started"));
        assert!(events.iter().any(|name| name == "session.exited"));

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn nonzero_exit_is_reported_as_crash() {
        let root = unique_temp_dir("crash-exit");
        let executable =
            write_fake_mame(&root, "printf '%s\\n' 'fatal fake failure' >&2\nexit 7\n");
        let supervisor = SessionSupervisor::default();

        supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect("fake MAME must launch");

        let crashed = wait_for_terminal(&supervisor);
        assert_eq!(crashed.state, SessionState::Crashed);
        assert_eq!(crashed.exit_code, Some(7));
        assert!(crashed.stderr_tail.contains("fatal fake failure"));

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn diagnostics_are_bounded_to_recent_tail() {
        let root = unique_temp_dir("bounded-output");
        let executable = write_fake_mame(
            &root,
            "i=0\nwhile [ \"$i\" -lt 9000 ]; do\n  printf '0123456789'\n  i=$((i + 1))\ndone\nexit 0\n",
        );
        let supervisor = SessionSupervisor::default();

        supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect("fake MAME must launch");

        let exited = wait_for_terminal(&supervisor);
        assert!(exited.stdout_truncated);
        assert!(exited.stdout_tail.len() <= DIAGNOSTIC_TAIL_LIMIT);

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn duplicate_active_launch_is_rejected() {
        let root = unique_temp_dir("duplicate-session");
        let executable = write_fake_mame(&root, "trap 'exit 0' TERM\nwhile :; do :; done\n");
        let supervisor = SessionSupervisor::default();

        let first = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect("first fake MAME must launch");

        let error = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("galaga"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect_err("second active launch must be rejected");
        assert_eq!(error.code, "MAME_SESSION_ALREADY_ACTIVE");

        supervisor
            .stop(&first.session_id)
            .expect("first fake MAME must stop");
        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn control_endpoint_is_session_scoped_and_torn_down_with_session() {
        let root = unique_temp_dir("control-endpoint");
        let executable = write_fake_mame(&root, "trap 'exit 0' TERM\nwhile :; do :; done\n");
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
            .expect("fake MAME control endpoint must become ready");

        assert!(started.effective_argv.iter().any(|arg| arg == "-console"));
        assert!(started
            .effective_argv
            .iter()
            .any(|arg| arg == "-autoboot_script"));
        assert!(!started.stdout_tail.contains("@@MAME_TAURI_CONTROL_V1@@"));
        {
            let inner = recover_lock(&supervisor.inner);
            let current = inner.current.as_ref().expect("managed session");
            let control = current.control.as_ref().expect("control channel");
            assert_eq!(control.state(), ControlChannelState::Ready);
        }

        supervisor
            .stop(&started.session_id)
            .expect("fake MAME must stop");
        {
            let inner = recover_lock(&supervisor.inner);
            let current = inner.current.as_ref().expect("managed session");
            let control = current.control.as_ref().expect("control channel");
            assert_eq!(control.state(), ControlChannelState::Closed);
        }

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn stop_escalates_when_soft_termination_is_ignored() {
        let root = unique_temp_dir("forced-stop");
        let executable = write_fake_mame(&root, "trap '' TERM\nwhile :; do :; done\n");
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
            .stop(&started.session_id)
            .expect("forced stop must complete");

        assert!(stopped.soft_stop_requested);
        assert!(stopped.forced_termination);
        assert_eq!(stopped.session.state, SessionState::Exited);
        assert!(stopped.session.forced_termination);

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    fn write_fake_mame(root: &PathBuf, launch_body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        fs::create_dir_all(root).expect("create fake MAME directory");
        let executable = root.join("fake mame executable");
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
if [ -n "$bootstrap" ]; then
  ready_frame=$(sed -n 's/^local ready_frame = "\(.*\)"$/\1/p' "$bootstrap")
  if [ -n "$ready_frame" ]; then
    printf '\n%s\n' "$ready_frame"
  fi
fi
{launch_body}"#
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
            "mame-tauri-session-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    #[cfg(unix)]
    fn wait_for_terminal(supervisor: &SessionSupervisor) -> super::SessionSnapshot {
        let started = std::time::Instant::now();
        loop {
            let snapshot = supervisor
                .current_session()
                .expect("session snapshot must be available")
                .expect("session must exist");
            if !snapshot.state.is_active() {
                return snapshot;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "fake MAME did not reach terminal state"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn target(machine: &str) -> MameLaunchTarget {
        MameLaunchTarget {
            machine: machine.to_owned(),
            software: None,
            project_paths: Vec::new(),
        }
    }
}
