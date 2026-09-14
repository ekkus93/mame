use std::{
    collections::HashMap,
    io::{self, Read},
    process::{Command, Stdio},
    str,
    thread,
    time::{Duration, Instant},
};

use quick_xml::{events::Event, Reader};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};

use super::{validate_executable_path, validate_short_identifier, MameExecutableSource};

const BIOS_QUERY_TIMEOUT: Duration = Duration::from_secs(15);
const BIOS_STDOUT_LIMIT: usize = 4 * 1024 * 1024;
const BIOS_STDERR_LIMIT: usize = 256 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BiosChoice {
    pub name: String,
    pub description: String,
    pub is_default: bool,
}

pub(crate) fn get_machine_bios_choices(
    source: &MameExecutableSource,
    machine: &str,
) -> AppResult<Vec<BiosChoice>> {
    validate_short_identifier("machine", machine)?;
    let path = validate_executable_path(source.path())?;
    let mut command = Command::new(&path);
    command
        .args(["-noreadconfig", "-listxml", machine])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|error| {
        AppError::new(
            "MAME_BIOS_QUERY_LAUNCH_FAILED",
            "MAME could not be launched to read BIOS metadata.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "machine": machine,
            "cause": error.to_string()
        }))
    })?;

    let stdout = child.stdout.take().ok_or_else(|| {
        AppError::new(
            "MAME_BIOS_QUERY_OUTPUT_FAILED",
            "MAME BIOS-query stdout was not available.",
        )
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        AppError::new(
            "MAME_BIOS_QUERY_OUTPUT_FAILED",
            "MAME BIOS-query stderr was not available.",
        )
    })?;
    let stdout_reader = thread::spawn(move || drain_bounded(stdout, BIOS_STDOUT_LIMIT));
    let stderr_reader = thread::spawn(move || drain_bounded(stderr, BIOS_STDERR_LIMIT));

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= BIOS_QUERY_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(AppError::new(
                    "MAME_BIOS_QUERY_TIMEOUT",
                    "MAME did not finish the BIOS query within the bounded timeout.",
                )
                .with_details(serde_json::json!({
                    "machine": machine,
                    "timeoutMs": BIOS_QUERY_TIMEOUT.as_millis()
                })));
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                let _ = child.kill();
                return Err(AppError::new(
                    "MAME_BIOS_QUERY_WAIT_FAILED",
                    "The MAME BIOS query process could not be observed.",
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
            "MAME_BIOS_QUERY_FAILED",
            "MAME rejected the requested BIOS query.",
        )
        .with_details(serde_json::json!({
            "machine": machine,
            "exitCode": status.code(),
            "stdout": stdout_text,
            "stderr": stderr_text,
            "stdoutTruncated": stdout.truncated,
            "stderrTruncated": stderr.truncated
        })));
    }
    if stdout.truncated {
        return Err(AppError::new(
            "MAME_BIOS_QUERY_TOO_LARGE",
            "MAME BIOS metadata exceeded the bounded capture limit.",
        )
        .with_details(serde_json::json!({
            "machine": machine,
            "limitBytes": BIOS_STDOUT_LIMIT
        })));
    }
    if stdout_text.trim().is_empty() {
        return Err(AppError::new(
            "MAME_BIOS_QUERY_EMPTY",
            "MAME returned no metadata for the requested machine.",
        )
        .with_details(serde_json::json!({ "machine": machine })));
    }

    parse_bios_choices(&stdout_text, machine)
}

pub(crate) fn validate_bios_selection(choices: &[BiosChoice], selected: &str) -> AppResult<()> {
    if choices.iter().any(|choice| choice.name == selected) {
        return Ok(());
    }
    Err(AppError::new(
        "MAME_BIOS_SELECTION_INVALID",
        "The selected BIOS is not reported for the requested machine.",
    )
    .with_details(serde_json::json!({ "bios": selected })))
}

fn parse_bios_choices(xml: &str, expected_machine: &str) -> AppResult<Vec<BiosChoice>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut in_expected_machine = false;
    let mut machine_seen = false;
    let mut choices = Vec::new();

    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer).map_err(xml_error)? {
            Event::Start(start) if start.name().as_ref() == b"machine" => {
                let attrs = attributes(&start)?;
                in_expected_machine = attrs.get("name").is_some_and(|name| name == expected_machine);
                machine_seen |= in_expected_machine;
            }
            Event::Empty(empty) if in_expected_machine && empty.name().as_ref() == b"biosset" => {
                choices.push(parse_biosset(&attributes(&empty)?)?);
            }
            Event::End(end) if end.name().as_ref() == b"machine" => in_expected_machine = false,
            Event::Eof => break,
            _ => {}
        }
    }

    if !machine_seen {
        return Err(AppError::new(
            "MAME_BIOS_MACHINE_MISMATCH",
            "MAME BIOS metadata did not contain the requested machine.",
        )
        .with_details(serde_json::json!({ "machine": expected_machine })));
    }
    if choices.iter().filter(|choice| choice.is_default).count() > 1 {
        return Err(AppError::new(
            "MAME_BIOS_METADATA_INVALID",
            "MAME reported more than one default BIOS for the machine.",
        ));
    }
    Ok(choices)
}

