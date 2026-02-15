//! Handler for the publish action and confirmation callbacks.

use teloxide::prelude::*;

use crate::keyboard::{
    build_destination_keyboard, build_inline_keyboard, raw_answer_callback_query,
    raw_copy_message, raw_send_message,
};
use crate::models::SessionData;
use crate::state::BotState;
use crate::Config;

/// Handle confirmation callbacks when in AwaitingConfirm state.
pub async fn handle_confirm_callback(
    bot: Bot,
    callback: CallbackQuery,
    dialogue: crate::Dialogue,
    config: Config,
    mut data: SessionData,
) -> crate::HandlerResult {
    let callback_data = callback.data.as_deref().unwrap_or("");
    let chat_id = callback.message.as_ref().map(|m| m.chat().id.0).unwrap_or(0);

    // Answer the callback
    raw_answer_callback_query(&config.bot_token, &callback.id, None).await?;

    match callback_data {
        "action_add" => {
            log::info!("User wants to add another button");

            // Clear current button fields
            data.clear_current_button();

            // Transition to AwaitingButtonText
            dialogue
                .update(BotState::AwaitingButtonText { data })
                .await?;

            bot.send_message(
                ChatId(chat_id),
                "✏️ Send me the <b>button text</b> for the next button.",
            )
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
        }

        "action_cancel" => {
            log::info!("User cancelled the operation");

            // Reset to idle
            dialogue.update(BotState::Idle).await?;

            bot.send_message(
                ChatId(chat_id),
                "❌ Cancelled. Send me content to start again.",
            )
            .await?;
        }

        "action_publish" => {
            log::info!("User wants to publish");

            // Verify we have all required data
            let (source_chat_id, source_message_id, destination_chat_id) = match (
                data.source_chat_id,
                data.source_message_id,
                data.destination_chat_id,
            ) {
                (Some(s), Some(m), Some(d)) => (s, m, d),
                _ => {
                    log::error!("Missing required data for publish: {:?}", data);
                    bot.send_message(
                        ChatId(chat_id),
                        "❌ Something went wrong - missing required data. Please start over.",
                    )
                    .await?;
                    dialogue.update(BotState::Idle).await?;
                    return Ok(());
                }
            };

            // Build the inline keyboard with styled buttons
            let reply_markup = build_inline_keyboard(&data.buttons);

            log::debug!("Publishing with keyboard: {:?}", reply_markup);

            // Execute copyMessage with the styled buttons
            match raw_copy_message(
                &config.bot_token,
                destination_chat_id,
                source_chat_id,
                source_message_id,
                reply_markup,
            )
            .await
            {
                Ok(_) => {
                    log::info!(
                        "Successfully published message to chat {}",
                        destination_chat_id
                    );
                    bot.send_message(
                        ChatId(chat_id),
                        "✅ Posted successfully! The buttons will work forever, even when I'm offline.\n\nSend me more content anytime!",
                    )
                    .await?;

                    // Only reset to idle on success
                    dialogue.update(BotState::Idle).await?;
                }
                Err(e) => {
                    log::error!("Failed to publish message: {}", e);

                    // Clear destination but KEEP the button config
                    data.destination_chat_id = None;
                    let button_count = data.buttons.len();

                    // Go back to destination selection
                    dialogue
                        .update(BotState::AwaitingDestination { data })
                        .await?;

                    // Show error and destination picker again
                    let error_msg = format!(
                        "❌ {}\n\n\
                        Your button configuration is saved ({} button(s)).\n\
                        Please choose a different chat where I'm an admin:",
                        e,
                        button_count
                    );

                    let keyboard = build_destination_keyboard();
                    raw_send_message(&config.bot_token, chat_id, &error_msg, Some(keyboard), None)
                        .await?;
                }
            }
        }

        _ => {
            log::warn!("Unknown callback data: {}", callback_data);
        }
    }

    Ok(())
}
