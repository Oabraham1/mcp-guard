//! Claude Desktop MCP configuration discovery.

use crate::discovery::clients::generic::parse_mcp_servers;
use crate::discovery::config::ServerConfig;
use crate::discovery::traits::McpClientDiscovery;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct ClaudeDesktopDiscovery;

impl McpClientDiscovery for ClaudeDesktopDiscovery {
    fn name(&self) -> &str {
        "Claude Desktop"
    }

    fn config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // macOS: ~/Library/Application Support/Claude/claude_desktop_config.json
        #[cfg(target_os = "macos")]
        if let Some(home) = dirs::home_dir() {
            paths.push(
                home.join("Library/Application Support/Claude/claude_desktop_config.json"),
            );
        }

        // Linux: ~/.config/Claude/claude_desktop_config.json
        #[cfg(target_os = "linux")]
        if let Some(config) = dirs::config_dir() {
            paths.push(config.join("Claude/claude_desktop_config.json"));
        }

        // Windows: %APPDATA%\Claude\claude_desktop_config.json
        #[cfg(target_os = "windows")]
        if let Some(appdata) = dirs::config_dir() {
            paths.push(appdata.join("Claude\\claude_desktop_config.json"));
        }

        paths
    }

    fn parse(&self, content: &str, config_path: &Path) -> Result<Vec<ServerConfig>> {
        parse_mcp_servers(content, config_path, self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_correct_name() {
        let discovery = ClaudeDesktopDiscovery;
        assert_eq!(discovery.name(), "Claude Desktop");
    }

    #[test]
    fn config_paths_not_empty() {
        let discovery = ClaudeDesktopDiscovery;
        let paths = discovery.config_paths();
        assert!(!paths.is_empty());
    }

    #[test]
    fn parse_sample_config() {
        let discovery = ClaudeDesktopDiscovery;
        let json = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
                },
                "memory": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-memory"]
                }
            }
        }"#;

        let servers = discovery
            .parse(json, &PathBuf::from("/test/config.json"))
            .unwrap();
        assert_eq!(servers.len(), 2);
    }
}
