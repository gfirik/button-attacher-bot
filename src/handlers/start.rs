//! Handlers for /start, /help, and /cancel commands.

use teloxide::prelude::*;
use teloxide::types::ParseMode;

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
) -> crate::HandlerResult {
    log::info!("User {} started the bot", msg.chat.id);

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
) -> crate::HandlerResult {
    log::info!("User {} requested help", msg.chat.id);

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
) -> crate::HandlerResult {
    log::info!("User {} cancelled operation", msg.chat.id);

    dialogue.update(BotState::Idle).await?;

    bot.send_message(
        msg.chat.id,
        "❌ Operation cancelled. Send me content to start again.",
    )
    .await?;

    Ok(())
}
