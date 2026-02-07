//! Windsurf (Codeium) MCP configuration discovery.

use crate::discovery::clients::generic::parse_mcp_servers;
use crate::discovery::config::ServerConfig;
use crate::discovery::traits::McpClientDiscovery;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct WindsurfDiscovery;

impl McpClientDiscovery for WindsurfDiscovery {
    fn name(&self) -> &str {
        "Windsurf"
    }

    fn config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(home) = dirs::home_dir() {
            // ~/.codeium/windsurf/mcp_config.json
            paths.push(home.join(".codeium/windsurf/mcp_config.json"));
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
        let discovery = WindsurfDiscovery;
        assert_eq!(discovery.name(), "Windsurf");
    }

    #[test]
    fn config_paths_correct() {
        let discovery = WindsurfDiscovery;
        let paths = discovery.config_paths();
        assert!(!paths.is_empty());
        assert!(paths[0].ends_with(".codeium/windsurf/mcp_config.json"));
    }
}
