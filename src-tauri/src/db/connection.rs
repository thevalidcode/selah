//! SQLite connection handling.

use rusqlite::Connection;
use std::path::Path;
use tracing::debug;

/// The versioned list of migrations. Add new migrations to the end — never
/// edit an applied migration, because it would corrupt existing databases.
pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial_schema",
    sql: include_str!("../../migrations/0001_initial.sql"),
}];

/// A single ordered schema migration.
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

/// Opens (and creates if needed) the SQLite database at `path`.
///
/// The caller owns the returned connection; every subsystem accesses the
/// database through a shared `Mutex<Database>` managed by [`crate::state`].
pub fn open(path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(path)?;

    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;

    debug!("opened sqlite database at {}", path.display());
    Ok(conn)
}

/// A place to store the resolved connection inside application state.
pub type DbConnection = std::sync::Mutex<Connection>;
