# Quick Start

## Discover Your MCP Servers

First, see what MCP servers are configured across your AI tools:

```bash
mcp-guard list
```

This scans configurations for Claude Desktop, Cursor, Windsurf, Zed, and other supported clients.

## Run a Security Scan

Scan all discovered servers:

```bash
mcp-guard scan
```

Or scan a specific client's servers:

```bash
mcp-guard scan --client claude
```

Or scan a specific server command:

```bash
mcp-guard scan --server "npx -y @modelcontextprotocol/server-filesystem /"
```

## Understanding Results

Scan results show threats by severity:

- **CRITICAL**: Immediate action required (e.g., prompt injection in remote servers)
- **HIGH**: Significant security risk (e.g., description drift, broad permissions)
- **MEDIUM**: Moderate risk worth reviewing (e.g., new tools added)
- **LOW**: Minor issues or informational (e.g., tools removed)
- **INFO**: Non-actionable information

Each threat includes:
- A description of the issue
- Evidence from the server configuration
- Remediation steps

## Output Formats

```bash
# Default table format
mcp-guard scan

# JSON for scripting
mcp-guard scan --output json

# SARIF for CI integration
mcp-guard scan --output sarif > results.sarif
```

## Start the Dashboard

For a visual interface:

```bash
mcp-guard serve
```

This opens a web dashboard at `http://localhost:9191` where you can:
- View scan results
- Browse audit logs
- Manage proxy rules

## Next Steps

- [Configure threat detection](./configuration.md)
- [Set up the proxy](./proxy-rules.md)
- [Integrate with CI/CD](./ci-integration.md)
