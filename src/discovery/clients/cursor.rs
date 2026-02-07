//! Cursor IDE MCP configuration discovery.

use crate::discovery::clients::generic::parse_mcp_servers;
use crate::discovery::config::ServerConfig;
use crate::discovery::traits::McpClientDiscovery;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct CursorDiscovery;

impl McpClientDiscovery for CursorDiscovery {
    fn name(&self) -> &str {
        "Cursor"
    }

    fn config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(home) = dirs::home_dir() {
            // ~/.cursor/mcp.json
            paths.push(home.join(".cursor/mcp.json"));
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
        let discovery = CursorDiscovery;
        assert_eq!(discovery.name(), "Cursor");
    }

    #[test]
    fn config_paths_correct() {
        let discovery = CursorDiscovery;
        let paths = discovery.config_paths();
        assert!(!paths.is_empty());
        assert!(paths[0].ends_with(".cursor/mcp.json"));
    }
}
