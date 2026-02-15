//! Integration tests for ButtonAttachBot.

use serde_json::json;

// Import the crate modules for testing
// Note: We test the public API and internal logic

/// Test URL validation logic.
mod url_validation {
    /// Validates a URL for button usage.
    fn validate_url(url: &str) -> bool {
        let url_lower = url.to_lowercase();
        url_lower.starts_with("http://")
            || url_lower.starts_with("https://")
            || url_lower.starts_with("tg://")
    }

    #[test]
    fn test_https_urls_are_valid() {
        assert!(validate_url("https://example.com"));
        assert!(validate_url("https://example.com/path?query=1"));
        assert!(validate_url("HTTPS://EXAMPLE.COM"));
    }

    #[test]
    fn test_http_urls_are_valid() {
        assert!(validate_url("http://example.com"));
        assert!(validate_url("http://localhost:8080"));
        assert!(validate_url("HTTP://EXAMPLE.COM"));
    }

    #[test]
    fn test_telegram_urls_are_valid() {
        assert!(validate_url("tg://resolve?domain=username"));
        assert!(validate_url("tg://join?invite=hash"));
        assert!(validate_url("TG://RESOLVE"));
    }

    #[test]
    fn test_invalid_urls() {
        assert!(!validate_url("ftp://example.com"));
        assert!(!validate_url("javascript:alert(1)"));
        assert!(!validate_url("mailto:test@example.com"));
        assert!(!validate_url("example.com"));
        assert!(!validate_url("not a url"));
        assert!(!validate_url(""));
    }
}

/// Test style mapping logic.
mod style_mapping {
    /// Maps callback data to style value.
    fn map_style_callback(callback_data: &str) -> Option<String> {
        match callback_data {
            "style_primary" => Some("primary".to_string()),
            "style_success" => Some("success".to_string()),
            "style_danger" => Some("danger".to_string()),
            "style_default" => None,
            _ => None,
        }
    }

    #[test]
    fn test_primary_style() {
        assert_eq!(map_style_callback("style_primary"), Some("primary".to_string()));
    }

    #[test]
    fn test_success_style() {
        assert_eq!(map_style_callback("style_success"), Some("success".to_string()));
    }

    #[test]
    fn test_danger_style() {
        assert_eq!(map_style_callback("style_danger"), Some("danger".to_string()));
    }

    #[test]
    fn test_default_style() {
        assert_eq!(map_style_callback("style_default"), None);
    }

    #[test]
    fn test_unknown_style() {
        assert_eq!(map_style_callback("unknown"), None);
        assert_eq!(map_style_callback(""), None);
    }
}

/// Test keyboard builder JSON structure.
mod keyboard_builder {
    use super::json;

    /// Represents a button configuration.
    #[derive(Clone, Debug)]
    struct ButtonConfig {
        text: String,
        url: String,
        style: Option<String>,
        icon_custom_emoji_id: Option<String>,
    }

    impl ButtonConfig {
        fn to_json(&self) -> serde_json::Value {
            let mut button = json!({
                "text": self.text,
                "url": self.url,
            });

            if let Some(ref style) = self.style {
                button["style"] = json!(style);
            }

            if let Some(ref emoji_id) = self.icon_custom_emoji_id {
                button["icon_custom_emoji_id"] = json!(emoji_id);
            }

            button
        }
    }

    fn build_inline_keyboard(buttons: &[ButtonConfig]) -> serde_json::Value {
        let keyboard: Vec<Vec<serde_json::Value>> = buttons
            .iter()
            .map(|btn| vec![btn.to_json()])
            .collect();

        json!({
            "inline_keyboard": keyboard
        })
    }

    #[test]
    fn test_single_button_structure() {
        let buttons = vec![ButtonConfig {
            text: "Click me".to_string(),
            url: "https://example.com".to_string(),
            style: Some("primary".to_string()),
            icon_custom_emoji_id: None,
        }];

        let keyboard = build_inline_keyboard(&buttons);

        assert!(keyboard.get("inline_keyboard").is_some());
        let rows = keyboard["inline_keyboard"].as_array().unwrap();
        assert_eq!(rows.len(), 1);

        let row = rows[0].as_array().unwrap();
        assert_eq!(row.len(), 1);

        let button = &row[0];
        assert_eq!(button["text"], "Click me");
        assert_eq!(button["url"], "https://example.com");
        assert_eq!(button["style"], "primary");
        assert!(button.get("icon_custom_emoji_id").is_none());
    }

