//! Roo Code MCP configuration discovery.

use crate::discovery::clients::generic::parse_mcp_servers;
use crate::discovery::config::ServerConfig;
use crate::discovery::traits::McpClientDiscovery;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct RooCodeDiscovery;

impl McpClientDiscovery for RooCodeDiscovery {
    fn name(&self) -> &str {
        "Roo Code"
    }

    fn config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(config) = dirs::config_dir() {
            // Similar path structure to Cline
            paths.push(
                config.join("Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/mcp_settings.json"),
            );
        }

        #[cfg(target_os = "windows")]
        if let Some(appdata) = dirs::config_dir() {
            paths.push(
                appdata.join("Code\\User\\globalStorage\\rooveterinaryinc.roo-cline\\settings\\mcp_settings.json"),
            );
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
        let discovery = RooCodeDiscovery;
        assert_eq!(discovery.name(), "Roo Code");
    }
}
