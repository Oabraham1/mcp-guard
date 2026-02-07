//! Server configuration types shared across all discovery clients.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub transport: TransportType,
    #[serde(skip)]
    pub source: ConfigSource,
}

impl ServerConfig {
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
            transport: TransportType::Stdio,
            source: ConfigSource::Manual,
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    #[cfg(test)]
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    #[cfg(test)]
    pub fn is_remote(&self) -> bool {
        matches!(
            self.transport,
            TransportType::Sse { .. } | TransportType::StreamableHttp { .. }
        )
    }

    pub fn display_source(&self) -> String {
        match &self.source {
            ConfigSource::Client { name, config_path } => {
                format!("{} ({})", name, config_path.display())
            }
            ConfigSource::Manual => "manual".to_string(),
        }
    }

    pub fn config_path(&self) -> Option<PathBuf> {
        match &self.source {
            ConfigSource::Client { config_path, .. } => Some(config_path.clone()),
            ConfigSource::Manual => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransportType {
    #[default]
    Stdio,
    Sse {
        url: String,
    },
    StreamableHttp {
        url: String,
    },
}

#[derive(Debug, Clone, Default)]
pub enum ConfigSource {
    Client {
        name: String,
        config_path: PathBuf,
    },
    #[default]
    Manual,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_builder_pattern() {
        let config = ServerConfig::new("test", "npx")
            .with_args(vec!["-y".to_string(), "server".to_string()])
            .with_env(HashMap::from([("KEY".to_string(), "value".to_string())]));

        assert_eq!(config.name, "test");
        assert_eq!(config.command, "npx");
        assert_eq!(config.args, vec!["-y", "server"]);
        assert_eq!(config.env.get("KEY"), Some(&"value".to_string()));
    }

    #[test]
    fn transport_type_serialization() {
        let stdio = TransportType::Stdio;
        let json = serde_json::to_string(&stdio).unwrap();
        assert!(json.contains("\"type\":\"stdio\""));

        let sse = TransportType::Sse {
            url: "http://localhost:8080".to_string(),
        };
        let json = serde_json::to_string(&sse).unwrap();
        assert!(json.contains("\"type\":\"sse\""));
        assert!(json.contains("\"url\":\"http://localhost:8080\""));
    }

    #[test]
    fn is_remote() {
        let stdio_server = ServerConfig::new("test", "cmd");
        assert!(!stdio_server.is_remote());

        let mut sse_server = ServerConfig::new("test", "");
        sse_server.transport = TransportType::Sse {
            url: "http://localhost".to_string(),
        };
        assert!(sse_server.is_remote());
    }
}
