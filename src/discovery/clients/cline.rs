//! Cline (Claude Dev) VS Code extension MCP configuration discovery.

use crate::discovery::clients::generic::parse_mcp_servers;
use crate::discovery::config::ServerConfig;
use crate::discovery::traits::McpClientDiscovery;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct ClineDiscovery;

impl McpClientDiscovery for ClineDiscovery {
    fn name(&self) -> &str {
        "Cline"
    }

    fn config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(config) = dirs::config_dir() {
            // Linux/macOS: ~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json
            paths.push(config.join(
                "Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json",
            ));
        }

        // Windows: %APPDATA%\Code\User\globalStorage\saoudrizwan.claude-dev\settings\cline_mcp_settings.json
        #[cfg(target_os = "windows")]
        if let Some(appdata) = dirs::config_dir() {
            paths.push(
                appdata.join("Code\\User\\globalStorage\\saoudrizwan.claude-dev\\settings\\cline_mcp_settings.json"),
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
        let discovery = ClineDiscovery;
        assert_eq!(discovery.name(), "Cline");
    }

    #[test]
    fn config_paths_contain_cline_path() {
        let discovery = ClineDiscovery;
        let paths = discovery.config_paths();
        assert!(!paths.is_empty());
        let path_str = paths[0].to_string_lossy();
        assert!(path_str.contains("saoudrizwan.claude-dev"));
    }
}
