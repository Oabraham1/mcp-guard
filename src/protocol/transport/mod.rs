//! Transport layer for MCP communication.

mod stdio;

pub use stdio::StdioTransport;

use crate::error::Result;
use crate::protocol::jsonrpc::{Request, Response};
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait McpTransport: Send + Sync {
    async fn send(&mut self, request: Request) -> Result<Response>;

    async fn send_notification(
        &mut self,
        notification: crate::protocol::jsonrpc::Notification,
    ) -> Result<()>;

    async fn close(&mut self) -> Result<()>;

    fn set_timeout(&mut self, timeout: Duration);
}
