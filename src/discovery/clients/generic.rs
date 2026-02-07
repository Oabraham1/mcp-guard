//! Generic MCP config parser for any JSON file with mcpServers key.

use crate::discovery::config::{ConfigSource, ServerConfig, TransportType};
use crate::error::{Error, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct McpConfig {
    #[serde(default)]
    mcp_servers: HashMap<String, RawServerConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawServerConfig {
    command: Option<String>,
    args: Option<Vec<String>>,
    #[serde(default)]
    env: HashMap<String, String>,
    url: Option<String>,
    transport: Option<String>,
}

pub fn parse_mcp_servers(
    content: &str,
    config_path: &Path,
    client_name: &str,
) -> Result<Vec<ServerConfig>> {
    let config: McpConfig = serde_json::from_str(content)
        .map_err(|e| Error::config_parse(config_path, e.to_string()))?;

    let mut servers = Vec::new();

    for (name, raw) in config.mcp_servers {
        let transport = determine_transport(&raw);

        // Remote servers don't need a command
        let command = match &transport {
            TransportType::Stdio => raw.command.ok_or_else(|| {
                Error::invalid_server_config(format!(
                    "Server '{}' is missing 'command' field",
                    name
                ))
            })?,
            TransportType::Sse { .. } | TransportType::StreamableHttp { .. } => {
                raw.command.unwrap_or_default()
            }
        };

        let server = ServerConfig {
            name,
            command,
            args: raw.args.unwrap_or_default(),
            env: raw.env,
            transport,
            source: ConfigSource::Client {
                name: client_name.to_string(),
                config_path: config_path.to_path_buf(),
            },
        };

        servers.push(server);
    }

    Ok(servers)
}

fn determine_transport(raw: &RawServerConfig) -> TransportType {
    if let Some(url) = &raw.url {
        match raw.transport.as_deref() {
            Some("sse") => TransportType::Sse { url: url.clone() },
            Some("streamable-http") | Some("http") => {
                TransportType::StreamableHttp { url: url.clone() }
            }
            _ => {
                // Default to SSE if URL is provided but transport not specified
                TransportType::Sse { url: url.clone() }
            }
        }
    } else {
        TransportType::Stdio
    }
}

pub struct GenericDiscovery {
    pub path: PathBuf,
}

impl GenericDiscovery {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn parse_file(&self) -> Result<Vec<ServerConfig>> {
        if !self.path.exists() {
            return Err(Error::ConfigNotFound {
                path: self.path.clone(),
            });
        }

        let content = std::fs::read_to_string(&self.path).map_err(|e| Error::ConfigRead {
            path: self.path.clone(),
            source: e,
        })?;

        parse_mcp_servers(&content, &self.path, "generic")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_config() {
        let json = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/"]
                }
            }
        }"#;

        let servers = parse_mcp_servers(json, &PathBuf::from("/test/config.json"), "test").unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "filesystem");
        assert_eq!(servers[0].command, "npx");
        assert_eq!(servers[0].args.len(), 3);
    }

    #[test]
    fn parse_config_with_env() {
        let json = r#"{
            "mcpServers": {
                "github": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-github"],
                    "env": {
                        "GITHUB_TOKEN": "secret"
                    }
                }
            }
        }"#;

        let servers = parse_mcp_servers(json, &PathBuf::from("/test/config.json"), "test").unwrap();
        assert_eq!(
            servers[0].env.get("GITHUB_TOKEN"),
            Some(&"secret".to_string())
        );
    }

    #[test]
    fn parse_remote_server() {
        let json = r#"{
            "mcpServers": {
                "remote": {
                    "url": "http://localhost:8080",
                    "transport": "sse"
                }
            }
        }"#;

        let servers = parse_mcp_servers(json, &PathBuf::from("/test/config.json"), "test").unwrap();
        assert!(
            matches!(&servers[0].transport, TransportType::Sse { url } if url == "http://localhost:8080")
        );
    }

    #[test]
    fn missing_command_for_stdio_fails() {
        let json = r#"{
            "mcpServers": {
                "bad": {
                    "args": ["test"]
                }
            }
        }"#;

        let result = parse_mcp_servers(json, &PathBuf::from("/test/config.json"), "test");
        assert!(result.is_err());
    }
}
