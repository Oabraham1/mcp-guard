# Contributing

We welcome contributions to mcp-scanner! This document outlines how to get started.

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/oabraham1/mcp-scanner
   cd mcp-scanner
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Run with logging:
   ```bash
   RUST_LOG=debug cargo run -- scan
   ```

## Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Pass clippy checks (`cargo clippy -- -D warnings`)
- Write tests for new functionality
- Document public APIs

## Submitting Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes
4. Push to your fork
5. Open a pull request

## Adding Threat Detectors

To add a new threat detector:

1. Create a file in `src/scanner/threats/`
2. Implement the `ThreatDetector` trait
3. Add it to the list in `threats/mod.rs`
4. Write tests

Example:

```rust
use crate::scanner::threats::ThreatDetector;
use crate::scanner::report::{Threat, Severity, ThreatCategory};

pub struct MyDetector;

impl ThreatDetector for MyDetector {
    fn detect(
        &self,
        server: &ServerConfig,
        tools: &[ToolInfo],
        resources: &[ResourceInfo],
    ) -> Vec<Threat> {
        // Detection logic here
        vec![]
    }
}
```

## Adding Client Parsers

To add support for a new AI client:

1. Create a file in `src/discovery/clients/`
2. Implement the `McpClientParser` trait
3. Add it to `all_clients()` in `discovery/mod.rs`
4. Write tests

## Reporting Issues

Please report issues on GitHub with:
- Steps to reproduce
- Expected behavior
- Actual behavior
- mcp-scanner version (`mcp-scanner --version`)
