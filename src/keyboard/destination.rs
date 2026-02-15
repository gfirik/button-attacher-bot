//! Destination chat picker keyboard builder.

use serde_json::json;

/// Build a ReplyKeyboardMarkup with chat picker buttons and "Send to me" option.
///
/// Telegram API requires separate buttons for groups vs channels:
/// - chat_is_channel: false = groups/supergroups
/// - chat_is_channel: true = channels
pub fn build_destination_keyboard() -> serde_json::Value {
    json!({
        "keyboard": [
            [
                {
                    "text": "👥 Choose a group",
                    "request_chat": {
                        "request_id": 1,
                        "chat_is_channel": false,
                        "bot_is_member": true
                    }
                },
                {
                    "text": "📢 Choose a channel",
                    "request_chat": {
                        "request_id": 2,
                        "chat_is_channel": true,
                        "bot_is_member": true
                    }
                }
            ],
            [
                {
                    "text": "📨 Send back to me"
                }
            ]
        ],
        "resize_keyboard": true,
        "one_time_keyboard": true
    })
}

/// Build a ReplyKeyboardRemove markup to hide the custom keyboard.
pub fn build_remove_keyboard() -> serde_json::Value {
    json!({
        "remove_keyboard": true
    })
}

/// The text of the "Send to me" button for matching.
pub const SEND_TO_ME_TEXT: &str = "📨 Send back to me";
