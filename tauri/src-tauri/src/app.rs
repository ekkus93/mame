use serde::{Deserialize, Serialize};
use tauri::{Emitter, Runtime};

use crate::errors::{AppError, AppResult};

pub const APP_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoRequest {
    pub protocol_version: u32,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoResponse {
    pub protocol_version: u32,
    pub app_version: &'static str,
    pub backend: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppReadyEventV1 {
    pub schema_version: u32,
    pub app_version: &'static str,
}

pub fn build_app_info(request: AppInfoRequest) -> AppResult<AppInfoResponse> {
    if request.protocol_version != APP_PROTOCOL_VERSION {
        return Err(AppError::new(
            "UNSUPPORTED_PROTOCOL_VERSION",
            format!(
                "Frontend protocol version {} is not supported by backend version {}.",
                request.protocol_version, APP_PROTOCOL_VERSION
            ),
        )
        .with_details(serde_json::json!({
            "requestedVersion": request.protocol_version,
            "supportedVersion": APP_PROTOCOL_VERSION
        })));
    }

    Ok(AppInfoResponse {
        protocol_version: APP_PROTOCOL_VERSION,
        app_version: env!("CARGO_PKG_VERSION"),
        backend: "rust-tauri",
    })
}

#[tauri::command]
pub fn get_app_info(request: AppInfoRequest) -> AppResult<AppInfoResponse> {
    build_app_info(request)
}

pub fn emit_ready<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<()> {
    let event = AppReadyEventV1 {
        schema_version: 1,
        app_version: env!("CARGO_PKG_VERSION"),
    };

    app.emit("app.ready", event).map_err(|error| {
        AppError::new(
            "APP_READY_EVENT_FAILED",
            "The backend could not emit the application-ready event.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

#[cfg(test)]
mod tests {
    use super::{build_app_info, AppInfoRequest, APP_PROTOCOL_VERSION};

    #[test]
    fn accepts_current_protocol() {
        let result = build_app_info(AppInfoRequest {
            protocol_version: APP_PROTOCOL_VERSION,
        })
        .expect("current protocol must be accepted");

        assert_eq!(result.protocol_version, APP_PROTOCOL_VERSION);
        assert_eq!(result.backend, "rust-tauri");
    }

    #[test]
    fn rejects_unknown_protocol() {
        let error = build_app_info(AppInfoRequest {
            protocol_version: APP_PROTOCOL_VERSION + 1,
        })
        .expect_err("unknown protocol must be rejected");

        assert_eq!(error.code, "UNSUPPORTED_PROTOCOL_VERSION");
    }
}
