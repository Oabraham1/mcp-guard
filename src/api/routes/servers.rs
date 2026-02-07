//! Server discovery endpoints.

use crate::discovery::{discover_all, discover_from_client, ServerConfig};
use axum::{
    extract::Query,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ServersQuery {
    pub client: Option<String>,
}

#[derive(Serialize)]
pub struct ServersResponse {
    pub servers: Vec<ServerInfo>,
}

#[derive(Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub source: String,
    pub transport: String,
}

impl From<ServerConfig> for ServerInfo {
    fn from(s: ServerConfig) -> Self {
        let transport = match &s.transport {
            crate::discovery::TransportType::Stdio => "stdio".to_string(),
            crate::discovery::TransportType::Sse { url } => format!("sse:{}", url),
            crate::discovery::TransportType::StreamableHttp { url } => format!("http:{}", url),
        };

        let source = s.display_source();

        Self {
            name: s.name,
            command: s.command,
            args: s.args,
            source,
            transport,
        }
    }
}

pub async fn list_servers(
    Query(query): Query<ServersQuery>,
) -> Result<Json<ServersResponse>, (StatusCode, String)> {
    let servers = if let Some(client) = query.client {
        discover_from_client(&client).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    } else {
        discover_all().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    Ok(Json(ServersResponse {
        servers: servers.into_iter().map(ServerInfo::from).collect(),
    }))
}
