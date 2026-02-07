//! Detects overly broad permission scopes in tools.

use crate::discovery::ServerConfig;
use crate::scanner::report::{ResourceInfo, Severity, Threat, ThreatCategory, ToolInfo};
use crate::scanner::threats::ThreatDetector;
use regex::Regex;
use std::sync::LazyLock;

static DANGEROUS_PATTERNS: LazyLock<Vec<DangerPattern>> = LazyLock::new(|| {
    vec![
        // Code execution
        DangerPattern::new(
            r"(?i)\b(exec|eval|execute|run|shell|command|spawn|system)\b",
            "Potential code execution capability",
            Severity::High,
            "Code execution tools can run arbitrary commands. Ensure proper input validation and sandboxing.",
        ),
        // Filesystem - broad access
        DangerPattern::new(
            r#"(?i)"path"\s*:\s*\{\s*"type"\s*:\s*"string""#,
            "Accepts arbitrary file paths",
            Severity::Medium,
            "Tools that accept arbitrary paths should validate against allowed directories.",
        ),
        // Network
        DangerPattern::new(
            r"(?i)\b(url|uri|endpoint|host|hostname|fetch|request|http|https)\b",
            "Network access capability",
            Severity::Medium,
            "Network-accessing tools can exfiltrate data. Consider restricting allowed domains.",
        ),
        // Database
        DangerPattern::new(
            r"(?i)\b(query|sql|database|db|select|insert|update|delete)\b",
            "Database access capability",
            Severity::Medium,
            "Database tools should use parameterized queries and limited permissions.",
        ),
        // Credentials
        DangerPattern::new(
            r"(?i)\b(password|secret|token|key|credential|auth|api.?key)\b",
            "Handles sensitive credentials",
            Severity::High,
            "Tools handling credentials should use secure storage and avoid logging values.",
        ),
    ]
});

struct DangerPattern {
    regex: Regex,
    title: &'static str,
    severity: Severity,
    remediation: &'static str,
}

impl DangerPattern {
    fn new(pattern: &str, title: &'static str, severity: Severity, remediation: &'static str) -> Self {
        Self {
            regex: Regex::new(pattern).expect("Invalid regex"),
            title,
            severity,
            remediation,
        }
    }
}

pub struct PermissionScopeDetector {
    root_path_patterns: Vec<Regex>,
}

impl PermissionScopeDetector {
    pub fn new() -> Self {
        Self {
            root_path_patterns: vec![
                Regex::new(r#"["']/["']"#).unwrap(),           // "/" alone
                Regex::new(r#"["']~["']"#).unwrap(),           // "~" home dir
                Regex::new(r#"["'][A-Z]:\\["']"#).unwrap(),    // Windows root like "C:\"
                Regex::new(r#"["']/home["']"#).unwrap(),       // /home
                Regex::new(r#"["']/Users["']"#).unwrap(),      // macOS /Users
                Regex::new(r#"["']/etc["']"#).unwrap(),        // system config
                Regex::new(r#"["']/var["']"#).unwrap(),        // system data
            ],
        }
    }

    fn check_tool(&self, tool: &ToolInfo) -> Vec<Threat> {
        let mut threats = Vec::new();

        let combined_text = format!(
            "{} {} {}",
            tool.name,
            tool.description.as_deref().unwrap_or(""),
            serde_json::to_string(&tool.input_schema).unwrap_or_default()
        );

        // Check for dangerous patterns
        for pattern in DANGEROUS_PATTERNS.iter() {
            if pattern.regex.is_match(&combined_text) {
                threats.push(
                    Threat::new(
                        format!("PERM-{}", threats.len() + 1),
                        pattern.severity,
                        ThreatCategory::PermissionScope,
                        pattern.title,
                    )
                    .with_message(format!(
                        "Tool '{}' appears to have {}",
                        tool.name,
                        pattern.title.to_lowercase()
                    ))
                    .with_evidence(tool.name.clone())
                    .with_remediation(pattern.remediation)
                    .with_tool(&tool.name),
                );
            }
        }

        // Check for root path access
        let schema_str = serde_json::to_string(&tool.input_schema).unwrap_or_default();
        for pattern in &self.root_path_patterns {
            if pattern.is_match(&schema_str) {
                threats.push(
                    Threat::new(
                        "PERM-ROOT",
                        Severity::High,
                        ThreatCategory::PermissionScope,
                        "Root filesystem access",
                    )
                    .with_message(format!(
                        "Tool '{}' appears to have access to root or system directories",
                        tool.name
                    ))
                    .with_evidence(pattern.find(&schema_str).map(|m| m.as_str().to_string()).unwrap_or_default())
                    .with_remediation("Restrict filesystem access to specific directories needed for the task")
                    .with_tool(&tool.name),
                );
                break;
            }
        }

        threats
    }
}

impl Default for PermissionScopeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreatDetector for PermissionScopeDetector {
    fn detect(
        &self,
        _server: &ServerConfig,
        tools: &[ToolInfo],
        _resources: &[ResourceInfo],
    ) -> Vec<Threat> {
        tools.iter().flat_map(|tool| self.check_tool(tool)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool(name: &str, description: &str, schema: serde_json::Value) -> ToolInfo {
        ToolInfo {
            name: name.to_string(),
            description: Some(description.to_string()),
            input_schema: schema,
        }
    }

    #[test]
    fn detects_code_execution() {
        let detector = PermissionScopeDetector::new();
        let tools = vec![make_tool(
            "run_command",
            "Execute a shell command",
            serde_json::json!({"type": "object", "properties": {"command": {"type": "string"}}}),
        )];

        let threats = detector.detect(&ServerConfig::new("test", "cmd"), &tools, &[]);
        assert!(!threats.is_empty());
        assert!(threats.iter().any(|t| t.title.contains("code execution")));
    }

    #[test]
    fn detects_credential_handling() {
        let detector = PermissionScopeDetector::new();
        let tools = vec![make_tool(
            "get_api_key",
            "Retrieve the API key from environment",
            serde_json::json!({}),
        )];

        let threats = detector.detect(&ServerConfig::new("test", "cmd"), &tools, &[]);
        assert!(threats.iter().any(|t| t.title.contains("credential")));
    }

    #[test]
    fn detects_root_path() {
        let detector = PermissionScopeDetector::new();
        let tools = vec![make_tool(
            "read_file",
            "Read a file",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "default": "/"}
                }
            }),
        )];

        let threats = detector.detect(&ServerConfig::new("test", "cmd"), &tools, &[]);
        assert!(threats.iter().any(|t| t.id == "PERM-ROOT"));
    }
}
