//! Handler for button configuration flow.

use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::keyboard::{
    build_confirm_keyboard, build_emoji_keyboard, build_style_keyboard,
    map_style_callback, raw_answer_callback_query, raw_edit_message_text,
    raw_send_message, style_display_name, validate_url,
};
use crate::models::SessionData;
use crate::state::BotState;
use crate::Config;

/// Handle button text input when in AwaitingButtonText state.
pub async fn handle_button_text(
    bot: Bot,
    msg: Message,
    dialogue: crate::Dialogue,
    mut data: SessionData,
) -> crate::HandlerResult {
    let text = match msg.text() {
        Some(t) => t.to_string(),
        None => {
            bot.send_message(msg.chat.id, "Please send me a text message for the button label.")
                .await?;
            return Ok(());
        }
    };

    log::info!("User {} set button text: {}", msg.chat.id, text);

    data.current_button_text = Some(text);

    // Transition to AwaitingUrl
    dialogue.update(BotState::AwaitingUrl { data }).await?;

    bot.send_message(
        msg.chat.id,
        "🔗 Now send me the <b>URL</b> for this button.\n\nSupported formats: <code>https://...</code>, <code>http://...</code>, <code>tg://...</code>",
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

/// Handle URL input when in AwaitingUrl state.
pub async fn handle_url(
    bot: Bot,
    msg: Message,
    dialogue: crate::Dialogue,
    config: Config,
    mut data: SessionData,
) -> crate::HandlerResult {
    let url = match msg.text() {
        Some(t) => t.to_string(),
        None => {
            bot.send_message(msg.chat.id, "Please send me a URL as a text message.")
                .await?;
            return Ok(());
        }
    };

    // Validate URL
    if !validate_url(&url) {
        bot.send_message(
            msg.chat.id,
            "⚠️ That doesn't look like a valid URL. Please send a link starting with https://, http://, or tg://",
        )
        .await?;
        return Ok(());
    }

    log::info!("User {} set button URL: {}", msg.chat.id, url);

    data.current_button_url = Some(url);

    // Transition to AwaitingStyle
    dialogue.update(BotState::AwaitingStyle { data }).await?;

    // Send style picker
    let keyboard = build_style_keyboard();
    raw_send_message(
        &config.bot_token,
        msg.chat.id.0,
        "🎨 Pick a button style:",
        Some(keyboard),
        None,
    )
    .await?;

    Ok(())
}

/// Handle style selection callback when in AwaitingStyle state.
pub async fn handle_style_callback(
    _bot: Bot,
    callback: CallbackQuery,
    dialogue: crate::Dialogue,
    config: Config,
    mut data: SessionData,
) -> crate::HandlerResult {
    let callback_data = callback.data.as_deref().unwrap_or("");
    let chat_id = callback.message.as_ref().map(|m| m.chat().id.0).unwrap_or(0);
    let message_id = callback.message.as_ref().and_then(|m| m.regular_message()).map(|m| m.id.0).unwrap_or(0);

    // Answer the callback to stop loading animation
    raw_answer_callback_query(&config.bot_token, &callback.id, None).await?;

    // Map callback to style
    let style = map_style_callback(callback_data);
    let style_name = style_display_name(style.as_deref());

    log::info!("User selected style: {:?}", style);

    data.current_button_style = style;

    // Edit the style message to show selection
    let edit_text = format!("🎨 Style: {} ✓", style_name);
    let _ = raw_edit_message_text(
        &config.bot_token,
        chat_id,
        message_id,
        &edit_text,
        None,
        None,
    )
    .await;

    // Transition to AwaitingEmoji
    dialogue.update(BotState::AwaitingEmoji { data }).await?;

    // Send emoji prompt
    let keyboard = build_emoji_keyboard();
    raw_send_message(
        &config.bot_token,
        chat_id,
        "🎭 Want to add a <b>custom emoji icon</b> to the button?\n\nSend the emoji ID, or tap Skip.",
        Some(keyboard),
        Some("HTML"),
    )
    .await?;

    Ok(())
}

/// Handle emoji input or skip when in AwaitingEmoji state.
pub async fn handle_emoji_callback(
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

    if callback_data == "emoji_skip" {
        log::info!("User skipped emoji");
        data.current_button_emoji = None;
    }

    // Finalize the button
    finalize_button_and_show_summary(bot, dialogue, config, data, chat_id).await
}

/// Handle emoji text input when in AwaitingEmoji state.
pub async fn handle_emoji_text(
    bot: Bot,
    msg: Message,
    dialogue: crate::Dialogue,
    config: Config,
    mut data: SessionData,
) -> crate::HandlerResult {
    let emoji_id = match msg.text() {
        Some(t) => t.to_string(),
        None => {
            bot.send_message(msg.chat.id, "Please send the emoji ID as text, or tap Skip.")
                .await?;
            return Ok(());
        }
    };

    log::info!("User {} set emoji ID: {}", msg.chat.id, emoji_id);
    data.current_button_emoji = Some(emoji_id);

    finalize_button_and_show_summary(bot, dialogue, config, data, msg.chat.id.0).await
}

/// Finalize the current button configuration and show the summary.
async fn finalize_button_and_show_summary(
    _bot: Bot,
    dialogue: crate::Dialogue,
    config: Config,
    mut data: SessionData,
    chat_id: i64,
) -> crate::HandlerResult {
    // Store emoji before finalizing
    let emoji_display = data
        .current_button_emoji
        .as_ref()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "None".to_string());

    // Finalize the button
    let button = data.finalize_current_button();

    let button = match button {
        Some(b) => b,
        None => {
            log::error!("Failed to finalize button - missing text or URL");
            raw_send_message(
                &config.bot_token,
                chat_id,
                "❌ Something went wrong. Please start over with /cancel.",
                None,
                None,
            )
            .await?;
            return Ok(());
        }
    };

    // Build summary message
    let style_display = style_display_name(button.style.as_deref());
    let summary = format!(
        "📋 Button configured:\n\n\
         📌 Label: {}\n\
         🔗 URL: {}\n\
         🎨 Style: {}\n\
         🎭 Emoji: {}\n\n\
         Total buttons: {}\n\n\
         What's next?",
        button.text,
        button.url,
        style_display,
        emoji_display,
        data.button_count()
    );

    // Transition to AwaitingConfirm
    dialogue.update(BotState::AwaitingConfirm { data }).await?;

    // Send summary with confirm keyboard
    let keyboard = build_confirm_keyboard();
    raw_send_message(&config.bot_token, chat_id, &summary, Some(keyboard), None).await?;

    Ok(())
}
