//! Threat detection framework and implementations.

mod description_drift;
mod description_injection;
mod no_auth;
mod permission_scope;
mod shadowing;

pub use description_drift::DescriptionDriftDetector;
pub use description_injection::DescriptionInjectionDetector;
pub use no_auth::NoAuthDetector;
pub use permission_scope::PermissionScopeDetector;

use crate::discovery::ServerConfig;
use crate::scanner::report::{ResourceInfo, Threat, ToolInfo};

pub trait ThreatDetector: Send + Sync {
    fn detect(
        &self,
        server: &ServerConfig,
        tools: &[ToolInfo],
        resources: &[ResourceInfo],
    ) -> Vec<Threat>;
}

pub fn all_detectors() -> Vec<Box<dyn ThreatDetector>> {
    vec![
        Box::new(DescriptionInjectionDetector::new()),
        Box::new(PermissionScopeDetector::new()),
        Box::new(NoAuthDetector),
    ]
}
