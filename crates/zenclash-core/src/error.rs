use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZenClashError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] crate::utils::HttpError),

    #[error("API error: {0}")]
    Api(#[from] crate::core::ApiError),

    #[error("Process error: {0}")]
    Process(#[from] crate::core::ProcessError),

    #[error("Core manager error: {0}")]
    CoreManager(#[from] crate::core::CoreManagerError),

    #[error("Selection error: {0}")]
    Selection(#[from] crate::proxy::SelectionError),

    #[error("Delay test error: {0}")]
    DelayTest(#[from] crate::proxy::DelayTestError),

    #[error("Override error: {0}")]
    Override(#[from] crate::config::OverrideError),

    #[error("Logger error: {0}")]
    Logger(#[from] crate::utils::LoggerError),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Subscription error: {0}")]
    Subscription(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, ZenClashError>;

impl From<url::ParseError> for ZenClashError {
    fn from(e: url::ParseError) -> Self {
        ZenClashError::Network(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for ZenClashError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        ZenClashError::Unknown(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ZenClashError::Config("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }
}
