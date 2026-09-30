use std::path::PathBuf;

use mame_tauri_lib::config::SettingsV2;
use mame_tauri_lib::effective_runtime::resolve_effective_mame_source;
use mame_tauri_lib::mame::{inspect_executable, MameExecutableSourceKind, MameExecutableTrust};

fn main() {
    let mut args = std::env::args_os().skip(1);
    let resource_dir = PathBuf::from(args.next().expect("missing resource directory"));
    let external_path = PathBuf::from(args.next().expect("missing external MAME path"));
    assert!(args.next().is_none(), "unexpected extra argument");

    let settings = SettingsV2 {
        mame_executable: Some(external_path.to_string_lossy().into_owned()),
        ..SettingsV2::default()
    };
    let source = resolve_effective_mame_source(&settings, &resource_dir).expect("override resolution");
    let identity = inspect_executable(source).expect("external MAME validation");
    assert_eq!(identity.source, MameExecutableSourceKind::External);
    assert_eq!(identity.trust, MameExecutableTrust::UserConfigured);
    assert_eq!(identity.path, external_path);

    let source = resolve_effective_mame_source(&SettingsV2::default(), &resource_dir)
        .expect("bundled reset resolution");
    let identity = inspect_executable(source).expect("bundled MAME validation");
    assert_eq!(identity.source, MameExecutableSourceKind::Bundled);
    assert_eq!(identity.trust, MameExecutableTrust::QualifiedBundled);

    println!("installed runtime override/reset qualification passed");
}
