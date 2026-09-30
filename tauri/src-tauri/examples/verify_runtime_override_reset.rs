use std::path::PathBuf;

use mame_tauri_lib::{
    config::SettingsV2,
    effective_runtime::resolve_effective_mame_source,
    mame::{inspect_executable, MameExecutableSourceKind, MameExecutableTrust},
};

fn main() {
    let mut args = std::env::args_os().skip(1);
    let resource_dir = PathBuf::from(args.next().expect("missing resource directory"));
    let external_path = PathBuf::from(args.next().expect("missing external MAME path"));
    assert!(args.next().is_none(), "unexpected extra argument");

    let override_settings = SettingsV2 {
        mame_executable: Some(external_path.to_string_lossy().into_owned()),
        ..SettingsV2::default()
    };
    let override_source = resolve_effective_mame_source(&override_settings, &resource_dir)
        .expect("external override must resolve");
    let override_identity = inspect_executable(override_source).expect("external MAME must validate");
    assert_eq!(
        override_identity.source,
        MameExecutableSourceKind::External
    );
    assert_eq!(
        override_identity.trust,
        MameExecutableTrust::UserConfigured
    );

    let reset_source = resolve_effective_mame_source(&SettingsV2::default(), &resource_dir)
        .expect("reset must restore bundled runtime");
    let reset_identity = inspect_executable(reset_source).expect("bundled MAME must validate");
    assert_eq!(reset_identity.source, MameExecutableSourceKind::Bundled);
    assert_eq!(
        reset_identity.trust,
        MameExecutableTrust::QualifiedBundled
    );

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "overrideIdentity": override_identity,
            "resetIdentity": reset_identity,
        }))
        .expect("override/reset report must serialize")
    );
}
