use std::path::PathBuf;

use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: Level,
    pub format: LogFormat,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum LogFormat {
    #[default]
    Pretty,
    Json,
}

impl LoggingConfig {
    pub fn from_env() -> Self {
        let level = Self::parse_level(&std::env::var("LOG_LEVEL").unwrap_or_default());
        let format = Self::parse_format(&std::env::var("LOG_FORMAT").unwrap_or_default());
        let path = std::env::var("LOG_PATH").ok().map(PathBuf::from);
        Self { level, format, path }
    }

    pub fn apply_cli_overrides(&mut self, level: Option<String>, format: Option<String>, path: Option<PathBuf>) {
        if let Some(l) = level {
            self.level = Self::parse_level(&l);
        }
        if let Some(f) = format {
            self.format = Self::parse_format(&f);
        }
        if let Some(p) = path {
            self.path = Some(p);
        }
    }

    pub fn init(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filter = EnvFilter::builder()
            .with_default_directive(self.level.into())
            .from_env_lossy();

        match (self.format, self.path) {
            (LogFormat::Pretty, None) => {
                fmt().with_env_filter(filter).with_writer(std::io::stderr).try_init()?;
            }
            (LogFormat::Json, None) => {
                fmt()
                    .json()
                    .with_env_filter(filter)
                    .with_writer(std::io::stderr)
                    .try_init()?;
            }
            (LogFormat::Pretty, Some(path)) => {
                let file = std::fs::File::create(&path)?;
                fmt().with_env_filter(filter).with_writer(file).try_init()?;
            }
            (LogFormat::Json, Some(path)) => {
                let file = std::fs::File::create(&path)?;
                fmt().json().with_env_filter(filter).with_writer(file).try_init()?;
            }
        }
        Ok(())
    }

    fn parse_level(s: &str) -> Level {
        match s.to_uppercase().as_str() {
            "OFF" => Level::ERROR,
            "ERROR" => Level::ERROR,
            "WARN" => Level::WARN,
            "INFO" => Level::INFO,
            "DEBUG" => Level::DEBUG,
            "TRACE" => Level::TRACE,
            _ => Level::WARN,
        }
    }

    fn parse_format(s: &str) -> LogFormat {
        match s.to_lowercase().as_str() {
            "json" => LogFormat::Json,
            _ => LogFormat::Pretty,
        }
    }
}