fn parse_biosset(attributes: &HashMap<String, String>) -> AppResult<BiosChoice> {
    let name = required_attr(attributes, "name")?.to_owned();
    validate_bios_identifier(&name)?;
    Ok(BiosChoice {
        name,
        description: required_attr(attributes, "description")?.to_owned(),
        is_default: match attributes.get("default").map(String::as_str) {
            None | Some("no") => false,
            Some("yes") => true,
            Some(value) => {
                return Err(AppError::new(
                    "MAME_BIOS_METADATA_INVALID",
                    "MAME reported an invalid BIOS default flag.",
                )
                .with_details(serde_json::json!({ "default": value })))
            }
        },
    })
}

pub(crate) fn validate_bios_identifier(value: &str) -> AppResult<()> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-' | b'.')
        });
    if valid {
        return Ok(());
    }
    Err(AppError::new(
        "MAME_BIOS_IDENTIFIER_INVALID",
        "The BIOS identifier contains unsupported characters or exceeds the bounded length.",
    )
    .with_details(serde_json::json!({ "bios": value })))
}

fn attributes(start: &quick_xml::events::BytesStart<'_>) -> AppResult<HashMap<String, String>> {
    let mut result = HashMap::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|error| {
            AppError::new(
                "MAME_BIOS_XML_ATTRIBUTE_INVALID",
                "MAME BIOS metadata contains a malformed XML attribute.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        let key = str::from_utf8(attribute.key.as_ref()).map_err(|error| {
            AppError::new(
                "MAME_BIOS_XML_ATTRIBUTE_INVALID",
                "MAME BIOS metadata contains a non-UTF-8 attribute name.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        let value = attribute
            .unescape_value()
            .map_err(|error| {
                AppError::new(
                    "MAME_BIOS_XML_ATTRIBUTE_INVALID",
                    "MAME BIOS metadata contains an attribute value that cannot be decoded.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() }))
            })?
            .into_owned();
        result.insert(key.to_owned(), value);
    }
    Ok(result)
}

fn required_attr<'a>(attributes: &'a HashMap<String, String>, name: &str) -> AppResult<&'a str> {
    attributes
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::new(
                "MAME_BIOS_REQUIRED_ATTRIBUTE_MISSING",
                "MAME BIOS metadata is missing a required attribute.",
            )
            .with_details(serde_json::json!({ "attribute": name }))
        })
}

fn xml_error(error: quick_xml::Error) -> AppError {
    AppError::new(
        "MAME_BIOS_XML_INVALID",
        "MAME BIOS metadata is not valid XML.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
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
                "MAME_BIOS_QUERY_OUTPUT_FAILED",
                format!("The MAME BIOS-query {stream} reader thread panicked."),
            )
        })?
        .map_err(|error| {
            AppError::new(
                "MAME_BIOS_QUERY_OUTPUT_FAILED",
                "MAME BIOS-query output could not be captured.",
            )
            .with_details(serde_json::json!({
                "stream": stream,
                "cause": error.to_string()
            }))
        })
}

#[cfg(test)]
mod tests {
    use super::{parse_bios_choices, validate_bios_identifier, validate_bios_selection};

    #[test]
    fn parses_bios_choices_for_requested_machine() {
        let xml = r#"<mame><machine name="pc"><biosset name="us" description="US BIOS" default="yes"/><biosset name="jp" description="Japan BIOS"/></machine></mame>"#;
        let choices = parse_bios_choices(xml, "pc").expect("bios metadata");
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].name, "us");
        assert!(choices[0].is_default);
        validate_bios_selection(&choices, "jp").expect("reported BIOS must validate");
        assert_eq!(
            validate_bios_selection(&choices, "eu")
                .expect_err("unreported BIOS must fail")
                .code,
            "MAME_BIOS_SELECTION_INVALID"
        );
    }

    #[test]
    fn bios_identifier_is_bounded_and_option_safe() {
        for valid in ["us", "v2.0", "rev-3", "jp_1"] {
            validate_bios_identifier(valid).expect("valid BIOS identifier");
        }
        for invalid in ["", "-bios", "US", "bad value", "bad/../value"] {
            assert_eq!(
                validate_bios_identifier(invalid)
                    .expect_err("invalid BIOS identifier")
                    .code,
                "MAME_BIOS_IDENTIFIER_INVALID"
            );
        }
    }
}
