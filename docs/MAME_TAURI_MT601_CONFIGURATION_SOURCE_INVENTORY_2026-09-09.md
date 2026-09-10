# MT-601 — MAME Configuration Source Inventory

**Date:** 2026-09-09  
**Repository:** `ekkus93/mame`  
**Task:** MT-601 — Inventory relevant MAME configuration sources  
**Parent:** MT-500 closure `4f95bcb997faf400b44b471b862e57a1f01a0010`

## Purpose

This document inventories the MAME configuration sources the Tauri frontend must respect before MT-602 defines application-level configuration precedence. The source of truth is the current MAME implementation, not assumptions made by the frontend.

This task does **not** define the Tauri precedence model yet. It records the configuration layers and the authority MAME already gives them.

## Authoritative MAME sources inspected

- `src/frontend/mame/clifront.cpp`
- `src/frontend/mame/mameopts.cpp`
- `src/frontend/mame/mameopts.h`
- `src/emu/emuopts.h`
- `src/emu/config.cpp`
- `src/osd/modules/lib/osdobj_common.h`
- `src/osd/sdl/sdlopts.cpp`

## 1. Command-line options

MAME parses the process argv through `m_options.parse_command_line(args, OPTION_PRIORITY_CMDLINE)` in `cli_frontend::start_execution`.

`mameopts.h` places command-line options above all standard INI priorities:

```text
OPTION_PRIORITY_SUBCMD = OPTION_PRIORITY_HIGH
OPTION_PRIORITY_CMDLINE

OPTION_PRIORITY_MAME_INI = OPTION_PRIORITY_NORMAL + 1
...
OPTION_PRIORITY_DRIVER_INI
OPTION_PRIORITY_INI
```

Therefore a Tauri launch-time override represented as a real argv option is naturally a high-priority MAME override. It must remain an argv vector; MT-602 must not model command-line options as if they were just another file layer.

Relevant configuration commands exposed by MAME include:

- `-createconfig` / `-cc`
- `-showconfig` / `-sc`
- `-showusage` / `-su`

The frontend can use `-showconfig` later as an evidence source for effective values, but MT-601 does not yet adopt it as the sole explainability mechanism.

## 2. Main INI and standard INI stack

When `readconfig` is enabled, `cli_frontend::start_execution` calls `mame_options::parse_standard_inis`.

MAME first attempts one parse of the main configuration file (`<configname>.ini`, normally `mame.ini`) specifically so that `inipath` and `[no]readconfig` can affect the subsequent search. It then parses the main INI again using the potentially updated INI path.

For a valid system, standard INI loading proceeds in increasing priority:

1. main MAME INI (`mame.ini` in ordinary MAME builds)
2. `debug.ini` when debug mode is enabled
3. `vertical.ini` or `horizont.ini`
4. the first video-output-type INI selected for the machine
5. `source/<sourcefile>.ini`
6. grandparent machine INI, if any
7. parent machine INI, if any
8. machine-specific `<machine>.ini`

The corresponding priority constants are:

```text
OPTION_PRIORITY_MAME_INI
OPTION_PRIORITY_DEBUG_INI
OPTION_PRIORITY_ORIENTATION_INI
OPTION_PRIORITY_SCREEN_INI
OPTION_PRIORITY_SOURCE_INI
OPTION_PRIORITY_GPARENT_INI
OPTION_PRIORITY_PARENT_INI
OPTION_PRIORITY_DRIVER_INI
```

`parse_one_ini` searches through `inipath`; missing INI files are normal and do not by themselves constitute an error. Parse errors are accumulated and surfaced separately.

### Platform INI path defaults

The SDL OSD layer supplies platform-specific defaults for `inipath`:

```text
Windows SDL: .;ini;ini/presets
macOS SDL:   $HOME/Library/Application Support/APP_NAME;$HOME/.APP_NAME;.;ini
other SDL:   $HOME/.APP_NAME;.;ini
```

The non-Windows/non-macOS SDL default is therefore relevant to the current Linux target, while MT-602 must avoid hard-coding it as a universal cross-platform path.

## 3. Machine-specific INI behavior

Machine-specific INI behavior is not a single file lookup. The selected machine influences multiple layers before `<machine>.ini` itself:

- orientation (`vertical.ini` or `horizont.ini`)
- video output type
- source driver file
- grandparent clone relationship
- parent clone relationship
- exact machine short name

The exact machine INI has the highest priority among the standard INIs listed above, but command-line options still have higher priority.

This matters for MT-602: a frontend “machine override” must be clearly distinguished from MAME's existing machine-specific INI layer. The application must not silently claim a value is machine-specific if MAME actually obtained it from source, parent, orientation, or another standard layer.

