#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

const VERIFY_BUNDLED_RUNTIME_ARG: &str = "--mame-tauri-verify-bundled-runtime";
const VERIFY_METADATA_BOOTSTRAP_ARG: &str = "--mame-tauri-verify-bundled-metadata-bootstrap";
const VERIFY_RESET_CONTENT_POLICY_ARG: &str = "--mame-tauri-verify-reset-content-policy";

fn main() {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if let Some(command) = args.first() {
        if command == OsStr::new(VERIFY_BUNDLED_RUNTIME_ARG) {
            verify_bundled_runtime(&args[1..]);
            return;
        }
        if command == OsStr::new(VERIFY_METADATA_BOOTSTRAP_ARG) {
            verify_metadata_bootstrap(&args[1..]);
            return;
        }
        if command == OsStr::new(VERIFY_RESET_CONTENT_POLICY_ARG) {
            verify_reset_content_policy(&args[1..]);
            return;
        }
    }

    if let Err(error) = mame_tauri_lib::run() {
        eprintln!("fatal Tauri runtime error: {error}");
        std::process::exit(1);
    }
}

fn verify_bundled_runtime(args: &[OsString]) {
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

fn verify_metadata_bootstrap(args: &[OsString]) {
    if !(2..=3).contains(&args.len()) {
        eprintln!(
            "usage: mame-tauri {} <tauri-resource-dir> <catalog-path> [probe-machine]",
            VERIFY_METADATA_BOOTSTRAP_ARG
        );
        std::process::exit(64);
    }

    let resource_dir = PathBuf::from(&args[0]);
    let catalog_path = PathBuf::from(&args[1]);
    let probe_machine = match args.get(2) {
        Some(value) => match value.to_str() {
            Some(value) => Some(value),
            None => {
                eprintln!("probe-machine must be valid UTF-8");
                std::process::exit(64);
            }
        },
        None => None,
    };

    match mame_tauri_lib::metadata::verify_bundled_metadata_bootstrap(
        &resource_dir,
        &catalog_path,
        probe_machine,
    ) {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .expect("metadata bootstrap report must serialize")
            );
        }
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&error)
                    .expect("metadata bootstrap error must serialize")
            );
            std::process::exit(1);
        }
    }
}

fn verify_reset_content_policy(args: &[OsString]) {
    if args.len() != 3 {
        eprintln!(
            "usage: mame-tauri {} <tauri-resource-dir> <empty-content-dir> <probe-machine>",
            VERIFY_RESET_CONTENT_POLICY_ARG
        );
        std::process::exit(64);
    }

    let resource_dir = PathBuf::from(&args[0]);
    let content_dir = PathBuf::from(&args[1]);
    let probe_machine = match args[2].to_str() {
        Some(value) => value,
        None => {
            eprintln!("probe-machine must be valid UTF-8");
            std::process::exit(64);
        }
    };

    match mame_tauri_lib::verify_reset_content_policy(&resource_dir, &content_dir, probe_machine) {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .expect("reset content-policy report must serialize")
            );
        }
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&error)
                    .expect("reset content-policy error must serialize")
            );
            std::process::exit(1);
        }
    }
}
