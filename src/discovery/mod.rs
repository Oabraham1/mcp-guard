//! MCP server discovery across all supported clients.

pub mod clients;
pub mod config;
pub mod traits;

pub use config::{ServerConfig, TransportType};
pub use traits::McpClientDiscovery;

use crate::error::Result;
use std::collections::HashMap;
use tracing::warn;

pub fn all_clients() -> Vec<Box<dyn McpClientDiscovery>> {
    vec![
        Box::new(clients::ClaudeDesktopDiscovery),
        Box::new(clients::CursorDiscovery),
        Box::new(clients::WindsurfDiscovery),
        Box::new(clients::ZedDiscovery),
        Box::new(clients::ClineDiscovery),
        Box::new(clients::ContinueDiscovery),
        Box::new(clients::VsCodeDiscovery),
        Box::new(clients::RooCodeDiscovery),
        Box::new(clients::ClaudeCodeDiscovery),
    ]
}

pub fn discover_all() -> Result<Vec<ServerConfig>> {
    let mut servers: Vec<ServerConfig> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();

    for client in all_clients() {
        match client.discover() {
            Ok(client_servers) => {
                for server in client_servers {
                    // Dedupe by server name - first discovery wins
                    if let Some(&idx) = seen.get(&server.name) {
                        warn!(
                            server_name = %server.name,
                            existing_source = %servers[idx].display_source(),
                            duplicate_source = %server.display_source(),
                            "Duplicate server name found, using first discovery"
                        );
                    } else {
                        seen.insert(server.name.clone(), servers.len());
                        servers.push(server);
                    }
                }
            }
            Err(e) => {
                warn!(
                    client = %client.name(),
                    error = %e,
                    "Failed to discover servers from client"
                );
            }
        }
    }

    Ok(servers)
}

pub fn discover_from_client(client_name: &str) -> Result<Vec<ServerConfig>> {
    let clients = all_clients();
    let client = clients
        .iter()
        .find(|c| c.name().eq_ignore_ascii_case(client_name))
        .ok_or_else(|| crate::error::Error::Other(format!("Unknown client: {}", client_name)))?;

    client.discover()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_clients_returns_expected_count() {
        let clients = all_clients();
        assert_eq!(clients.len(), 9);
    }

    #[test]
    fn all_clients_have_unique_names() {
        let clients = all_clients();
        let names: Vec<_> = clients.iter().map(|c| c.name()).collect();
        let mut unique_names = names.clone();
        unique_names.sort();
        unique_names.dedup();
        assert_eq!(names.len(), unique_names.len());
    }
}
