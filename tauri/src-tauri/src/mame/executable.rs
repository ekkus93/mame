//! MAME executable source selection, validation, and identity probing.

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;

use crate::errors::{AppError, AppResult};

const VERSION_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const VERSION_OUTPUT_LIMIT: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MameExecutableSourceKind {
    Bundled,
    External,
    DevelopmentTree,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MameExecutableTrust {
    QualifiedBundled,
    UserConfigured,
    Development,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MameExecutableSource {
    kind: MameExecutableSourceKind,
    path: PathBuf,
}

impl MameExecutableSource {
    pub fn bundled(path: impl Into<PathBuf>) -> Self {
        Self {
            kind: MameExecutableSourceKind::Bundled,
            path: path.into(),
        }
    }

    pub fn external(path: impl Into<PathBuf>) -> Self {
        Self {
            kind: MameExecutableSourceKind::External,
            path: path.into(),
        }
    }

    pub fn development_tree(path: impl Into<PathBuf>) -> Self {
        Self {
            kind: MameExecutableSourceKind::DevelopmentTree,
            path: path.into(),
        }
    }

    pub fn kind(&self) -> MameExecutableSourceKind {
        self.kind
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn trust(&self) -> MameExecutableTrust {
        match self.kind {
            MameExecutableSourceKind::Bundled => MameExecutableTrust::QualifiedBundled,
            MameExecutableSourceKind::External => MameExecutableTrust::UserConfigured,
            MameExecutableSourceKind::DevelopmentTree => MameExecutableTrust::Development,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MameExecutableIdentity {
    pub source: MameExecutableSourceKind,
    pub trust: MameExecutableTrust,
    pub path: String,
    pub version: String,
    pub build: Option<String>,
    pub raw_version_line: String,
}

pub fn configured_external_source(path: Option<&str>) -> AppResult<Option<MameExecutableSource>> {
    let Some(path) = path else {
        return Ok(None);
    };

    if path.trim().is_empty() {
        return Err(AppError::new(
            "MAME_EXECUTABLE_PATH_EMPTY",
            "The configured MAME executable path is empty.",
        ));
    }

    Ok(Some(MameExecutableSource::external(PathBuf::from(path))))
}

pub fn inspect_executable(source: MameExecutableSource) -> AppResult<MameExecutableIdentity> {
    let canonical_path = validate_executable_path(source.path())?;
    let probe = run_version_probe(&canonical_path)?;
    let raw_version_line = select_version_line(&probe.stdout, &probe.stderr).ok_or_else(|| {
        AppError::new(
            "MAME_VERSION_UNRECOGNIZED",
            "The executable ran, but its version output did not contain a MAME version line.",
        )
        .with_details(serde_json::json!({
            "path": canonical_path,
            "stdout": probe.stdout,
            "stderr": probe.stderr,
            "stdoutTruncated": probe.stdout_truncated,
            "stderrTruncated": probe.stderr_truncated
        }))
    })?;
    let (version, build) = parse_version_line(&raw_version_line);

    Ok(MameExecutableIdentity {
        source: source.kind(),
        trust: source.trust(),
        path: canonical_path.to_string_lossy().into_owned(),
        version,
        build,
        raw_version_line,
    })
}

pub fn validate_executable_path(path: &Path) -> AppResult<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err(AppError::new(
            "MAME_EXECUTABLE_PATH_EMPTY",
            "The MAME executable path is empty.",
        ));
    }

    let canonical_path = fs::canonicalize(path).map_err(|error| {
        let code = if error.kind() == io::ErrorKind::NotFound {
            "MAME_EXECUTABLE_NOT_FOUND"
        } else {
            "MAME_EXECUTABLE_PATH_INVALID"
        };
        AppError::new(code, "The configured MAME executable path is not usable.").with_details(
            serde_json::json!({
                "path": path,
                "cause": error.to_string()
            }),
        )
    })?;

    let metadata = fs::metadata(&canonical_path).map_err(|error| {
        AppError::new(
            "MAME_EXECUTABLE_METADATA_FAILED",
            "The configured MAME executable could not be inspected.",
        )
        .with_details(serde_json::json!({
            "path": canonical_path,
            "cause": error.to_string()
        }))
    })?;

    if !metadata.is_file() {
        return Err(AppError::new(
            "MAME_EXECUTABLE_NOT_FILE",
            "The configured MAME executable path does not refer to a regular file.",
        )
        .with_details(serde_json::json!({ "path": canonical_path })));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(AppError::new(
                "MAME_EXECUTABLE_NOT_EXECUTABLE",
                "The configured MAME executable file is not marked executable.",
            )
            .with_details(serde_json::json!({ "path": canonical_path })));
        }
    }

    Ok(canonical_path)
}

fn run_version_probe(path: &Path) -> AppResult<ProbeOutput> {
    let mut command = Command::new(path);
    command
        .args(["-noreadconfig", "-version"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    capture_command(command, VERSION_PROBE_TIMEOUT, VERSION_OUTPUT_LIMIT).map_err(|error| {
        match error {
            CaptureError::Spawn(error) => AppError::new(
                "MAME_EXECUTABLE_LAUNCH_FAILED",
                "The configured MAME executable could not be launched.",
            )
            .with_details(serde_json::json!({
                "path": path,
                "cause": error.to_string()
            })),
            CaptureError::Wait(error) => AppError::new(
                "MAME_VERSION_PROBE_WAIT_FAILED",
                "The MAME version probe could not observe the child process.",
            )
            .with_details(serde_json::json!({
                "path": path,
                "cause": error.to_string()
            })),
            CaptureError::Timeout => AppError::new(
                "MAME_VERSION_PROBE_TIMEOUT",
                "The configured executable did not complete the MAME version probe in time.",
            )
            .with_details(serde_json::json!({
                "path": path,
                "timeoutMs": VERSION_PROBE_TIMEOUT.as_millis()
            })),
            CaptureError::Reader(error) => AppError::new(
                "MAME_VERSION_PROBE_OUTPUT_FAILED",
                "The MAME version probe output could not be captured.",
            )
            .with_details(serde_json::json!({
                "path": path,
                "cause": error
            })),
            CaptureError::Exit(output) => AppError::new(
                "MAME_VERSION_PROBE_FAILED",
                "The configured executable rejected the MAME version probe.",
            )
            .with_details(serde_json::json!({
                "path": path,
                "exitCode": output.status.code(),
                "stdout": output.stdout,
                "stderr": output.stderr,
                "stdoutTruncated": output.stdout_truncated,
                "stderrTruncated": output.stderr_truncated
            })),
        }
    })
}

#[derive(Debug)]
struct ProbeOutput {
    status: ExitStatus,
    stdout: String,
    stderr: String,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

#[derive(Debug)]
enum CaptureError {
    Spawn(io::Error),
    Wait(io::Error),
    Timeout,
    Reader(String),
    Exit(ProbeOutput),
}

fn capture_command(
    mut command: Command,
    timeout: Duration,
    output_limit: usize,
) -> Result<ProbeOutput, CaptureError> {
    let mut child = command.spawn().map_err(CaptureError::Spawn)?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CaptureError::Reader("stdout pipe was not available".to_owned()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CaptureError::Reader("stderr pipe was not available".to_owned()))?;

    let stdout_reader = thread::spawn(move || drain_bounded(stdout, output_limit));
    let stderr_reader = thread::spawn(move || drain_bounded(stderr, output_limit));

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= timeout => {
                if child.kill().is_ok() {
                    let _ = child.wait();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                }
                return Err(CaptureError::Timeout);
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                let _ = child.kill();
                return Err(CaptureError::Wait(error));
            }
        }
    };

    let stdout = join_reader(stdout_reader, "stdout")?;
    let stderr = join_reader(stderr_reader, "stderr")?;
    let output = ProbeOutput {
        status,
        stdout: String::from_utf8_lossy(&stdout.bytes).into_owned(),
        stderr: String::from_utf8_lossy(&stderr.bytes).into_owned(),
        stdout_truncated: stdout.truncated,
        stderr_truncated: stderr.truncated,
    };

    if !output.status.success() {
        return Err(CaptureError::Exit(output));
    }

    Ok(output)
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
) -> Result<BoundedBytes, CaptureError> {
    handle
        .join()
        .map_err(|_| CaptureError::Reader(format!("{stream} reader thread panicked")))?
        .map_err(|error| CaptureError::Reader(format!("{stream}: {error}")))
}

fn select_version_line(stdout: &str, stderr: &str) -> Option<String> {
    stdout
        .lines()
        .chain(stderr.lines())
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

fn parse_version_line(line: &str) -> (String, Option<String>) {
    let trimmed = line.trim();

    if let Some((version, build)) = trimmed.split_once(' ') {
        let build = build.trim();
        (
            version.to_owned(),
            (!build.is_empty()).then(|| build.to_owned()),
        )
    } else {
        (trimmed.to_owned(), None)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        configured_external_source, inspect_executable, parse_version_line, MameExecutableSource,
        MameExecutableSourceKind, MameExecutableTrust,
    };

    #[test]
    fn external_configuration_is_explicit() {
        assert_eq!(
            configured_external_source(None).expect("unset config is valid"),
            None
        );

        let source = configured_external_source(Some("/opt/mame/mame"))
            .expect("configured path is valid")
            .expect("source must be present");
        assert_eq!(source.kind(), MameExecutableSourceKind::External);
        assert_eq!(source.trust(), MameExecutableTrust::UserConfigured);
    }

    #[test]
    fn empty_external_configuration_is_rejected() {
        let error =
            configured_external_source(Some("   ")).expect_err("empty configured path must fail");
        assert_eq!(error.code, "MAME_EXECUTABLE_PATH_EMPTY");
    }

    #[test]
    fn parses_version_and_build_identity_without_guessing() {
        assert_eq!(
            parse_version_line("0.288 (mame0287-823-gf4f6f34f2a8)"),
            (
                "0.288".to_owned(),
                Some("(mame0287-823-gf4f6f34f2a8)".to_owned())
            )
        );
        assert_eq!(parse_version_line("0.289"), ("0.289".to_owned(), None));
    }

    #[cfg(unix)]
    #[test]
    fn inspect_executable_records_source_trust_and_version() {
        use std::os::unix::fs::PermissionsExt;

        let root = unique_temp_dir("mame executable 日本語");
        fs::create_dir_all(&root).expect("create test directory");
        let executable = root.join("fake mame");
        fs::write(
            &executable,
            "#!/bin/sh\n[ \"$1\" = '-noreadconfig' ] || exit 41\n[ \"$2\" = '-version' ] || exit 42\nprintf '%s\\n' '0.288 (mame0287-823-gf4f6f34f2a8)'\n",
        )
        .expect("write fake executable");
        let mut permissions = fs::metadata(&executable)
            .expect("read fake executable metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).expect("mark fake executable executable");

        let identity = inspect_executable(MameExecutableSource::development_tree(&executable))
            .expect("fake MAME must probe");

        assert_eq!(identity.source, MameExecutableSourceKind::DevelopmentTree);
        assert_eq!(identity.trust, MameExecutableTrust::Development);
        assert_eq!(identity.version, "0.288");
        assert_eq!(
            identity.build.as_deref(),
            Some("(mame0287-823-gf4f6f34f2a8)")
        );
        assert_eq!(
            identity.path,
            fs::canonicalize(&executable)
                .expect("canonical executable path")
                .to_string_lossy()
                .into_owned()
        );

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn missing_executable_is_rejected_with_structured_error() {
        let root = unique_temp_dir_portable("mame-missing-executable");
        let executable = root.join("definitely-missing-mame");

        let error = inspect_executable(MameExecutableSource::external(&executable))
            .expect_err("missing executable must fail");
        assert_eq!(error.code, "MAME_EXECUTABLE_NOT_FOUND");
    }

    #[cfg(unix)]
    #[test]
    fn non_executable_file_is_rejected_before_probe() {
        let root = unique_temp_dir("mame-non-executable");
        fs::create_dir_all(&root).expect("create test directory");
        let executable = root.join("mame");
        fs::write(&executable, "not executable").expect("write fake executable");

        let error = inspect_executable(MameExecutableSource::external(&executable))
            .expect_err("non-executable file must fail");
        assert_eq!(error.code, "MAME_EXECUTABLE_NOT_EXECUTABLE");

        fs::remove_dir_all(root).expect("remove test directory");
    }

    fn unique_temp_dir_portable(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "mame-tauri-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    #[cfg(unix)]
    fn unique_temp_dir(label: &str) -> PathBuf {
        unique_temp_dir_portable(label)
    }
}
