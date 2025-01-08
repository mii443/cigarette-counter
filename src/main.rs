//! Main entry point for the Cigarette Counter Discord bot.
//!
//! This module handles the initialization and setup of the bot, including:
//! - Configuration loading
//! - Database connection
//! - Command framework setup
//! - Discord client creation

mod commands;
mod config;
mod database;

use std::sync::Arc;

use config::{Config, ConfigError};
use commands::create_cigarette_ui;
use database::Database;
use poise::{
    serenity_prelude::{self as serenity, futures::lock::Mutex},
    PrefixFrameworkOptions,
};
use sqlx::PgPool;
use tracing::{error, info};

/// Shared application state containing the database connection
pub struct Data {
    /// Thread-safe, async database connection wrapped in Arc<Mutex>
    pub database: Arc<Mutex<Database>>,
}

/// Type alias for boxed errors that can be sent between threads
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Type alias for command context containing application state
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Main error type for the bot application
#[derive(Debug, thiserror::Error)]
pub enum BotError {
    /// Error occurred while loading or parsing configuration
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    /// Error occurred during database operations
    #[error("Database connection error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Error occurred in the Discord client
    #[error("Client error: {0}")]
    Client(#[from] serenity::Error),
}

/// Sets up the command framework with bot configuration and commands
///
/// # Arguments
/// * `config` - Loaded bot configuration
/// * `db` - Database connection to be shared across commands
///
/// # Returns
/// Configured Poise framework instance
async fn setup_framework(config: &Config, db: Database) -> poise::Framework<Data, Error> {
    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![create_cigarette_ui()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(config.command_prefix.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    database: Arc::new(Mutex::new(db)),
                })
            })
        })
        .build()
}

/// Creates and configures the Discord client with the command framework
///
/// # Arguments
/// * `config` - Loaded bot configuration containing the bot token
/// * `framework` - Configured command framework
///
/// # Returns
/// Result containing the initialized Discord client or a BotError
async fn create_client(config: &Config, framework: poise::Framework<Data, Error>) -> Result<serenity::Client, BotError> {
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::all();
    
    serenity::ClientBuilder::new(&config.bot_token, intents)
        .framework(framework)
        .await
        .map_err(BotError::from)
}

/// Establishes a connection to the PostgreSQL database
///
/// # Arguments
/// * `config` - Loaded bot configuration containing the database URL
///
/// # Returns
/// Result containing the database connection pool or a BotError
async fn connect_database(config: &Config) -> Result<PgPool, BotError> {
    PgPool::connect(&config.database_url)
        .await
        .map_err(BotError::from)
}

/// Main entry point for the bot application
///
/// Initializes the bot by:
/// 1. Setting up logging
/// 2. Loading configuration
/// 3. Connecting to the database
/// 4. Setting up the command framework
/// 5. Creating and starting the Discord client
///
/// # Returns
/// Result indicating success or a BotError
#[tokio::main]
async fn main() -> Result<(), BotError> {
    tracing_subscriber::fmt::init();
    info!("Starting cigarette counter bot...");

    let config = Config::load()?;
    let pool = connect_database(&config).await?;
    let db = Database::new(pool);
    
    let framework = setup_framework(&config, db).await;
    let mut client = create_client(&config, framework).await?;

    info!("Bot is running!");
    client.start().await?;

    Ok(())
}
