//! Database module for SQLite persistence and analytics.

mod pool;
mod schema;
mod analytics;

pub use pool::Database;
pub use schema::run_migrations;
pub use analytics::{Analytics, EventType};
