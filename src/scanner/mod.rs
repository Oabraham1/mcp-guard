//! MCP server scanner for security vulnerabilities.

pub mod report;
pub mod snapshot;
pub mod threats;

pub use report::{ResourceInfo, ScanResult, Severity, Threat, ToolInfo};
pub use snapshot::SnapshotStore;
pub use threats::{all_detectors, ThreatDetector};

use crate::discovery::ServerConfig;
use crate::error::{Error, Result};
use crate::protocol::mcp::{methods, InitializeParams};
use crate::protocol::transport::StdioTransport;
use crate::protocol::{McpTransport, Request};
use chrono::Utc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

pub struct Scanner {
    detectors: Vec<Box<dyn ThreatDetector>>,
    snapshot_store: Option<SnapshotStore>,
    timeout: Duration,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            detectors: all_detectors(),
            snapshot_store: SnapshotStore::new().ok(),
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn scan(&self, server: &ServerConfig) -> Result<ScanResult> {
        let start = Instant::now();
        info!(server = %server.name, "Starting scan");

        let (tools, resources) = match self.fetch_server_info(server).await {
            Ok(info) => info,
            Err(e) => {
                warn!(server = %server.name, error = %e, "Failed to connect to server");
                return Err(Error::scan_failed(&server.name, e.to_string()));
            }
        };

        debug!(server = %server.name, tools = tools.len(), resources = resources.len(), "Fetched server info");

        // Run threat detectors
        let mut threats: Vec<Threat> = self
            .detectors
            .iter()
            .flat_map(|d| d.detect(server, &tools, &resources))
            .collect();

        // Check for description drift
        let snapshot_diff = if let Some(ref store) = self.snapshot_store {
            match store.compare(&server.name, &tools) {
                Ok(Some(diff)) => {
                    let drift_threats = threats::DescriptionDriftDetector::threats_from_diff(&diff);
                    threats.extend(drift_threats);
                    Some(diff)
                }
                Ok(None) => None,
                Err(e) => {
                    warn!(error = %e, "Failed to compare snapshots");
                    None
                }
            }
        } else {
            None
        };

        // Save new snapshot
        if let Some(ref store) = self.snapshot_store {
            if let Err(e) = store.save(&server.name, &tools) {
                warn!(error = %e, "Failed to save snapshot");
            }
        }

        // Sort threats by severity
        threats.sort_by(|a, b| a.severity.cmp(&b.severity));

        let scan_duration = start.elapsed();
        info!(
            server = %server.name,
            tools = tools.len(),
            threats = threats.len(),
            duration_ms = scan_duration.as_millis(),
            "Scan complete"
        );

        Ok(ScanResult {
            server: server.clone(),
            tools,
            resources,
            threats,
            snapshot_diff,
            scan_duration,
            scanned_at: Utc::now(),
        })
    }

    async fn fetch_server_info(
        &self,
        server: &ServerConfig,
    ) -> Result<(Vec<ToolInfo>, Vec<ResourceInfo>)> {
        let mut transport =
            StdioTransport::spawn(&server.command, &server.args, &server.env).await?;
        transport.set_timeout(self.timeout);

        // Initialize
        let init_params = InitializeParams::default();
        let init_request = Request::new(
            methods::INITIALIZE,
            Some(serde_json::to_value(&init_params)?),
        );

        let init_response = transport.send(init_request).await?;
        let init_result: crate::protocol::mcp::InitializeResult =
            serde_json::from_value(init_response.result)?;

        debug!(
            server_name = %init_result.server_info.name,
            version = %init_result.server_info.version,
            "Server initialized"
        );

        // Send initialized notification
        let notification = crate::protocol::Notification::new(
            crate::protocol::mcp::NOTIFICATION_INITIALIZED,
            Some(serde_json::json!({})),
        );
        transport.send_notification(notification).await?;

        // Fetch tools
        let tools = if init_result.capabilities.has_tools() {
            let tools_request = Request::new(methods::TOOLS_LIST, Some(serde_json::json!({})));
            let tools_response = transport.send(tools_request).await?;
            let tools_result: crate::protocol::mcp::ToolsListResult =
                serde_json::from_value(tools_response.result)?;
            tools_result.tools.into_iter().map(ToolInfo::from).collect()
        } else {
            Vec::new()
        };

        // Fetch resources
        let resources = if init_result.capabilities.has_resources() {
            let resources_request =
                Request::new(methods::RESOURCES_LIST, Some(serde_json::json!({})));
            let resources_response = transport.send(resources_request).await?;
            let resources_result: crate::protocol::mcp::ResourcesListResult =
                serde_json::from_value(resources_response.result)?;
            resources_result
                .resources
                .into_iter()
                .map(ResourceInfo::from)
                .collect()
        } else {
            Vec::new()
        };

        transport.close().await?;

        Ok((tools, resources))
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_creation() {
        let scanner = Scanner::new();
        assert!(!scanner.detectors.is_empty());
    }

    #[test]
    fn scanner_with_timeout() {
        let scanner = Scanner::new().with_timeout(Duration::from_secs(60));
        assert_eq!(scanner.timeout, Duration::from_secs(60));
    }
}
