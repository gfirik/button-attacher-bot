//! Handlers for /start, /help, and /cancel commands.

use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::db::{Analytics, EventType};
use crate::state::BotState;

/// The welcome/help message shown to users.
const WELCOME_MESSAGE: &str = r#"🤖 *Welcome to ButtonAttachBot\!*

Send me any content \(photo, video, text, document, etc\.\) and I'll help you post it with custom styled buttons to any chat where I'm an admin\.

✨ *Features:*
• Colored buttons \(🔵 Primary, 🟢 Success, 🔴 Danger\)
• Custom emoji icons on buttons
• Multiple buttons per post
• Post to channels, groups, or yourself

*Commands:*
/start \- Show this message
/help \- Show this message
/cancel \- Cancel current operation

Just send me any message to get started\!"#;

/// Handle /start command - show welcome message.
pub async fn handle_start(
    bot: Bot,
    msg: Message,
    dialogue: crate::Dialogue,
    analytics: Analytics,
) -> crate::HandlerResult {
    log::info!("User {} started the bot", msg.chat.id);

    // Track user and event
    if let Some(user) = msg.from.as_ref() {
        let _ = analytics.track_user(
            user.id.0 as i64,
            user.username.as_deref(),
            Some(&user.first_name),
            user.last_name.as_deref(),
            user.language_code.as_deref(),
        );
        let _ = analytics.track_event(user.id.0 as i64, EventType::Start, None);
    }

    // Reset to idle state
    dialogue.update(BotState::Idle).await?;

    bot.send_message(msg.chat.id, WELCOME_MESSAGE)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

/// Handle /help command - show welcome message.
pub async fn handle_help(
    bot: Bot,
    msg: Message,
    analytics: Analytics,
) -> crate::HandlerResult {
    log::info!("User {} requested help", msg.chat.id);

    // Track event
    if let Some(user) = msg.from.as_ref() {
        let _ = analytics.track_user(
            user.id.0 as i64,
            user.username.as_deref(),
            Some(&user.first_name),
            user.last_name.as_deref(),
            user.language_code.as_deref(),
        );
        let _ = analytics.track_event(user.id.0 as i64, EventType::Help, None);
    }

    bot.send_message(msg.chat.id, WELCOME_MESSAGE)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

/// Handle /cancel command - cancel current operation and reset to idle.
pub async fn handle_cancel(
    bot: Bot,
    msg: Message,
    dialogue: crate::Dialogue,
    analytics: Analytics,
) -> crate::HandlerResult {
    log::info!("User {} cancelled operation", msg.chat.id);

    // Track event
    if let Some(user) = msg.from.as_ref() {
        let _ = analytics.track_event(user.id.0 as i64, EventType::Cancelled, None);
    }

    dialogue.update(BotState::Idle).await?;

    bot.send_message(
        msg.chat.id,
        "❌ Operation cancelled. Send me content to start again.",
    )
    .await?;

    Ok(())
}
