//! Database schema migrations.

use crate::error::Result;
use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            server_name TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            description TEXT,
            description_hash TEXT NOT NULL,
            input_schema TEXT,
            scanned_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS scan_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            server_name TEXT NOT NULL,
            server_source TEXT NOT NULL,
            tool_count INTEGER NOT NULL,
            resource_count INTEGER NOT NULL,
            threat_count INTEGER NOT NULL,
            threats_json TEXT NOT NULL,
            scan_duration_ms INTEGER NOT NULL,
            scanned_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            server_name TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            tool_args TEXT,
            result TEXT,
            blocked INTEGER NOT NULL DEFAULT 0,
            block_reason TEXT,
            duration_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS proxy_rules (
            id TEXT PRIMARY KEY,
            tool_pattern TEXT NOT NULL,
            action TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
        CREATE INDEX IF NOT EXISTS idx_audit_server ON audit_log(server_name);
        CREATE INDEX IF NOT EXISTS idx_audit_tool ON audit_log(tool_name);
        CREATE INDEX IF NOT EXISTS idx_snapshots_server_tool ON snapshots(server_name, tool_name);
        CREATE INDEX IF NOT EXISTS idx_scan_results_server ON scan_results(server_name);
        "#,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_migrations_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"audit_log".to_string()));
        assert!(tables.contains(&"proxy_rules".to_string()));
        assert!(tables.contains(&"scan_results".to_string()));
        assert!(tables.contains(&"snapshots".to_string()));
    }

    #[test]
    fn run_migrations_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap(); // Should not fail
    }
}
