//! Handler for receiving user content (any message type).

use teloxide::prelude::*;

use crate::db::{Analytics, EventType};
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
    analytics: Analytics,
) -> crate::HandlerResult {
    log::info!(
        "User {} sent content (message_id: {})",
        msg.chat.id,
        msg.id.0
    );

    // Check if user is blocked
    if let Some(user) = msg.from.as_ref() {
        let user_id = user.id.0 as i64;

        if analytics.is_user_blocked(user_id).unwrap_or(false) {
            log::warn!("Blocked user {} attempted to use the bot", user_id);
            return Ok(());
        }

        // Track user and event
        let _ = analytics.track_user(
            user_id,
            user.username.as_deref(),
            Some(&user.first_name),
            user.last_name.as_deref(),
            user.language_code.as_deref(),
        );
        let _ = analytics.track_event(user_id, EventType::ContentReceived, None);
    }

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
