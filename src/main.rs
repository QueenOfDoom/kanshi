mod event;
mod log;
mod persistence;

use crate::log::setup_logger;
use crate::persistence::{connect_db, initialize_db};
use ::log::{error, info};
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ChannelId;
use serenity::GatewayIntents;
use tokio::signal;

// User Data
struct Data {
    log_channel: ChannelId,
    environment: String,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
// type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    let conn = connect_db().expect("Database Connection Failure");
    initialize_db(&conn).expect("Database Initialization Failure");
    setup_logger().expect("Failed to initialize logger");
    info!("Database & Logging are available.");

    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let log_channel =
        std::env::var("LOG_CHANNEL").expect("Expected a log channel in the environment");
    let environment = std::env::var("ENV").unwrap_or("production".to_string());
    info!("Environment is set up");

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
