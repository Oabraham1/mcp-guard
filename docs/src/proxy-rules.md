# Proxy Rules

The mcp-guard proxy intercepts tool calls between AI clients and MCP servers, applying rules for filtering and rate limiting.

## Setting Up the Proxy

### 1. Update Client Configuration

Replace the server command with mcp-guard proxy:

**Before:**
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/"]
    }
  }
}
```

**After:**
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

### 2. Configure Rules

Rules can be configured via the web dashboard (`mcp-guard serve`) or the API.

## Rule Types

### Block Rules

Prevent specific tools from being called:

```json
{
  "rule_type": "block",
  "pattern": "delete_*",
  "reason": "Prevent destructive operations"
}
```

### Rate Limit Rules

Limit how often a tool can be called:

```json
{
  "rule_type": "rate_limit",
  "pattern": "send_email",
  "max_calls": 10,
  "window_seconds": 3600,
  "reason": "Limit email sending"
}
```

## Pattern Matching

Rules use glob patterns:

| Pattern | Matches |
|---------|---------|
| `read_file` | Exact match only |
| `read_*` | `read_file`, `read_dir`, etc. |
| `*_file` | `read_file`, `write_file`, etc. |
| `*` | All tools |

## Rule Priority

Rules are evaluated in priority order (lower numbers first). The first matching rule is applied.

## Audit Logging

All proxied tool calls are logged to the SQLite database, including:
- Timestamp
- Server name
- Tool name and arguments
- Result or error
- Whether the call was blocked
- Execution duration
