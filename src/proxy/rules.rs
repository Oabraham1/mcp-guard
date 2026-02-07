//! Proxy rule engine for filtering and rate-limiting tool calls.

#[cfg(test)]
use crate::error::Result;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRule {
    pub id: String,
    pub tool_pattern: String,
    pub action: RuleAction,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleAction {
    Allow,
    Block { reason: String },
    RateLimit { max_calls: u32, window_secs: u64 },
    Log,
}

pub struct RuleEngine {
    rules: Vec<CompiledRule>,
    rate_limiters: Mutex<HashMap<String, RateLimiter>>,
}

struct CompiledRule {
    rule: ProxyRule,
    pattern: Pattern,
}

struct RateLimiter {
    max_calls: u32,
    window: Duration,
    calls: Vec<Instant>,
}

impl RateLimiter {
    fn new(max_calls: u32, window_secs: u64) -> Self {
        Self {
            max_calls,
            window: Duration::from_secs(window_secs),
            calls: Vec::new(),
        }
    }

    fn check_and_record(&mut self) -> bool {
        let now = Instant::now();
        self.calls.retain(|&t| now.duration_since(t) < self.window);

        if self.calls.len() >= self.max_calls as usize {
            return false;
        }

        self.calls.push(now);
        true
    }
}

#[derive(Debug)]
pub enum RuleResult {
    Allow,
    Block { reason: String },
    RateLimited { tool: String },
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            rate_limiters: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    pub fn add_rule(&mut self, rule: ProxyRule) -> Result<()> {
        let pattern = Pattern::new(&rule.tool_pattern)
            .map_err(|e| crate::error::Error::Other(format!("Invalid rule pattern: {}", e)))?;

        self.rules.push(CompiledRule { rule, pattern });
        self.rules.sort_by(|a, b| b.rule.priority.cmp(&a.rule.priority));

        Ok(())
    }

    pub fn evaluate(&self, tool_name: &str) -> RuleResult {
        for compiled in &self.rules {
            if compiled.pattern.matches(tool_name) {
                match &compiled.rule.action {
                    RuleAction::Allow => return RuleResult::Allow,
                    RuleAction::Block { reason } => {
                        return RuleResult::Block {
                            reason: reason.clone(),
                        }
                    }
                    RuleAction::RateLimit {
                        max_calls,
                        window_secs,
                    } => {
                        let mut limiters = self.rate_limiters.lock().unwrap();
                        let key = format!("{}:{}", compiled.rule.id, tool_name);
                        let limiter = limiters
                            .entry(key)
                            .or_insert_with(|| RateLimiter::new(*max_calls, *window_secs));

                        if !limiter.check_and_record() {
                            return RuleResult::RateLimited {
                                tool: tool_name.to_string(),
                            };
                        }
                    }
                    RuleAction::Log => {
                        // Log action doesn't block, but marks for audit
                        // Continue checking other rules
                    }
                }
            }
        }

        // Default: allow
        RuleResult::Allow
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_rule() {
        let mut engine = RuleEngine::new();
        engine
            .add_rule(ProxyRule {
                id: "1".to_string(),
                tool_pattern: "dangerous_*".to_string(),
                action: RuleAction::Block {
                    reason: "Blocked by policy".to_string(),
                },
                priority: 0,
            })
            .unwrap();

        match engine.evaluate("dangerous_tool") {
            RuleResult::Block { reason } => assert_eq!(reason, "Blocked by policy"),
            _ => panic!("Expected block"),
        }

        assert!(matches!(engine.evaluate("safe_tool"), RuleResult::Allow));
    }

    #[test]
    fn rate_limit_rule() {
        let mut engine = RuleEngine::new();
        engine
            .add_rule(ProxyRule {
                id: "1".to_string(),
                tool_pattern: "api_*".to_string(),
                action: RuleAction::RateLimit {
                    max_calls: 2,
                    window_secs: 60,
                },
                priority: 0,
            })
            .unwrap();

        // First two calls should succeed
        assert!(matches!(engine.evaluate("api_call"), RuleResult::Allow));
        assert!(matches!(engine.evaluate("api_call"), RuleResult::Allow));

        // Third call should be rate limited
        assert!(matches!(
            engine.evaluate("api_call"),
            RuleResult::RateLimited { .. }
        ));
    }

    #[test]
    fn priority_ordering() {
        let mut engine = RuleEngine::new();

        // Low priority block
        engine
            .add_rule(ProxyRule {
                id: "1".to_string(),
                tool_pattern: "*".to_string(),
                action: RuleAction::Block {
                    reason: "Default block".to_string(),
                },
                priority: 0,
            })
            .unwrap();

        // High priority allow
        engine
            .add_rule(ProxyRule {
                id: "2".to_string(),
                tool_pattern: "safe_*".to_string(),
                action: RuleAction::Allow,
                priority: 10,
            })
            .unwrap();

        // safe_* should be allowed due to higher priority
        assert!(matches!(engine.evaluate("safe_tool"), RuleResult::Allow));

        // Other tools should be blocked
        assert!(matches!(
            engine.evaluate("other_tool"),
            RuleResult::Block { .. }
        ));
    }
}
