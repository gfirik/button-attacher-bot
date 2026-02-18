//! Analytics tracking and reporting.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::Database;

/// Event types for analytics tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// User started the bot
    Start,
    /// User sent content to attach buttons
    ContentReceived,
    /// User selected a destination
    DestinationSelected,
    /// User configured a button
    ButtonConfigured,
    /// User published content
    Published,
    /// User cancelled an operation
    Cancelled,
    /// User requested help
    Help,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Start => "start",
            EventType::ContentReceived => "content_received",
            EventType::DestinationSelected => "destination_selected",
            EventType::ButtonConfigured => "button_configured",
            EventType::Published => "published",
            EventType::Cancelled => "cancelled",
            EventType::Help => "help",
        }
    }
}

/// User statistics for admin reporting.
#[derive(Debug, Serialize)]
pub struct UserStats {
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_seen_at: String,
    pub total_messages: i64,
    pub total_publications: i64,
}

/// Overall bot statistics.
#[derive(Debug, Serialize)]
pub struct BotStats {
    pub total_users: i64,
    pub active_users_24h: i64,
    pub active_users_7d: i64,
    pub active_users_30d: i64,
    pub total_publications: i64,
    pub publications_24h: i64,
    pub publications_7d: i64,
    pub total_buttons_configured: i64,
    pub most_popular_style: Option<String>,
    pub top_destination_chats: Vec<(i64, String, i64)>,
}

/// Analytics service for tracking and reporting.
#[derive(Clone)]
pub struct Analytics {
    db: Database,
}

impl Analytics {
    /// Create a new analytics service.
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Track a user interaction, creating or updating user record.
    pub fn track_user(
        &self,
        user_id: i64,
        username: Option<&str>,
        first_name: Option<&str>,
        last_name: Option<&str>,
        language_code: Option<&str>,
    ) -> Result<()> {
        let conn = self.db.conn()?;

        conn.execute(
            r#"
            INSERT INTO users (user_id, username, first_name, last_name, language_code, total_messages)
            VALUES (?1, ?2, ?3, ?4, ?5, 1)
            ON CONFLICT(user_id) DO UPDATE SET
                username = COALESCE(?2, username),
                first_name = COALESCE(?3, first_name),
                last_name = COALESCE(?4, last_name),
                language_code = COALESCE(?5, language_code),
                last_seen_at = datetime('now'),
                total_messages = total_messages + 1
            "#,
            rusqlite::params![user_id, username, first_name, last_name, language_code],
        )
        .context("Failed to track user")?;

        // Update daily stats
        let today = Utc::now().format("%Y-%m-%d").to_string();
        conn.execute(
            r#"
            INSERT INTO daily_stats (date, stat_key, stat_value)
            VALUES (?1, 'active_users', 1)
            ON CONFLICT(date, stat_key) DO UPDATE SET stat_value = stat_value + 1
            "#,
            rusqlite::params![today],
        )?;

        Ok(())
    }

    /// Track an event.
    pub fn track_event(
        &self,
        user_id: i64,
        event_type: EventType,
        event_data: Option<&str>,
    ) -> Result<()> {
        let conn = self.db.conn()?;

        conn.execute(
            "INSERT INTO events (user_id, event_type, event_data) VALUES (?1, ?2, ?3)",
            rusqlite::params![user_id, event_type.as_str(), event_data],
        )
        .context("Failed to track event")?;

        // Update daily stats for this event type
        let today = Utc::now().format("%Y-%m-%d").to_string();
        conn.execute(
            r#"
            INSERT INTO daily_stats (date, stat_key, stat_value)
            VALUES (?1, ?2, 1)
            ON CONFLICT(date, stat_key) DO UPDATE SET stat_value = stat_value + 1
            "#,
            rusqlite::params![today, event_type.as_str()],
        )?;

        Ok(())
    }

    /// Track a successful publication.
    pub fn track_publication(
        &self,
        user_id: i64,
        destination_chat_id: i64,
        button_count: usize,
        styles: &[Option<String>],
    ) -> Result<()> {
        let conn = self.db.conn()?;

        // Update user publication count
        conn.execute(
            "UPDATE users SET total_publications = total_publications + 1 WHERE user_id = ?1",
            rusqlite::params![user_id],
        )?;

        // Track destination chat
        conn.execute(
            r#"
            INSERT INTO destination_chats (chat_id, total_publications)
            VALUES (?1, 1)
            ON CONFLICT(chat_id) DO UPDATE SET
                last_used_at = datetime('now'),
                total_publications = total_publications + 1
            "#,
            rusqlite::params![destination_chat_id],
        )?;

        // Track button styles
        for style in styles {
            conn.execute(
                "INSERT INTO button_styles (user_id, style, has_emoji) VALUES (?1, ?2, 0)",
                rusqlite::params![user_id, style.as_deref()],
            )?;
        }

        // Track the event
        let event_data = serde_json::json!({
            "destination_chat_id": destination_chat_id,
            "button_count": button_count,
        })
        .to_string();

        self.track_event(user_id, EventType::Published, Some(&event_data))?;

        Ok(())
    }

