//! Session data structures for the bot conversation flow.

use serde::{Deserialize, Serialize};

/// Configuration for a single inline button.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ButtonConfig {
    /// The label text displayed on the button.
    pub text: String,
    /// The URL the button links to.
    pub url: String,
    /// Optional button style: "primary", "success", "danger", or None for default.
    pub style: Option<String>,
    /// Optional custom emoji ID to display as an icon on the button.
    pub icon_custom_emoji_id: Option<String>,
}

impl ButtonConfig {
    /// Create a new ButtonConfig with the given parameters.
    pub fn new(
        text: String,
        url: String,
        style: Option<String>,
        icon_custom_emoji_id: Option<String>,
    ) -> Self {
        Self {
            text,
            url,
            style,
            icon_custom_emoji_id,
        }
    }

    /// Convert this button config to a JSON value for the Telegram API.
    pub fn to_json(&self) -> serde_json::Value {
        let mut button = serde_json::json!({
            "text": self.text,
            "url": self.url,
        });

        if let Some(ref style) = self.style {
            button["style"] = serde_json::json!(style);
        }

        if let Some(ref emoji_id) = self.icon_custom_emoji_id {
            button["icon_custom_emoji_id"] = serde_json::json!(emoji_id);
        }

        button
    }
}

/// Session data persisted across conversation states.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SessionData {
    /// The chat ID where the original content was sent from.
    pub source_chat_id: Option<i64>,
    /// The message ID of the original content.
    pub source_message_id: Option<i32>,
    /// The destination chat ID where content will be posted.
    pub destination_chat_id: Option<i64>,
    /// List of configured buttons to attach.
    pub buttons: Vec<ButtonConfig>,
    /// Currently being configured: button text.
    pub current_button_text: Option<String>,
    /// Currently being configured: button URL.
    pub current_button_url: Option<String>,
    /// Currently being configured: button style.
    pub current_button_style: Option<String>,
    /// Currently being configured: button emoji ID.
    pub current_button_emoji: Option<String>,
}

impl SessionData {
    /// Create a new empty session.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all session data.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Clear only the current button configuration fields.
    pub fn clear_current_button(&mut self) {
        self.current_button_text = None;
        self.current_button_url = None;
        self.current_button_style = None;
        self.current_button_emoji = None;
    }

    /// Finalize the current button configuration and add it to the buttons list.
    pub fn finalize_current_button(&mut self) -> Option<ButtonConfig> {
        if let (Some(text), Some(url)) = (self.current_button_text.take(), self.current_button_url.take()) {
            let button = ButtonConfig::new(
                text,
                url,
                self.current_button_style.take(),
                self.current_button_emoji.take(),
            );
            self.buttons.push(button.clone());
            Some(button)
        } else {
            None
        }
    }

    /// Get the number of configured buttons.
    pub fn button_count(&self) -> usize {
        self.buttons.len()
    }
}
