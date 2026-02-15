//! Configuration loading from environment variables.

use anyhow::{Context, Result};

/// Application configuration loaded from environment.
#[derive(Clone, Debug)]
pub struct Config {
    /// Telegram Bot API token.
    pub bot_token: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Requires `BOT_TOKEN` to be set.
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists (ignore errors if it doesn't)
        let _ = dotenvy::dotenv();

        let bot_token = std::env::var("BOT_TOKEN")
            .context("BOT_TOKEN environment variable is required. Please set it in .env file or environment.")?;

        if bot_token.is_empty() || bot_token == "your_bot_token_here" {
            anyhow::bail!("BOT_TOKEN is not set or still contains placeholder value. Please set your actual bot token.");
        }

        Ok(Self { bot_token })
    }
}
