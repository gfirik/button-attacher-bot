//! Inline keyboard builder with Bot API 9.4 style support.
//!
//! NOTE: Using raw HTTP request because teloxide does not yet support
//! Bot API 9.4 style/icon_custom_emoji_id fields on InlineKeyboardButton.
//! TODO: Replace with native teloxide method when support is added.

use anyhow::{Context, Result};
use serde_json::json;

use crate::models::ButtonConfig;

/// Build an InlineKeyboardMarkup JSON structure from a list of button configurations.
///
/// Each button gets its own row in the keyboard.
pub fn build_inline_keyboard(buttons: &[ButtonConfig]) -> serde_json::Value {
    let keyboard: Vec<Vec<serde_json::Value>> = buttons
        .iter()
        .map(|btn| vec![btn.to_json()])
        .collect();

    json!({
        "inline_keyboard": keyboard
    })
}

/// Build a style selection inline keyboard.
pub fn build_style_keyboard() -> serde_json::Value {
    json!({
        "inline_keyboard": [
            [
                {"text": "🔵 Primary", "callback_data": "style_primary"},
                {"text": "🟢 Success", "callback_data": "style_success"}
            ],
            [
                {"text": "🔴 Danger", "callback_data": "style_danger"},
                {"text": "⚪ Default", "callback_data": "style_default"}
            ]
        ]
    })
}

/// Build an emoji skip inline keyboard.
pub fn build_emoji_keyboard() -> serde_json::Value {
    json!({
        "inline_keyboard": [
            [{"text": "⏭ Skip", "callback_data": "emoji_skip"}]
        ]
    })
}

/// Build a confirmation inline keyboard.
pub fn build_confirm_keyboard() -> serde_json::Value {
    json!({
        "inline_keyboard": [
            [
                {"text": "✅ Publish", "callback_data": "action_publish"},
                {"text": "➕ Add another button", "callback_data": "action_add"}
            ],
            [{"text": "❌ Cancel", "callback_data": "action_cancel"}]
        ]
    })
}

/// Map style callback data to the actual style value.
pub fn map_style_callback(callback_data: &str) -> Option<String> {
    match callback_data {
        "style_primary" => Some("primary".to_string()),
        "style_success" => Some("success".to_string()),
        "style_danger" => Some("danger".to_string()),
        "style_default" => None,
        _ => None,
    }
}

/// Get a human-readable style name.
pub fn style_display_name(style: Option<&str>) -> &'static str {
    match style {
        Some("primary") => "🔵 Primary",
        Some("success") => "🟢 Success",
        Some("danger") => "🔴 Danger",
        _ => "⚪ Default",
    }
}

/// Validate a URL for button usage.
///
/// Returns true if the URL starts with http://, https://, or tg://
pub fn validate_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    url_lower.starts_with("http://")
        || url_lower.starts_with("https://")
        || url_lower.starts_with("tg://")
}

/// Raw copyMessage call that supports Bot API 9.4 button fields.
///
/// NOTE: Using raw HTTP request because teloxide does not yet support
/// Bot API 9.4 style/icon_custom_emoji_id fields on InlineKeyboardButton.
/// TODO: Replace with native teloxide method when support is added.
pub async fn raw_copy_message(
    token: &str,
    chat_id: i64,
    from_chat_id: i64,
    message_id: i32,
    reply_markup: serde_json::Value,
) -> Result<()> {
    let url = format!("https://api.telegram.org/bot{}/copyMessage", token);
    let body = json!({
        "chat_id": chat_id,
        "from_chat_id": from_chat_id,
        "message_id": message_id,
        "reply_markup": reply_markup
    });

    log::debug!("Sending copyMessage request: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to send request to Telegram API")?;

    let result: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse Telegram API response")?;

    log::debug!("copyMessage response: {}", serde_json::to_string_pretty(&result).unwrap_or_default());

    if result["ok"].as_bool() != Some(true) {
        let description = result["description"]
            .as_str()
            .unwrap_or("Unknown error");

        // Provide user-friendly error messages based on common Telegram API errors
        if description.contains("not enough rights")
            || description.contains("CHAT_ADMIN_REQUIRED")
            || description.contains("need administrator rights")
            || description.contains("have no rights")
        {
            anyhow::bail!("I couldn't post to that chat. Make sure I'm an admin with permission to post messages.");
        } else if description.contains("chat not found") || description.contains("CHAT_NOT_FOUND") {
            anyhow::bail!("The chat was not found. It may have been deleted or I was removed.");
        } else if description.contains("bot was kicked") || description.contains("bot is not a member") {
            anyhow::bail!("I'm not a member of that chat. Please add me first.");
        } else if description.contains("message to copy not found") {
            anyhow::bail!("The original message was deleted. Please send new content.");
        } else {
            anyhow::bail!("Telegram API error: {}", description);
        }
    }

    Ok(())
}

/// Raw sendMessage call with custom reply markup.
///
/// NOTE: Using raw HTTP request for consistency with other raw API calls
/// and to handle potential edge cases with reply_markup serialization.
pub async fn raw_send_message(
    token: &str,
    chat_id: i64,
    text: &str,
    reply_markup: Option<serde_json::Value>,
    parse_mode: Option<&str>,
) -> Result<serde_json::Value> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let mut body = json!({
        "chat_id": chat_id,
        "text": text,
    });

    if let Some(markup) = reply_markup {
        body["reply_markup"] = markup;
    }

    if let Some(mode) = parse_mode {
        body["parse_mode"] = json!(mode);
    }

    log::debug!("Sending sendMessage request: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to send request to Telegram API")?;

    let result: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse Telegram API response")?;

    log::debug!("sendMessage response: {}", serde_json::to_string_pretty(&result).unwrap_or_default());

    if result["ok"].as_bool() != Some(true) {
        let description = result["description"]
            .as_str()
            .unwrap_or("Unknown error");
        anyhow::bail!("Telegram API error: {}", description);
    }

    Ok(result)
}

/// Raw editMessageText call.
pub async fn raw_edit_message_text(
    token: &str,
    chat_id: i64,
    message_id: i32,
    text: &str,
    reply_markup: Option<serde_json::Value>,
    parse_mode: Option<&str>,
) -> Result<()> {
    let url = format!("https://api.telegram.org/bot{}/editMessageText", token);
    let mut body = json!({
        "chat_id": chat_id,
        "message_id": message_id,
        "text": text,
    });

    if let Some(markup) = reply_markup {
        body["reply_markup"] = markup;
    }

    if let Some(mode) = parse_mode {
        body["parse_mode"] = json!(mode);
    }

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to send request to Telegram API")?;

    let result: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse Telegram API response")?;

    if result["ok"].as_bool() != Some(true) {
        let description = result["description"]
            .as_str()
            .unwrap_or("Unknown error");
        // Ignore "message is not modified" errors
        if !description.contains("message is not modified") {
            anyhow::bail!("Telegram API error: {}", description);
        }
    }

    Ok(())
}

/// Raw answerCallbackQuery call.
pub async fn raw_answer_callback_query(
    token: &str,
    callback_query_id: &str,
    text: Option<&str>,
) -> Result<()> {
    let url = format!("https://api.telegram.org/bot{}/answerCallbackQuery", token);
    let mut body = json!({
        "callback_query_id": callback_query_id,
    });

    if let Some(t) = text {
        body["text"] = json!(t);
    }

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to send request to Telegram API")?;

    let result: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse Telegram API response")?;

    if result["ok"].as_bool() != Some(true) {
        log::warn!("answerCallbackQuery failed: {:?}", result);
    }

    Ok(())
}