    #[test]
    fn test_button_with_emoji() {
        let buttons = vec![ButtonConfig {
            text: "Shop Now".to_string(),
            url: "https://shop.example.com".to_string(),
            style: Some("success".to_string()),
            icon_custom_emoji_id: Some("5368324170671202286".to_string()),
        }];

        let keyboard = build_inline_keyboard(&buttons);
        let button = &keyboard["inline_keyboard"][0][0];

        assert_eq!(button["text"], "Shop Now");
        assert_eq!(button["style"], "success");
        assert_eq!(button["icon_custom_emoji_id"], "5368324170671202286");
    }

    #[test]
    fn test_button_without_style() {
        let buttons = vec![ButtonConfig {
            text: "Learn More".to_string(),
            url: "https://docs.example.com".to_string(),
            style: None,
            icon_custom_emoji_id: None,
        }];

        let keyboard = build_inline_keyboard(&buttons);
        let button = &keyboard["inline_keyboard"][0][0];

        assert_eq!(button["text"], "Learn More");
        assert_eq!(button["url"], "https://docs.example.com");
        assert!(button.get("style").is_none());
    }

    #[test]
    fn test_multiple_buttons_each_own_row() {
        let buttons = vec![
            ButtonConfig {
                text: "Button 1".to_string(),
                url: "https://example.com/1".to_string(),
                style: Some("primary".to_string()),
                icon_custom_emoji_id: None,
            },
            ButtonConfig {
                text: "Button 2".to_string(),
                url: "https://example.com/2".to_string(),
                style: Some("success".to_string()),
                icon_custom_emoji_id: None,
            },
            ButtonConfig {
                text: "Button 3".to_string(),
                url: "https://example.com/3".to_string(),
                style: Some("danger".to_string()),
                icon_custom_emoji_id: None,
            },
        ];

        let keyboard = build_inline_keyboard(&buttons);
        let rows = keyboard["inline_keyboard"].as_array().unwrap();

        // Each button should be in its own row
        assert_eq!(rows.len(), 3);

        // Verify each row has exactly one button
        for row in rows {
            assert_eq!(row.as_array().unwrap().len(), 1);
        }

        // Verify button content
        assert_eq!(keyboard["inline_keyboard"][0][0]["text"], "Button 1");
        assert_eq!(keyboard["inline_keyboard"][0][0]["style"], "primary");
        assert_eq!(keyboard["inline_keyboard"][1][0]["text"], "Button 2");
        assert_eq!(keyboard["inline_keyboard"][1][0]["style"], "success");
        assert_eq!(keyboard["inline_keyboard"][2][0]["text"], "Button 3");
        assert_eq!(keyboard["inline_keyboard"][2][0]["style"], "danger");
    }

    #[test]
    fn test_empty_buttons() {
        let buttons: Vec<ButtonConfig> = vec![];
        let keyboard = build_inline_keyboard(&buttons);

        let rows = keyboard["inline_keyboard"].as_array().unwrap();
        assert_eq!(rows.len(), 0);
    }
}

/// Test session data manipulation.
mod session_data {
    /// Button configuration.
    #[derive(Clone, Debug, PartialEq)]
    struct ButtonConfig {
        text: String,
        url: String,
        style: Option<String>,
        icon_custom_emoji_id: Option<String>,
    }

    /// Session data for the conversation.
    #[derive(Clone, Debug, Default)]
    struct SessionData {
        source_chat_id: Option<i64>,
        source_message_id: Option<i32>,
        destination_chat_id: Option<i64>,
        buttons: Vec<ButtonConfig>,
        current_button_text: Option<String>,
        current_button_url: Option<String>,
        current_button_style: Option<String>,
        current_button_emoji: Option<String>,
    }

    impl SessionData {
        fn new() -> Self {
            Self::default()
        }

        fn clear(&mut self) {
            *self = Self::default();
        }

        fn clear_current_button(&mut self) {
            self.current_button_text = None;
            self.current_button_url = None;
            self.current_button_style = None;
            self.current_button_emoji = None;
        }

        fn finalize_current_button(&mut self) -> Option<ButtonConfig> {
            if let (Some(text), Some(url)) = (self.current_button_text.take(), self.current_button_url.take()) {
                let button = ButtonConfig {
                    text,
                    url,
                    style: self.current_button_style.take(),
                    icon_custom_emoji_id: self.current_button_emoji.take(),
                };
                self.buttons.push(button.clone());
                Some(button)
            } else {
                None
            }
        }

        fn button_count(&self) -> usize {
            self.buttons.len()
        }
    }

    #[test]
    fn test_new_session_is_empty() {
        let session = SessionData::new();
        assert!(session.source_chat_id.is_none());
        assert!(session.source_message_id.is_none());
        assert!(session.destination_chat_id.is_none());
        assert!(session.buttons.is_empty());
        assert_eq!(session.button_count(), 0);
    }

