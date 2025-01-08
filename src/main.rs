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

pub struct Data {
    pub database: Arc<Mutex<Database>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug, thiserror::Error)]
pub enum BotError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("Database connection error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Client error: {0}")]
    Client(#[from] serenity::Error),
}

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

async fn create_client(config: &Config, framework: poise::Framework<Data, Error>) -> Result<serenity::Client, BotError> {
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::all();
    
    serenity::ClientBuilder::new(&config.bot_token, intents)
        .framework(framework)
        .await
        .map_err(BotError::from)
}

async fn connect_database(config: &Config) -> Result<PgPool, BotError> {
    PgPool::connect(&config.database_url)
        .await
        .map_err(BotError::from)
}

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
