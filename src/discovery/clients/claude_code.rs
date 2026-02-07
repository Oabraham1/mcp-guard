//! Claude Code MCP configuration discovery.
//!
//! Claude Code stores MCP config in ~/.claude/settings.json or project-level .mcp.json

use crate::discovery::config::{ConfigSource, ServerConfig, TransportType};
use crate::discovery::traits::McpClientDiscovery;
use crate::error::{Error, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct ClaudeCodeSettings {
    #[serde(default, rename = "mcpServers")]
    mcp_servers: HashMap<String, ClaudeCodeServerConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeCodeServerConfig {
    command: Option<String>,
    args: Option<Vec<String>>,
    #[serde(default)]
    env: HashMap<String, String>,
    url: Option<String>,
}

pub struct ClaudeCodeDiscovery;

impl McpClientDiscovery for ClaudeCodeDiscovery {
    fn name(&self) -> &str {
        "Claude Code"
    }

    fn config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Project-level .mcp.json takes precedence
        paths.push(PathBuf::from(".mcp.json"));

        if let Some(home) = dirs::home_dir() {
            // User-level ~/.claude/settings.json
            paths.push(home.join(".claude/settings.json"));
        }

        paths
    }

    fn parse(&self, content: &str, config_path: &Path) -> Result<Vec<ServerConfig>> {
        let settings: ClaudeCodeSettings = serde_json::from_str(content)
            .map_err(|e| Error::config_parse(config_path, e.to_string()))?;

        let mut servers = Vec::new();

        for (name, raw) in settings.mcp_servers {
            let transport = if let Some(url) = &raw.url {
                TransportType::Sse { url: url.clone() }
            } else {
                TransportType::Stdio
            };

            let command = match &transport {
                TransportType::Stdio => raw.command.ok_or_else(|| {
                    Error::invalid_server_config(format!(
                        "Server '{}' is missing 'command' field",
                        name
                    ))
                })?,
                _ => raw.command.unwrap_or_default(),
            };

            let server = ServerConfig {
                name,
                command,
                args: raw.args.unwrap_or_default(),
                env: raw.env,
                transport,
                source: ConfigSource::Client {
                    name: self.name().to_string(),
                    config_path: config_path.to_path_buf(),
                },
            };

            servers.push(server);
        }

        Ok(servers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_correct_name() {
        let discovery = ClaudeCodeDiscovery;
        assert_eq!(discovery.name(), "Claude Code");
    }

    #[test]
    fn parse_claude_code_config() {
        let discovery = ClaudeCodeDiscovery;
        let json = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/"]
                }
            }
        }"#;

        let servers = discovery
            .parse(json, &PathBuf::from("/test/settings.json"))
            .unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "filesystem");
    }

    #[test]
    fn includes_project_level_config() {
        let discovery = ClaudeCodeDiscovery;
        let paths = discovery.config_paths();
        assert!(paths.iter().any(|p| p.ends_with(".mcp.json")));
    }
}
