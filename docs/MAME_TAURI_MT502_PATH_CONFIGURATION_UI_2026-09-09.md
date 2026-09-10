# MT-502 — Path configuration UI

## Scope

MT-502 exposes the MT-501 ROM, software, and CHD path model in the desktop UI. It owns native directory selection, ordered add/remove/reorder controls, live path accessibility feedback, and durable persistence of edits. MAME audit semantics remain deferred to MT-503 and later tasks.

## Architecture

The frontend uses three dedicated Tauri commands:

- `get_content_path_configuration` loads the persisted MT-501 model and returns validation for each configured path.
- `set_content_path_configuration` writes the complete ordered path model while preserving unrelated settings such as `mameExecutable`, synchronizes the settings file to storage, and returns fresh validation.
- `pick_content_directory` invokes the native desktop folder picker from Rust and returns a platform-safe path. Android and iOS return an explicit unsupported error rather than pretending folder selection succeeded.

The official Tauri dialog plugin is initialized in Rust. The webview is not granted direct dialog-plugin capability; the native picker stays behind the dedicated application command.

## Path fidelity

Existing `PlatformPath` wire values remain either UTF-8 strings or the encoded MT-501 representation. Frontend reorder and removal operations preserve the original value instead of coercing it through JavaScript string conversion. Encoded paths are rendered explicitly with their encoding and payload so they remain visible without lossy display conversion.

## Validation behavior

Each rendered path shows one of the MT-501 states:

- accessible
- missing
- not a directory
- permission denied
- unreadable

Invalid or unavailable paths remain persisted and visible. MT-502 does not silently delete, normalize, or replace them.

## Persistence behavior

Every successful add, remove, or reorder operation submits the complete ordered path model to Rust. Rust reloads the current settings file, replaces only `contentPaths`, writes formatted schema-v2 JSON, and calls `sync_all` before reporting success. Errors are returned through the existing structured `AppError` envelope.

## Test coverage

Rust tests cover persistence order, preservation of the existing MAME executable setting, and returned validation for an unavailable path. Frontend tests cover typed command envelopes, preservation of encoded platform paths, duplicate detection, ordering, movement, removal, and explicit encoded-path display.

## Deferred work

MT-503 determines the authoritative MAME audit command and exit/output semantics. MT-504 and later tasks parse, persist, surface, and automatically refresh audit state. MT-502 does not infer machine availability from file presence.

## Qualification evidence

The normalized implementation commit is `10ebff53f8b2d81b705b490a0663d8be90d113af`, with the closed MT-501 head `16582976a21ce3c2d7b137d2859f3652e565dfe4` as its single parent.

GitHub Actions Tauri run `34421486030` completed successfully against that exact normalized implementation SHA. The successful qualification covered frontend formatting, ESLint, TypeScript typechecking, frontend tests, production build, Rust formatting, the full Rust test suite, the inherited 100,000-row MT-410 performance qualification, Clippy with warnings denied, and lockfile integrity.

GitHub Actions documentation run `34421486034` also completed successfully against the same normalized implementation SHA, including HTML and PDF documentation builds.

MT-502 is therefore closed. MT-503 is the next active task.
