//! Proxy rules CRUD endpoints.

use crate::api::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRule {
    pub id: String,
    pub tool_pattern: String,
    pub action: RuleAction,
    pub priority: i32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleAction {
    Allow,
    Block { reason: String },
    RateLimit { max_calls: u32, window_secs: u64 },
    Log,
}

#[derive(Deserialize)]
pub struct CreateRuleRequest {
    pub tool_pattern: String,
    pub action: RuleAction,
    pub priority: Option<i32>,
}

#[derive(Serialize)]
pub struct RulesResponse {
    pub rules: Vec<ProxyRule>,
}

pub async fn list_rules(
    State(state): State<AppState>,
) -> Result<Json<RulesResponse>, (StatusCode, String)> {
    let conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, tool_pattern, action, priority, enabled FROM proxy_rules ORDER BY priority DESC",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rules: Vec<ProxyRule> = stmt
        .query_map([], |row| {
            let action_json: String = row.get(2)?;
            let action: RuleAction = serde_json::from_str(&action_json).unwrap_or(RuleAction::Log);

            Ok(ProxyRule {
                id: row.get(0)?,
                tool_pattern: row.get(1)?,
                action,
                priority: row.get(3)?,
                enabled: row.get::<_, i32>(4)? != 0,
            })
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(RulesResponse { rules }))
}

pub async fn create_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateRuleRequest>,
) -> Result<Json<ProxyRule>, (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();
    let action_json =
        serde_json::to_string(&req.action).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    conn.execute(
        "INSERT INTO proxy_rules (id, tool_pattern, action, priority) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, req.tool_pattern, action_json, req.priority.unwrap_or(0)],
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ProxyRule {
        id,
        tool_pattern: req.tool_pattern,
        action: req.action,
        priority: req.priority.unwrap_or(0),
        enabled: true,
    }))
}

pub async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rows = conn
        .execute("DELETE FROM proxy_rules WHERE id = ?1", [&id])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows == 0 {
        return Err((StatusCode::NOT_FOUND, "Rule not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateRuleRequest>,
) -> Result<Json<ProxyRule>, (StatusCode, String)> {
    let action_json =
        serde_json::to_string(&req.action).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let conn = state
        .db
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rows = conn
        .execute(
            "UPDATE proxy_rules SET tool_pattern = ?1, action = ?2, priority = ?3, updated_at = datetime('now') WHERE id = ?4",
            rusqlite::params![req.tool_pattern, action_json, req.priority.unwrap_or(0), id],
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows == 0 {
        return Err((StatusCode::NOT_FOUND, "Rule not found".to_string()));
    }

    Ok(Json(ProxyRule {
        id,
        tool_pattern: req.tool_pattern,
        action: req.action,
        priority: req.priority.unwrap_or(0),
        enabled: true,
    }))
}
