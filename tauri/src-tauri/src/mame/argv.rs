//! Safe construction of MAME command-line argument vectors.

use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
};

use crate::{
    config::{AudioPreference, LaunchPreferencesV1, RendererPreference, WindowPreference},
    errors::{AppError, AppResult},
};

use super::bios::validate_bios_identifier;

const MAX_IDENTIFIER_SEGMENT_LEN: usize = 16;
const MAX_SOFTWARE_LIST_IDENTIFIER_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MameLaunchTarget {
    pub machine: String,
    pub software: Option<String>,
    pub bios: Option<String>,
    pub project_paths: Vec<ProjectPathArgument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPathArgument {
    pub option: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MameArgv {
    args: Vec<OsString>,
}

impl MameArgv {
    pub fn as_slice(&self) -> &[OsString] {
        &self.args
    }

    pub fn into_vec(self) -> Vec<OsString> {
        self.args
    }
}

pub fn build_launch_argv(target: &MameLaunchTarget) -> AppResult<MameArgv> {
    validate_short_identifier("machine", &target.machine)?;

    let mut args = Vec::with_capacity(4 + (target.project_paths.len() * 2));
    args.push(OsString::from(&target.machine));

    if let Some(software) = &target.software {
        validate_software_identifier(software)?;
        args.push(OsString::from(software));
    }

    if let Some(bios) = &target.bios {
        validate_bios_identifier(bios)?;
        push_option(&mut args, "bios", bios);
    }

    for project_path in &target.project_paths {
        validate_option_name(&project_path.option)?;
        validate_project_path_argument(&project_path.option, &project_path.path)?;
        args.push(OsString::from(format!("-{}", project_path.option)));
        args.push(project_path.path.as_os_str().to_owned());
    }

    Ok(MameArgv { args })
}

pub fn build_launch_argv_with_preferences(
    target: &MameLaunchTarget,
    preferences: &LaunchPreferencesV1,
) -> AppResult<MameArgv> {
    let mut args = build_launch_argv(target)?.into_vec();

    match preferences.window_mode {
        WindowPreference::Inherit => {}
        WindowPreference::Windowed => args.push(OsString::from("-window")),
        WindowPreference::Fullscreen => args.push(OsString::from("-nowindow")),
    }

    match preferences.renderer {
        RendererPreference::Inherit => {}
        RendererPreference::Auto => push_option(&mut args, "video", "auto"),
        RendererPreference::Bgfx => push_option(&mut args, "video", "bgfx"),
        RendererPreference::OpenGl => push_option(&mut args, "video", "opengl"),
        RendererPreference::Software => push_option(&mut args, "video", "soft"),
    }

    match preferences.audio {
        AudioPreference::Inherit => {}
        AudioPreference::Auto => push_option(&mut args, "sound", "auto"),
        AudioPreference::Disabled => push_option(&mut args, "sound", "none"),
    }

    Ok(MameArgv { args })
}

pub fn force_in_app_gameplay_video(argv: &mut Vec<OsString>) {
    let mut filtered = Vec::with_capacity(argv.len() + 2);
    let mut index = 0_usize;

    while index < argv.len() {
        let argument = &argv[index];
        if argument == "-video" {
            index = index.saturating_add(2);
            continue;
        }
        if argument == "-window" || argument == "-nowindow" {
            index += 1;
            continue;
        }
        filtered.push(argument.clone());
        index += 1;
    }

    push_option(&mut filtered, "video", "none");
    *argv = filtered;
}

pub fn compose_mame_path_list<'a, I>(paths: I) -> AppResult<Option<OsString>>
where
    I: IntoIterator<Item = &'a Path>,
{
    let mut path_list = OsString::new();
    let mut count = 0_usize;

    for path in paths {
        validate_project_controlled_path(path)?;
        if count != 0 {
            path_list.push(";");
        }
        path_list.push(path.as_os_str());
        count += 1;
    }

    if count == 0 {
        Ok(None)
    } else {
        Ok(Some(path_list))
    }
}

fn push_option(args: &mut Vec<OsString>, option: &str, value: &str) {
    args.push(OsString::from(format!("-{option}")));
    args.push(OsString::from(value));
}

pub fn validate_short_identifier(field: &str, value: &str) -> AppResult<()> {
    validate_identifier_segment(field, value, MAX_IDENTIFIER_SEGMENT_LEN)
}

pub fn validate_software_list_identifier(value: &str) -> AppResult<()> {
    validate_identifier_segment("softwareList", value, MAX_SOFTWARE_LIST_IDENTIFIER_LEN)
}

pub fn validate_software_identifier(value: &str) -> AppResult<()> {
    if value.is_empty() {
        return Err(invalid_identifier_error(
            "software",
            value,
            "Software identifier must not be empty.",
        ));
    }

    let segments: Vec<_> = value.split(':').collect();
    if segments.len() > 3 {
        return Err(invalid_identifier_error(
            "software",
            value,
            "Software identifier may contain at most list, item, and part segments.",
        ));
    }

    if segments.len() == 1 {
        validate_identifier_segment("software", segments[0], MAX_IDENTIFIER_SEGMENT_LEN)?;
    } else {
        validate_software_list_identifier(segments[0])?;
        for segment in &segments[1..] {
            validate_identifier_segment("software", segment, MAX_IDENTIFIER_SEGMENT_LEN)?;
        }
    }

    Ok(())
}

fn validate_project_path_argument(option: &str, path: &Path) -> AppResult<()> {
    if option == "rompath" {
        validate_project_controlled_path_list(path)
    } else {
        validate_project_controlled_path(path)
    }
}

pub fn validate_project_controlled_path(path: &Path) -> AppResult<()> {
    if path.as_os_str().is_empty() {
        return Err(AppError::new(
            "MAME_PROJECT_PATH_EMPTY",
            "A project-controlled MAME path is empty.",
        ));
    }

    if !path.is_absolute() {
        return Err(AppError::new(
            "MAME_PROJECT_PATH_NOT_ABSOLUTE",
            "Project-controlled MAME paths must be absolute.",
        )
        .with_details(serde_json::json!({ "path": path })));
    }

    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(AppError::new(
            "MAME_PROJECT_PATH_TRAVERSAL",
            "Project-controlled MAME paths must not contain parent-directory traversal.",
        )
        .with_details(serde_json::json!({ "path": path })));
    }

    let path_text = path.to_string_lossy();
    if path_text.contains(';') {
        return Err(AppError::new(
            "MAME_PROJECT_PATH_LIST_SEPARATOR",
            "A project-controlled MAME path must represent exactly one path.",
        )
        .with_details(serde_json::json!({ "path": path })));
    }

    #[cfg(unix)]
    if path_text.contains('$') {
        return Err(AppError::new(
            "MAME_PROJECT_PATH_EXPANSION",
            "Project-controlled MAME paths must not contain MAME environment-variable expressions.",
        )
        .with_details(serde_json::json!({ "path": path })));
    }

    #[cfg(windows)]
    if path_text.contains('%') {
        return Err(AppError::new(
            "MAME_PROJECT_PATH_EXPANSION",
            "Project-controlled MAME paths must not contain MAME environment-variable expressions.",
        )
        .with_details(serde_json::json!({ "path": path })));
    }

    Ok(())
}

fn validate_project_controlled_path_list(path_list: &Path) -> AppResult<()> {
    let path_text = path_list.to_string_lossy();
    if !path_text.contains(';') {
        return validate_project_controlled_path(path_list);
    }

    for segment in path_text.split(';') {
        if segment.is_empty() {
            return Err(AppError::new(
                "MAME_PROJECT_PATH_LIST_EMPTY_SEGMENT",
                "A MAME path list must not contain an empty path segment.",
            )
            .with_details(serde_json::json!({ "pathList": path_text })));
        }
        validate_project_controlled_path(Path::new(segment))?;
    }

    Ok(())
}

fn validate_identifier_segment(field: &str, value: &str, max_len: usize) -> AppResult<()> {
    if value.is_empty() {
        return Err(invalid_identifier_error(
            field,
            value,
            "Identifier must not be empty.",
        ));
    }

    if value.len() > max_len {
        return Err(invalid_identifier_error(
            field,
            value,
            &format!("Identifier exceeds the supported {max_len}-character limit."),
        ));
    }

    if !value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(invalid_identifier_error(
            field,
            value,
            "Identifier may contain only lowercase ASCII letters, decimal digits, and underscores.",
        ));
    }

    Ok(())
}

