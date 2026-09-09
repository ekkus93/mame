//! MAME executable identity, validation, argument construction, and invocation.
//!
//! The Rust backend owns this boundary so frontend input never becomes a shell
//! command string. Process supervision itself is implemented in `sessions`.

mod argv;
mod executable;
mod software;

pub use argv::{
    build_launch_argv, validate_project_controlled_path, validate_short_identifier,
    validate_software_identifier, validate_software_list_identifier, MameArgv, MameLaunchTarget,
    ProjectPathArgument,
};
pub use executable::{
    configured_external_source, inspect_executable, validate_executable_path,
    MameExecutableIdentity, MameExecutableSource, MameExecutableSourceKind, MameExecutableTrust,
};
pub(crate) use software::get_software_list_xml;
