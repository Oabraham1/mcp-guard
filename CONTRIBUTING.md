# Contributing to mcp-guard

Thank you for your interest in contributing to mcp-guard! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We want mcp-guard to be a welcoming project for everyone.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Git

### Setup

```bash
git clone https://github.com/oabraham1/mcp-guard
cd mcp-guard
cargo build
cargo test
```

### Running Locally

```bash
# With debug logging
RUST_LOG=debug cargo run -- scan

# Run specific tests
cargo test test_name

# Check formatting and lints
cargo fmt --check
cargo clippy -- -D warnings
```

## How to Contribute

### Reporting Bugs

Before reporting a bug:
1. Check if it's already reported in [GitHub Issues](https://github.com/oabraham1/mcp-guard/issues)
2. Try to reproduce with the latest version

When reporting, include:
- mcp-guard version (`mcp-guard --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or error messages

### Suggesting Features

Open a GitHub issue with:
- Clear description of the feature
- Use case / motivation
- Proposed implementation (optional)

### Submitting Code

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Add/update tests
5. Run `cargo fmt` and `cargo clippy -- -D warnings`
6. Commit with a clear message
7. Push to your fork
8. Open a pull request

## Code Guidelines

### Style

- Follow Rust standard formatting (`cargo fmt`)
- Write clear, self-documenting code
- Add comments for complex logic
- Keep functions focused and reasonably sized

### Testing

- Write tests for new functionality
- Maintain or improve test coverage
- Tests should be deterministic
- Use descriptive test names

### Commits

- Use clear, descriptive commit messages
- Keep commits focused on a single change
- Reference issues when applicable

## Project Structure

```
src/
├── main.rs          # Entry point
├── cli.rs           # CLI commands
├── error.rs         # Error types
├── discovery/       # Server discovery
├── scanner/         # Security scanning
├── proxy/           # STDIO proxy
├── protocol/        # MCP/JSON-RPC
├── db/              # Database
└── web/             # Web UI/API
```

### Adding a Threat Detector

1. Create `src/scanner/threats/my_detector.rs`
2. Implement `ThreatDetector` trait
3. Add to `all_detectors()` in `threats/mod.rs`
4. Write tests

### Adding a Client Parser

1. Create `src/discovery/clients/my_client.rs`
2. Implement `McpClientParser` trait
3. Add to `all_clients()` in `discovery/mod.rs`
4. Write tests

## Review Process

1. All changes require review before merging
2. CI must pass (tests, formatting, lints)
3. Reviewers may request changes
4. Once approved, maintainers will merge

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).