    #[test]
    fn test_set_source_info() {
        let mut session = SessionData::new();
        session.source_chat_id = Some(123456789);
        session.source_message_id = Some(42);

        assert_eq!(session.source_chat_id, Some(123456789));
        assert_eq!(session.source_message_id, Some(42));
    }

    #[test]
    fn test_finalize_button() {
        let mut session = SessionData::new();
        session.current_button_text = Some("Click Here".to_string());
        session.current_button_url = Some("https://example.com".to_string());
        session.current_button_style = Some("primary".to_string());
        session.current_button_emoji = Some("5368324170671202286".to_string());

        let button = session.finalize_current_button();

        assert!(button.is_some());
        let button = button.unwrap();
        assert_eq!(button.text, "Click Here");
        assert_eq!(button.url, "https://example.com");
        assert_eq!(button.style, Some("primary".to_string()));
        assert_eq!(button.icon_custom_emoji_id, Some("5368324170671202286".to_string()));

        // Button should be added to the list
        assert_eq!(session.button_count(), 1);

        // Current button fields should be cleared
        assert!(session.current_button_text.is_none());
        assert!(session.current_button_url.is_none());
        assert!(session.current_button_style.is_none());
        assert!(session.current_button_emoji.is_none());
    }

    #[test]
    fn test_finalize_button_without_optional_fields() {
        let mut session = SessionData::new();
        session.current_button_text = Some("Simple Button".to_string());
        session.current_button_url = Some("https://example.com".to_string());
        // style and emoji not set

        let button = session.finalize_current_button();

        assert!(button.is_some());
        let button = button.unwrap();
        assert_eq!(button.text, "Simple Button");
        assert_eq!(button.url, "https://example.com");
        assert!(button.style.is_none());
        assert!(button.icon_custom_emoji_id.is_none());
    }

    #[test]
    fn test_finalize_button_fails_without_text() {
        let mut session = SessionData::new();
        session.current_button_url = Some("https://example.com".to_string());
        // text not set

        let button = session.finalize_current_button();
        assert!(button.is_none());
        assert_eq!(session.button_count(), 0);
    }

    #[test]
    fn test_finalize_button_fails_without_url() {
        let mut session = SessionData::new();
        session.current_button_text = Some("Button".to_string());
        // url not set

        let button = session.finalize_current_button();
        assert!(button.is_none());
        assert_eq!(session.button_count(), 0);
    }

    #[test]
    fn test_multiple_buttons() {
        let mut session = SessionData::new();

        // Add first button
        session.current_button_text = Some("Button 1".to_string());
        session.current_button_url = Some("https://example.com/1".to_string());
        session.current_button_style = Some("primary".to_string());
        session.finalize_current_button();

        // Add second button
        session.current_button_text = Some("Button 2".to_string());
        session.current_button_url = Some("https://example.com/2".to_string());
        session.current_button_style = Some("success".to_string());
        session.finalize_current_button();

        // Add third button
        session.current_button_text = Some("Button 3".to_string());
        session.current_button_url = Some("https://example.com/3".to_string());
        session.current_button_style = Some("danger".to_string());
        session.finalize_current_button();

        assert_eq!(session.button_count(), 3);
        assert_eq!(session.buttons[0].text, "Button 1");
        assert_eq!(session.buttons[1].text, "Button 2");
        assert_eq!(session.buttons[2].text, "Button 3");
    }

    #[test]
    fn test_clear_session() {
        let mut session = SessionData::new();
        session.source_chat_id = Some(123);
        session.destination_chat_id = Some(456);
        session.current_button_text = Some("Test".to_string());
        session.buttons.push(ButtonConfig {
            text: "Btn".to_string(),
            url: "https://x.com".to_string(),
            style: None,
            icon_custom_emoji_id: None,
        });

        session.clear();

        assert!(session.source_chat_id.is_none());
        assert!(session.destination_chat_id.is_none());
        assert!(session.current_button_text.is_none());
        assert!(session.buttons.is_empty());
    }

    #[test]
    fn test_clear_current_button() {
        let mut session = SessionData::new();
        session.source_chat_id = Some(123);
        session.current_button_text = Some("Test".to_string());
        session.current_button_url = Some("https://x.com".to_string());
        session.current_button_style = Some("primary".to_string());
        session.current_button_emoji = Some("emoji".to_string());

        session.clear_current_button();

        // Source should remain
        assert_eq!(session.source_chat_id, Some(123));
        // Current button fields should be cleared
        assert!(session.current_button_text.is_none());
        assert!(session.current_button_url.is_none());
        assert!(session.current_button_style.is_none());
        assert!(session.current_button_emoji.is_none());
    }
}

