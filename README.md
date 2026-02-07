# mcp-guard

[![CI](https://github.com/oabraham1/mcp-guard/actions/workflows/ci.yml/badge.svg)](https://github.com/oabraham1/mcp-guard/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/mcp-guard.svg)](https://crates.io/crates/mcp-guard)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

Security scanner and proxy for MCP (Model Context Protocol) servers.

mcp-guard discovers, scans, and proxies MCP servers configured across your AI tools (Claude Desktop, Cursor, Windsurf, VS Code, and more), detecting security vulnerabilities like prompt injection in tool descriptions, overly broad permissions, and suspicious changes.

## Features

- **Auto-discovery**: Finds MCP servers configured in Claude Desktop, Cursor, Windsurf, Zed, Cline, Continue, VS Code, Roo Code, and Claude Code
- **Security scanning**: Detects prompt injection, permission scope issues, missing auth, tool shadowing, and description drift
- **STDIO proxy**: Intercepts tool calls between clients and servers with rule-based filtering
- **Web dashboard**: htmx-powered UI for viewing scan results and managing proxy rules
- **Audit logging**: SQLite-backed logging of all proxied tool calls

## Installation

```bash
cargo install mcp-guard
```

Or build from source:

```bash
git clone https://github.com/oabraham1/mcp-guard
cd mcp-guard
cargo build --release
```

## Quick Start

```bash
# Scan all discovered MCP servers
mcp-guard scan

# List discovered servers
mcp-guard list

# Start the web dashboard
mcp-guard serve

# Proxy a specific server
mcp-guard proxy --server "npx -y @modelcontextprotocol/server-filesystem /"
```

## CLI Reference

### `mcp-guard scan`

Scan MCP servers for security vulnerabilities.

```bash
mcp-guard scan                           # Scan all discovered servers
mcp-guard scan --client claude           # Scan only Claude Desktop servers
mcp-guard scan --server "npx server.js"  # Scan a specific server command
mcp-guard scan --config ./mcp.json       # Scan servers from config file
mcp-guard scan --output json             # Output as JSON
mcp-guard scan --output sarif            # Output as SARIF (for CI integration)
```

### `mcp-guard list`

List discovered MCP servers.

```bash
mcp-guard list                    # List all servers
mcp-guard list --client cursor    # List only Cursor servers
```

### `mcp-guard serve`

Start the web dashboard and API server.

```bash
mcp-guard serve                   # Start on localhost:9191
mcp-guard serve --port 8080       # Use custom port
mcp-guard serve --headless        # Don't open browser
```

### `mcp-guard proxy`

Proxy an MCP server with filtering and audit logging.

```bash
mcp-guard proxy --server "npx -y @modelcontextprotocol/server-filesystem /"
```

To use the proxy, update your client config to point to mcp-guard:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "mcp-guard",
      "args": ["proxy", "--server", "npx -y @modelcontextprotocol/server-filesystem /"]
    }
  }
}
```

### `mcp-guard init`

Create default configuration.

```bash
mcp-guard init           # Create ~/.mcp-guard/config.toml
mcp-guard init --force   # Overwrite existing config
```

### `mcp-guard completions`

Generate shell completions.

```bash
mcp-guard completions --shell bash >> ~/.bashrc
mcp-guard completions --shell zsh >> ~/.zshrc
mcp-guard completions --shell fish >> ~/.config/fish/completions/mcp-guard.fish
```

## Threat Categories

mcp-guard detects the following security issues:

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

Config file location: `~/.mcp-guard/config.toml`

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

mcp-guard stores data in `~/.mcp-guard/`:

- `mcp-guard.db` - SQLite database (audit logs, scan results, rules)
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
