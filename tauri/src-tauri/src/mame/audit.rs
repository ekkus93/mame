use serde::{Deserialize, Serialize};

const RAW_EXCERPT_LIMIT: usize = 16 * 1024;

/// Structured aggregate result derived from MAME's audit output and exit status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MameAuditClassification {
    Complete,
    BestAvailable,
    MissingRequired,
    Incorrect,
    MixedFailure,
    Unknown,
}

/// Fine-grained facts that remain useful even when the aggregate result is acceptable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MameAuditFacts {
    pub optional_missing: bool,
    pub no_good_dump_known: bool,
    pub needs_redump: bool,
    pub missing_required: bool,
    pub incorrect_checksum: bool,
    pub incorrect_length: bool,
    pub target_not_found: bool,
}

/// Conservative parse result for a completed MAME audit process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MameAuditParseResult {
    pub classification: MameAuditClassification,
    pub facts: MameAuditFacts,
    pub exit_code: Option<i32>,
    pub raw_excerpt: String,
    pub raw_truncated: bool,
}

/// Parse MAME audit stdout/stderr together with the process exit code.
///
/// The parser intentionally does not infer media availability from filesystem
/// presence. Unknown, contradictory, or insufficient output remains `Unknown`.
pub fn parse_mame_audit_output(
    stdout: &str,
    stderr: &str,
    exit_code: Option<i32>,
) -> MameAuditParseResult {
    let mut facts = MameAuditFacts::default();
    let mut saw_good_summary = false;
    let mut saw_best_available_summary = false;
    let mut saw_bad_summary = false;

    for line in stdout.lines().chain(stderr.lines()) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if is_target_not_found(line) {
            facts.target_not_found = true;
        }

        if line.contains("NOT FOUND BUT OPTIONAL") {
            facts.optional_missing = true;
        } else if line.contains("NOT FOUND - NO GOOD DUMP KNOWN") {
            facts.no_good_dump_known = true;
        } else if line.contains(" - NOT FOUND") || is_missing_set_summary(line) {
            facts.missing_required = true;
        }

        if line.contains("INCORRECT CHECKSUM:") {
            facts.incorrect_checksum = true;
        }
        if line.contains("INCORRECT LENGTH:") {
            facts.incorrect_length = true;
        }
        if line.contains("NEEDS REDUMP") {
            facts.needs_redump = true;
        }
        if line.contains("NO GOOD DUMP KNOWN") {
            facts.no_good_dump_known = true;
        }

        if line.starts_with("romset ") {
            if line.ends_with(" is good") {
                saw_good_summary = true;
            } else if line.ends_with(" is best available") {
                saw_best_available_summary = true;
            } else if line.ends_with(" is bad") {
                saw_bad_summary = true;
            }
        }
    }

    let classification = classify(
        exit_code,
        facts,
        saw_good_summary,
        saw_best_available_summary,
        saw_bad_summary,
    );
    let (raw_excerpt, raw_truncated) = build_raw_excerpt(stdout, stderr);

    MameAuditParseResult {
        classification,
        facts,
        exit_code,
        raw_excerpt,
        raw_truncated,
    }
}

fn classify(
    exit_code: Option<i32>,
    facts: MameAuditFacts,
    saw_good_summary: bool,
    saw_best_available_summary: bool,
    saw_bad_summary: bool,
) -> MameAuditClassification {
    let has_incorrect_content = facts.incorrect_checksum || facts.incorrect_length;

    match exit_code {
        Some(0) => {
            if facts.target_not_found
                || facts.missing_required
                || has_incorrect_content
                || saw_bad_summary
            {
                MameAuditClassification::Unknown
            } else if saw_best_available_summary
                || facts.optional_missing
                || facts.no_good_dump_known
                || facts.needs_redump
            {
                MameAuditClassification::BestAvailable
            } else if saw_good_summary {
                MameAuditClassification::Complete
            } else {
                MameAuditClassification::Unknown
            }
        }
        Some(2) => {
            if facts.target_not_found {
                MameAuditClassification::Unknown
            } else if facts.missing_required && has_incorrect_content {
                MameAuditClassification::MixedFailure
            } else if has_incorrect_content {
                MameAuditClassification::Incorrect
            } else if facts.missing_required {
                MameAuditClassification::MissingRequired
            } else {
                // MAME uses exit 2 for multiple media-audit failures. A bare
                // `is bad` line does not identify which failure occurred.
                MameAuditClassification::Unknown
            }
        }
        Some(5) | None => MameAuditClassification::Unknown,
        Some(_) => MameAuditClassification::Unknown,
    }
}

