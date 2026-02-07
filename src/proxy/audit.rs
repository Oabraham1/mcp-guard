//! Audit logging for proxy tool calls.

use crate::db::{AuditEntry, AuditLog, DbPool};
use chrono::Utc;
use std::time::Duration;

pub struct ProxyAudit {
    log: AuditLog,
}

impl ProxyAudit {
    pub fn new(pool: DbPool) -> Self {
        Self {
            log: AuditLog::new(pool),
        }
    }

    #[allow(clippy::too_many_arguments)] // All fields needed to construct audit entry
    pub fn record_call(
        &self,
        server_name: &str,
        tool_name: &str,
        tool_args: Option<serde_json::Value>,
        result: Option<serde_json::Value>,
        blocked: bool,
        block_reason: Option<String>,
        duration: Duration,
    ) {
        let entry = AuditEntry {
            id: 0,
            timestamp: Utc::now(),
            server_name: server_name.to_string(),
            tool_name: tool_name.to_string(),
            tool_args,
            result,
            blocked,
            block_reason,
            duration_ms: duration.as_millis() as u64,
        };

        if let Err(e) = self.log.insert(&entry) {
            tracing::error!(error = %e, "Failed to record audit entry");
        }
    }
}
