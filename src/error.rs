//! Error types for mcp-guard using thiserror.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to spawn MCP server process: {0}")]
    ProcessSpawn(#[source] std::io::Error),

    #[error("Failed to communicate with MCP server: {0}")]
    ProcessIo(#[source] std::io::Error),

    #[error("MCP server process exited unexpectedly with code: {0:?}")]
    ProcessExit(Option<i32>),

    #[error("Timeout waiting for MCP server response after {timeout_secs}s")]
    Timeout { timeout_secs: u64 },

    #[error("HTTP transport error: {0}")]
    HttpTransport(#[from] reqwest::Error),

    #[error("Invalid JSON-RPC message: {0}")]
    JsonRpcParse(#[source] serde_json::Error),

    #[error("JSON-RPC error: code={code}, message={message}")]
    JsonRpc {
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    },

    #[error("Config file not found: {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Failed to read config file {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file {path}: {message}")]
    ConfigParse { path: PathBuf, message: String },

    #[error("Invalid server configuration: {message}")]
    InvalidServerConfig { message: String },

    #[error("Failed to scan server '{server_name}': {message}")]
    ScanFailed {
        server_name: String,
        message: String,
    },

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Database pool error: {0}")]
    DatabasePool(#[from] r2d2::Error),

    #[error("Proxy error: {message}")]
    Proxy { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn config_parse(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::ConfigParse {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn invalid_server_config(message: impl Into<String>) -> Self {
        Self::InvalidServerConfig {
            message: message.into(),
        }
    }

    pub fn scan_failed(server_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ScanFailed {
            server_name: server_name.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Timeout { timeout_secs: 30 };
        assert_eq!(
            err.to_string(),
            "Timeout waiting for MCP server response after 30s"
        );

        let err = Error::JsonRpc {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        };
        assert_eq!(
            err.to_string(),
            "JSON-RPC error: code=-32600, message=Invalid Request"
        );
    }

    #[test]
    fn test_error_helpers() {
        let err = Error::scan_failed("test_server", "connection refused");
        assert!(
            matches!(err, Error::ScanFailed { server_name, .. } if server_name == "test_server")
        );
    }
}
