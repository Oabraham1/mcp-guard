//! Detects servers without authentication configured.

use crate::discovery::{ServerConfig, TransportType};
use crate::scanner::report::{ResourceInfo, Severity, Threat, ThreatCategory, ToolInfo};
use crate::scanner::threats::ThreatDetector;

pub struct NoAuthDetector;

impl ThreatDetector for NoAuthDetector {
    fn detect(
        &self,
        server: &ServerConfig,
        _tools: &[ToolInfo],
        _resources: &[ResourceInfo],
    ) -> Vec<Threat> {
        let mut threats = Vec::new();

        match &server.transport {
            TransportType::Stdio => {
                // For STDIO servers, check if there are any auth-related env vars
                let has_auth_env = server.env.keys().any(|k| {
                    let k_lower = k.to_lowercase();
                    k_lower.contains("token")
                        || k_lower.contains("key")
                        || k_lower.contains("secret")
                        || k_lower.contains("auth")
                        || k_lower.contains("password")
                });

                if !has_auth_env {
                    // Info-level for local STDIO servers - it's often fine
                    threats.push(
                        Threat::new(
                            "NO-AUTH-LOCAL",
                            Severity::Info,
                            ThreatCategory::NoAuth,
                            "No authentication configured",
                        )
                        .with_message(format!(
                            "Server '{}' has no authentication environment variables configured",
                            server.name
                        ))
                        .with_evidence("No TOKEN, KEY, SECRET, AUTH, or PASSWORD env vars found")
                        .with_remediation(
                            "Consider adding authentication if this server accesses sensitive resources",
                        ),
                    );
                }
            }
            TransportType::Sse { url } | TransportType::StreamableHttp { url } => {
                // Remote servers MUST have auth - this is critical
                let has_auth_env = server.env.keys().any(|k| {
                    let k_lower = k.to_lowercase();
                    k_lower.contains("token")
                        || k_lower.contains("key")
                        || k_lower.contains("auth")
                        || k_lower.contains("bearer")
                });

                if !has_auth_env {
                    threats.push(
                        Threat::new(
                            "NO-AUTH-REMOTE",
                            Severity::Critical,
                            ThreatCategory::NoAuth,
                            "Remote server without authentication",
                        )
                        .with_message(format!(
                            "Remote server '{}' at {} has no authentication configured",
                            server.name, url
                        ))
                        .with_evidence(format!("URL: {}, no auth headers/tokens found", url))
                        .with_remediation(
                            "Add authentication tokens or API keys for remote MCP servers. Never expose remote servers without auth.",
                        ),
                    );
                }
            }
        }

        threats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_server_no_auth_is_info() {
        let detector = NoAuthDetector;
        let server = ServerConfig::new("test", "npx");

        let threats = detector.detect(&server, &[], &[]);
        assert!(!threats.is_empty());
        assert_eq!(threats[0].severity, Severity::Info);
    }

    #[test]
    fn local_server_with_token_no_warning() {
        let detector = NoAuthDetector;
        let mut server = ServerConfig::new("test", "npx");
        server
            .env
            .insert("GITHUB_TOKEN".to_string(), "xxx".to_string());

        let threats = detector.detect(&server, &[], &[]);
        assert!(threats.is_empty());
    }

    #[test]
    fn remote_server_no_auth_is_critical() {
        let detector = NoAuthDetector;
        let mut server = ServerConfig::new("test", "");
        server.transport = TransportType::Sse {
            url: "https://example.com/mcp".to_string(),
        };

        let threats = detector.detect(&server, &[], &[]);
        assert!(!threats.is_empty());
        assert_eq!(threats[0].severity, Severity::Critical);
    }

    #[test]
    fn remote_server_with_auth_no_warning() {
        let detector = NoAuthDetector;
        let mut server = ServerConfig::new("test", "");
        server.transport = TransportType::Sse {
            url: "https://example.com/mcp".to_string(),
        };
        server
            .env
            .insert("AUTH_TOKEN".to_string(), "xxx".to_string());

        let threats = detector.detect(&server, &[], &[]);
        assert!(threats.is_empty());
    }
}
