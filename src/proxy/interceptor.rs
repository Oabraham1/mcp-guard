//! STDIO man-in-the-middle proxy between MCP client and server.

use crate::db::DbPool;
use crate::error::{Error, Result};
use crate::protocol::jsonrpc::{ErrorResponse, JsonRpcError, Message, RequestId};
use crate::proxy::audit::ProxyAudit;
use crate::proxy::rules::{RuleEngine, RuleResult};
use std::process::Stdio;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

enum InterceptResult {
    Forward(String),
    BlockWithResponse(String),
}

pub struct ProxyInterceptor {
    server_command: String,
    server_args: Vec<String>,
    rule_engine: RuleEngine,
    audit: Option<ProxyAudit>,
}

impl ProxyInterceptor {
    pub fn new(server_command: String, server_args: Vec<String>) -> Self {
        Self {
            server_command,
            server_args,
            rule_engine: RuleEngine::new(),
            audit: None,
        }
    }

    pub fn with_db(mut self, pool: DbPool) -> Self {
        self.audit = Some(ProxyAudit::new(pool));
        self
    }

    #[cfg(test)]
    pub fn with_rules(mut self, engine: RuleEngine) -> Self {
        self.rule_engine = engine;
        self
    }

    pub async fn run(&self) -> Result<()> {
        let mut child = Command::new(&self.server_command)
            .args(&self.server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(Error::ProcessSpawn)?;

        let server_stdin = child.stdin.take().ok_or_else(|| Error::Proxy {
            message: "Failed to open server stdin".to_string(),
        })?;

        let server_stdout = child.stdout.take().ok_or_else(|| Error::Proxy {
            message: "Failed to open server stdout".to_string(),
        })?;

        let client_stdin = tokio::io::stdin();
        let mut client_stdout = tokio::io::stdout();

        let mut server_stdin = server_stdin;
        let mut server_stdout = BufReader::new(server_stdout);
        let mut client_stdin = BufReader::new(client_stdin);

        let server_name = self.server_command.clone();

        loop {
            let mut client_line = String::new();
            let mut server_line = String::new();

            tokio::select! {
                result = client_stdin.read_line(&mut client_line) => {
                    match result {
                        Ok(0) => break,
                        Ok(_) => {
                            let start = Instant::now();
                            match self.intercept_client_message(&client_line, &server_name, start) {
                                InterceptResult::Forward(msg) => {
                                    server_stdin.write_all(msg.as_bytes()).await.map_err(Error::ProcessIo)?;
                                    server_stdin.flush().await.map_err(Error::ProcessIo)?;
                                }
                                InterceptResult::BlockWithResponse(response) => {
                                    client_stdout.write_all(response.as_bytes()).await.map_err(Error::ProcessIo)?;
                                    client_stdout.flush().await.map_err(Error::ProcessIo)?;
                                }
                            }
                        }
                        Err(e) => return Err(Error::ProcessIo(e)),
                    }
                }

                result = server_stdout.read_line(&mut server_line) => {
                    match result {
                        Ok(0) => break,
                        Ok(_) => {
                            client_stdout.write_all(server_line.as_bytes()).await.map_err(Error::ProcessIo)?;
                            client_stdout.flush().await.map_err(Error::ProcessIo)?;
                        }
                        Err(e) => return Err(Error::ProcessIo(e)),
                    }
                }
            }
        }

        Ok(())
    }

    fn intercept_client_message(
        &self,
        line: &str,
        server_name: &str,
        start: Instant,
    ) -> InterceptResult {
        let message = match Message::parse(line) {
            Ok(m) => m,
            Err(_) => return InterceptResult::Forward(line.to_string()),
        };

        if let Message::Request(ref request) = message {
            if request.method == "tools/call" {
                if let Some(params) = &request.params {
                    if let Some(tool_name) = params.get("name").and_then(|n| n.as_str()) {
                        match self.rule_engine.evaluate(tool_name) {
                            RuleResult::Allow => {
                                if let Some(ref audit) = self.audit {
                                    audit.record_call(
                                        server_name,
                                        tool_name,
                                        params.get("arguments").cloned(),
                                        None,
                                        false,
                                        None,
                                        start.elapsed(),
                                    );
                                }
                            }
                            RuleResult::Block { reason } => {
                                tracing::warn!(tool = tool_name, reason = %reason, "Blocked tool call");

                                if let Some(ref audit) = self.audit {
                                    audit.record_call(
                                        server_name,
                                        tool_name,
                                        params.get("arguments").cloned(),
                                        None,
                                        true,
                                        Some(reason.clone()),
                                        start.elapsed(),
                                    );
                                }

                                return self.create_error_response(
                                    request.id.clone(),
                                    &format!("Tool call blocked: {}", reason),
                                );
                            }
                            RuleResult::RateLimited { tool } => {
                                tracing::warn!(tool = tool, "Rate limited tool call");

                                if let Some(ref audit) = self.audit {
                                    audit.record_call(
                                        server_name,
                                        tool_name,
                                        params.get("arguments").cloned(),
                                        None,
                                        true,
                                        Some("Rate limit exceeded".to_string()),
                                        start.elapsed(),
                                    );
                                }

                                return self.create_error_response(
                                    request.id.clone(),
                                    "Rate limit exceeded for this tool",
                                );
                            }
                        }
                    }
                }
            }
        }

        InterceptResult::Forward(line.to_string())
    }

    fn create_error_response(&self, id: RequestId, message: &str) -> InterceptResult {
        let error = JsonRpcError::new(-32000, message);
        let response = ErrorResponse::new(error, id);
        let json = serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"Internal error"},"id":null}"#
                .to_string()
        });
        InterceptResult::BlockWithResponse(format!("{}\n", json))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::rules::{ProxyRule, RuleAction};

    #[test]
    fn interceptor_creation() {
        let interceptor = ProxyInterceptor::new("echo".to_string(), vec!["test".to_string()]);
        assert_eq!(interceptor.server_command, "echo");
    }

    #[test]
    fn rule_engine_integration() {
        let mut engine = RuleEngine::new();
        engine
            .add_rule(ProxyRule {
                id: "test".to_string(),
                tool_pattern: "blocked_*".to_string(),
                action: RuleAction::Block {
                    reason: "Test block".to_string(),
                },
                priority: 0,
            })
            .unwrap();

        let interceptor = ProxyInterceptor::new("echo".to_string(), vec![]).with_rules(engine);

        let message =
            r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"blocked_tool"},"id":1}"#;
        let result = interceptor.intercept_client_message(message, "test", Instant::now());
        assert!(matches!(result, InterceptResult::BlockWithResponse(_)));
    }

    #[test]
    fn allowed_tool_forwards() {
        let interceptor = ProxyInterceptor::new("echo".to_string(), vec![]);

        let message =
            r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"safe_tool"},"id":1}"#;
        let result = interceptor.intercept_client_message(message, "test", Instant::now());
        assert!(matches!(result, InterceptResult::Forward(_)));
    }
}
