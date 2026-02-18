//! Database schema and migrations.

use anyhow::{Context, Result};
use rusqlite::Connection;

/// Current schema version.
const SCHEMA_VERSION: i32 = 1;

/// Run all database migrations.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migrations table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .context("Failed to create migrations table")?;

    // Get current version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    log::info!(
        "Database schema version: {}, target: {}",
        current_version,
        SCHEMA_VERSION
    );

    // Apply migrations
    if current_version < 1 {
        migrate_v1(conn)?;
    }

    Ok(())
}

/// Migration v1: Initial schema with users, events, and daily stats.
fn migrate_v1(conn: &Connection) -> Result<()> {
    log::info!("Applying migration v1: Initial schema");

    conn.execute_batch(
        r#"
        -- Users table: tracks all users who interact with the bot
        CREATE TABLE IF NOT EXISTS users (
            user_id INTEGER PRIMARY KEY,
            username TEXT,
            first_name TEXT,
            last_name TEXT,
            language_code TEXT,
            first_seen_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_seen_at TEXT NOT NULL DEFAULT (datetime('now')),
            is_blocked INTEGER NOT NULL DEFAULT 0,
            total_messages INTEGER NOT NULL DEFAULT 0,
            total_publications INTEGER NOT NULL DEFAULT 0
        );

        -- Events table: detailed event log for analytics
        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            event_data TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (user_id) REFERENCES users(user_id)
        );

        -- Daily stats table: aggregated daily statistics
        CREATE TABLE IF NOT EXISTS daily_stats (
            date TEXT NOT NULL,
            stat_key TEXT NOT NULL,
            stat_value INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (date, stat_key)
        );

        -- Destination chats: tracks where content is published
        CREATE TABLE IF NOT EXISTS destination_chats (
            chat_id INTEGER PRIMARY KEY,
            chat_type TEXT,
            title TEXT,
            first_used_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_used_at TEXT NOT NULL DEFAULT (datetime('now')),
            total_publications INTEGER NOT NULL DEFAULT 0
        );

        -- Button styles usage: tracks which styles are popular
        CREATE TABLE IF NOT EXISTS button_styles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            style TEXT,
            has_emoji INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (user_id) REFERENCES users(user_id)
        );

        -- Indexes for common queries
        CREATE INDEX IF NOT EXISTS idx_events_user_id ON events(user_id);
        CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at);
        CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
        CREATE INDEX IF NOT EXISTS idx_users_last_seen ON users(last_seen_at);
        CREATE INDEX IF NOT EXISTS idx_daily_stats_date ON daily_stats(date);

        -- Record migration
        INSERT INTO schema_migrations (version) VALUES (1);
        "#,
    )
    .context("Failed to apply migration v1")?;

    log::info!("Migration v1 applied successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations() {
        let conn = Connection::open_in_memory().expect("Failed to open memory database");
        run_migrations(&conn).expect("Failed to run migrations");

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"events".to_string()));
        assert!(tables.contains(&"daily_stats".to_string()));
    }
}
