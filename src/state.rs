//! Finite State Machine (FSM) states for the bot conversation flow.

use serde::{Deserialize, Serialize};

use crate::models::SessionData;

/// The possible states of the bot conversation.
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub enum BotState {
    /// Idle state: waiting for user to send content.
    #[default]
    Idle,
    /// Content received, waiting for user to pick a destination chat.
    AwaitingDestination {
        data: SessionData,
    },
    /// Destination chosen, waiting for button text input.
    AwaitingButtonText {
        data: SessionData,
    },
    /// Button text received, waiting for URL input.
    AwaitingUrl {
        data: SessionData,
    },
    /// URL received, waiting for style selection.
    AwaitingStyle {
        data: SessionData,
    },
    /// Style picked, waiting for optional emoji ID or skip.
    AwaitingEmoji {
        data: SessionData,
    },
    /// Button configured, waiting for confirmation: Publish / Add another / Cancel.
    AwaitingConfirm {
        data: SessionData,
    },
}

impl BotState {
    /// Get a reference to the session data if this state contains it.
    pub fn data(&self) -> Option<&SessionData> {
        match self {
            BotState::Idle => None,
            BotState::AwaitingDestination { data } => Some(data),
            BotState::AwaitingButtonText { data } => Some(data),
            BotState::AwaitingUrl { data } => Some(data),
            BotState::AwaitingStyle { data } => Some(data),
            BotState::AwaitingEmoji { data } => Some(data),
            BotState::AwaitingConfirm { data } => Some(data),
        }
    }

    /// Get a mutable reference to the session data if this state contains it.
    pub fn data_mut(&mut self) -> Option<&mut SessionData> {
        match self {
            BotState::Idle => None,
            BotState::AwaitingDestination { data } => Some(data),
            BotState::AwaitingButtonText { data } => Some(data),
            BotState::AwaitingUrl { data } => Some(data),
            BotState::AwaitingStyle { data } => Some(data),
            BotState::AwaitingEmoji { data } => Some(data),
            BotState::AwaitingConfirm { data } => Some(data),
        }
    }

    /// Get the name of the current state for logging.
    pub fn name(&self) -> &'static str {
        match self {
            BotState::Idle => "Idle",
            BotState::AwaitingDestination { .. } => "AwaitingDestination",
            BotState::AwaitingButtonText { .. } => "AwaitingButtonText",
            BotState::AwaitingUrl { .. } => "AwaitingUrl",
            BotState::AwaitingStyle { .. } => "AwaitingStyle",
            BotState::AwaitingEmoji { .. } => "AwaitingEmoji",
            BotState::AwaitingConfirm { .. } => "AwaitingConfirm",
        }
    }
}
