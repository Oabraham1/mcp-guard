//! Discovery traits for MCP client implementations.

use crate::discovery::config::ServerConfig;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub trait McpClientDiscovery: Send + Sync {
    fn name(&self) -> &str;

    fn config_paths(&self) -> Vec<PathBuf>;

    fn parse(&self, content: &str, config_path: &Path) -> Result<Vec<ServerConfig>>;

    fn discover(&self) -> Result<Vec<ServerConfig>> {
        for path in self.config_paths() {
            if path.exists() {
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    crate::error::Error::ConfigRead {
                        path: path.clone(),
                        source: e,
                    }
                })?;

                return self.parse(&content, &path);
            }
        }
        Ok(Vec::new())
    }
}