    /// Check if a user is blocked.
    pub fn is_user_blocked(&self, user_id: i64) -> Result<bool> {
        let conn = self.db.conn()?;

        let blocked: i32 = conn
            .query_row(
                "SELECT is_blocked FROM users WHERE user_id = ?1",
                rusqlite::params![user_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(blocked == 1)
    }

    /// Block or unblock a user.
    pub fn set_user_blocked(&self, user_id: i64, blocked: bool) -> Result<()> {
        let conn = self.db.conn()?;

        conn.execute(
            "UPDATE users SET is_blocked = ?1 WHERE user_id = ?2",
            rusqlite::params![blocked as i32, user_id],
        )
        .context("Failed to update user blocked status")?;

        Ok(())
    }

    /// Get overall bot statistics.
    pub fn get_bot_stats(&self) -> Result<BotStats> {
        let conn = self.db.conn()?;

        let total_users: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .unwrap_or(0);

        let active_users_24h: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE last_seen_at > datetime('now', '-1 day')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let active_users_7d: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE last_seen_at > datetime('now', '-7 days')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let active_users_30d: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE last_seen_at > datetime('now', '-30 days')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let total_publications: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total_publications), 0) FROM users",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let publications_24h: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE event_type = 'published' AND created_at > datetime('now', '-1 day')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let publications_7d: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE event_type = 'published' AND created_at > datetime('now', '-7 days')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let total_buttons_configured: i64 = conn
            .query_row("SELECT COUNT(*) FROM button_styles", [], |row| row.get(0))
            .unwrap_or(0);

        let most_popular_style: Option<String> = conn
            .query_row(
                r#"
                SELECT style FROM button_styles
                GROUP BY style
                ORDER BY COUNT(*) DESC
                LIMIT 1
                "#,
                [],
                |row| row.get(0),
            )
            .ok();

        let mut stmt = conn.prepare(
            r#"
            SELECT chat_id, COALESCE(title, 'Unknown'), total_publications
            FROM destination_chats
            ORDER BY total_publications DESC
            LIMIT 5
            "#,
        )?;

        let top_destination_chats: Vec<(i64, String, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(BotStats {
            total_users,
            active_users_24h,
            active_users_7d,
            active_users_30d,
            total_publications,
            publications_24h,
            publications_7d,
            total_buttons_configured,
            most_popular_style,
            top_destination_chats,
        })
    }

    /// Get a list of users with pagination.
    pub fn get_users(&self, limit: usize, offset: usize) -> Result<Vec<UserStats>> {
        let conn = self.db.conn()?;

        let mut stmt = conn.prepare(
            r#"
            SELECT user_id, username, first_name, last_seen_at, total_messages, total_publications
            FROM users
            ORDER BY last_seen_at DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )?;

        let users: Vec<UserStats> = stmt
            .query_map(rusqlite::params![limit as i64, offset as i64], |row| {
                Ok(UserStats {
                    user_id: row.get(0)?,
                    username: row.get(1)?,
                    first_name: row.get(2)?,
                    last_seen_at: row.get(3)?,
                    total_messages: row.get(4)?,
                    total_publications: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(users)
    }

    /// Get user count.
    pub fn get_user_count(&self) -> Result<i64> {
        let conn = self.db.conn()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get recent events for a specific user.
    pub fn get_user_events(&self, user_id: i64, limit: usize) -> Result<Vec<(String, String, String)>> {
        let conn = self.db.conn()?;

        let mut stmt = conn.prepare(
            r#"
            SELECT event_type, COALESCE(event_data, ''), created_at
            FROM events
            WHERE user_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )?;

        let events: Vec<(String, String, String)> = stmt
            .query_map(rusqlite::params![user_id, limit as i64], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{run_migrations, Database};

    fn setup_test_db() -> (Database, Analytics) {
        let db = Database::new(":memory:").expect("Failed to create test database");
        {
            let conn = db.conn().expect("Failed to get connection");
            run_migrations(&conn).expect("Failed to run migrations");
        }
        let analytics = Analytics::new(db.clone());
        (db, analytics)
    }

    #[test]
    fn test_track_user() {
        let (_db, analytics) = setup_test_db();
        analytics
            .track_user(123456, Some("testuser"), Some("Test"), Some("User"), Some("en"))
            .expect("Failed to track user");

        let stats = analytics.get_bot_stats().expect("Failed to get stats");
        assert_eq!(stats.total_users, 1);
    }

    #[test]
    fn test_track_event() {
        let (_db, analytics) = setup_test_db();
        analytics
            .track_user(123456, None, None, None, None)
            .expect("Failed to track user");
        analytics
            .track_event(123456, EventType::Start, None)
            .expect("Failed to track event");
    }

    #[test]
    fn test_get_users() {
        let (_db, analytics) = setup_test_db();
        analytics
            .track_user(123456, Some("user1"), Some("User"), Some("One"), None)
            .expect("Failed to track user");
        analytics
            .track_user(789012, Some("user2"), Some("User"), Some("Two"), None)
            .expect("Failed to track user");

        let users = analytics.get_users(10, 0).expect("Failed to get users");
        assert_eq!(users.len(), 2);
    }
}
