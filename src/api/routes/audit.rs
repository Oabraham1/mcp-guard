//! Audit log endpoints.

use crate::api::state::AppState;
use crate::db::AuditLog;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AuditQuery {
    pub server: Option<String>,
    pub tool: Option<String>,
    pub blocked: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Serialize)]
pub struct AuditResponse {
    pub entries: Vec<AuditEntryInfo>,
    pub total: i64,
}

#[derive(Serialize)]
pub struct AuditEntryInfo {
    pub id: i64,
    pub timestamp: String,
    pub server_name: String,
    pub tool_name: String,
    pub tool_args: Option<serde_json::Value>,
    pub blocked: bool,
    pub block_reason: Option<String>,
    pub duration_ms: u64,
}

pub async fn list_audit(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<AuditResponse>, (StatusCode, String)> {
    let pool = state.db.as_ref().clone();
    let audit_log = AuditLog::new(pool.clone());

    let limit = query.limit.unwrap_or(50).min(1000);
    let offset = query.offset.unwrap_or(0);

    let entries =
        if query.server.is_some() || query.tool.is_some() || query.blocked.unwrap_or(false) {
            audit_log
                .search(
                    query.server.as_deref(),
                    query.tool.as_deref(),
                    query.blocked.unwrap_or(false),
                    limit,
                )
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        } else {
            audit_log
                .list(limit, offset)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        };

    let total = audit_log
        .count()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let entry_infos: Vec<AuditEntryInfo> = entries
        .into_iter()
        .map(|e| AuditEntryInfo {
            id: e.id,
            timestamp: e.timestamp.to_rfc3339(),
            server_name: e.server_name,
            tool_name: e.tool_name,
            tool_args: e.tool_args,
            blocked: e.blocked,
            block_reason: e.block_reason,
            duration_ms: e.duration_ms,
        })
        .collect();

    Ok(Json(AuditResponse {
        entries: entry_infos,
        total,
    }))
}