## 4. Controller and input configuration sources

### Controller profile selection

Core options define:

- `ctrlrpath` — controller configuration search path
- `ctrlr` — selected controller configuration name
- `cfg_directory` — ordinary MAME XML configuration directory
- `input_directory` — input recording/playback directory

During `configuration_manager::load_settings`, MAME performs controller/config XML loading in this order:

1. if `-ctrlr <name>` is set, load `<name>.cfg` from `ctrlrpath` as a controller configuration;
2. load `default.cfg` from `cfg_directory`;
3. load `<machine>.cfg` from `cfg_directory`.

Failure to load an explicitly selected controller configuration is fatal. Missing ordinary `default.cfg` or `<machine>.cfg` is tolerated.

A controller `.cfg` can contain matching sections for:

- `default`
- exact system name
- source file name
- parent/grandparent relationship
- BIOS root

MAME assigns the corresponding configuration level when applying those entries.

### Generic OSD controller mapping

The generic OSD layer also exposes a separate `controller_map` option (`OSDOPTION_CONTROLLER_MAP_FILE`). This is distinct from the core `-ctrlr` controller profile mechanism and must not be conflated with it in the Tauri data model.

The OSD layer also owns `background_input`, while core input options own items such as `mouse`, `joystick`, `lightgun`, joystick map/deadzone/saturation/threshold, and device-selection options.

## 5. Renderer and display options relevant to the frontend

Renderer/display configuration is split between core emulation options, generic OSD options, and platform OSD options.

### Core render/display options

Examples in `emuopts.h` include:

- `keepaspect`
- `unevenstretch`, `unevenstretchx`, `unevenstretchy`
- `autostretchxy`
- integer scaling options
- rotation/flip options
- brightness, contrast, gamma
- artwork options

### Generic OSD video options

`osd_options` exposes frontend-relevant choices including:

- `video`
- `numscreens`
- `window`
- `maximize`
- `waitvsync`
- `syncrefresh`
- `screen` / indexed screen variants
- `aspect`
- `resolution`
- `view`
- `switchres`
- `filter`
- `prescale`
- OpenGL/GLSL options
- BGFX path/backend/debug/screen-chain/shadow-mask/LUT options

The renderer is selected through the OSD module system, and unsupported requested module values can fall back to automatic module selection with a warning. A future UI must therefore distinguish “requested renderer” from “effective renderer” where MAME performs fallback.

### SDL-specific video options

The Linux SDL layer additionally exposes options such as:

- `videodriver`
- `renderdriver`
- `scalemode`
- `attach_window` on X11 builds
- SDL full-screen/monitor-related options

These are platform/backend-specific and should not be presented as universally available settings.

## 6. Audio options relevant to the frontend

Core audio-related options include:

- `samplerate`
- `samples`
- `volume`

The generic OSD layer additionally exposes:

- `sound` — OSD sound module/provider selection
- `audio_latency`

The SDL layer exposes:

- `audiodriver` — SDL low-level audio driver selection

As with video, MT-602 must model platform capability and requested/effective values explicitly rather than assuming a single universal audio setting source.

## 7. Configuration-file preservation behavior

MAME's XML `configuration_manager` preserves unhandled nodes when loading and re-saving ordinary default/system configuration files. This is a useful precedent for MT-603: the Tauri frontend must not overwrite unrelated user configuration merely because it understands only a subset of settings.

The application should prefer narrowly scoped writes and preserve unknown data. Any future direct editing of MAME-owned files must have round-trip tests and recovery/atomic-write behavior.

## 8. Implications for MT-602

MT-602 must define application precedence **around**, not in place of, MAME's existing rules.

At minimum it must distinguish:

```text
application defaults
MAME/global defaults and standard INI stack
application profile defaults
application machine overrides
transient launch overrides (argv)
```

The next task must answer explicitly:

- whether application profile/machine overrides are persisted in app-owned storage or translated into MAME-owned files;
- where those layers sit relative to MAME's source/parent/machine INIs;
- how effective values are calculated and explained;
- how platform-specific OSD options are capability-gated;
- how requested renderer/audio modules are distinguished from MAME fallback/effective values;
- how existing user `mame.ini`, machine INIs, controller `.cfg`, `default.cfg`, and `<machine>.cfg` remain non-destructively respected.

## MT-601 acceptance

The inventory covers all five requested domains:

- command-line options;
- `mame.ini` and platform INI-path behavior;
- machine-specific INI behavior;
- controller mappings/configuration sources;
- renderer/audio options relevant to the frontend.

MT-601 is documentation-only; no runtime behavior is changed by this task.
