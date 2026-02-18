//! Database connection pool management.

use anyhow::{Context, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;
use std::sync::Arc;

/// Type alias for the connection pool.
pub type DbPool = Pool<SqliteConnectionManager>;

/// Database wrapper providing connection pool access.
#[derive(Clone)]
pub struct Database {
    pool: Arc<DbPool>,
}

impl Database {
    /// Create a new database connection pool.
    ///
    /// Creates the database file and parent directories if they don't exist.
    pub fn new(database_url: &str) -> Result<Self> {
        // Handle in-memory database for testing
        let manager = if database_url == ":memory:" {
            SqliteConnectionManager::memory()
        } else {
            // Ensure parent directory exists
            if let Some(parent) = Path::new(database_url).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)
                        .context("Failed to create database directory")?;
                }
            }
            SqliteConnectionManager::file(database_url)
        };

        let pool = Pool::builder()
            .max_size(10)
            .min_idle(Some(1))
            .build(manager)
            .context("Failed to create database connection pool")?;

        // Configure SQLite for better performance and reliability
        {
            let conn = pool.get().context("Failed to get database connection")?;
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA busy_timeout = 5000;",
            )
            .context("Failed to configure SQLite pragmas")?;
        }

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Get a connection from the pool.
    pub fn conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().context("Failed to get database connection")
    }

    /// Get access to the underlying pool.
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_memory_database() {
        let db = Database::new(":memory:").expect("Failed to create memory database");
        let conn = db.conn().expect("Failed to get connection");
        let result: i32 = conn
            .query_row("SELECT 1", [], |row| row.get(0))
            .expect("Failed to execute query");
        assert_eq!(result, 1);
    }
}
