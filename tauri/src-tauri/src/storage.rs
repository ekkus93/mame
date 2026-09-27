//! SQLite and project-owned filesystem persistence primitives.
//!
//! This module remains below library/session domain logic and does not expose a
//! generic arbitrary-filesystem API to the WebView.

mod migrations;

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use rusqlite::Connection;
use tauri::{Manager, Runtime};

use crate::errors::{AppError, AppResult};

pub use migrations::CATALOG_SCHEMA_VERSION;

const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MameUserDirectories {
    pub root: PathBuf,
    pub cfg: PathBuf,
    pub nvram: PathBuf,
    pub state: PathBuf,
    pub snapshot: PathBuf,
    pub diff: PathBuf,
}

pub fn mame_user_directories<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> AppResult<MameUserDirectories> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| {
            AppError::new(
                "MAME_USER_DATA_ROOT_UNAVAILABLE",
                "The platform application data directory for MAME state is unavailable.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?
        .join("mame");

    let directories = MameUserDirectories {
        cfg: root.join("cfg"),
        nvram: root.join("nvram"),
        state: root.join("state"),
        snapshot: root.join("snap"),
        diff: root.join("diff"),
        root,
    };
    for path in [
        &directories.root,
        &directories.cfg,
        &directories.nvram,
        &directories.state,
        &directories.snapshot,
        &directories.diff,
    ] {
        fs::create_dir_all(path).map_err(|error| {
            AppError::new(
                "MAME_USER_DATA_DIRECTORY_CREATE_FAILED",
                "A user-writable MAME data directory could not be created.",
            )
            .with_details(serde_json::json!({
                "path": path,
                "cause": error.to_string()
            }))
        })?;
    }
    Ok(directories)
}

pub fn catalog_path<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|root| root.join("catalog.sqlite3"))
        .map_err(|error| {
            AppError::new(
                "CATALOG_ROOT_UNAVAILABLE",
                "The platform application data directory is unavailable.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })
}

pub(crate) fn open_catalog_connection(path: &Path) -> AppResult<Connection> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::new(
                "CATALOG_DIRECTORY_CREATE_FAILED",
                "The catalog database directory could not be created.",
            )
            .with_details(serde_json::json!({
                "path": parent,
                "cause": error.to_string()
            }))
        })?;
    }

    let mut connection = Connection::open(path).map_err(|error| {
        AppError::new(
            "CATALOG_DATABASE_OPEN_FAILED",
            "The catalog database could not be opened.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "cause": error.to_string()
        }))
    })?;
    configure_connection(&mut connection)?;
    Ok(connection)
}

#[cfg(test)]
pub(crate) fn open_catalog_memory() -> AppResult<Connection> {
    let mut connection = Connection::open_in_memory().map_err(|error| {
        AppError::new(
            "CATALOG_DATABASE_OPEN_FAILED",
            "The in-memory catalog database could not be opened.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    configure_connection(&mut connection)?;
    Ok(connection)
}

fn configure_connection(connection: &mut Connection) -> AppResult<()> {
    connection
        .busy_timeout(SQLITE_BUSY_TIMEOUT)
        .map_err(|error| {
            AppError::new(
                "CATALOG_DATABASE_CONFIG_FAILED",
                "The catalog database busy timeout could not be configured.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
    migrations::migrate(connection)
}
