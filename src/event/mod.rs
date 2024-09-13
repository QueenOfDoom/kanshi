use log::{debug, info, warn};
use crate::{Data, Error};

use poise::serenity_prelude::{self as serenity, Colour, CreateEmbed, CreateEmbedFooter, CreateMessage, Mentionable, UserId};
use serenity::FullEvent;
use crate::persistence::{connect_db, get_message_by_id, get_message_count, insert_message, update_message_by_id};

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
                let msg = CreateMessage::new()
                    .content("Logging Channel Found :white_check_mark:");
                data.log_channel.send_message(&ctx.http, msg)
                    .await.expect("Could not send message to logging channel.");
            }
        }
        FullEvent::Message { new_message, .. } => {
            let db = connect_db().expect("Database Connection Failure");
            if new_message.author.id == framework.bot_id {
               return Ok(())
            }
            insert_message(&db, new_message.id.get(), new_message.author.id.get(), new_message.content.as_str())
                .expect("Unable to insert message");

            let entries = get_message_count(&db).expect("This call must succeed.");
            if entries % 1000u64 == 0 {
                info!("Database contains {} messages", entries);
                let msg = CreateMessage::new()
                    .content(format!("Database now contains {} messages.", entries));
                data.log_channel.send_message(&ctx.http, msg)
                    .await.expect("Could not send message to logging channel.");
            }
        }
        FullEvent::MessageUpdate { event, .. } => {
            let db = connect_db().expect("Database Connection Failure");
            debug!("Message {:?} updated to: {:?}", event.id, event.content);
            match &event.content {
                Some(content) => {
                    let (user_id, previous_content) = get_message_by_id(&db, event.id.get())
                        .expect("Error making the get request.")
                        .unwrap_or((0, "<unknown message>".to_string()));
                    let mut user_name = "<unknown user>".to_string();

                    if framework.bot_id == user_id {
                        return Ok(())
                    }

                    let mut embed = CreateEmbed::new().title("Message Updated")
                        .colour(Colour::ORANGE);

                    if user_id != 0 {
                        let author = match &event.author {
                            Some(author) => author,
                            None => &UserId::new(user_id).to_user(&ctx.http)
                                .await.expect("User should exist and be accessible at this time.")
                        };
                        user_name = author.mention().to_string();
                        embed = embed.footer(CreateEmbedFooter::new(&author.name)
                            .icon_url(author.face()))
                    }

                    embed = embed.field("Author", format!("{} ({})", user_name, user_id.to_string()), false)
                        .field("Old Message", previous_content, false)
                        .field("New Message", content, false);

                    data.log_channel.send_message(&ctx.http, CreateMessage::new().embed(embed))
                        .await.expect("Unable to send message");

                    update_message_by_id(&db, event.id.get(), content)
                        .expect("Error making the update request.");
                }
                None => {
                    warn!("Message Update occurred not in message content")
                }
            }
        }
        FullEvent::MessageDelete { deleted_message_id, .. } => {
            let db = connect_db().expect("Database Connection Failure");
            let (user_id, content) = get_message_by_id(&db, deleted_message_id.get())
                .expect("Error making the database request.")
                .map_or((0u64, "<unknown message>".to_string()), |(u, s)| (u, s));

            if user_id == framework.bot_id.get() {
                return Ok(())
            }

            let mut embed = CreateEmbed::new()
                .title("Message Deleted")
                .colour(Colour::DARK_RED);

            if user_id != 0 {
                let id = UserId::new(user_id);
                match id.to_user(&ctx.http).await {
                    Ok(user) => {
                        embed = embed.footer(CreateEmbedFooter::new(&user.name)
                            .icon_url(user.face()))
                    }
                    Err(_) => {
                        embed = embed.footer(CreateEmbedFooter::new("<unknown user>"));
                    }
                }
                embed = embed.field("Author", format!("{} ({})", id.mention().to_string(), user_id.to_string()), false);
            }

            embed = embed.field("Deleted Message", content, false);

            data.log_channel.send_message(&ctx.http, CreateMessage::new().embed(embed))
                .await.expect("Unable to send message");
        }
        _ => {}
    }
    Ok(())
}