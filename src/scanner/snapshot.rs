//! Snapshot storage and comparison for detecting description drift.

use crate::error::{Error, Result};
use crate::scanner::report::{DescriptionChange, SnapshotDiff, ToolInfo};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct SnapshotStore {
    data_dir: PathBuf,
}

impl SnapshotStore {
    pub fn new() -> Result<Self> {
        let data_dir = Self::default_data_dir()?;
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    fn default_data_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| Error::Other("Could not find home directory".to_string()))?;
        Ok(home.join(".mcp-guard").join("snapshots"))
    }

    fn snapshot_path(&self, server_name: &str) -> PathBuf {
        let safe_name = server_name.replace(['/', '\\', ':'], "_");
        self.data_dir.join(format!("{}.json", safe_name))
    }

    pub fn load(&self, server_name: &str) -> Result<Option<Snapshot>> {
        let path = self.snapshot_path(server_name);
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let snapshot: Snapshot = serde_json::from_str(&content)?;
        Ok(Some(snapshot))
    }

    pub fn save(&self, server_name: &str, tools: &[ToolInfo]) -> Result<()> {
        let snapshot = Snapshot::from_tools(tools);
        let path = self.snapshot_path(server_name);
        let content = serde_json::to_string_pretty(&snapshot)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn compare(&self, server_name: &str, current_tools: &[ToolInfo]) -> Result<Option<SnapshotDiff>> {
        let previous = match self.load(server_name)? {
            Some(s) => s,
            None => return Ok(None),
        };

        let current = Snapshot::from_tools(current_tools);
        Ok(Some(previous.diff(&current)))
    }
}

impl Default for SnapshotStore {
    fn default() -> Self {
        Self::new().expect("Failed to create snapshot store")
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    pub tools: HashMap<String, ToolSnapshot>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolSnapshot {
    pub description: Option<String>,
    pub description_hash: String,
}

impl Snapshot {
    pub fn from_tools(tools: &[ToolInfo]) -> Self {
        let mut tool_map = HashMap::new();

        for tool in tools {
            let desc = tool.description.as_deref().unwrap_or("");
            let hash = hash_description(desc);

            tool_map.insert(
                tool.name.clone(),
                ToolSnapshot {
                    description: tool.description.clone(),
                    description_hash: hash,
                },
            );
        }

        Self {
            tools: tool_map,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn diff(&self, current: &Snapshot) -> SnapshotDiff {
        let mut added_tools = Vec::new();
        let mut removed_tools = Vec::new();
        let mut changed_descriptions = Vec::new();

        // Find added and changed tools
        for (name, current_tool) in &current.tools {
            match self.tools.get(name) {
                Some(old_tool) => {
                    if old_tool.description_hash != current_tool.description_hash {
                        changed_descriptions.push(DescriptionChange {
                            tool_name: name.clone(),
                            old_description: old_tool.description.clone().unwrap_or_default(),
                            new_description: current_tool.description.clone().unwrap_or_default(),
                            old_hash: old_tool.description_hash.clone(),
                            new_hash: current_tool.description_hash.clone(),
                        });
                    }
                }
                None => {
                    added_tools.push(name.clone());
                }
            }
        }

        // Find removed tools
        for name in self.tools.keys() {
            if !current.tools.contains_key(name) {
                removed_tools.push(name.clone());
            }
        }

        SnapshotDiff {
            added_tools,
            removed_tools,
            changed_descriptions,
        }
    }
}

fn hash_description(description: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(description.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool(name: &str, description: &str) -> ToolInfo {
        ToolInfo {
            name: name.to_string(),
            description: Some(description.to_string()),
            input_schema: serde_json::json!({}),
        }
    }

    #[test]
    fn snapshot_from_tools() {
        let tools = vec![
            make_tool("read_file", "Read a file"),
            make_tool("write_file", "Write a file"),
        ];

        let snapshot = Snapshot::from_tools(&tools);
        assert_eq!(snapshot.tools.len(), 2);
        assert!(snapshot.tools.contains_key("read_file"));
    }

    #[test]
    fn diff_detects_added_tool() {
        let old_tools = vec![make_tool("tool1", "desc1")];
        let new_tools = vec![make_tool("tool1", "desc1"), make_tool("tool2", "desc2")];

        let old_snapshot = Snapshot::from_tools(&old_tools);
        let new_snapshot = Snapshot::from_tools(&new_tools);

        let diff = old_snapshot.diff(&new_snapshot);
        assert_eq!(diff.added_tools, vec!["tool2"]);
        assert!(diff.removed_tools.is_empty());
        assert!(diff.changed_descriptions.is_empty());
    }

    #[test]
    fn diff_detects_removed_tool() {
        let old_tools = vec![make_tool("tool1", "desc1"), make_tool("tool2", "desc2")];
        let new_tools = vec![make_tool("tool1", "desc1")];

        let old_snapshot = Snapshot::from_tools(&old_tools);
        let new_snapshot = Snapshot::from_tools(&new_tools);

        let diff = old_snapshot.diff(&new_snapshot);
        assert!(diff.added_tools.is_empty());
        assert_eq!(diff.removed_tools, vec!["tool2"]);
    }

    #[test]
    fn diff_detects_changed_description() {
        let old_tools = vec![make_tool("tool1", "old description")];
        let new_tools = vec![make_tool("tool1", "new description with injection")];

        let old_snapshot = Snapshot::from_tools(&old_tools);
        let new_snapshot = Snapshot::from_tools(&new_tools);

        let diff = old_snapshot.diff(&new_snapshot);
        assert!(diff.added_tools.is_empty());
        assert!(diff.removed_tools.is_empty());
        assert_eq!(diff.changed_descriptions.len(), 1);
        assert_eq!(diff.changed_descriptions[0].tool_name, "tool1");
    }

    #[test]
    fn hash_is_deterministic() {
        let hash1 = hash_description("test description");
        let hash2 = hash_description("test description");
        assert_eq!(hash1, hash2);

        let hash3 = hash_description("different description");
        assert_ne!(hash1, hash3);
    }
}
