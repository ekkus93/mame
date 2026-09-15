//! Bounded MAME-style displayed-list export.
//!
//! The WebView supplies only the current typed browser filter/search state.
//! Destination selection is performed by the native dialog plugin and all
//! filesystem/database work remains Rust-authoritative.

use std::{io::Write, path::Path};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_dialog::DialogExt;
use tempfile::Builder;

use crate::{
    config::settings_path,
    errors::{AppError, AppResult},
    mame_ui::{validated_query, MameUiMachineFilterRequest, MameUiMachineSearchRequest},
    metadata::{CatalogRepository, MachineAvailability},
    storage,
};

const EXPORT_PAGE_SIZE: u32 = 200;
const MAX_EXPORT_ROWS: u64 = 100_000;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportMameUiDisplayedListRequest {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub filter: MameUiMachineFilterRequest,
    #[serde(default)]
    pub filter_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportMameUiDisplayedListResult {
    pub schema_version: u32,
    pub canceled: bool,
    pub rows: u64,
    pub path: Option<String>,
}

#[tauri::command]
pub async fn export_mame_ui_displayed_list(
    request: ExportMameUiDisplayedListRequest,
    app: AppHandle,
) -> AppResult<ExportMameUiDisplayedListResult> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let selected = app
            .dialog()
            .file()
            .set_title("Export displayed MAME machine list")
            .add_filter("CSV", &["csv"])
            .set_file_name("mame-displayed-list.csv")
            .blocking_save_file();

        let Some(selected) = selected else {
            return Ok(ExportMameUiDisplayedListResult {
                schema_version: 1,
                canceled: true,
                rows: 0,
                path: None,
            });
        };
        let destination = selected.into_path().map_err(|error| {
            AppError::new(
                "MAME_UI_EXPORT_DESTINATION_INVALID",
                "The selected export destination could not be represented as a local path.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        let catalog_path = storage::catalog_path(&app)?;
        let settings_path = settings_path(&app)?;

        return tauri::async_runtime::spawn_blocking(move || {
            export_displayed_list_to_path(request, &catalog_path, &settings_path, &destination)
        })
        .await
        .map_err(|error| {
            AppError::new(
                "MAME_UI_EXPORT_WORKER_FAILED",
                "The displayed-list export worker did not complete normally.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (request, app);
        Err(AppError::new(
            "MAME_UI_EXPORT_UNAVAILABLE",
            "Displayed-list export is not supported on this platform.",
        ))
    }
}

fn export_displayed_list_to_path(
    request: ExportMameUiDisplayedListRequest,
    catalog_path: &Path,
    settings_path: &Path,
    destination: &Path,
) -> AppResult<ExportMameUiDisplayedListResult> {
    let mut query = validated_query(export_search_request(request), catalog_path, settings_path)?;
    let repository = CatalogRepository::open(catalog_path)?;
    let parent = destination.parent().ok_or_else(|| {
        AppError::new(
            "MAME_UI_EXPORT_DESTINATION_INVALID",
            "The selected export destination has no parent directory.",
        )
    })?;
    if !parent.is_dir() {
        return Err(AppError::new(
            "MAME_UI_EXPORT_DESTINATION_INVALID",
            "The selected export destination directory does not exist.",
        ));
    }

    let mut temp = Builder::new()
        .prefix(".mame-tauri-list-export-")
        .tempfile_in(parent)
        .map_err(export_write_error)?;
    write_csv_row(
        &mut temp,
        &[
            "Description",
            "Short name",
            "Year",
            "Manufacturer",
            "Driver status",
            "Availability",
            "Parent",
            "Source file",
        ],
    )?;

    let mut rows = 0_u64;
    loop {
        query.offset = u32::try_from(rows).map_err(|_| {
            AppError::new(
                "MAME_UI_EXPORT_RESULT_LIMIT_EXCEEDED",
                "The displayed-list export exceeds the supported row range.",
            )
        })?;
        let page = repository.query_mame_ui_machines(&query)?;
        if page.total > MAX_EXPORT_ROWS {
            return Err(AppError::new(
                "MAME_UI_EXPORT_RESULT_LIMIT_EXCEEDED",
                format!("Displayed-list export is limited to {MAX_EXPORT_ROWS} machines."),
            )
            .with_details(serde_json::json!({ "rows": page.total, "maxRows": MAX_EXPORT_ROWS })));
        }
        if page.items.is_empty() {
            break;
        }

        for item in &page.items {
            let availability = page
                .availability_by_short_name
                .get(&item.short_name)
                .copied()
                .unwrap_or(MachineAvailability::Unknown);
            write_csv_row(
                &mut temp,
                &[
                    &item.description,
                    &item.short_name,
                    item.year.as_deref().unwrap_or(""),
                    item.manufacturer.as_deref().unwrap_or(""),
                    item.driver_status.as_deref().unwrap_or(""),
                    availability_token(availability),
                    item.clone_of.as_deref().unwrap_or(""),
                    item.source_file.as_deref().unwrap_or(""),
                ],
            )?;
        }
        rows = rows.checked_add(page.items.len() as u64).ok_or_else(|| {
            AppError::new(
                "MAME_UI_EXPORT_RESULT_LIMIT_EXCEEDED",
                "The displayed-list export row count overflowed.",
            )
        })?;
        if rows >= page.total {
            break;
        }
    }

    temp.flush().map_err(export_write_error)?;
    temp.as_file().sync_all().map_err(export_write_error)?;
    let persisted = temp
        .persist(destination)
        .map_err(|error| export_write_error(error.error))?;
    persisted.sync_all().map_err(export_write_error)?;

    #[cfg(unix)]
    std::fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(export_write_error)?;

    Ok(ExportMameUiDisplayedListResult {
        schema_version: 1,
        canceled: false,
        rows,
        path: Some(destination.to_string_lossy().into_owned()),
    })
}

fn export_search_request(request: ExportMameUiDisplayedListRequest) -> MameUiMachineSearchRequest {
    MameUiMachineSearchRequest {
        text: request.text,
        filter: request.filter,
        filter_value: request.filter_value,
        preferred_machine: None,
        limit: EXPORT_PAGE_SIZE,
        offset: 0,
    }
}

fn write_csv_row(writer: &mut impl Write, values: &[&str]) -> AppResult<()> {
    let encoded = values
        .iter()
        .map(|value| csv_field(value))
        .collect::<Vec<_>>();
    writeln!(writer, "{}", encoded.join(",")).map_err(export_write_error)
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn availability_token(value: MachineAvailability) -> &'static str {
    match value {
        MachineAvailability::Available => "available",
        MachineAvailability::Missing => "missing",
        MachineAvailability::Unknown => "unknown",
    }
}

fn export_write_error(error: std::io::Error) -> AppError {
    AppError::new(
        "MAME_UI_EXPORT_WRITE_FAILED",
        "The displayed machine list could not be written.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use super::{
        csv_field, export_search_request, ExportMameUiDisplayedListRequest, EXPORT_PAGE_SIZE,
    };
    use crate::mame_ui::MameUiMachineFilterRequest;

    #[test]
    fn csv_encoding_quotes_delimiters_quotes_and_newlines() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("a\"b"), "\"a\"\"b\"");
        assert_eq!(csv_field("a\nb"), "\"a\nb\"");
    }

    #[test]
    fn export_reuses_browser_query_semantics_without_paging_or_preferred_selection() {
        let request = export_search_request(ExportMameUiDisplayedListRequest {
            text: Some("galax".to_owned()),
            filter: MameUiMachineFilterRequest::Manufacturer,
            filter_value: Some("Namco".to_owned()),
        });
        assert_eq!(request.text.as_deref(), Some("galax"));
        assert_eq!(request.filter, MameUiMachineFilterRequest::Manufacturer);
        assert_eq!(request.filter_value.as_deref(), Some("Namco"));
        assert_eq!(request.preferred_machine, None);
        assert_eq!(request.limit, EXPORT_PAGE_SIZE);
        assert_eq!(request.offset, 0);
    }
}
