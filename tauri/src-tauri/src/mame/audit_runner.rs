use std::{
    ffi::OsString,
    io::{self, Read},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::{
    config::ContentPathsV1,
    errors::{AppError, AppResult},
};

use super::{
    parse_mame_audit_output, validate_executable_path, validate_project_controlled_path,
    validate_short_identifier, MameAuditParseResult, MameExecutableSource,
};

const MACHINE_AUDIT_TIMEOUT: Duration = Duration::from_secs(120);
const AUDIT_STREAM_LIMIT: usize = 256 * 1024;

pub(crate) fn audit_machine(
    source: &MameExecutableSource,
    machine: &str,
    content_paths: &ContentPathsV1,
) -> AppResult<MameAuditParseResult> {
    let executable_path = validate_executable_path(source.path())?;
    let argv = build_machine_audit_argv(machine, content_paths)?;

    let mut command = Command::new(&executable_path);
    command
        .args(&argv)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let captured = capture_audit_command(command, &executable_path)?;
    let mut result =
        parse_mame_audit_output(&captured.stdout, &captured.stderr, captured.status.code());
    result.raw_truncated |= captured.stdout_truncated || captured.stderr_truncated;
    Ok(result)
}

fn build_machine_audit_argv(
    machine: &str,
    content_paths: &ContentPathsV1,
) -> AppResult<Vec<OsString>> {
    validate_short_identifier("machine", machine)?;

    let mut argv = vec![OsString::from("-noreadconfig")];
    if let Some(media_path) = compose_media_search_path(content_paths)? {
        argv.push(OsString::from("-rompath"));
        argv.push(media_path);
    }
    argv.push(OsString::from("-verifyroms"));
    argv.push(OsString::from(machine));
    Ok(argv)
}

fn compose_media_search_path(content_paths: &ContentPathsV1) -> AppResult<Option<OsString>> {
    let paths = content_paths
        .rom_paths
        .iter()
        .chain(content_paths.software_paths.iter())
        .chain(content_paths.chd_paths.iter());

    let mut media_path = OsString::new();
    let mut count = 0_usize;
    for path in paths {
        validate_project_controlled_path(path.as_path()).map_err(|error| {
            AppError::new(
                "MAME_AUDIT_CONTENT_PATH_INVALID",
                "A configured MAME content path cannot be used for auditing.",
            )
            .with_details(serde_json::json!({
                "causeCode": error.code,
                "causeMessage": error.message,
                "causeDetails": error.details
            }))
        })?;
        if count != 0 {
            media_path.push(";");
        }
        media_path.push(path.as_path().as_os_str());
        count += 1;
    }

    if count == 0 {
        Ok(None)
    } else {
        Ok(Some(media_path))
    }
}

#[derive(Debug)]
struct CapturedAuditOutput {
    status: ExitStatus,
    stdout: String,
    stderr: String,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

fn capture_audit_command(
    mut command: Command,
    executable_path: &std::path::Path,
) -> AppResult<CapturedAuditOutput> {
    let mut child = command.spawn().map_err(|error| {
        AppError::new(
            "MAME_AUDIT_LAUNCH_FAILED",
            "MAME could not be launched for the machine audit.",
        )
        .with_details(serde_json::json!({
            "path": executable_path,
            "cause": error.to_string()
        }))
    })?;

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate_child(&mut child);
            return Err(AppError::new(
                "MAME_AUDIT_STDOUT_MISSING",
                "The MAME audit process did not provide stdout.",
            ));
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            terminate_child(&mut child);
            return Err(AppError::new(
                "MAME_AUDIT_STDERR_MISSING",
                "The MAME audit process did not provide stderr.",
            ));
        }
    };

    let stdout_reader = thread::spawn(move || drain_bounded(stdout, AUDIT_STREAM_LIMIT));
    let stderr_reader = thread::spawn(move || drain_bounded(stderr, AUDIT_STREAM_LIMIT));
    let started = Instant::now();

    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= MACHINE_AUDIT_TIMEOUT => {
                terminate_child(&mut child);
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(AppError::new(
                    "MAME_AUDIT_TIMEOUT",
                    "The MAME machine audit exceeded its time limit.",
                )
                .with_details(serde_json::json!({
                    "timeoutMs": MACHINE_AUDIT_TIMEOUT.as_millis()
                })));
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                terminate_child(&mut child);
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(AppError::new(
                    "MAME_AUDIT_WAIT_FAILED",
                    "The MAME audit process could not be observed to completion.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() })));
            }
        }
    };

    let stdout = join_reader(stdout_reader, "stdout")?;
    let stderr = join_reader(stderr_reader, "stderr")?;

    Ok(CapturedAuditOutput {
        status,
        stdout: String::from_utf8_lossy(&stdout.bytes).into_owned(),
        stderr: String::from_utf8_lossy(&stderr.bytes).into_owned(),
        stdout_truncated: stdout.truncated,
        stderr_truncated: stderr.truncated,
    })
}

#[derive(Debug)]
struct BoundedBytes {
    bytes: Vec<u8>,
    truncated: bool,
}

