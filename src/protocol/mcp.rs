//! MCP (Model Context Protocol) message types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

impl Default for InitializeParams {
    fn default() -> Self {
        Self {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "mcp-guard".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
}

impl ServerCapabilities {
    pub fn has_tools(&self) -> bool {
        self.tools.is_some()
    }

    pub fn has_resources(&self) -> bool {
        self.resources.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsListResult {
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesListResult {
    pub resources: Vec<Resource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

pub const NOTIFICATION_INITIALIZED: &str = "notifications/initialized";

pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const TOOLS_LIST: &str = "tools/list";
    pub const RESOURCES_LIST: &str = "resources/list";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_params_default() {
        let params = InitializeParams::default();
        assert_eq!(params.protocol_version, MCP_PROTOCOL_VERSION);
        assert_eq!(params.client_info.name, "mcp-guard");
    }

    #[test]
    fn test_tool_serialization() {
        let tool = Tool {
            name: "read_file".to_string(),
            description: Some("Read a file from disk".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                },
                "required": ["path"]
            }),
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("\"name\":\"read_file\""));
        assert!(json.contains("\"description\":\"Read a file from disk\""));
        assert!(json.contains("\"inputSchema\""));
    }

    #[test]
    fn test_server_capabilities() {
        let caps = ServerCapabilities {
            tools: Some(serde_json::json!({"listChanged": true})),
            resources: None,
            prompts: None,
            logging: None,
            experimental: None,
        };

        assert!(caps.has_tools());
        assert!(!caps.has_resources());
    }

    #[test]
    fn test_initialize_result_deserialization() {
        let json = r#"{
            "protocolVersion": "2025-11-25",
            "capabilities": {
                "tools": {"listChanged": true},
                "resources": {"subscribe": true, "listChanged": false}
            },
            "serverInfo": {
                "name": "test-server",
                "version": "1.0.0"
            }
        }"#;

        let result: InitializeResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.protocol_version, "2025-11-25");
        assert_eq!(result.server_info.name, "test-server");
        assert!(result.capabilities.has_tools());
        assert!(result.capabilities.has_resources());
    }
}