fn is_target_not_found(line: &str) -> bool {
    line.starts_with("No matching systems found for '")
        || line.starts_with("No matching software lists found for '")
        || line.starts_with("No software list items are defined for systems matching '")
}

fn is_missing_set_summary(line: &str) -> bool {
    (line.starts_with("romset \"") && line.ends_with("\" not found!"))
        || line.starts_with("No software items found for systems matching '")
        || line.starts_with("no romsets found for software list \"")
}

fn build_raw_excerpt(stdout: &str, stderr: &str) -> (String, bool) {
    let mut excerpt = String::new();
    let mut truncated = false;

    append_stream(&mut excerpt, &mut truncated, "stdout", stdout);
    append_stream(&mut excerpt, &mut truncated, "stderr", stderr);

    (excerpt, truncated)
}

fn append_stream(excerpt: &mut String, truncated: &mut bool, label: &str, stream: &str) {
    if stream.trim().is_empty() {
        return;
    }

    let separator = if excerpt.is_empty() { "" } else { "\n" };
    append_bounded(excerpt, truncated, separator);
    append_bounded(excerpt, truncated, label);
    append_bounded(excerpt, truncated, ":\n");
    append_bounded(excerpt, truncated, stream);
}

fn append_bounded(excerpt: &mut String, truncated: &mut bool, value: &str) {
    let remaining = RAW_EXCERPT_LIMIT.saturating_sub(excerpt.len());
    if remaining == 0 {
        if !value.is_empty() {
            *truncated = true;
        }
        return;
    }

    if value.len() <= remaining {
        excerpt.push_str(value);
        return;
    }

    let mut end = remaining;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    excerpt.push_str(&value[..end]);
    *truncated = true;
}

#[cfg(test)]
mod tests {
    use super::{parse_mame_audit_output, MameAuditClassification, RAW_EXCERPT_LIMIT};

    #[test]
    fn classifies_confirmed_good_set_as_complete() {
        let result = parse_mame_audit_output(
            "romset pacman is good\n1 romsets found, 1 were OK.\n",
            "",
            Some(0),
        );

        assert_eq!(result.classification, MameAuditClassification::Complete);
        assert!(!result.facts.optional_missing);
        assert!(!result.facts.missing_required);
    }

    #[test]
    fn preserves_optional_missing_as_best_available() {
        let result = parse_mame_audit_output(
            "pacman: optional.bin (1 bytes) - NOT FOUND BUT OPTIONAL\nromset pacman is best available\n1 romsets found, 1 were OK.\n",
            "",
            Some(0),
        );

        assert_eq!(
            result.classification,
            MameAuditClassification::BestAvailable
        );
        assert!(result.facts.optional_missing);
        assert!(!result.facts.missing_required);
    }

    #[test]
    fn known_undumped_media_is_not_required_missing() {
        let result = parse_mame_audit_output(
            "pacman: mystery.bin (1 bytes) - NOT FOUND - NO GOOD DUMP KNOWN\nromset pacman is best available\n",
            "",
            Some(0),
        );

        assert_eq!(
            result.classification,
            MameAuditClassification::BestAvailable
        );
        assert!(result.facts.no_good_dump_known);
        assert!(!result.facts.missing_required);
    }

    #[test]
    fn needs_redump_remains_best_available() {
        let result = parse_mame_audit_output(
            "pacman: suspect.bin (1 bytes) - NEEDS REDUMP\nromset pacman is best available\n",
            "",
            Some(0),
        );

        assert_eq!(
            result.classification,
            MameAuditClassification::BestAvailable
        );
        assert!(result.facts.needs_redump);
    }

