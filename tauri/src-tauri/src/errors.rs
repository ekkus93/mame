use std::fmt;

use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub details: serde_json::Value,
    pub retryable: bool,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: serde_json::Value::Object(serde_json::Map::new()),
            retryable: false,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn serializes_stable_error_envelope() {
        let value = serde_json::to_value(AppError::new("TEST_ERROR", "test failure"))
            .expect("AppError must be serializable");

        assert_eq!(value["code"], "TEST_ERROR");
        assert_eq!(value["message"], "test failure");
        assert_eq!(value["retryable"], false);
        assert!(value.get("details").is_some());
    }
}
