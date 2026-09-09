use std::{
    io::{self, Read},
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    errors::{AppError, AppResult},
    mame::{inspect_executable, validate_executable_path, MameExecutableSource},
};

use super::{
    catalog::CatalogRepository,
    model::{MetadataRefreshResult, MetadataStatus},
};

const STDERR_LIMIT: usize = 64 * 1024;

pub(crate) fn refresh_catalog(
    source: MameExecutableSource,
    catalog_path: &Path,
) -> AppResult<MetadataRefreshResult> {
    let identity = inspect_executable(source.clone())?;
    let executable_path = validate_executable_path(source.path())?;
    let generated_at_epoch_ms = epoch_millis()?;

    let mut command = Command::new(&executable_path);
    command
        .args(["-noreadconfig", "-listxml"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|error| {
        AppError::new(
            "MAME_METADATA_LAUNCH_FAILED",
            "MAME could not be launched to generate metadata.",
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
                "MAME_METADATA_STDOUT_MISSING",
                "MAME metadata generation did not provide a stdout stream.",
            ));
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            terminate_child(&mut child);
            return Err(AppError::new(
                "MAME_METADATA_STDERR_MISSING",
                "MAME metadata generation did not provide a stderr stream.",
            ));
        }
    };
    let stderr_reader = thread::spawn(move || drain_bounded(stderr, STDERR_LIMIT));

    let mut repository = match CatalogRepository::open(catalog_path) {
        Ok(repository) => repository,
        Err(error) => {
            terminate_child(&mut child);
            return Err(with_stderr(error, join_stderr(stderr_reader)));
        }
    };
    let mut import = match repository.begin_import(&identity, generated_at_epoch_ms) {
        Ok(import) => import,
        Err(error) => {
            terminate_child(&mut child);
            return Err(with_stderr(error, join_stderr(stderr_reader)));
        }
    };

    let listxml = match import.import_listxml(io::BufReader::new(stdout)) {
        Ok(summary) => summary,
        Err(error) => {
            drop(import);
            terminate_child(&mut child);
            return Err(with_stderr(error, join_stderr(stderr_reader)));
        }
    };

    if let Some(build) = listxml.build.as_deref() {
        if build != identity.raw_version_line {
            drop(import);
            terminate_child(&mut child);
            return Err(with_stderr(
                AppError::new(
                    "MAME_METADATA_IDENTITY_MISMATCH",
                    "MAME -listxml reported a different build than the executable identity probe.",
                )
                .with_details(serde_json::json!({
                    "versionProbe": identity.raw_version_line,
                    "listxmlBuild": build
                })),
                join_stderr(stderr_reader),
            ));
        }
    } else {
        drop(import);
        terminate_child(&mut child);
        return Err(with_stderr(
            AppError::new(
                "MAME_METADATA_BUILD_MISSING",
                "MAME -listxml did not report its build identity.",
            ),
            join_stderr(stderr_reader),
        ));
    }

    let status = match child.wait() {
        Ok(status) => status,
        Err(error) => {
            drop(import);
            return Err(with_stderr(
                AppError::new(
                    "MAME_METADATA_WAIT_FAILED",
                    "The MAME metadata process could not be observed to completion.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() })),
                join_stderr(stderr_reader),
            ));
        }
    };
    let stderr = join_stderr(stderr_reader);

    if !status.success() {
        drop(import);
        return Err(AppError::new(
            "MAME_METADATA_GENERATION_FAILED",
            "MAME exited unsuccessfully while generating metadata.",
        )
        .with_details(serde_json::json!({
            "exitCode": status.code(),
            "stderr": stderr.text,
            "stderrTruncated": stderr.truncated,
            "stderrReadError": stderr.error
        })));
    }
    if let Some(read_error) = stderr.error {
        drop(import);
        return Err(AppError::new(
            "MAME_METADATA_STDERR_READ_FAILED",
            "MAME metadata diagnostics could not be read completely.",
        )
        .with_details(serde_json::json!({
            "cause": read_error,
            "stderr": stderr.text,
            "stderrTruncated": stderr.truncated
        })));
    }

    let imported_at_epoch_ms = epoch_millis()?;
    let generation = import.finish(listxml, imported_at_epoch_ms)?;

    Ok(MetadataRefreshResult {
        schema_version: 1,
        generation,
    })
}

pub(crate) fn metadata_status(
    source: MameExecutableSource,
    catalog_path: &Path,
) -> AppResult<MetadataStatus> {
    let identity = inspect_executable(source)?;
    CatalogRepository::open(catalog_path)?.metadata_status(identity)
}

fn epoch_millis() -> AppResult<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            AppError::new(
                "SYSTEM_CLOCK_INVALID",
                "The system clock cannot represent the metadata timestamp.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
    u64::try_from(duration.as_millis()).map_err(|error| {
        AppError::new(
            "SYSTEM_CLOCK_OVERFLOW",
            "The metadata timestamp exceeds the supported range.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[derive(Debug)]
struct BoundedText {
    text: String,
    truncated: bool,
    error: Option<String>,
}

fn drain_bounded<R: Read>(mut reader: R, limit: usize) -> BoundedText {
    let mut retained = Vec::with_capacity(limit.min(8192));
    let mut buffer = [0_u8; 4096];
    let mut truncated = false;

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                let remaining = limit.saturating_sub(retained.len());
                let keep = remaining.min(count);
                retained.extend_from_slice(&buffer[..keep]);
                truncated |= keep < count;
            }
            Err(error) => {
                return BoundedText {
                    text: String::from_utf8_lossy(&retained).into_owned(),
                    truncated,
                    error: Some(error.to_string()),
                }
            }
        }
    }

    BoundedText {
        text: String::from_utf8_lossy(&retained).into_owned(),
        truncated,
        error: None,
    }
}

fn join_stderr(handle: thread::JoinHandle<BoundedText>) -> BoundedText {
    handle.join().unwrap_or_else(|_| BoundedText {
        text: String::new(),
        truncated: false,
        error: Some("stderr reader thread panicked".to_owned()),
    })
}

fn with_stderr(error: AppError, stderr: BoundedText) -> AppError {
    if stderr.text.is_empty() && stderr.error.is_none() {
        return error;
    }

    AppError::new(error.code, error.message).with_details(serde_json::json!({
        "causeDetails": error.details,
        "stderr": stderr.text,
        "stderrTruncated": stderr.truncated,
        "stderrReadError": stderr.error
    }))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::refresh_catalog;
    use crate::{
        mame::MameExecutableSource,
        metadata::catalog::{CatalogRepository, CloneFilter, MachineQuery},
    };

    #[cfg(unix)]
    #[test]
    fn refresh_streams_fake_mame_into_catalog() {
        let root = unique_temp_dir("generator-success");
        let executable = write_fake_mame(&root, 0);
        let database = root.join("catalog.sqlite3");

        let result = refresh_catalog(MameExecutableSource::external(&executable), &database)
            .expect("fake metadata generation must succeed");
        assert_eq!(result.generation.machine_count, 4);

        let repository = CatalogRepository::open(&database).expect("open imported catalog");
        let page = repository
            .query_machines(&MachineQuery {
                text: Some("Apple IIe".to_owned()),
                manufacturer: None,
                year: None,
                driver_status: None,
                clone_filter: CloneFilter::All,
                include_devices: false,
                limit: 25,
                offset: 0,
            })
            .expect("query imported catalog");
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].short_name, "apple2e");

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn failed_mame_generation_does_not_activate_partial_catalog() {
        let root = unique_temp_dir("generator-failure");
        let good = write_fake_mame(&root, 0);
        let database = root.join("catalog.sqlite3");
        let first = refresh_catalog(MameExecutableSource::external(&good), &database)
            .expect("seed catalog");

        let bad = root.join("bad fake mame");
        write_script(
            &bad,
            r#"if [ "$1" = '-noreadconfig' ] && [ "$2" = '-version' ]; then
  printf '%s\n' '0.288 test-fixture'
  exit 0
fi
if [ "$1" = '-noreadconfig' ] && [ "$2" = '-listxml' ]; then
  printf '%s\n' '<mame build="0.288 test-fixture" mameconfig="10"><machine name="partial"><description>Partial</description></machine>'
  printf '%s\n' 'metadata failed' >&2
  exit 17
fi
exit 99
"#,
        );

        let error = refresh_catalog(MameExecutableSource::external(&bad), &database)
            .expect_err("failed generation must fail");
        assert!(matches!(
            error.code.as_str(),
            "MAME_METADATA_XML_INVALID"
                | "MAME_METADATA_XML_STRUCTURE_INVALID"
                | "MAME_METADATA_GENERATION_FAILED"
        ));

        let active = CatalogRepository::open(&database)
            .expect("open catalog")
            .active_generation()
            .expect("active generation")
            .expect("seed generation retained");
        assert_eq!(active.generation_id, first.generation.generation_id);

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[cfg(unix)]
    fn write_fake_mame(root: &PathBuf, exit_code: i32) -> PathBuf {
        let executable = root.join("fake mame");
        fs::create_dir_all(root).expect("create test directory");
        let fixture = include_str!("../../tests/fixtures/listxml-representative.xml");
        let body = format!(
            "if [ \"$1\" = '-noreadconfig' ] && [ \"$2\" = '-version' ]; then\n  printf '%s\\n' '0.288 test-fixture'\n  exit 0\nfi\nif [ \"$1\" = '-noreadconfig' ] && [ \"$2\" = '-listxml' ]; then\n  cat <<'MAME_XML'\n{fixture}\nMAME_XML\n  exit {exit_code}\nfi\nexit 99\n"
        );
        write_script(&executable, &body);
        executable
    }

    #[cfg(unix)]
    fn write_script(path: &PathBuf, body: &str) {
        use std::os::unix::fs::PermissionsExt;
        let script = format!("#!/bin/sh\n{body}");
        fs::write(path, script).expect("write fake MAME executable");
        let mut permissions = fs::metadata(path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("mark executable");
    }

    #[cfg(unix)]
    fn unique_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "mame-tauri-metadata-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
