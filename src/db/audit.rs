//! Audit log storage and queries.

use crate::db::DbPool;
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub server_name: String,
    pub tool_name: String,
    pub tool_args: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub blocked: bool,
    pub block_reason: Option<String>,
    pub duration_ms: u64,
}

pub struct AuditLog {
    pool: DbPool,
}

impl AuditLog {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn insert(&self, entry: &AuditEntry) -> Result<i64> {
        let conn = self.pool.get()?;

        conn.execute(
            r#"
            INSERT INTO audit_log (timestamp, server_name, tool_name, tool_args, result, blocked, block_reason, duration_ms)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            rusqlite::params![
                entry.timestamp.to_rfc3339(),
                entry.server_name,
                entry.tool_name,
                entry.tool_args.as_ref().map(|v| v.to_string()),
                entry.result.as_ref().map(|v| v.to_string()),
                entry.blocked as i32,
                entry.block_reason,
                entry.duration_ms as i64,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<AuditEntry>> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, timestamp, server_name, tool_name, tool_args, result, blocked, block_reason, duration_ms
            FROM audit_log
            ORDER BY timestamp DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )?;

        let entries = stmt
            .query_map([limit as i64, offset as i64], |row| {
                Ok(AuditEntry {
                    id: row.get(0)?,
                    timestamp: parse_datetime(row.get::<_, String>(1)?),
                    server_name: row.get(2)?,
                    tool_name: row.get(3)?,
                    tool_args: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    result: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    blocked: row.get::<_, i32>(6)? != 0,
                    block_reason: row.get(7)?,
                    duration_ms: row.get::<_, i64>(8)? as u64,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    pub fn search(
        &self,
        server: Option<&str>,
        tool: Option<&str>,
        blocked_only: bool,
        limit: usize,
    ) -> Result<Vec<AuditEntry>> {
        let conn = self.pool.get()?;

        let mut query = String::from(
            "SELECT id, timestamp, server_name, tool_name, tool_args, result, blocked, block_reason, duration_ms FROM audit_log WHERE 1=1",
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(s) = server {
            query.push_str(" AND server_name = ?");
            params.push(Box::new(s.to_string()));
        }
        if let Some(t) = tool {
            query.push_str(" AND tool_name = ?");
            params.push(Box::new(t.to_string()));
        }
        if blocked_only {
            query.push_str(" AND blocked = 1");
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT ?");
        params.push(Box::new(limit as i64));

        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let entries = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(AuditEntry {
                    id: row.get(0)?,
                    timestamp: parse_datetime(row.get::<_, String>(1)?),
                    server_name: row.get(2)?,
                    tool_name: row.get(3)?,
                    tool_args: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    result: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    blocked: row.get::<_, i32>(6)? != 0,
                    block_reason: row.get(7)?,
                    duration_ms: row.get::<_, i64>(8)? as u64,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    pub fn count(&self) -> Result<i64> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))?;
        Ok(count)
    }
}

fn parse_datetime(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_pool;
    use tempfile::tempdir;

    fn test_pool() -> (tempfile::TempDir, DbPool) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = create_pool(&db_path).unwrap();
        (dir, pool)
    }

    #[test]
    fn insert_and_list_entries() {
        let (_dir, pool) = test_pool();
        let log = AuditLog::new(pool);

        let entry = AuditEntry {
            id: 0,
            timestamp: Utc::now(),
            server_name: "test-server".to_string(),
            tool_name: "read_file".to_string(),
            tool_args: Some(serde_json::json!({"path": "/tmp/test"})),
            result: Some(serde_json::json!({"content": "hello"})),
            blocked: false,
            block_reason: None,
            duration_ms: 100,
        };

        let id = log.insert(&entry).unwrap();
        assert!(id > 0);

        let entries = log.list(10, 0).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].server_name, "test-server");
    }

    #[test]
    fn search_by_server() {
        let (_dir, pool) = test_pool();
        let log = AuditLog::new(pool);

        for server in ["server-a", "server-b"] {
            let entry = AuditEntry {
                id: 0,
                timestamp: Utc::now(),
                server_name: server.to_string(),
                tool_name: "test".to_string(),
                tool_args: None,
                result: None,
                blocked: false,
                block_reason: None,
                duration_ms: 50,
            };
            log.insert(&entry).unwrap();
        }

        let results = log.search(Some("server-a"), None, false, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].server_name, "server-a");
    }
}
