# mcp-scanner

[![CI](https://github.com/oabraham1/mcp-scanner/actions/workflows/ci.yml/badge.svg)](https://github.com/oabraham1/mcp-scanner/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/mcp-scanner.svg)](https://crates.io/crates/mcp-scanner)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

Security scanner and proxy for MCP (Model Context Protocol) servers.

mcp-scanner discovers, scans, and proxies MCP servers configured across your AI tools (Claude Desktop, Cursor, Windsurf, VS Code, and more), detecting security vulnerabilities like prompt injection in tool descriptions, overly broad permissions, and suspicious changes.

## Features

- **Auto-discovery**: Finds MCP servers configured in Claude Desktop, Cursor, Windsurf, Zed, Cline, Continue, VS Code, Roo Code, and Claude Code
- **Security scanning**: Detects prompt injection, permission scope issues, missing auth, tool shadowing, and description drift
- **STDIO proxy**: Intercepts tool calls between clients and servers with rule-based filtering
- **Web dashboard**: htmx-powered UI for viewing scan results and managing proxy rules
- **Audit logging**: SQLite-backed logging of all proxied tool calls

## Installation

### Homebrew (macOS/Linux)

```bash
brew install oabraham1/tap/mcp-scanner
```

### Shell Installer

```bash
curl -fsSL https://raw.githubusercontent.com/oabraham1/mcp-scanner/main/install.sh | sh
```

### Download Binary

Download pre-built binaries from [GitHub Releases](https://github.com/oabraham1/mcp-scanner/releases).

### Cargo (requires Rust)

```bash
cargo install mcp-scanner
```

### Build from Source

```bash
git clone https://github.com/oabraham1/mcp-scanner
cd mcp-scanner
cargo build --release
```

## Quick Start

```bash
# Scan all discovered MCP servers
mcp-scanner scan

# List discovered servers
mcp-scanner list

# Start the web dashboard
mcp-scanner serve

# Proxy a specific server
mcp-scanner proxy --server "npx -y @modelcontextprotocol/server-filesystem /"
```

## CLI Reference

### `mcp-scanner scan`

Scan MCP servers for security vulnerabilities.

```bash
mcp-scanner scan                           # Scan all discovered servers
mcp-scanner scan --client claude           # Scan only Claude Desktop servers
mcp-scanner scan --server "npx server.js"  # Scan a specific server command
mcp-scanner scan --config ./mcp.json       # Scan servers from config file
mcp-scanner scan --output json             # Output as JSON
mcp-scanner scan --output sarif            # Output as SARIF (for CI integration)
```

### `mcp-scanner list`

List discovered MCP servers.

```bash
mcp-scanner list                    # List all servers
mcp-scanner list --client cursor    # List only Cursor servers
```

### `mcp-scanner serve`

Start the web dashboard and API server.

```bash
mcp-scanner serve                   # Start on localhost:9191
mcp-scanner serve --port 8080       # Use custom port
mcp-scanner serve --headless        # Don't open browser
```

### `mcp-scanner proxy`

Proxy an MCP server with filtering and audit logging.

```bash
mcp-scanner proxy --server "npx -y @modelcontextprotocol/server-filesystem /"
```

To use the proxy, update your client config to point to mcp-scanner:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "mcp-scanner",
      "args": ["proxy", "--server", "npx -y @modelcontextprotocol/server-filesystem /"]
    }
  }
}
```

### `mcp-scanner init`

Create default configuration.

```bash
mcp-scanner init           # Create ~/.mcp-scanner/config.toml
mcp-scanner init --force   # Overwrite existing config
```

### `mcp-scanner completions`

Generate shell completions.

```bash
mcp-scanner completions --shell bash >> ~/.bashrc
mcp-scanner completions --shell zsh >> ~/.zshrc
mcp-scanner completions --shell fish >> ~/.config/fish/completions/mcp-scanner.fish
```

## Threat Categories

mcp-scanner detects the following security issues:

### Description Injection (Critical/High)
Prompt injection patterns in tool descriptions, including:
- "Ignore previous instructions" patterns
- Hidden Unicode characters
- Base64-encoded payloads
- System prompt injection attempts

### Permission Scope (High/Medium)
Overly broad capabilities:
- Arbitrary code execution
- Root filesystem access
- Unrestricted network access
- Database query access

### No Auth (Critical for remote, Info for local)
Servers without authentication:
- Remote servers without auth tokens (Critical)
- Local servers without env-based auth (Info)

### Tool Shadowing (High/Medium)
Name conflicts across servers:
- Exact name collisions
- Similar names (potential typosquatting)

### Description Drift (High/Medium)
Changes since last scan:
- Modified tool descriptions
- Added/removed tools

## Configuration

Config file location: `~/.mcp-scanner/config.toml`

```toml
[scan]
timeout = 30  # seconds per server

[output]
format = "table"  # table, json, sarif
```

## API Endpoints

The web server exposes a JSON API:

- `GET /api/health` - Health check
- `GET /api/servers` - List discovered servers
- `POST /api/scan` - Run a scan
- `GET /api/audit` - List audit log entries
- `GET /api/rules` - List proxy rules
- `POST /api/rules` - Create proxy rule
- `PUT /api/rules/:id` - Update proxy rule
- `DELETE /api/rules/:id` - Delete proxy rule

## Data Storage

mcp-scanner stores data in `~/.mcp-scanner/`:

- `mcp-scanner.db` - SQLite database (audit logs, scan results, rules)
- `snapshots/` - Tool description snapshots for drift detection
- `config.toml` - Configuration file

## Supported Clients

| Client | Config Path |
|--------|-------------|
| Claude Desktop | `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) |
| Cursor | `~/.cursor/mcp.json` |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` |
| Zed | `~/.config/zed/settings.json` |
| Cline | `~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json` |
| Continue | `~/.continue/config.json` |
| VS Code | `.vscode/mcp.json` |
| Roo Code | `~/.config/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/mcp_settings.json` |
| Claude Code | `~/.claude/settings.json` or `.mcp.json` |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
