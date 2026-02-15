//! Keyboard building utilities.

pub mod builder;
pub mod destination;

pub use builder::{
    build_confirm_keyboard, build_emoji_keyboard, build_inline_keyboard, build_style_keyboard,
    map_style_callback, raw_answer_callback_query, raw_copy_message, raw_edit_message_text,
    raw_send_message, style_display_name, validate_url,
};
pub use destination::{build_destination_keyboard, build_remove_keyboard, SEND_TO_ME_TEXT};
