use std::env;

#[derive(Debug)]
pub struct Config {
    pub bot_token: String,
    pub database_url: String,
    pub command_prefix: String,
}

impl Config {
    /// Loads configuration from environment variables
    ///
    /// # Returns
    /// - `Ok(Config)` if all required environment variables are present
    /// - `Err(ConfigError)` if any required environment variables are missing
    ///
    /// # Environment Variables
    /// - `BOT_TOKEN`: Required, bot authentication token
    /// - `DATABASE_URL`: Required, database connection string
    /// - `COMMAND_PREFIX`: Optional, defaults to "c:"
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