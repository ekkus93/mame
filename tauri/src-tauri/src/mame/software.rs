use std::{
    io::{self, Read},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::errors::{AppError, AppResult};

use super::{validate_executable_path, validate_software_list_identifier, MameExecutableSource};

const SOFTWARE_QUERY_TIMEOUT: Duration = Duration::from_secs(15);
const SOFTWARE_STDOUT_LIMIT: usize = 32 * 1024 * 1024;
const SOFTWARE_STDERR_LIMIT: usize = 256 * 1024;

pub(crate) fn get_software_list_xml(
    source: &MameExecutableSource,
    software_list: &str,
) -> AppResult<String> {
    validate_software_list_identifier(software_list)?;
    let path = validate_executable_path(source.path())?;
    let mut command = Command::new(&path);
    command
        .args(["-noreadconfig", "-getsoftlist", software_list])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|error| {
        AppError::new(
            "MAME_SOFTWARE_QUERY_LAUNCH_FAILED",
            "MAME could not be launched to read software-list metadata.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "softwareList": software_list,
            "cause": error.to_string()
        }))
    })?;

    let stdout = child.stdout.take().ok_or_else(|| {
        AppError::new(
            "MAME_SOFTWARE_QUERY_OUTPUT_FAILED",
            "MAME software-list stdout was not available.",
        )
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        AppError::new(
            "MAME_SOFTWARE_QUERY_OUTPUT_FAILED",
            "MAME software-list stderr was not available.",
        )
    })?;
    let stdout_reader = thread::spawn(move || drain_bounded(stdout, SOFTWARE_STDOUT_LIMIT));
    let stderr_reader = thread::spawn(move || drain_bounded(stderr, SOFTWARE_STDERR_LIMIT));

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= SOFTWARE_QUERY_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(AppError::new(
                    "MAME_SOFTWARE_QUERY_TIMEOUT",
                    "MAME did not finish the software-list query within the bounded timeout.",
                )
                .with_details(serde_json::json!({
                    "softwareList": software_list,
                    "timeoutMs": SOFTWARE_QUERY_TIMEOUT.as_millis()
                })));
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                let _ = child.kill();
                return Err(AppError::new(
                    "MAME_SOFTWARE_QUERY_WAIT_FAILED",
                    "The MAME software-list query process could not be observed.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() })));
            }
        }
    };

    let stdout = join_reader(stdout_reader, "stdout")?;
    let stderr = join_reader(stderr_reader, "stderr")?;
    let stdout_text = String::from_utf8_lossy(&stdout.bytes).into_owned();
    let stderr_text = String::from_utf8_lossy(&stderr.bytes).into_owned();

    if !status.success() {
        return Err(AppError::new(
            "MAME_SOFTWARE_QUERY_FAILED",
            "MAME rejected the requested software-list query.",
        )
        .with_details(serde_json::json!({
            "softwareList": software_list,
            "exitCode": status.code(),
            "stdout": stdout_text,
            "stderr": stderr_text,
            "stdoutTruncated": stdout.truncated,
            "stderrTruncated": stderr.truncated
        })));
    }
    if stdout.truncated {
        return Err(AppError::new(
            "MAME_SOFTWARE_QUERY_TOO_LARGE",
            "MAME software-list metadata exceeded the bounded capture limit.",
        )
        .with_details(serde_json::json!({
            "softwareList": software_list,
            "limitBytes": SOFTWARE_STDOUT_LIMIT
        })));
    }
    if stdout_text.trim().is_empty() {
        return Err(AppError::new(
            "MAME_SOFTWARE_QUERY_EMPTY",
            "MAME returned no software-list metadata for the requested list.",
        )
        .with_details(serde_json::json!({ "softwareList": software_list })));
    }

    Ok(stdout_text)
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
                "MAME_SOFTWARE_QUERY_OUTPUT_FAILED",
                format!("The MAME software-list {stream} reader thread panicked."),
            )
        })?
        .map_err(|error| {
            AppError::new(
                "MAME_SOFTWARE_QUERY_OUTPUT_FAILED",
                "MAME software-list output could not be captured.",
            )
            .with_details(serde_json::json!({
                "stream": stream,
                "cause": error.to_string()
            }))
        })
}

#[cfg(all(test, unix))]
mod tests {
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::get_software_list_xml;
    use crate::mame::MameExecutableSource;

    #[test]
    fn invokes_getsoftlist_without_shell_interpolation() {
        let root = unique_temp_dir();
        fs::create_dir_all(&root).expect("create temp directory");
        let executable = root.join("fake mame");
        fs::write(
            &executable,
            "#!/bin/sh\n[ \"$1\" = '-noreadconfig' ] || exit 41\n[ \"$2\" = '-getsoftlist' ] || exit 42\n[ \"$3\" = 'apple2_flop_clcracked' ] || exit 43\nprintf '%s\\n' '<softwarelists><softwarelist name=\"apple2_flop_clcracked\"><software name=\"x\"><description>X</description><year>1980</year><publisher>P</publisher></software></softwarelist></softwarelists>'\n",
        )
        .expect("write fake executable");
        let mut permissions = fs::metadata(&executable)
            .expect("fake executable metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).expect("mark executable");

        let xml = get_software_list_xml(
            &MameExecutableSource::development_tree(&executable),
            "apple2_flop_clcracked",
        )
        .expect("query must succeed");
        assert!(xml.contains("apple2_flop_clcracked"));
        fs::remove_dir_all(root).expect("remove temp directory");
    }

    fn unique_temp_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "mame-tauri-software-query-{}-{nonce}",
            std::process::id()
        ))
    }
}
