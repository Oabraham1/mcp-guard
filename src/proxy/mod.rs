//! STDIO proxy for MCP servers with rule-based filtering and audit logging.

pub mod audit;
pub mod interceptor;
pub mod rules;

pub use interceptor::ProxyInterceptor;