fn invalid_identifier_error(field: &str, value: &str, reason: &str) -> AppError {
    AppError::new(
        "MAME_IDENTIFIER_INVALID",
        format!("The {field} identifier is invalid."),
    )
    .with_details(serde_json::json!({
        "field": field,
        "value": value,
        "reason": reason
    }))
}

fn validate_option_name(option: &str) -> AppResult<()> {
    if option.is_empty()
        || !option
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(AppError::new(
            "MAME_OPTION_INVALID",
            "A project-controlled MAME option name is invalid.",
        )
        .with_details(serde_json::json!({ "option": option })));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use crate::config::{
        AudioPreference, LaunchPreferencesV1, RendererPreference, WindowPreference,
    };

    use super::{
        build_launch_argv, build_launch_argv_with_preferences, compose_mame_path_list,
        force_in_app_gameplay_video, validate_project_controlled_path, validate_short_identifier,
        validate_software_identifier, validate_software_list_identifier, MameLaunchTarget,
        ProjectPathArgument,
    };

    #[test]
    fn launch_arguments_are_discrete_argv_entries() {
        let path = absolute_test_path("ROMs with spaces/日本語/&[]!");
        let target = MameLaunchTarget {
            machine: "pacman".to_owned(),
            software: Some("list_name:item_name:cart".to_owned()),
            bios: None,
            project_paths: vec![ProjectPathArgument {
                option: "rompath".to_owned(),
                path: path.clone(),
            }],
        };

        let argv = build_launch_argv(&target).expect("arguments must validate");
        assert_eq!(argv.as_slice().len(), 4);
        assert_eq!(argv.as_slice()[0], "pacman");
        assert_eq!(argv.as_slice()[1], "list_name:item_name:cart");
        assert_eq!(argv.as_slice()[2], "-rompath");
        assert_eq!(PathBuf::from(&argv.as_slice()[3]), path);
    }

    #[test]
    fn rompath_option_accepts_valid_mame_path_list() {
        let rom_a = absolute_test_path("rom-a");
        let rom_b = absolute_test_path("rom-b");
        let path_list = format!("{};{}", rom_a.display(), rom_b.display());
        let target = MameLaunchTarget {
            machine: "pacman".to_owned(),
            software: None,
            bios: None,
            project_paths: vec![ProjectPathArgument {
                option: "rompath".to_owned(),
                path: PathBuf::from(&path_list),
            }],
        };

        let argv = build_launch_argv(&target).expect("rompath path list must validate");
        assert_eq!(argv.as_slice()[0], "pacman");
        assert_eq!(argv.as_slice()[1], "-rompath");
        assert_eq!(argv.as_slice()[2], OsString::from(&path_list));
    }

    #[test]
    fn non_rompath_options_still_reject_path_lists() {
        let rom_a = absolute_test_path("rom-a");
        let rom_b = absolute_test_path("rom-b");
        let path_list = format!("{};{}", rom_a.display(), rom_b.display());
        let target = MameLaunchTarget {
            machine: "pacman".to_owned(),
            software: None,
            bios: None,
            project_paths: vec![ProjectPathArgument {
                option: "snapshot_directory".to_owned(),
                path: PathBuf::from(path_list),
            }],
        };

        let error = build_launch_argv(&target).expect_err("non-rompath list must fail");
        assert_eq!(error.code, "MAME_PROJECT_PATH_LIST_SEPARATOR");
    }

    #[test]
    fn composed_mame_path_list_preserves_order() {
        let rom_a = absolute_test_path("rom-a");
        let rom_b = absolute_test_path("rom-b");
        let composed = compose_mame_path_list([rom_a.as_path(), rom_b.as_path()])
            .expect("compose path list")
            .expect("non-empty path list");
        let expected = format!("{};{}", rom_a.display(), rom_b.display());
        assert_eq!(composed, OsString::from(expected));
    }

    #[test]
    fn omitted_bios_adds_no_bios_arguments() {
        for software in [None, Some("list_name:item_name:cart".to_owned())] {
            let target = MameLaunchTarget {
                machine: "pc".to_owned(),
                software,
                bios: None,
                project_paths: Vec::new(),
            };
            let argv = build_launch_argv(&target).expect("omitted BIOS must validate");
            assert!(
                !argv.as_slice().iter().any(|arg| arg == "-bios"),
                "omitted BIOS must remain omitted from MAME argv: {:?}",
                argv.as_slice()
            );
        }
    }

    #[test]
    fn launch_arguments_include_only_typed_bios_option() {
        let target = MameLaunchTarget {
            machine: "pc".to_owned(),
            software: None,
            bios: Some("rev-3".to_owned()),
            project_paths: Vec::new(),
        };
        let argv = build_launch_argv(&target).expect("BIOS must validate");
        assert_eq!(argv.as_slice(), ["pc", "-bios", "rev-3"]);
    }

    #[test]
    fn launch_preferences_map_to_bounded_mame_options() {
        let target = MameLaunchTarget {
            machine: "pacman".to_owned(),
            software: None,
            bios: None,
            project_paths: Vec::new(),
        };
        let preferences = LaunchPreferencesV1 {
            window_mode: WindowPreference::Fullscreen,
            renderer: RendererPreference::Bgfx,
            audio: AudioPreference::Disabled,
        };

        let argv = build_launch_argv_with_preferences(&target, &preferences)
            .expect("bounded preferences must produce safe argv");
        assert_eq!(
            argv.as_slice(),
            ["pacman", "-nowindow", "-video", "bgfx", "-sound", "none"]
        );
    }

    #[test]
    fn inherited_launch_preferences_add_no_arguments() {
        let target = MameLaunchTarget {
            machine: "pacman".to_owned(),
            software: None,
            bios: None,
            project_paths: Vec::new(),
        };
        let argv = build_launch_argv_with_preferences(&target, &LaunchPreferencesV1::default())
            .expect("inherited preferences must preserve base argv");
        assert_eq!(argv.as_slice(), ["pacman"]);
    }

    #[test]
    fn in_app_gameplay_forces_headless_video_without_disabling_audio() {
        let target = MameLaunchTarget {
            machine: "pacman".to_owned(),
            software: None,
            bios: None,
            project_paths: Vec::new(),
        };
        let preferences = LaunchPreferencesV1 {
            window_mode: WindowPreference::Fullscreen,
            renderer: RendererPreference::Bgfx,
            audio: AudioPreference::Auto,
        };
        let mut argv = build_launch_argv_with_preferences(&target, &preferences)
            .expect("preferences")
            .into_vec();

        force_in_app_gameplay_video(&mut argv);

        assert_eq!(argv, ["pacman", "-sound", "auto", "-video", "none"]);
        assert!(!argv
            .iter()
            .any(|argument| argument == "-window" || argument == "-nowindow"));
    }

    #[test]
    fn automatic_renderer_and_audio_are_explicit_when_requested() {
        let target = MameLaunchTarget {
            machine: "pacman".to_owned(),
            software: None,
            bios: None,
            project_paths: Vec::new(),
        };
        let preferences = LaunchPreferencesV1 {
            window_mode: WindowPreference::Windowed,
            renderer: RendererPreference::Auto,
            audio: AudioPreference::Auto,
        };
        let argv = build_launch_argv_with_preferences(&target, &preferences)
            .expect("automatic providers must produce safe argv");
        assert_eq!(
            argv.as_slice(),
            ["pacman", "-window", "-video", "auto", "-sound", "auto"]
        );
    }

    #[test]
    fn software_list_prefix_may_exceed_short_name_limit() {
        validate_software_list_identifier("apple2_flop_clcracked")
            .expect("known MAME software-list name must validate");
        validate_software_identifier("apple2_flop_clcracked:agentusa")
            .expect("list-qualified software target must validate");
    }

    #[test]
    fn malformed_machine_identifiers_are_rejected() {
        for value in ["", "-help", "PacMan", "pac man", "pac/man", "pac;man"] {
            let error = validate_short_identifier("machine", value)
                .expect_err("malformed machine identifier must fail");
            assert_eq!(error.code, "MAME_IDENTIFIER_INVALID", "{value:?}");
        }
    }

    #[test]
    fn malformed_software_identifiers_are_rejected() {
        for value in [
            "",
            "-help",
            "list:item:part:extra",
            "list::item",
            "item with spaces",
            "item;rm",
        ] {
            let error = validate_software_identifier(value)
                .expect_err("malformed software identifier must fail");
            assert_eq!(error.code, "MAME_IDENTIFIER_INVALID", "{value:?}");
        }
    }

    #[test]
    fn project_paths_require_absolute_non_traversing_paths() {
        let relative = Path::new("roms/test");
        assert_eq!(
            validate_project_controlled_path(relative)
                .expect_err("relative path must fail")
                .code,
            "MAME_PROJECT_PATH_NOT_ABSOLUTE"
        );

        let traversal = absolute_test_path("roms/../escape");
        assert_eq!(
            validate_project_controlled_path(&traversal)
                .expect_err("traversal path must fail")
                .code,
            "MAME_PROJECT_PATH_TRAVERSAL"
        );
    }

    #[test]
    fn project_paths_reject_mame_path_list_injection() {
        let path = absolute_test_path("roms;other-roms");
        assert_eq!(
            validate_project_controlled_path(&path)
                .expect_err("MAME path-list delimiter must fail")
                .code,
            "MAME_PROJECT_PATH_LIST_SEPARATOR"
        );
    }

    #[cfg(unix)]
    #[test]
    fn project_paths_reject_mame_environment_expansion() {
        let path = absolute_test_path("roms/$HOME");
        assert_eq!(
            validate_project_controlled_path(&path)
                .expect_err("MAME environment expansion must fail")
                .code,
            "MAME_PROJECT_PATH_EXPANSION"
        );
    }

    fn absolute_test_path(suffix: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(suffix);
        path
    }
}
