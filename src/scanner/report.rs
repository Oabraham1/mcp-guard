//! Scan result and threat types.

use crate::discovery::ServerConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub server: ServerConfig,
    pub tools: Vec<ToolInfo>,
    pub resources: Vec<ResourceInfo>,
    pub threats: Vec<Threat>,
    /// Diff against the last stored snapshot for this server.
    /// `None` on the first scan when no prior snapshot exists.
    pub snapshot_diff: Option<SnapshotDiff>,
    #[serde(with = "duration_millis")]
    pub scan_duration: Duration,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

impl From<crate::protocol::mcp::Tool> for ToolInfo {
    fn from(tool: crate::protocol::mcp::Tool) -> Self {
        Self {
            name: tool.name,
            description: tool.description,
            input_schema: tool.input_schema,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

impl From<crate::protocol::mcp::Resource> for ResourceInfo {
    fn from(resource: crate::protocol::mcp::Resource) -> Self {
        Self {
            uri: resource.uri,
            name: resource.name,
            description: resource.description,
            mime_type: resource.mime_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    pub id: String,
    pub severity: Severity,
    pub category: ThreatCategory,
    pub title: String,
    pub message: String,
    pub evidence: String,
    pub remediation: String,
    pub tool_name: Option<String>,
}

impl Threat {
    pub fn new(
        id: impl Into<String>,
        severity: Severity,
        category: ThreatCategory,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity,
            category,
            title: title.into(),
            message: String::new(),
            evidence: String::new(),
            remediation: String::new(),
            tool_name: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence = evidence.into();
        self
    }

    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = remediation.into();
        self
    }

    pub fn with_tool(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Info => "info",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatCategory {
    DescriptionInjection,
    PermissionScope,
    NoAuth,
    ToolShadowing,
    DescriptionDrift,
}

impl ThreatCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThreatCategory::DescriptionInjection => "description_injection",
            ThreatCategory::PermissionScope => "permission_scope",
            ThreatCategory::NoAuth => "no_auth",
            ThreatCategory::ToolShadowing => "tool_shadowing",
            ThreatCategory::DescriptionDrift => "description_drift",
        }
    }
}

impl std::fmt::Display for ThreatCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub added_tools: Vec<String>,
    pub removed_tools: Vec<String>,
    pub changed_descriptions: Vec<DescriptionChange>,
}

#[cfg(test)]
impl SnapshotDiff {
    pub fn is_empty(&self) -> bool {
        self.added_tools.is_empty()
            && self.removed_tools.is_empty()
            && self.changed_descriptions.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionChange {
    pub tool_name: String,
    pub old_description: String,
    pub new_description: String,
    pub old_hash: String,
    pub new_hash: String,
}

mod duration_millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(Severity::Critical < Severity::High);
        assert!(Severity::High < Severity::Medium);
        assert!(Severity::Medium < Severity::Low);
        assert!(Severity::Low < Severity::Info);
    }

    #[test]
    fn threat_builder() {
        let threat = Threat::new(
            "TEST-001",
            Severity::High,
            ThreatCategory::DescriptionInjection,
            "Test Threat",
        )
        .with_message("This is a test")
        .with_evidence("suspicious content")
        .with_tool("test_tool");

        assert_eq!(threat.id, "TEST-001");
        assert_eq!(threat.severity, Severity::High);
        assert_eq!(threat.tool_name, Some("test_tool".to_string()));
    }

    #[test]
    fn snapshot_diff_is_empty() {
        let empty = SnapshotDiff {
            added_tools: vec![],
            removed_tools: vec![],
            changed_descriptions: vec![],
        };
        assert!(empty.is_empty());

        let not_empty = SnapshotDiff {
            added_tools: vec!["new_tool".to_string()],
            removed_tools: vec![],
            changed_descriptions: vec![],
        };
        assert!(!not_empty.is_empty());
    }
}