fn drain_bounded<R: Read>(mut reader: R, limit: usize) -> io::Result<BoundedBytes> {
    let mut retained = Vec::with_capacity(limit.min(8192));
    let mut buffer = [0_u8; 4096];
    let mut truncated = false;

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(retained.len());
        let keep = remaining.min(count);
        retained.extend_from_slice(&buffer[..keep]);
        truncated |= keep < count;
    }

    Ok(BoundedBytes {
        bytes: retained,
        truncated,
    })
}

fn join_reader(
    handle: thread::JoinHandle<io::Result<BoundedBytes>>,
    stream: &str,
) -> AppResult<BoundedBytes> {
    handle
        .join()
        .map_err(|_| {
            AppError::new(
                "MAME_AUDIT_OUTPUT_FAILED",
                format!("The MAME audit {stream} reader thread panicked."),
            )
        })?
        .map_err(|error| {
            AppError::new(
                "MAME_AUDIT_OUTPUT_FAILED",
                format!("The MAME audit {stream} stream could not be read."),
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::PathBuf};

    use crate::config::{ContentPathsV1, PlatformPath};

    use super::{audit_machine, build_machine_audit_argv, compose_media_search_path};
    use crate::mame::{MameAuditClassification, MameExecutableSource};

    fn absolute_path(name: &str) -> PathBuf {
        #[cfg(windows)]
        {
            PathBuf::from(format!(r"C:\{name}"))
        }
        #[cfg(not(windows))]
        {
            PathBuf::from(format!("/{name}"))
        }
    }

    #[test]
    fn empty_configuration_keeps_mame_builtin_media_default() {
        let argv =
            build_machine_audit_argv("pacman", &ContentPathsV1::default()).expect("audit argv");
        assert_eq!(
            argv,
            vec![
                OsString::from("-noreadconfig"),
                OsString::from("-verifyroms"),
                OsString::from("pacman"),
            ]
        );
    }

    #[test]
    fn media_search_path_preserves_group_and_user_order() {
        let rom_a = absolute_path("rom-a");
        let rom_b = absolute_path("rom-b");
        let software = absolute_path("software");
        let chd = absolute_path("chd");
        let paths = ContentPathsV1 {
            rom_paths: vec![PlatformPath::new(&rom_a), PlatformPath::new(&rom_b)],
            software_paths: vec![PlatformPath::new(&software)],
            chd_paths: vec![PlatformPath::new(&chd)],
        };

        let composed = compose_media_search_path(&paths)
            .expect("composed path")
            .expect("non-empty path");
        let expected = format!(
            "{};{};{};{}",
            rom_a.display(),
            rom_b.display(),
            software.display(),
            chd.display()
        );
        assert_eq!(composed, OsString::from(expected));

        let argv = build_machine_audit_argv("pacman", &paths).expect("audit argv");
        assert_eq!(argv[0], "-noreadconfig");
        assert_eq!(argv[1], "-rompath");
        assert_eq!(argv[2], composed);
        assert_eq!(argv[3], "-verifyroms");
        assert_eq!(argv[4], "pacman");
    }

    #[test]
    fn multipath_separator_in_configured_path_is_rejected() {
        let paths = ContentPathsV1 {
            rom_paths: vec![PlatformPath::new(absolute_path("rom;other"))],
            software_paths: Vec::new(),
            chd_paths: Vec::new(),
        };
        let error =
            compose_media_search_path(&paths).expect_err("ambiguous multipath must be rejected");
        assert_eq!(error.code, "MAME_AUDIT_CONTENT_PATH_INVALID");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_paths_are_composed_without_loss() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let first = vec![b'/', b'r', b'o', b'm', 0xff];
        let second = vec![b'/', b'c', b'h', b'd', 0xfe];
        let paths = ContentPathsV1 {
            rom_paths: vec![PlatformPath::new(PathBuf::from(OsString::from_vec(
                first.clone(),
            )))],
            software_paths: Vec::new(),
            chd_paths: vec![PlatformPath::new(PathBuf::from(OsString::from_vec(
                second.clone(),
            )))],
        };

        let composed = compose_media_search_path(&paths)
            .expect("composed path")
            .expect("non-empty path");
        let mut expected = first;
        expected.push(b';');
        expected.extend(second);
        assert_eq!(composed.into_vec(), expected);
    }

    #[cfg(unix)]
    #[test]
    fn nonzero_media_failure_is_parsed_instead_of_becoming_transport_error() {
        use std::{
            fs,
            os::unix::fs::PermissionsExt,
            time::{SystemTime, UNIX_EPOCH},
        };

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mame-audit-runner-{nonce}"));
        fs::create_dir_all(&root).expect("temp root");
        let executable = root.join("fake-mame");
        fs::write(
            &executable,
            "#!/bin/sh\nprintf 'pacman: required.bin (1 bytes) - NOT FOUND\\nromset pacman is bad\\n'\nexit 2\n",
        )
        .expect("fake mame");
        let mut permissions = fs::metadata(&executable).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).expect("executable permissions");

        let result = audit_machine(
            &MameExecutableSource::external(&executable),
            "pacman",
            &ContentPathsV1::default(),
        )
        .expect("exit 2 is an audit result");
        assert_eq!(
            result.classification,
            MameAuditClassification::MissingRequired
        );
        assert_eq!(result.exit_code, Some(2));
        assert!(result.facts.missing_required);

        fs::remove_dir_all(root).expect("cleanup");
    }
}
