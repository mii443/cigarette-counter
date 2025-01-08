use std::env;

#[derive(Debug)]
pub struct Config {
    pub bot_token: String,
    pub database_url: String,
    pub command_prefix: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            bot_token: env::var("BOT_TOKEN").map_err(|_| ConfigError::MissingBotToken)?,
            database_url: env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?,
            command_prefix: env::var("COMMAND_PREFIX").unwrap_or_else(|_| "c:".to_string()),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing BOT_TOKEN environment variable")]
    MissingBotToken,
    #[error("Missing DATABASE_URL environment variable")]
    MissingDatabaseUrl,
}