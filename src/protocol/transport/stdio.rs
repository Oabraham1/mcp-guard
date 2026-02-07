//! STDIO transport for MCP servers.
//!
//! Spawns a child process and communicates via newline-delimited JSON-RPC
//! over stdin/stdout.

use crate::error::{Error, Result};
use crate::protocol::jsonrpc::{Message, Notification, Request, Response};
use crate::protocol::transport::McpTransport;
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

#[derive(Debug)]
pub struct StdioTransport {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    timeout: Duration,
}

impl StdioTransport {
    pub async fn spawn(
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Self> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .envs(env);

        let mut child = cmd.spawn().map_err(Error::ProcessSpawn)?;

        let stdin = child.stdin.take().ok_or_else(|| {
            Error::ProcessIo(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Failed to open stdin",
            ))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            Error::ProcessIo(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Failed to open stdout",
            ))
        })?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            timeout: Duration::from_secs(30),
        })
    }

    async fn read_line(&mut self) -> Result<String> {
        let mut line = String::new();

        let bytes_read = timeout(self.timeout, self.stdout.read_line(&mut line))
            .await
            .map_err(|_| Error::Timeout {
                timeout_secs: self.timeout.as_secs(),
            })?
            .map_err(Error::ProcessIo)?;

        if bytes_read == 0 {
            let exit_status = self.child.try_wait().map_err(Error::ProcessIo)?;
            return Err(Error::ProcessExit(exit_status.and_then(|s| s.code())));
        }

        Ok(line)
    }

    async fn write_line(&mut self, line: &str) -> Result<()> {
        timeout(self.timeout, async {
            self.stdin
                .write_all(line.as_bytes())
                .await
                .map_err(Error::ProcessIo)?;
            self.stdin.flush().await.map_err(Error::ProcessIo)
        })
        .await
        .map_err(|_| Error::Timeout {
            timeout_secs: self.timeout.as_secs(),
        })?
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send(&mut self, request: Request) -> Result<Response> {
        let expected_id = request.id.clone();
        let line = request.to_json_line().map_err(Error::JsonRpcParse)?;

        self.write_line(&line).await?;

        // Read responses until we get one matching our request ID
        loop {
            let response_line = self.read_line().await?;
            let message = Message::parse(&response_line).map_err(Error::JsonRpcParse)?;

            match message {
                Message::Response(response) => {
                    if response.id == expected_id {
                        return Ok(response);
                    }
                    // ID mismatch - could be response to a different request, skip it
                }
                Message::Error(error) => {
                    if error.id == expected_id {
                        return Err(Error::JsonRpc {
                            code: error.error.code,
                            message: error.error.message,
                            data: error.error.data,
                        });
                    }
                }
                Message::Notification(_) => {
                    // Server-initiated notification, skip it
                }
                Message::Request(req) => {
                    tracing::debug!(method = %req.method, "Ignoring server-initiated request");
                }
            }
        }
    }

    async fn send_notification(&mut self, notification: Notification) -> Result<()> {
        let line = notification.to_json_line().map_err(Error::JsonRpcParse)?;
        self.write_line(&line).await
    }

    async fn close(&mut self) -> Result<()> {
        self.child.kill().await.ok();
        Ok(())
    }

    fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn spawn_nonexistent_command_fails() {
        let result = StdioTransport::spawn("nonexistent-command-12345", &[], &HashMap::new()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ProcessSpawn(_)));
    }

    #[tokio::test]
    async fn spawn_echo_succeeds() {
        // Just verify we can spawn a simple command
        let result = StdioTransport::spawn("echo", &["test".to_string()], &HashMap::new()).await;
        assert!(result.is_ok());
    }
}
