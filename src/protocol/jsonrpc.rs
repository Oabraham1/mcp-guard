//! JSON-RPC 2.0 types for MCP communication.
//!
//! Implements the JSON-RPC 2.0 specification as used by MCP.
//! See: <https://www.jsonrpc.org/specification>

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global request ID counter for generating unique IDs.
static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a new unique request ID.
pub fn next_request_id() -> RequestId {
    RequestId::Number(REQUEST_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// JSON-RPC 2.0 version string.
pub const JSONRPC_VERSION: &str = "2.0";

/// Request ID - can be string, number, or null.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(u64),
    Null,
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestId::String(s) => write!(f, "{}", s),
            RequestId::Number(n) => write!(f, "{}", n),
            RequestId::Null => write!(f, "null"),
        }
    }
}

impl From<u64> for RequestId {
    fn from(n: u64) -> Self {
        RequestId::Number(n)
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        RequestId::String(s)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        RequestId::String(s.to_string())
    }
}

/// A JSON-RPC 2.0 request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// JSON-RPC version, must be "2.0".
    pub jsonrpc: String,

    /// The method to invoke.
    pub method: String,

    /// Optional parameters for the method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,

    /// Request ID for correlating responses.
    pub id: RequestId,
}

impl Request {
    /// Create a new JSON-RPC request with auto-generated ID.
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
            id: next_request_id(),
        }
    }

    #[cfg(test)]
    pub fn with_id(
        method: impl Into<String>,
        params: Option<serde_json::Value>,
        id: RequestId,
    ) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
            id,
        }
    }

    /// Serialize this request to a JSON string with newline for STDIO transport.
    pub fn to_json_line(&self) -> Result<String, serde_json::Error> {
        let mut json = serde_json::to_string(self)?;
        json.push('\n');
        Ok(json)
    }
}

/// A JSON-RPC 2.0 notification message (request without ID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// JSON-RPC version, must be "2.0".
    pub jsonrpc: String,

    /// The method to invoke.
    pub method: String,

    /// Optional parameters for the method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl Notification {
    /// Create a new JSON-RPC notification.
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
        }
    }

    /// Serialize this notification to a JSON string with newline for STDIO transport.
    pub fn to_json_line(&self) -> Result<String, serde_json::Error> {
        let mut json = serde_json::to_string(self)?;
        json.push('\n');
        Ok(json)
    }
}

/// A JSON-RPC 2.0 successful response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// JSON-RPC version, must be "2.0".
    pub jsonrpc: String,

    /// The result of the method call.
    pub result: serde_json::Value,

    /// Request ID this response is for.
    pub id: RequestId,
}

impl Response {
    #[cfg(test)]
    pub fn new(result: serde_json::Value, id: RequestId) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result,
            id,
        }
    }
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,

    /// Error message.
    pub message: String,

    /// Optional additional data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Create a new error.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

#[cfg(test)]
impl JsonRpcError {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    pub fn parse_error() -> Self {
        Self::new(Self::PARSE_ERROR, "Parse error")
    }

    pub fn invalid_request() -> Self {
        Self::new(Self::INVALID_REQUEST, "Invalid Request")
    }

    pub fn method_not_found() -> Self {
        Self::new(Self::METHOD_NOT_FOUND, "Method not found")
    }

    pub fn invalid_params() -> Self {
        Self::new(Self::INVALID_PARAMS, "Invalid params")
    }

    pub fn internal_error() -> Self {
        Self::new(Self::INTERNAL_ERROR, "Internal error")
    }
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for JsonRpcError {}

/// A JSON-RPC 2.0 error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// JSON-RPC version, must be "2.0".
    pub jsonrpc: String,

    /// The error object.
    pub error: JsonRpcError,

    /// Request ID this response is for.
    pub id: RequestId,
}

impl ErrorResponse {
    pub fn new(error: JsonRpcError, id: RequestId) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            error,
            id,
        }
    }
}

/// A message that can be either a request, notification, response, or error.
/// Used for parsing incoming messages when the type is unknown.
#[derive(Debug, Clone)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
    Error(ErrorResponse),
}

impl Message {
    /// Parse a JSON string into a Message, determining the type automatically.
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(json)?;

        // Check if it's a response (has "result" field)
        if value.get("result").is_some() {
            let response: Response = serde_json::from_value(value)?;
            return Ok(Message::Response(response));
        }

