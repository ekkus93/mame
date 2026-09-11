use std::{env, fs, path::PathBuf};

fn main() {
    compose_runtime_control();
    tauri_build::build();
}

fn replace_once(source: String, from: &str, to: &str, label: &str) -> String {
    assert_eq!(
        source.matches(from).count(),
        1,
        "MT-710 composition anchor drifted: {label}"
    );
    source.replacen(from, to, 1)
}

fn compose_runtime_control() {
    println!("cargo:rerun-if-changed=src/sessions/control.rs");
    println!("cargo:rerun-if-changed=src/sessions/control_exit.rs");
    println!("cargo:rerun-if-changed=src/sessions/control_save_state.rs");
    println!("cargo:rerun-if-changed=src/sessions/control_load_state.rs");
    println!("cargo:rerun-if-changed=src/sessions/control_mute.rs");
    println!("cargo:rerun-if-changed=src/sessions/control_query_state.rs");
    println!("cargo:rerun-if-changed=src/sessions/control_pause_resume.lua");
    println!("cargo:rerun-if-changed=src/sessions/control_load_state.lua");
    println!("cargo:rerun-if-changed=src/sessions/control_mute.lua");
    println!("cargo:rerun-if-changed=src/sessions/control_query_state.lua");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR"));
    let lua_extension = fs::read_to_string("src/sessions/control_load_state.lua")
        .expect("read MT-708 load-state Lua extension");
    let mute_extension = fs::read_to_string("src/sessions/control_mute.lua")
        .expect("read MT-709 user-mute Lua extension");
    let query_extension = fs::read_to_string("src/sessions/control_query_state.lua")
        .expect("read MT-710 query-state Lua extension");
    let lua_source = fs::read_to_string("src/sessions/control_pause_resume.lua")
        .expect("read qualified runtime-control Lua source");
    let lua_source = replace_once(
        lua_source,
        "local supported_commands = { pause = true, resume = true, reset = true, exit = true, save_state = true }",
        "local supported_commands = { pause = true, resume = true, reset = true, exit = true, save_state = true, load_state = true }",
        "supported command set",
    );
    let lua_source = replace_once(
        lua_source,
        "    pending_reset = nil,\n    subscriptions = {}",
        "    pending_reset = nil,\n    pending_load = nil,\n    subscriptions = {}",
        "load correlation state",
    );
    let lua_source = replace_once(
        lua_source,
        "            commands = { \"pause\", \"resume\", \"reset\", \"exit\", \"save_state\" },",
        "            commands = { \"pause\", \"resume\", \"reset\", \"exit\", \"save_state\", \"load_state\" },",
        "ready capability list",
    );
    let lua_source = replace_once(
        lua_source,
        "local function has_only_request_fields(request)",
        &format!("{lua_extension}\n\nlocal function has_only_request_fields(request)"),
        "load-state Lua helpers",
    );
    let lua_source = replace_once(
        lua_source,
        "    elseif request.command == \"save_state\" then\n        if not has_only_save_state_params(request.params) then\n            emit_rejected(request_id, \"PROTOCOL_INVALID_PARAMS\", \"Save state requires exactly a bounded UTF-8 path and logical slot.\", {}, false)\n            return\n        end\n    elseif not empty_table(request.params) then",
        "    elseif request.command == \"save_state\" then\n        if not has_only_save_state_params(request.params) then\n            emit_rejected(request_id, \"PROTOCOL_INVALID_PARAMS\", \"Save state requires exactly a bounded UTF-8 path and logical slot.\", {}, false)\n            return\n        end\n    elseif request.command == \"load_state\" then\n        if not has_only_load_state_params(request.params) then\n            emit_rejected(request_id, \"PROTOCOL_INVALID_PARAMS\", \"Load state requires exactly a bounded state path, logical slot, completion path, and completion token.\", {}, false)\n            return\n        end\n    elseif not empty_table(request.params) then",
        "load-state parameter validation",
    );
    let lua_source = replace_once(
        lua_source,
        "    if request.command == \"exit\" then",
        "    if request.command == \"load_state\" then\n        handle_load_state(request, request_id)\n        return\n    end\n\n    if request.command == \"exit\" then",
        "load-state dispatcher",
    );
    let lua_source = replace_once(
        lua_source,
        "local supported_commands = { pause = true, resume = true, reset = true, exit = true, save_state = true, load_state = true }",
        "local supported_commands = { pause = true, resume = true, reset = true, exit = true, save_state = true, load_state = true, set_mute = true }",
        "MT-709 supported command set",
    );
    let lua_source = replace_once(
        lua_source,
        "            commands = { \"pause\", \"resume\", \"reset\", \"exit\", \"save_state\", \"load_state\" },",
        "            commands = { \"pause\", \"resume\", \"reset\", \"exit\", \"save_state\", \"load_state\", \"set_mute\" },",
        "MT-709 ready capability list",
    );
    let lua_source = replace_once(
        lua_source,
        "local function has_only_request_fields(request)",
        &format!("{mute_extension}\n\nlocal function has_only_request_fields(request)"),
        "MT-709 mute Lua helpers",
    );
    let lua_source = replace_once(
        lua_source,
        "    elseif request.command == \"load_state\" then\n        if not has_only_load_state_params(request.params) then\n            emit_rejected(request_id, \"PROTOCOL_INVALID_PARAMS\", \"Load state requires exactly a bounded state path, logical slot, completion path, and completion token.\", {}, false)\n            return\n        end\n    elseif not empty_table(request.params) then",
        "    elseif request.command == \"load_state\" then\n        if not has_only_load_state_params(request.params) then\n            emit_rejected(request_id, \"PROTOCOL_INVALID_PARAMS\", \"Load state requires exactly a bounded state path, logical slot, completion path, and completion token.\", {}, false)\n            return\n        end\n    elseif request.command == \"set_mute\" then\n        if not has_only_set_mute_params(request.params) then\n            emit_rejected(request_id, \"PROTOCOL_INVALID_PARAMS\", \"Set mute requires exactly { muted = boolean }.\", {}, false)\n            return\n        end\n    elseif not empty_table(request.params) then",
        "MT-709 mute parameter validation",
    );
    let lua_source = replace_once(
        lua_source,
        "    if request.command == \"exit\" then",
        "    if request.command == \"set_mute\" then\n        handle_set_mute(request, request_id)\n        return\n    end\n\n    if request.command == \"exit\" then",
        "MT-709 mute dispatcher",
    );
    let lua_source = replace_once(
        lua_source,
        "local supported_commands = { pause = true, resume = true, reset = true, exit = true, save_state = true, load_state = true, set_mute = true }",
        "local supported_commands = { pause = true, resume = true, reset = true, exit = true, save_state = true, load_state = true, set_mute = true, query_state = true }",
        "MT-710 supported command set",
    );
    let lua_source = replace_once(
        lua_source,
        "            commands = { \"pause\", \"resume\", \"reset\", \"exit\", \"save_state\", \"load_state\", \"set_mute\" },",
        "            commands = { \"pause\", \"resume\", \"reset\", \"exit\", \"save_state\", \"load_state\", \"set_mute\", \"query_state\" },",
        "MT-710 ready capability list",
    );
    let lua_source = replace_once(
        lua_source,
        "local function has_only_request_fields(request)",
        &format!("{query_extension}\n\nlocal function has_only_request_fields(request)"),
        "MT-710 query-state Lua helpers",
    );
    let lua_source = replace_once(
        lua_source,
        "    elseif request.command == \"set_mute\" then\n        if not has_only_set_mute_params(request.params) then\n            emit_rejected(request_id, \"PROTOCOL_INVALID_PARAMS\", \"Set mute requires exactly { muted = boolean }.\", {}, false)\n            return\n        end\n    elseif not empty_table(request.params) then",
        "    elseif request.command == \"set_mute\" then\n        if not has_only_set_mute_params(request.params) then\n            emit_rejected(request_id, \"PROTOCOL_INVALID_PARAMS\", \"Set mute requires exactly { muted = boolean }.\", {}, false)\n            return\n        end\n    elseif request.command == \"query_state\" then\n        if not empty_table(request.params) then\n            emit_rejected(request_id, \"PROTOCOL_INVALID_PARAMS\", \"Query state requires an empty parameter object.\", {}, false)\n            return\n        end\n    elseif not empty_table(request.params) then",
        "MT-710 query-state parameter validation",
    );
    let lua_source = replace_once(
        lua_source,
        "    if request.command == \"exit\" then",
        "    if request.command == \"query_state\" then\n        handle_query_state(request_id)\n        return\n    end\n\n    if request.command == \"exit\" then",
        "MT-710 query-state dispatcher",
    );
    fs::write(out_dir.join("runtime_control_mt710.lua"), lua_source)
        .expect("write composed MT-710 runtime-control Lua source");

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
            "include_str!(concat!(env!(\"OUT_DIR\"), \"/runtime_control_mt710.lua\"))",
        );

    let mut composed = source;
    composed.push_str(
        "\n\ninclude!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/sessions/control_exit.rs\"));\n",
    );
    composed.push_str(
        "\ninclude!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/sessions/control_save_state.rs\"));\n",
    );
    composed.push_str(
        "\ninclude!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/sessions/control_load_state.rs\"));\n",
    );
    composed.push_str(
        "\ninclude!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/sessions/control_mute.rs\"));\n",
    );
    composed.push_str(
        "\ninclude!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/sessions/control_query_state.rs\"));\n",
    );

    fs::write(out_dir.join("runtime_control_mt710.rs"), composed)
        .expect("write composed MT-710 runtime-control source");
}