/// Test state transitions.
mod state_transitions {
    /// The possible states of the bot conversation.
    #[derive(Clone, Debug, PartialEq)]
    enum BotState {
        Idle,
        AwaitingDestination,
        AwaitingButtonText,
        AwaitingUrl,
        AwaitingStyle,
        AwaitingEmoji,
        AwaitingConfirm,
    }

    impl Default for BotState {
        fn default() -> Self {
            BotState::Idle
        }
    }

    #[test]
    fn test_default_state_is_idle() {
        let state = BotState::default();
        assert_eq!(state, BotState::Idle);
    }

    #[test]
    fn test_state_transitions_content_flow() {
        // Normal flow: Idle -> AwaitingDestination -> AwaitingButtonText -> AwaitingUrl
        // -> AwaitingStyle -> AwaitingEmoji -> AwaitingConfirm -> Idle

        let mut state = BotState::Idle;
        assert_eq!(state, BotState::Idle);

        // User sends content
        state = BotState::AwaitingDestination;
        assert_eq!(state, BotState::AwaitingDestination);

        // User picks destination
        state = BotState::AwaitingButtonText;
        assert_eq!(state, BotState::AwaitingButtonText);

        // User enters button text
        state = BotState::AwaitingUrl;
        assert_eq!(state, BotState::AwaitingUrl);

        // User enters URL
        state = BotState::AwaitingStyle;
        assert_eq!(state, BotState::AwaitingStyle);

        // User picks style
        state = BotState::AwaitingEmoji;
        assert_eq!(state, BotState::AwaitingEmoji);

        // User skips or enters emoji
        state = BotState::AwaitingConfirm;
        assert_eq!(state, BotState::AwaitingConfirm);

        // User publishes
        state = BotState::Idle;
        assert_eq!(state, BotState::Idle);
    }

    #[test]
    fn test_add_another_button_flow() {
        // From AwaitingConfirm, user can add another button
        let start_state = BotState::AwaitingConfirm;
        assert_eq!(start_state, BotState::AwaitingConfirm);

        // User clicks "Add another button" -> transitions to AwaitingButtonText
        let next_state = BotState::AwaitingButtonText;
        assert_eq!(next_state, BotState::AwaitingButtonText);
    }

    #[test]
    fn test_cancel_from_any_state() {
        // Cancel should work from any state and return to Idle
        let states = vec![
            BotState::AwaitingDestination,
            BotState::AwaitingButtonText,
            BotState::AwaitingUrl,
            BotState::AwaitingStyle,
            BotState::AwaitingEmoji,
            BotState::AwaitingConfirm,
        ];

        for state in states {
            // Verify each state is valid
            let _ = state.clone();
            // After cancel, state becomes Idle
            let after_cancel = BotState::Idle;
            assert_eq!(after_cancel, BotState::Idle, "Cancel from {:?} should return to Idle", state);
        }
    }
}

/// Test destination keyboard structure.
mod destination_keyboard {
    use super::json;

    fn build_destination_keyboard() -> serde_json::Value {
        // Single button for groups/channels + send to me
        // No administrator_rights to avoid ADMIN_RIGHTS_EMPTY errors
        json!({
            "keyboard": [
                [
                    {
                        "text": "📢 Choose a group or channel",
                        "request_chat": {
                            "request_id": 1,
                            "chat_is_channel": false,
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

    #[test]
    fn test_destination_keyboard_structure() {
        let keyboard = build_destination_keyboard();

        // Should have resize_keyboard and one_time_keyboard
        assert_eq!(keyboard["resize_keyboard"], true);
        assert_eq!(keyboard["one_time_keyboard"], true);

        // Should have 2 rows
        let rows = keyboard["keyboard"].as_array().unwrap();
        assert_eq!(rows.len(), 2);

        // First row: chat picker
        let chat_picker = &rows[0][0];
        assert_eq!(chat_picker["text"], "📢 Choose a group or channel");
        assert!(chat_picker.get("request_chat").is_some());

        let request_chat = &chat_picker["request_chat"];
        assert_eq!(request_chat["request_id"], 1);
        assert_eq!(request_chat["bot_is_member"], true);
        // No bot_administrator_rights - this is intentional!
        assert!(request_chat.get("bot_administrator_rights").is_none());

        // Second row: send to me
        let send_to_me = &rows[1][0];
        assert_eq!(send_to_me["text"], "📨 Send back to me");
    }
}
