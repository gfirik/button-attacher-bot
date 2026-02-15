//! Handler for receiving user content (any message type).

use teloxide::prelude::*;

use crate::keyboard::{build_destination_keyboard, raw_send_message};
use crate::models::SessionData;
use crate::state::BotState;
use crate::Config;

/// Handle any incoming content when in Idle state.
///
/// Saves the message info and prompts user to pick a destination.
pub async fn handle_content(
    _bot: Bot,
    msg: Message,
    dialogue: crate::Dialogue,
    config: Config,
) -> crate::HandlerResult {
    log::info!(
        "User {} sent content (message_id: {})",
        msg.chat.id,
        msg.id.0
    );

    // Create session data with source info
    let mut data = SessionData::new();
    data.source_chat_id = Some(msg.chat.id.0);
    data.source_message_id = Some(msg.id.0);

    // Transition to AwaitingDestination
    dialogue
        .update(BotState::AwaitingDestination { data })
        .await?;

    // Send destination picker with custom keyboard
    let text = "📬 Where should I post this?\n\nTap below to pick a chat where I'm an admin, or send it back to yourself.";
    let keyboard = build_destination_keyboard();

    raw_send_message(&config.bot_token, msg.chat.id.0, text, Some(keyboard), None).await?;

    Ok(())
}
