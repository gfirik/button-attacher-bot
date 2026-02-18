//! Configuration loading from environment variables.

use anyhow::{Context, Result};

/// Application configuration loaded from environment.
#[derive(Clone, Debug)]
pub struct Config {
    /// Telegram Bot API token.
    pub bot_token: String,
    /// Path to SQLite database file.
    pub database_url: String,
    /// List of admin user IDs who can access admin commands.
    pub admin_user_ids: Vec<i64>,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required:
    /// - `BOT_TOKEN`: Telegram bot token
    ///
    /// Optional:
    /// - `DATABASE_URL`: Path to SQLite database (default: ./data/bot.db)
    /// - `ADMIN_USER_IDS`: Comma-separated list of admin Telegram user IDs
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists (ignore errors if it doesn't)
        let _ = dotenvy::dotenv();

        let bot_token = std::env::var("BOT_TOKEN")
            .context("BOT_TOKEN environment variable is required. Please set it in .env file or environment.")?;

        if bot_token.is_empty() || bot_token == "your_bot_token_here" {
            anyhow::bail!("BOT_TOKEN is not set or still contains placeholder value. Please set your actual bot token.");
        }

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "./data/bot.db".to_string());

        let admin_user_ids: Vec<i64> = std::env::var("ADMIN_USER_IDS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        if admin_user_ids.is_empty() {
            log::warn!("No ADMIN_USER_IDS configured. Admin commands will be inaccessible.");
        } else {
            log::info!("Configured {} admin user(s)", admin_user_ids.len());
        }

        Ok(Self {
            bot_token,
            database_url,
            admin_user_ids,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_admin_ids() {
        std::env::set_var("BOT_TOKEN", "test_token");
        std::env::set_var("ADMIN_USER_IDS", "123456789, 987654321, 111222333");

        let config = Config::from_env().unwrap();

        assert_eq!(config.admin_user_ids.len(), 3);
        assert!(config.admin_user_ids.contains(&123456789));
        assert!(config.admin_user_ids.contains(&987654321));
        assert!(config.admin_user_ids.contains(&111222333));

        // Cleanup
        std::env::remove_var("ADMIN_USER_IDS");
    }
}
