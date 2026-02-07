//! Continue.dev MCP configuration discovery.

use crate::discovery::clients::generic::parse_mcp_servers;
use crate::discovery::config::ServerConfig;
use crate::discovery::traits::McpClientDiscovery;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct ContinueDiscovery;

impl McpClientDiscovery for ContinueDiscovery {
    fn name(&self) -> &str {
        "Continue"
    }

    fn config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(home) = dirs::home_dir() {
            // ~/.continue/config.json
            paths.push(home.join(".continue/config.json"));
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
        let discovery = ContinueDiscovery;
        assert_eq!(discovery.name(), "Continue");
    }

    #[test]
    fn config_paths_correct() {
        let discovery = ContinueDiscovery;
        let paths = discovery.config_paths();
        assert!(!paths.is_empty());
        assert!(paths[0].ends_with(".continue/config.json"));
    }
}
