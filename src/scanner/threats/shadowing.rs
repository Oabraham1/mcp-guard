//! Detects tool name collisions and similar names across servers.
//! This module is only used in tests for now.

#[cfg(test)]
use crate::discovery::ServerConfig;
#[cfg(test)]
use crate::scanner::report::{Severity, Threat, ThreatCategory, ToolInfo};
#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
pub struct ShadowingDetector {
    similarity_threshold: usize,
}

#[cfg(test)]
impl ShadowingDetector {
    pub fn new() -> Self {
        Self {
            similarity_threshold: 3,
        }
    }

    pub fn detect(&self, all_servers: &[(ServerConfig, Vec<ToolInfo>)]) -> Vec<(String, Threat)> {
        let mut threats = Vec::new();
        let mut tool_registry: HashMap<String, Vec<&str>> = HashMap::new();

        for (server, tools) in all_servers {
            for tool in tools {
                tool_registry
                    .entry(tool.name.clone())
                    .or_default()
                    .push(&server.name);
            }
        }

        for (tool_name, servers) in &tool_registry {
            if servers.len() > 1 {
                let threat = Threat::new(
                    format!("SHADOW-{}", tool_name),
                    Severity::High,
                    ThreatCategory::ToolShadowing,
                    "Tool name collision",
                )
                .with_message(format!(
                    "Tool '{}' is registered by multiple servers: {}",
                    tool_name,
                    servers.join(", ")
                ))
                .with_evidence(format!("Servers: {}", servers.join(", ")))
                .with_remediation(
                    "Rename one of the tools to avoid conflicts. The tool loaded last may shadow earlier ones.",
                )
                .with_tool(tool_name);

                threats.push((servers[0].to_string(), threat));
            }
        }

        let tool_names: Vec<&String> = tool_registry.keys().collect();
        for (i, name1) in tool_names.iter().enumerate() {
            for name2 in tool_names.iter().skip(i + 1) {
                let distance = strsim::levenshtein(name1, name2);
                if distance > 0 && distance <= self.similarity_threshold {
                    let servers1 = &tool_registry[*name1];
                    let servers2 = &tool_registry[*name2];

                    if servers1 != servers2 {
                        let threat = Threat::new(
                            format!("SHADOW-SIM-{}-{}", name1, name2),
                            Severity::Medium,
                            ThreatCategory::ToolShadowing,
                            "Similar tool names detected",
                        )
                        .with_message(format!(
                            "Tools '{}' and '{}' have similar names (distance: {})",
                            name1, name2, distance
                        ))
                        .with_evidence(format!(
                            "'{}' from {}, '{}' from {}",
                            name1,
                            servers1.join(", "),
                            name2,
                            servers2.join(", ")
                        ))
                        .with_remediation(
                            "Verify these are intentionally different tools. Similar names could indicate typosquatting.",
                        );

                        threats.push((servers1[0].to_string(), threat));
                    }
                }
            }
        }

        threats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_server(name: &str, tool_names: &[&str]) -> (ServerConfig, Vec<ToolInfo>) {
        let tools = tool_names
            .iter()
            .map(|&n| ToolInfo {
                name: n.to_string(),
                description: None,
                input_schema: serde_json::json!({}),
            })
            .collect();
        (ServerConfig::new(name, "cmd"), tools)
    }

    #[test]
    fn detects_exact_collision() {
        let detector = ShadowingDetector::new();
        let servers = vec![
            make_server("server1", &["read_file", "write_file"]),
            make_server("server2", &["read_file", "delete_file"]),
        ];

        let threats = detector.detect(&servers);
        assert!(threats.iter().any(|(_, t)| t.title == "Tool name collision"));
    }

    #[test]
    fn detects_similar_names() {
        let detector = ShadowingDetector::new();
        let servers = vec![
            make_server("trusted", &["read_file"]),
            make_server("suspicious", &["read_fi1e"]),
        ];

        let threats = detector.detect(&servers);
        assert!(threats
            .iter()
            .any(|(_, t)| t.title == "Similar tool names detected"));
    }

    #[test]
    fn no_false_positive_for_different_names() {
        let detector = ShadowingDetector::new();
        let servers = vec![
            make_server("server1", &["read_file"]),
            make_server("server2", &["write_database"]),
        ];

        let threats = detector.detect(&servers);
        assert!(threats.is_empty());
    }

    #[test]
    fn same_server_similar_names_not_flagged() {
        let detector = ShadowingDetector::new();
        let servers = vec![make_server("server1", &["read_file", "read_files"])];

        let threats = detector.detect(&servers);
        assert!(threats
            .iter()
            .all(|(_, t)| t.category != ThreatCategory::ToolShadowing));
    }
}
