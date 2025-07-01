use log::{info, warn};
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::ai::create_ai_backend;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
    #[command(description = "chat with AI - send your message after the command.")]
    General(String),
}

pub async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    // Log incoming message details
    let chat_type = match msg.chat.is_private() {
        true => "Private",
        false => match msg.chat.is_group() {
            true => "Group",
            false => match msg.chat.is_supergroup() {
                true => "Supergroup",
                false => "Channel",
            },
        },
    };

    let username = msg
        .from
        .as_ref()
        .and_then(|user| user.username.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("<no_username>");
    let user_id = msg.from.as_ref().map(|user| user.id.0).unwrap_or(0);
    let message_text = msg.text().unwrap_or("<no_text>");

    info!(
        "📨 Received message in {} chat (ID: {}) from @{} ({}): '{}'",
        chat_type, msg.chat.id, username, user_id, message_text
    );
    info!("💬 Processing command: {cmd:?}");

    match cmd {
        Command::Help => {
            let response = Command::descriptions().to_string();
            info!("📤 Sending help response to chat {}", msg.chat.id);
            bot.send_message(msg.chat.id, response).await?
        }
        Command::Username(username) => {
            let response = format!("Your username is @{username}.");
            info!(
                "📤 Sending username response to chat {}: '{}'",
                msg.chat.id, response
            );
            bot.send_message(msg.chat.id, response).await?
        }
        Command::UsernameAndAge { username, age } => {
            let response = format!("Your username is @{username} and age is {age}.");
            info!(
                "📤 Sending username+age response to chat {}: '{}'",
                msg.chat.id, response
            );
            bot.send_message(msg.chat.id, response).await?
        }
        Command::General(message) => {
            if message.trim().is_empty() {
                let response = "Please provide a message. You can either use /general <message> or just mention me with your message.";
                info!(
                    "📤 Sending empty message help to chat {}: '{}'",
                    msg.chat.id, response
                );
                bot.send_message(msg.chat.id, response).await?
            } else {
                info!(
                    "🤖 Processing AI request from chat {}: '{}'",
                    msg.chat.id, message
                );
                // Send typing indicator
                bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing)
                    .await?;

                match create_ai_backend() {
                    Ok(ai_backend) => {
                        info!("✅ AI backend created successfully");
                        match ai_backend.chat(&message).await {
                            Ok(response) => {
                                info!(
                                    "📤 Sending AI response to chat {} (length: {} chars)",
                                    msg.chat.id,
                                    response.len()
                                );
                                info!("🤖 AI response: '{response}'");
                                bot.send_message(msg.chat.id, response).await?
                            }
                            Err(e) => {
                                let error_msg = format!("AI Error: {e}");
                                warn!("❌ AI request failed for chat {}: {}", msg.chat.id, e);
                                info!(
                                    "📤 Sending AI error response to chat {}: '{}'",
                                    msg.chat.id, error_msg
                                );
                                bot.send_message(msg.chat.id, error_msg).await?
                            }
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Configuration Error: {e}");
                        warn!(
                            "⚙️ AI backend configuration failed for chat {}: {}",
                            msg.chat.id, e
                        );
                        info!(
                            "📤 Sending config error response to chat {}: '{}'",
                            msg.chat.id, error_msg
                        );
                        bot.send_message(msg.chat.id, error_msg).await?
                    }
                }
            }
        }
    };

    Ok(())
}