        // Check if it's an error response (has "error" field)
        if value.get("error").is_some() {
            let error: ErrorResponse = serde_json::from_value(value)?;
            return Ok(Message::Error(error));
        }

        // Check if it's a request or notification (has "method" field)
        if value.get("method").is_some() {
            // If it has an "id" field, it's a request; otherwise, notification
            if value.get("id").is_some() {
                let request: Request = serde_json::from_value(value)?;
                return Ok(Message::Request(request));
            } else {
                let notification: Notification = serde_json::from_value(value)?;
                return Ok(Message::Notification(notification));
            }
        }

        // Invalid message - doesn't match any known type
        Err(serde::de::Error::custom(
            "Invalid JSON-RPC message: missing required fields",
        ))
    }
}

#[cfg(test)]
impl Message {
    pub fn is_request(&self) -> bool {
        matches!(self, Message::Request(_))
    }

    pub fn is_notification(&self) -> bool {
        matches!(self, Message::Notification(_))
    }

    pub fn is_response(&self) -> bool {
        matches!(self, Message::Response(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Message::Error(_))
    }

    pub fn id(&self) -> Option<&RequestId> {
        match self {
            Message::Request(r) => Some(&r.id),
            Message::Response(r) => Some(&r.id),
            Message::Error(e) => Some(&e.id),
            Message::Notification(_) => None,
        }
    }

    pub fn method(&self) -> Option<&str> {
        match self {
            Message::Request(r) => Some(&r.method),
            Message::Notification(n) => Some(&n.method),
            _ => None,
        }
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Message::Request(r) => r.serialize(serializer),
            Message::Notification(n) => n.serialize(serializer),
            Message::Response(r) => r.serialize(serializer),
            Message::Error(e) => e.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = Request::with_id(
            "tools/list",
            Some(serde_json::json!({})),
            RequestId::Number(1),
        );

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tools/list\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_request_to_json_line() {
        let request = Request::with_id("test", None, RequestId::Number(1));
        let line = request.to_json_line().unwrap();
        assert!(line.ends_with('\n'));
        assert!(!line.ends_with("\n\n"));
    }

    #[test]
    fn test_response_serialization() {
        let response = Response::new(serde_json::json!({"tools": []}), RequestId::Number(1));

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse::new(JsonRpcError::method_not_found(), RequestId::Number(1));

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32601"));
    }

    #[test]
    fn test_notification_no_id() {
        let notification = Notification::new("notifications/progress", None);

        let json = serde_json::to_string(&notification).unwrap();
        assert!(!json.contains("\"id\""));
    }

    #[test]
    fn test_message_parse_request() {
        let json = r#"{"jsonrpc":"2.0","method":"test","params":{},"id":1}"#;
        let msg = Message::parse(json).unwrap();
        assert!(msg.is_request());
        assert_eq!(msg.method(), Some("test"));
        assert_eq!(msg.id(), Some(&RequestId::Number(1)));
    }

    #[test]
    fn test_message_parse_notification() {
        let json = r#"{"jsonrpc":"2.0","method":"test","params":{}}"#;
        let msg = Message::parse(json).unwrap();
        assert!(msg.is_notification());
        assert_eq!(msg.method(), Some("test"));
        assert_eq!(msg.id(), None);
    }

    #[test]
    fn test_message_parse_response() {
        let json = r#"{"jsonrpc":"2.0","result":{"data":true},"id":1}"#;
        let msg = Message::parse(json).unwrap();
        assert!(msg.is_response());
        assert_eq!(msg.id(), Some(&RequestId::Number(1)));
    }

    #[test]
    fn test_message_parse_error() {
        let json =
            r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":1}"#;
        let msg = Message::parse(json).unwrap();
        assert!(msg.is_error());
    }

    #[test]
    fn test_request_id_display() {
        assert_eq!(RequestId::Number(42).to_string(), "42");
        assert_eq!(RequestId::String("abc".to_string()).to_string(), "abc");
        assert_eq!(RequestId::Null.to_string(), "null");
    }

    #[test]
    fn test_jsonrpc_error_codes() {
        assert_eq!(JsonRpcError::parse_error().code, -32700);
        assert_eq!(JsonRpcError::invalid_request().code, -32600);
        assert_eq!(JsonRpcError::method_not_found().code, -32601);
        assert_eq!(JsonRpcError::invalid_params().code, -32602);
        assert_eq!(JsonRpcError::internal_error().code, -32603);
    }
}
