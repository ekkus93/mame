use std::{env, fs, path::PathBuf};

fn main() {
    compose_runtime_control();
    tauri_build::build();
}

fn compose_runtime_control() {
    println!("cargo:rerun-if-changed=src/sessions/control.rs");
    println!("cargo:rerun-if-changed=src/sessions/control_exit.rs");
    println!("cargo:rerun-if-changed=src/sessions/control_save_state.rs");
    println!("cargo:rerun-if-changed=src/sessions/control_pause_resume.lua");

    let source = fs::read_to_string("src/sessions/control.rs")
        .expect("read qualified runtime-control source");
    let source = source
        .lines()
        .map(|line| {
            line.strip_prefix("//!")
                .map_or_else(|| line.to_owned(), |rest| format!("//{rest}"))
        })
        .collect::<Vec<_>>()
        .join("\n")
        .replace(
            "include_str!(\"control_pause_resume.lua\")",
            "include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/sessions/control_pause_resume.lua\"))",
        );

    let mut composed = source;
    composed.push_str(
        "\n\ninclude!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/sessions/control_exit.rs\"));\n",
    );
    composed.push_str(
        "\ninclude!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/sessions/control_save_state.rs\"));\n",
    );

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR"));
    fs::write(out_dir.join("runtime_control_mt707.rs"), composed)
        .expect("write composed MT-707 runtime-control source");
}
