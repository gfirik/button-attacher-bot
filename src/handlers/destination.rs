//! Handler for destination selection (chat_shared event or "Send to me").

use teloxide::prelude::*;
use teloxide::types::MessageKind;

use crate::keyboard::{
    build_confirm_keyboard, build_remove_keyboard, raw_send_message, style_display_name,
    SEND_TO_ME_TEXT,
};
use crate::models::SessionData;
use crate::state::BotState;
use crate::Config;

/// Handle destination selection when in AwaitingDestination state.
///
/// Handles two cases:
/// 1. ChatShared update - user selected a chat from the picker
/// 2. Text message "📨 Send back to me" - user wants to send to themselves
pub async fn handle_destination(
    bot: Bot,
    msg: Message,
    dialogue: crate::Dialogue,
    config: Config,
    mut data: SessionData,
) -> crate::HandlerResult {
    // Check for chat_shared event via MessageKind
    if let MessageKind::ChatShared(msg_chat_shared) = &msg.kind {
        let chat_id = msg_chat_shared.chat_shared.chat_id.0;
        log::info!(
            "User {} selected chat {} as destination",
            msg.chat.id,
            chat_id
        );
        data.destination_chat_id = Some(chat_id);
    }
    // Check for "Send to me" text
    else if let Some(text) = msg.text() {
        if text == SEND_TO_ME_TEXT {
            log::info!("User {} selected to send to themselves", msg.chat.id);
            data.destination_chat_id = Some(msg.chat.id.0);
        } else {
            // Unexpected text, remind user to pick a destination
            bot.send_message(
                msg.chat.id,
                "Please use the buttons below to pick a destination, or type /cancel to start over.",
            )
            .await?;
            return Ok(());
        }
    } else {
        // Other message types - ignore
        return Ok(());
    }

    // Remove the reply keyboard
    let remove_keyboard = build_remove_keyboard();
    raw_send_message(
        &config.bot_token,
        msg.chat.id.0,
        "✅ Destination selected!",
        Some(remove_keyboard),
        None,
    )
    .await?;

    // Check if buttons are already configured (re-selecting destination after failed publish)
    if !data.buttons.is_empty() {
        // Go straight to confirm - buttons are already ready
        let button_summary: Vec<String> = data
            .buttons
            .iter()
            .map(|b| {
                format!(
                    "  • {} → {}",
                    b.text,
                    style_display_name(b.style.as_deref())
                )
            })
            .collect();

        let summary = format!(
            "📋 Ready to publish!\n\n\
            Buttons configured:\n{}\n\n\
            What's next?",
            button_summary.join("\n")
        );

        dialogue
            .update(BotState::AwaitingConfirm { data })
            .await?;

        let keyboard = build_confirm_keyboard();
        raw_send_message(&config.bot_token, msg.chat.id.0, &summary, Some(keyboard), None)
            .await?;
    } else {
        // No buttons yet - go to button configuration
        dialogue
            .update(BotState::AwaitingButtonText { data })
            .await?;

        bot.send_message(
            msg.chat.id,
            "✏️ Now send me the <b>button text</b> — this is the label users will see on the button.",
        )
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    }

    Ok(())
}
