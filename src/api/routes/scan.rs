//! Scan endpoints.

use crate::discovery::{discover_all, ServerConfig};
use crate::scanner::{ScanResult, Scanner};
use axum::{
    extract::Query,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize)]
pub struct ScanQuery {
    pub server: Option<String>,
    pub timeout: Option<u64>,
}

#[derive(Serialize)]
pub struct ScanResponse {
    pub results: Vec<ScanResultSummary>,
    pub total_threats: usize,
    pub servers_scanned: usize,
    pub servers_failed: usize,
}

#[derive(Serialize)]
pub struct ScanResultSummary {
    pub server_name: String,
    pub tool_count: usize,
    pub resource_count: usize,
    pub threat_count: usize,
    pub threats: Vec<ThreatInfo>,
    pub scan_duration_ms: u64,
}

#[derive(Serialize)]
pub struct ThreatInfo {
    pub id: String,
    pub severity: String,
    pub category: String,
    pub title: String,
    pub message: String,
    pub tool_name: Option<String>,
}

impl From<&ScanResult> for ScanResultSummary {
    fn from(r: &ScanResult) -> Self {
        Self {
            server_name: r.server.name.clone(),
            tool_count: r.tools.len(),
            resource_count: r.resources.len(),
            threat_count: r.threats.len(),
            threats: r.threats.iter().map(|t| ThreatInfo {
                id: t.id.clone(),
                severity: t.severity.to_string(),
                category: t.category.to_string(),
                title: t.title.clone(),
                message: t.message.clone(),
                tool_name: t.tool_name.clone(),
            }).collect(),
            scan_duration_ms: r.scan_duration.as_millis() as u64,
        }
    }
}

pub async fn run_scan(
    Query(query): Query<ScanQuery>,
) -> Result<Json<ScanResponse>, (StatusCode, String)> {
    let servers: Vec<ServerConfig> = if let Some(server_name) = query.server {
        // Scan specific server by name
        let all_servers = discover_all()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        all_servers
            .into_iter()
            .filter(|s| s.name == server_name)
            .collect()
    } else {
        discover_all().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    if servers.is_empty() {
        return Ok(Json(ScanResponse {
            results: vec![],
            total_threats: 0,
            servers_scanned: 0,
            servers_failed: 0,
        }));
    }

    let timeout = Duration::from_secs(query.timeout.unwrap_or(30));
    let scanner = Scanner::new().with_timeout(timeout);

    let mut results = Vec::new();
    let mut servers_failed = 0;

    for server in &servers {
        match scanner.scan(server).await {
            Ok(result) => results.push(result),
            Err(_) => servers_failed += 1,
        }
    }

    let total_threats: usize = results.iter().map(|r| r.threats.len()).sum();
    let summaries: Vec<ScanResultSummary> = results.iter().map(ScanResultSummary::from).collect();

    Ok(Json(ScanResponse {
        results: summaries,
        total_threats,
        servers_scanned: results.len(),
        servers_failed,
    }))
}
