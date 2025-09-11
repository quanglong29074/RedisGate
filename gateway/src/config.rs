// config.rs
use serde::Deserialize;

/// Server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
}

/// Redis configuration
#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
    pub pool_max_size: usize,
    pub pool_timeout_seconds: u64,
    pub default_password: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String, // ví dụ: "info", "debug", "warn"
}

/// Global configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub logging: LoggingConfig
}

impl Config {
    /// Load config from `config/default.toml` and environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?;

        let cfg: Config = settings.try_deserialize()?;
        Ok(cfg)
    }
}
