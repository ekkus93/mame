#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::OsStr, path::PathBuf};

const VERIFY_BUNDLED_RUNTIME_ARG: &str = "--mame-tauri-verify-bundled-runtime";

fn main() {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args
        .first()
        .is_some_and(|arg| arg == OsStr::new(VERIFY_BUNDLED_RUNTIME_ARG))
    {
        verify_bundled_runtime(&args[1..]);
        return;
    }

    if let Err(error) = mame_tauri_lib::run() {
        eprintln!("fatal Tauri runtime error: {error}");
        std::process::exit(1);
    }
}

fn verify_bundled_runtime(args: &[std::ffi::OsString]) {
    if args.len() != 1 {
        eprintln!(
            "usage: mame-tauri {} <tauri-resource-dir>",
            VERIFY_BUNDLED_RUNTIME_ARG
        );
        std::process::exit(64);
    }

    let resource_dir = PathBuf::from(&args[0]);
    match mame_tauri_lib::verify_bundled_runtime_resource_dir(&resource_dir) {
        Ok(identity) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&identity)
                    .expect("MAME runtime identity must serialize")
            );
        }
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&error).expect("MAME runtime error must serialize")
            );
            std::process::exit(1);
        }
    }
}
