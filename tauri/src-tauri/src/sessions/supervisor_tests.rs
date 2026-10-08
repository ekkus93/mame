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
        classify_early_exit_output, recover_lock, ControlChannelState, EffectiveLaunchConfig, EventSink,
        SessionLifecycleEventV1, SessionState, SessionSupervisor, DIAGNOSTIC_TAIL_LIMIT,
    };

    fn no_op_sink() -> EventSink {
        Arc::new(|_, _| Ok(()))
    }

    #[test]
    fn startup_output_classifies_permission_path_renderer_runtime_and_crash_failures() {
        let cases = [
            ("Permission denied while opening /games/roms", "MAME_CONTENT_PERMISSION_DENIED"),
            ("configured path is not a directory", "MAME_CONTENT_PATH_INVALID"),
            ("Unable to initialize SDL video subsystem", "MAME_RENDERER_STARTUP_FAILED"),
            ("Unknown option: -notreal", "MAME_RUNTIME_CONFIGURATION_FAILED"),
            ("Fatal error: assertion failed in device startup", "MAME_CHILD_CRASHED"),
        ];

        for (stderr, expected_code) in cases {
            let (code, _message, content_failure) = classify_early_exit_output("", stderr);
            assert_eq!(code, expected_code, "stderr={stderr}");
            assert!(!content_failure, "stderr={stderr}");
        }
    }

    #[test]
    fn best_available_romset_summary_is_not_misreported_as_missing_content() {
        let (code, _message, content_failure) = classify_early_exit_output(
            "romset breakout is best available\n1 romsets found, 1 were OK.",
            "",
        );

        assert_eq!(code, "MAME_EARLY_EXIT");
        assert!(!content_failure);
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
    fn pre_ready_missing_content_exit_is_actionable_and_preserves_bounded_tails() {
        let root = unique_temp_dir("pre-ready-missing-content");
        let executable = write_pre_ready_fake_mame(
            &root,
            "printf '%s\\n' 'Required files are missing, the machine cannot be run.' >&2\nexit 2\n",
        );
        let supervisor = SessionSupervisor::default();

        let error = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect_err("pre-ready missing content must fail launch");

        assert_eq!(error.code, "MAME_CONTENT_LAUNCH_FAILED");
        assert!(error.message.contains("required ROM/content"));
        assert!(!error.message.contains("control channel"));
        assert_eq!(error.details["earlyExit"], true);
        assert_eq!(error.details["contentFailure"], true);
        assert_eq!(error.details["runtimeControlCode"], "CONTROL_CHANNEL_CLOSED");
        assert!(error.details["stderrTail"]
            .as_str()
            .unwrap_or_default()
            .contains("Required files are missing"));

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn unrelated_pre_ready_exit_is_not_misreported_as_protocol_failure() {
        let root = unique_temp_dir("pre-ready-generic-exit");
        let executable =
            write_pre_ready_fake_mame(&root, "printf '%s\\n' 'startup failed' >&2\nexit 9\n");
        let supervisor = SessionSupervisor::default();

        let error = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect_err("pre-ready process exit must fail launch");

        assert_eq!(error.code, "MAME_EARLY_EXIT");
        assert_eq!(error.details["exitCode"], 9);
        assert_eq!(error.details["runtimeControlCode"], "CONTROL_CHANNEL_CLOSED");
        assert!(error.details["stderrTail"]
            .as_str()
            .unwrap_or_default()
            .contains("startup failed"));

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
    #[test]
    fn dropping_supervisor_reaps_active_child_and_closes_gameplay_resources() {
        let root = unique_temp_dir("drop-reaps-child");
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
        assert_eq!(started.state, SessionState::Running);

        let child = {
            let inner = recover_lock(&supervisor.inner);
            let current = inner.current.as_ref().expect("managed session");
            current.child.clone().expect("supervised child")
        };

        drop(supervisor);

        let status = super::wait_for_child(&child, Duration::from_secs(2))
            .expect("observe child after supervisor drop");
        assert!(status.is_some(), "dropping the supervisor must reap MAME");

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn deterministic_fake_frame_producer_reaches_session_mailbox() {
        let root = unique_temp_dir("gameplay-frame-producer");
        let executable = write_fake_mame(
            &root,
            r#"session_id=$(sed -n 's/^local session_id = "\(.*\)"$/\1/p' "$bootstrap")
python3 - "$MAME_TAURI_FRAME_PIPE" "$MAME_TAURI_FRAME_TOKEN" "$session_id" <<'PY'
import struct
import sys

pipe, token, session_id = sys.argv[1:]
width, height = 2, 1
pixels = bytes([3, 2, 1, 0, 30, 20, 10, 0])
header_len = 56 + len(session_id.encode()) + len(token.encode())
header = b"MTFRAME1" + struct.pack(
    "<HHQIIIIQHHHHHH",
    1,
    header_len,
    1,
    width,
    height,
    width * 4,
    len(pixels),
    1234,
    0,
    0,
    1,
    len(session_id.encode()),
    len(token.encode()),
    0,
)
with open(pipe, "wb", buffering=0) as stream:
    stream.write(header)
    stream.write(session_id.encode())
    stream.write(token.encode())
    stream.write(pixels)
PY
trap 'exit 0' TERM
while :; do sleep 1; done
"#,
        );
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

        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let frame = loop {
            match supervisor.take_latest_frame(&started.session_id) {
                Ok(frame) => break frame,
                Err(error) if error.code == "MAME_FRAME_NOT_READY" => {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "fake gameplay frame did not arrive before deadline: {error:?}"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("unexpected gameplay frame error: {error:?}"),
            }
        };

        let bytes = frame.to_client_bytes().expect("serialize gameplay frame");
        assert_eq!(&bytes[..8], b"MTGFRM01");
        assert_eq!(u64::from_le_bytes(bytes[12..20].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(bytes[20..24].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(bytes[24..28].try_into().unwrap()), 1);
        let metrics = supervisor
            .frame_metrics(&started.session_id)
            .expect("frame metrics");
        assert_eq!(metrics.received, 1);
        assert_eq!(metrics.delivered, 1);
        assert_eq!(metrics.dropped, 0);

        supervisor
            .stop(&started.session_id)
            .expect("fake MAME must stop cleanly");
        let error = supervisor
            .take_latest_frame(&started.session_id)
            .expect_err("terminal session must not serve stale gameplay video");
        assert_eq!(error.code, "MAME_FRAME_SESSION_NOT_RUNNING");

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    fn write_pre_ready_fake_mame(root: &PathBuf, launch_body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        fs::create_dir_all(root).expect("create fake MAME directory");
        let executable = root.join("fake pre-ready mame");
        let script = format!(
            r#"#!/bin/sh
if [ "$1" = '-noreadconfig' ] && [ "$2" = '-version' ]; then
  printf '%s\n' '0.288 test-build'
  exit 0
fi
{launch_body}"#
        );
        fs::write(&executable, script).expect("write pre-ready fake MAME executable");
        let mut permissions = fs::metadata(&executable)
            .expect("read fake MAME metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).expect("mark fake MAME executable");
        executable
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
            bios: None,
            project_paths: Vec::new(),
        }
    }
}
