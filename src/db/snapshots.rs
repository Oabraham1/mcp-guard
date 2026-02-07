//! Snapshot storage in SQLite (used only for testing).

#[cfg(test)]
mod tests {
    use crate::db::{create_pool, DbPool};
    use crate::error::Result;
    use crate::scanner::report::{DescriptionChange, SnapshotDiff, ToolInfo};
    use sha2::{Digest, Sha256};
    use tempfile::tempdir;

    pub struct SnapshotDb {
        pool: DbPool,
    }

    impl SnapshotDb {
        pub fn new(pool: DbPool) -> Self {
            Self { pool }
        }

        pub fn save(&self, server_name: &str, tools: &[ToolInfo]) -> Result<()> {
            let conn = self.pool.get()?;

            conn.execute(
                "DELETE FROM snapshots WHERE server_name = ?1",
                [server_name],
            )?;

            for tool in tools {
                let desc = tool.description.as_deref().unwrap_or("");
                let hash = hash_description(desc);

                conn.execute(
                    r#"
                    INSERT INTO snapshots (server_name, tool_name, description, description_hash, input_schema)
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                    rusqlite::params![
                        server_name,
                        tool.name,
                        tool.description,
                        hash,
                        serde_json::to_string(&tool.input_schema).ok(),
                    ],
                )?;
            }

            Ok(())
        }

        pub fn compare(
            &self,
            server_name: &str,
            current_tools: &[ToolInfo],
        ) -> Result<Option<SnapshotDiff>> {
            let conn = self.pool.get()?;

            let mut stmt = conn.prepare(
                "SELECT tool_name, description, description_hash FROM snapshots WHERE server_name = ?1",
            )?;

            let old_tools: Vec<(String, Option<String>, String)> = stmt
                .query_map([server_name], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?
                .filter_map(|r| r.ok())
                .collect();

            if old_tools.is_empty() {
                return Ok(None);
            }

            let mut added_tools = Vec::new();
            let mut removed_tools = Vec::new();
            let mut changed_descriptions = Vec::new();

            let old_map: std::collections::HashMap<_, _> = old_tools
                .iter()
                .map(|(name, desc, hash)| (name.clone(), (desc.clone(), hash.clone())))
                .collect();

            let current_map: std::collections::HashMap<_, _> = current_tools
                .iter()
                .map(|t| {
                    let desc = t.description.clone();
                    let hash = hash_description(desc.as_deref().unwrap_or(""));
                    (t.name.clone(), (desc, hash))
                })
                .collect();

            for (name, (new_desc, new_hash)) in &current_map {
                match old_map.get(name) {
                    Some((old_desc, old_hash)) => {
                        if old_hash != new_hash {
                            changed_descriptions.push(DescriptionChange {
                                tool_name: name.clone(),
                                old_description: old_desc.clone().unwrap_or_default(),
                                new_description: new_desc.clone().unwrap_or_default(),
                                old_hash: old_hash.clone(),
                                new_hash: new_hash.clone(),
                            });
                        }
                    }
                    None => {
                        added_tools.push(name.clone());
                    }
                }
            }

            for name in old_map.keys() {
                if !current_map.contains_key(name) {
                    removed_tools.push(name.clone());
                }
            }

            Ok(Some(SnapshotDiff {
                added_tools,
                removed_tools,
                changed_descriptions,
            }))
        }
    }

    fn hash_description(description: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(description.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn test_pool() -> (tempfile::TempDir, DbPool) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = create_pool(&db_path).unwrap();
        (dir, pool)
    }

    fn make_tool(name: &str, desc: &str) -> ToolInfo {
        ToolInfo {
            name: name.to_string(),
            description: Some(desc.to_string()),
            input_schema: serde_json::json!({}),
        }
    }

    #[test]
    fn save_and_compare_no_changes() {
        let (_dir, pool) = test_pool();
        let db = SnapshotDb::new(pool);

        let tools = vec![make_tool("tool1", "desc1")];
        db.save("server1", &tools).unwrap();

        let diff = db.compare("server1", &tools).unwrap().unwrap();
        assert!(diff.added_tools.is_empty());
        assert!(diff.removed_tools.is_empty());
        assert!(diff.changed_descriptions.is_empty());
    }

    #[test]
    fn compare_detects_new_tool() {
        let (_dir, pool) = test_pool();
        let db = SnapshotDb::new(pool);

        let old_tools = vec![make_tool("tool1", "desc1")];
        db.save("server1", &old_tools).unwrap();

        let new_tools = vec![make_tool("tool1", "desc1"), make_tool("tool2", "desc2")];
        let diff = db.compare("server1", &new_tools).unwrap().unwrap();

        assert_eq!(diff.added_tools, vec!["tool2"]);
    }

    #[test]
    fn compare_detects_removed_tool() {
        let (_dir, pool) = test_pool();
        let db = SnapshotDb::new(pool);

        let old_tools = vec![make_tool("tool1", "desc1"), make_tool("tool2", "desc2")];
        db.save("server1", &old_tools).unwrap();

        let new_tools = vec![make_tool("tool1", "desc1")];
        let diff = db.compare("server1", &new_tools).unwrap().unwrap();

        assert_eq!(diff.removed_tools, vec!["tool2"]);
    }

    #[test]
    fn compare_detects_changed_description() {
        let (_dir, pool) = test_pool();
        let db = SnapshotDb::new(pool);

        let old_tools = vec![make_tool("tool1", "old description")];
        db.save("server1", &old_tools).unwrap();

        let new_tools = vec![make_tool("tool1", "new description")];
        let diff = db.compare("server1", &new_tools).unwrap().unwrap();

        assert_eq!(diff.changed_descriptions.len(), 1);
        assert_eq!(diff.changed_descriptions[0].tool_name, "tool1");
    }

    #[test]
    fn compare_returns_none_for_new_server() {
        let (_dir, pool) = test_pool();
        let db = SnapshotDb::new(pool);

        let tools = vec![make_tool("tool1", "desc1")];
        let diff = db.compare("never-seen", &tools).unwrap();

        assert!(diff.is_none());
    }
}