    #[test]
    fn classifies_required_missing_content() {
        let result = parse_mame_audit_output(
            "pacman: required.bin (1 bytes) - NOT FOUND\nromset pacman is bad\n",
            "1 romsets found, 0 were OK.\n",
            Some(2),
        );

        assert_eq!(
            result.classification,
            MameAuditClassification::MissingRequired
        );
        assert!(result.facts.missing_required);
    }

    #[test]
    fn classifies_whole_set_not_found_as_required_missing() {
        let result = parse_mame_audit_output("", "romset \"pacman\" not found!\n", Some(2));

        assert_eq!(
            result.classification,
            MameAuditClassification::MissingRequired
        );
        assert!(result.facts.missing_required);
    }

    #[test]
    fn classifies_bad_checksum_as_incorrect() {
        let result = parse_mame_audit_output(
            "pacman: bad.bin (1 bytes) - INCORRECT CHECKSUM:\nEXPECTED: CRC(00000000)\n   FOUND: CRC(11111111)\nromset pacman is bad\n",
            "",
            Some(2),
        );

        assert_eq!(result.classification, MameAuditClassification::Incorrect);
        assert!(result.facts.incorrect_checksum);
        assert!(!result.facts.missing_required);
    }

    #[test]
    fn classifies_wrong_length_as_incorrect() {
        let result = parse_mame_audit_output(
            "pacman: bad.bin (1 bytes) - INCORRECT LENGTH: 2 bytes\nromset pacman is bad\n",
            "",
            Some(2),
        );

        assert_eq!(result.classification, MameAuditClassification::Incorrect);
        assert!(result.facts.incorrect_length);
    }

    #[test]
    fn preserves_mixed_missing_and_incorrect_failure() {
        let result = parse_mame_audit_output(
            "pacman: missing.bin (1 bytes) - NOT FOUND\npacman: bad.bin (1 bytes) - INCORRECT LENGTH: 2 bytes\nromset pacman is bad\n",
            "",
            Some(2),
        );

        assert_eq!(result.classification, MameAuditClassification::MixedFailure);
        assert!(result.facts.missing_required);
        assert!(result.facts.incorrect_length);
    }

    #[test]
    fn does_not_guess_from_exit_two_and_bad_summary_alone() {
        let result = parse_mame_audit_output("romset pacman is bad\n", "", Some(2));

        assert_eq!(result.classification, MameAuditClassification::Unknown);
    }

    #[test]
    fn unknown_target_is_not_missing_content() {
        let result = parse_mame_audit_output(
            "",
            "No matching systems found for 'not-a-system'\n",
            Some(5),
        );

        assert_eq!(result.classification, MameAuditClassification::Unknown);
        assert!(result.facts.target_not_found);
        assert!(!result.facts.missing_required);
    }

    #[test]
    fn contradictory_success_exit_and_bad_output_is_unknown() {
        let result = parse_mame_audit_output(
            "pacman: required.bin (1 bytes) - NOT FOUND\nromset pacman is bad\n",
            "",
            Some(0),
        );

        assert_eq!(result.classification, MameAuditClassification::Unknown);
    }

    #[test]
    fn unrecognized_success_output_is_unknown() {
        let result = parse_mame_audit_output("unexpected output\n", "", Some(0));

        assert_eq!(result.classification, MameAuditClassification::Unknown);
    }

    #[test]
    fn missing_process_exit_status_is_unknown() {
        let result = parse_mame_audit_output("romset pacman is good\n", "", None);

        assert_eq!(result.classification, MameAuditClassification::Unknown);
    }

    #[test]
    fn raw_diagnostics_are_bounded_without_breaking_utf8() {
        let stdout = "é".repeat(RAW_EXCERPT_LIMIT);
        let result = parse_mame_audit_output(&stdout, "stderr detail", Some(0));

        assert!(result.raw_truncated);
        assert!(result.raw_excerpt.len() <= RAW_EXCERPT_LIMIT);
        assert!(result
            .raw_excerpt
            .is_char_boundary(result.raw_excerpt.len()));
    }
}
