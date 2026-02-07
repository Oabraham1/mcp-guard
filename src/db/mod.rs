//! SQLite database for storing scan results, audit logs, and proxy rules.

mod audit;
mod migrations;
mod snapshots;

pub use audit::{AuditEntry, AuditLog};
pub use migrations::run_migrations;

use crate::error::{Error, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool(db_path: &PathBuf) -> Result<DbPool> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(4)
        .build(manager)
        .map_err(Error::DatabasePool)?;

    // Run migrations
    let conn = pool.get().map_err(Error::DatabasePool)?;
    run_migrations(&conn)?;

    Ok(pool)
}

pub fn default_db_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| Error::Other("Could not find home directory".to_string()))?;
    Ok(home.join(".mcp-guard").join("mcp-guard.db"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_pool_creates_db() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let pool = create_pool(&db_path).unwrap();
        assert!(db_path.exists());

        let conn = pool.get().unwrap();
        let table_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_count > 0);
    }
}
