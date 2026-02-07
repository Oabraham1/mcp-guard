//! VS Code MCP configuration discovery.

use crate::discovery::clients::generic::parse_mcp_servers;
use crate::discovery::config::ServerConfig;
use crate::discovery::traits::McpClientDiscovery;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct VsCodeDiscovery;

impl McpClientDiscovery for VsCodeDiscovery {
    fn name(&self) -> &str {
        "VS Code"
    }

    fn config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Check current directory for .vscode/mcp.json (workspace-level)
        paths.push(PathBuf::from(".vscode/mcp.json"));

        if let Some(config) = dirs::config_dir() {
            // User-level settings
            #[cfg(target_os = "macos")]
            paths.push(config.join("Code/User/mcp.json"));

            #[cfg(target_os = "linux")]
            paths.push(config.join("Code/User/mcp.json"));

            #[cfg(target_os = "windows")]
            paths.push(config.join("Code\\User\\mcp.json"));
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
        let discovery = VsCodeDiscovery;
        assert_eq!(discovery.name(), "VS Code");
    }

    #[test]
    fn includes_workspace_config() {
        let discovery = VsCodeDiscovery;
        let paths = discovery.config_paths();
        assert!(paths.iter().any(|p| p.ends_with(".vscode/mcp.json")));
    }
}
