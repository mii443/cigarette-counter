mod commands;
mod database;

use std::sync::Arc;

use anyhow::Result;
use commands::create_cigarette_ui;
use database::Database;
use poise::{
    serenity_prelude::{self as serenity, futures::lock::Mutex},
    PrefixFrameworkOptions,
};
use sqlx::PgPool;

pub struct Data {
    pub database: Arc<Mutex<Database>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let token = std::env::var("BOT_TOKEN").expect("missing BOT_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::all();

    let pool =
        PgPool::connect(&std::env::var("DATABASE_URL").expect("missing DATABASE_URL")).await?;
    let db = Database::new(pool);

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![create_cigarette_ui()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(String::from("c:")),
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
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await?;

    Ok(())
}
