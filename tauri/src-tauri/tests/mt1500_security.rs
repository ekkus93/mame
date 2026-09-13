use std::path::PathBuf;

use mame_tauri_lib::mame::{
    validate_project_controlled_path, validate_short_identifier, validate_software_identifier,
};

#[test]
fn shell_metacharacters_never_become_machine_identifiers() {
    for value in [
        "pac;man",
        "pac&&man",
        "pac|man",
        "pac$(id)",
        "pac`id`",
        "pac\"man",
        "pac'man",
        "pac\\man",
        "pac man",
        "-help",
    ] {
        let error = validate_short_identifier("machine", value)
            .expect_err("shell-like machine identifier must fail closed");
        assert_eq!(error.code, "MAME_IDENTIFIER_INVALID", "{value:?}");
    }
}

#[test]
fn shell_metacharacters_never_become_software_identifiers() {
    for value in [
        "item;rm",
        "item&&rm",
        "item|rm",
        "item$(id)",
        "item`id`",
        "item\"quoted",
        "item'quoted",
        "item\\path",
        "item with spaces",
        "-help",
    ] {
        let error = validate_software_identifier(value)
            .expect_err("shell-like software identifier must fail closed");
        assert_eq!(error.code, "MAME_IDENTIFIER_INVALID", "{value:?}");
    }
}

#[test]
fn project_paths_reject_traversal_and_mame_path_list_injection() {
    let traversal = absolute_test_path("roms/../escape");
    assert_eq!(
        validate_project_controlled_path(&traversal)
            .expect_err("parent traversal must fail")
            .code,
        "MAME_PROJECT_PATH_TRAVERSAL"
    );

    let path_list = absolute_test_path("roms;other-roms");
    assert_eq!(
        validate_project_controlled_path(&path_list)
            .expect_err("MAME path-list injection must fail")
            .code,
        "MAME_PROJECT_PATH_LIST_SEPARATOR"
    );
}

#[cfg(unix)]
#[test]
fn project_paths_reject_environment_expansion_on_unix() {
    let expansion = absolute_test_path("roms/$HOME");
    assert_eq!(
        validate_project_controlled_path(&expansion)
            .expect_err("environment expansion must fail")
            .code,
        "MAME_PROJECT_PATH_EXPANSION"
    );
}

fn absolute_test_path(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(suffix);
    path
}
