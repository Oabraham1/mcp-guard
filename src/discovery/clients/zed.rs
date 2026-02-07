//! Zed editor MCP configuration discovery.
//!
//! Zed uses a different config format with `context_servers` key in settings.json.

use crate::discovery::config::{ConfigSource, ServerConfig, TransportType};
use crate::discovery::traits::McpClientDiscovery;
use crate::error::{Error, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct ZedSettings {
    #[serde(default)]
    context_servers: HashMap<String, ZedContextServer>,
}

#[derive(Debug, Deserialize)]
struct ZedContextServer {
    command: Option<String>,
    args: Option<Vec<String>>,
    #[serde(default)]
    env: HashMap<String, String>,
}

pub struct ZedDiscovery;

impl McpClientDiscovery for ZedDiscovery {
    fn name(&self) -> &str {
        "Zed"
    }

    fn config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(config) = dirs::config_dir() {
            paths.push(config.join("zed/settings.json"));
        }

        paths
    }

    fn parse(&self, content: &str, config_path: &Path) -> Result<Vec<ServerConfig>> {
        let settings: ZedSettings = serde_json::from_str(content)
            .map_err(|e| Error::config_parse(config_path, e.to_string()))?;

        let mut servers = Vec::new();

        for (name, raw) in settings.context_servers {
            let command = raw.command.ok_or_else(|| {
                Error::invalid_server_config(format!(
                    "Server '{}' is missing 'command' field",
                    name
                ))
            })?;

            let server = ServerConfig {
                name,
                command,
                args: raw.args.unwrap_or_default(),
                env: raw.env,
                transport: TransportType::Stdio,
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
        let discovery = ZedDiscovery;
        assert_eq!(discovery.name(), "Zed");
    }

    #[test]
    fn parse_zed_config() {
        let discovery = ZedDiscovery;
        let json = r#"{
            "context_servers": {
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
    fn empty_context_servers_returns_empty() {
        let discovery = ZedDiscovery;
        let json = r#"{}"#;

        let servers = discovery
            .parse(json, &PathBuf::from("/test/settings.json"))
            .unwrap();
        assert!(servers.is_empty());
    }
}
