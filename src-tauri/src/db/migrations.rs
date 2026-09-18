//! Schema migrations.
//!
//! Migrations are embedded at compile time and applied in order inside a
//! transaction. The `schema_migrations` table records what has already been
//! applied so repeated launches are idempotent.

use crate::errors::AppError;
use rusqlite::Connection;
use tracing::{debug, info};

use super::connection::MIGRATIONS;

/// Applies pending migrations to the given connection.
pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version    INTEGER PRIMARY KEY,
            name       TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    for migration in MIGRATIONS {
        let applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            [migration.version],
            |row| row.get(0),
        )?;

        if applied {
            continue;
        }

        info!(
            version = migration.version,
            name = migration.name,
            "applying database migration"
        );

        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(migration.sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, name, applied_at)
             VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            rusqlite::params![migration.version, migration.name],
        )?;
        tx.commit()?;

        debug!(version = migration.version, "database migration applied");
    }

    Ok(())
}
