mod commands;
mod error;
mod event;
mod log;
mod persistence;
#[cfg(test)]
mod tests;
mod util;

use crate::log::setup_logger;
use crate::persistence::{
    establish_connection, sqlite_pool_handler, SqlitePool, SqlitePooledConnection,
};
use ::log::{error, info};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, CreateMessage};
use serenity::GatewayIntents;
use std::error::Error as StdError;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

// User Data
pub struct Data {
    log_channel: ChannelId,
    environment: String,
    pool: SqlitePool,
}
pub type Error = Box<dyn StdError + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(mut conn: SqlitePooledConnection) -> Result<(), Box<dyn StdError + Send + Sync>> {
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let log_channel = std::env::var("LOG_CHANNEL")
        .expect("Expected a log channel in the environment")
        .parse()
        .expect("Channel ID: Not a proper Discord Snowflake");
    let environment = std::env::var("ENV").unwrap_or("production".to_string());
    info!("Environment is set up");

    let pool = establish_connection();
    let connection = sqlite_pool_handler(&pool).expect("Pooled Connection failed.");
    run_migrations(connection).expect("Database should be initialize-able.");
    setup_logger().expect("Failed to initialize logger");
    info!("Database & Logging are available.");

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let framework_environment = environment.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::changelog()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("$".into()),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    Duration::from_secs(3600),
                ))),
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),

            event_handler: |ctx, event, framework, data| {
                Box::pin(event::event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                let commands = &framework.options().commands;
                if !commands.is_empty() {
                    poise::builtins::register_globally(ctx, commands).await?;
                }
                Ok(Data {
                    log_channel: ChannelId::new(log_channel),
                    environment: framework_environment,
                    pool,
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
            if environment != "develop" {
                let message = CreateMessage::default().content("Goodbye :saluting_face:");
                client.http.send_message(ChannelId::new(log_channel), vec![], &message)
                    .await.expect("Couldn't send Goodbye Message...");
            }
            client.shard_manager.shutdown_all().await;
        }
        result = client.start() => {
            if let Err(why) = result {
                error!("Client error: {:?}", why);
            }
        }
    }
}
