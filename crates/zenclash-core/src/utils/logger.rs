use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub log_file: Option<std::path::PathBuf>,
    pub level: String,
    pub with_ansi: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            log_file: Some(super::dirs::core_log_path()),
            level: "info".to_string(),
            with_ansi: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "warn" | "warning" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoggerError {
    #[error("Failed to create log directory: {0}")]
    DirectoryError(#[from] std::io::Error),

    #[error("Failed to create log file: {0}")]
    FileError(String),

    #[error("Logger already initialized")]
    AlreadyInitialized,
}

static LOGGER_INITIALIZED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

pub fn setup_logger(config: LoggerConfig) -> Result<(), LoggerError> {
    if LOGGER_INITIALIZED.get().is_some() {
        return Err(LoggerError::AlreadyInitialized);
    }

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    let fmt_layer = fmt::layer()
        .with_ansi(config.with_ansi)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true);

    if let Some(log_path) = &config.log_file {
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|e| LoggerError::FileError(e.to_string()))?;

        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_target(true)
            .with_line_number(true)
            .with_writer(std::sync::Arc::new(file));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(file_layer)
            .try_init()
            .map_err(|_| LoggerError::AlreadyInitialized)?;
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|_| LoggerError::AlreadyInitialized)?;
    }

    let _ = LOGGER_INITIALIZED.set(true);
    Ok(())
}

pub fn setup_default_logger() -> Result<(), LoggerError> {
    setup_logger(LoggerConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from("trace"), LogLevel::Trace);
        assert_eq!(LogLevel::from("debug"), LogLevel::Debug);
        assert_eq!(LogLevel::from("info"), LogLevel::Info);
        assert_eq!(LogLevel::from("warn"), LogLevel::Warn);
        assert_eq!(LogLevel::from("warning"), LogLevel::Warn);
        assert_eq!(LogLevel::from("error"), LogLevel::Error);
        assert_eq!(LogLevel::from("unknown"), LogLevel::Info);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Trace), "trace");
        assert_eq!(format!("{}", LogLevel::Debug), "debug");
        assert_eq!(format!("{}", LogLevel::Info), "info");
        assert_eq!(format!("{}", LogLevel::Warn), "warn");
        assert_eq!(format!("{}", LogLevel::Error), "error");
    }

    #[test]
    fn test_logger_config_default() {
        let config = LoggerConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.with_ansi);
        assert!(config.log_file.is_some());
    }
}
