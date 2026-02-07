//! MCP protocol implementation including JSON-RPC 2.0 and transport layers.

pub mod jsonrpc;
pub mod mcp;
pub mod transport;

pub use jsonrpc::{Notification, Request};
pub use transport::McpTransport;
