mod event;
mod log;
mod persistence;
mod util;

use crate::log::setup_logger;
use crate::persistence::establish_connection;
use ::log::{error, info};
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ChannelId;
use serenity::GatewayIntents;
use std::error::Error as StdError;
use tokio::signal;

// User Data
struct Data {
    log_channel: ChannelId,
    environment: String,
}
type Error = Box<dyn StdError + Send + Sync>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(conn: &mut SqliteConnection) -> Result<(), Box<dyn StdError + Send + Sync>> {
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let log_channel = std::env::var("LOG_CHANNEL").expect("Expected a log channel in the environment");
    let environment = std::env::var("ENV").unwrap_or("production".to_string());
    info!("Environment is set up");
    
    let connection = &mut establish_connection();
    run_migrations(connection).expect("Database should be initialize-able.");
    setup_logger().expect("Failed to initialize logger");
    info!("Database & Logging are available.");
    
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event::event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let commands = &framework.options().commands;
                if !commands.is_empty() {
                    poise::builtins::register_globally(ctx, commands).await?;
                }
                Ok(Data {
                    log_channel: ChannelId::new(
                        log_channel
                            .parse()
                            .expect("Channel ID: Not a proper Discord Snowflake"),
                    ),
                    environment,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    let ctrl_c = signal::ctrl_c();
    tokio::select! {
        _ = ctrl_c => {
            info!("Shutting down...");
            client.shard_manager.shutdown_all().await;
        }
        result = client.start() => {
            if let Err(why) = result {
                error!("Client error: {:?}", why);
            }
        }
    }
}
