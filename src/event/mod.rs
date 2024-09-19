use crate::{Data, Error};
use log::{debug, info, warn};

use poise::serenity_prelude::{self as serenity, Colour, CreateEmbed, CreateEmbedFooter, CreateMessage, Mentionable, Timestamp, UserId};
use serenity::FullEvent;
use crate::persistence::{create_message, exists_message, get_author_from_message, get_message_content_and_author_by_id, get_message_content_by_id, get_message_count, update_message_content};
use crate::util::UNKNOWN_USER;

fn construct_msg_ref(guild_id: u64, channel_id: u64, message_id: u64) -> String {
    format!("https://discord.com/channels/{}/{}/{}",
            guild_id.to_string(),
            channel_id.to_string(),
            message_id.to_string())
}

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            info!("Logged in as {}", data_about_bot.user.tag());
            if data.environment != "develop" {
                let msg = CreateMessage::new().content("Logging Channel Found :white_check_mark:");
                data.log_channel
                    .send_message(&ctx.http, msg)
                    .await
                    .expect("Could not send message to logging channel.");
            }
        }
        FullEvent::Message { new_message, .. } => {
            if new_message.author.id == framework.bot_id {
                return Ok(());
            }
            
            create_message(
                new_message.id.get(),
                new_message.author.id.get(),
                new_message.content.clone()
            );

            let entries = get_message_count();
            if entries % 1000 == 0 {
                info!("Database contains {} messages", entries);
                let msg = CreateMessage::new()
                    .content(format!("Database now contains {} messages.", entries));
                data.log_channel
                    .send_message(&ctx.http, msg)
                    .await
                    .expect("Could not send message to logging channel.");
            }
        }
        FullEvent::MessageUpdate { event, .. } => {
            debug!("Message {:?} updated to: {:?}", event.id, event.content);

            let message_id = event.id.get();
            let user_id = match &event.author {
                Some(user) => user.id.get(),
                None => get_author_from_message(message_id)
            };
            if user_id == UNKNOWN_USER { return Ok(()) }

            let user = UserId::new(user_id).to_user(&ctx.http)
                .await.expect("User should exist.");
            if user.bot { return Ok(()) }

            let guild_id = event.guild_id.map_or(0, |id| id.get());
            if guild_id == 0 { return Ok(()) }

            match &event.content {
                Some(content) => {
                    let mut previous_content = get_message_content_by_id(message_id);
                    if previous_content.eq(content) {
                        warn!("TODO: Implement non-content message updates (i.e. Embeds)");
                        return Ok(());
                    }

                    previous_content.truncate(1024);

                    let mut current_content = content.clone();
                    current_content.truncate(1024);

                    if framework.bot_id == user_id { return Ok(()); }

                    let embed = CreateEmbed::new()
                        .title("Message Updated")
                        .url(construct_msg_ref(guild_id, event.channel_id.get(), message_id))
                        .timestamp(Timestamp::now())
                        .colour(Colour::ORANGE)
                        .field(
                            "Author",
                            format!("{} ({})", user.mention(), user_id.to_string()),
                            false,
                        )
                        .field("Old Message", previous_content, false)
                        .field("New Message", current_content, false)
                        .footer(CreateEmbedFooter::new(&user.name)
                            .icon_url(user.face()));

                    data.log_channel
                        .send_message(&ctx.http, CreateMessage::new().embed(embed))
                        .await
                        .expect("Unable to send message");
                    
                    if exists_message(message_id) {
                        update_message_content(message_id, content.clone());
                    } else {
                        create_message(message_id, user.id.get(), content.clone());
                    }
                }
                None => {
                    warn!("Message Update occurred not in message content")
                }
            }
        }
        FullEvent::MessageDelete {
            deleted_message_id, ..
        } => {
            let (user_id, content) = get_message_content_and_author_by_id(deleted_message_id.get());

            if user_id == framework.bot_id.get() {
                return Ok(());
            }

            debug!("Message {:?} deleted.", deleted_message_id);

            let mut embed = CreateEmbed::new()
                .title("Message Deleted")
                .timestamp(Timestamp::now())
                .colour(Colour::DARK_RED);

            if user_id != 0 {
                let id = UserId::new(user_id);
                match id.to_user(&ctx.http).await {
                    Ok(user) => {
                        embed =
                            embed.footer(CreateEmbedFooter::new(&user.name).icon_url(user.face()))
                    }
                    Err(_) => {
                        embed = embed.footer(CreateEmbedFooter::new("<unknown user>"));
                    }
                }
                embed = embed.field(
                    "Author",
                    format!("{} ({})", id.mention().to_string(), user_id.to_string()),
                    false,
                );
            }

            embed = embed.field("Deleted Message", content, false);

            data.log_channel
                .send_message(&ctx.http, CreateMessage::new().embed(embed))
                .await
                .expect("Unable to send message");
        }
        _ => {}
    }
    Ok(())
}
