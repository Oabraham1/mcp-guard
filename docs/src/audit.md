# Audit Logging

mcp-guard maintains a detailed audit log of all proxied tool calls.

## What's Logged

Each audit entry includes:

| Field | Description |
|-------|-------------|
| `timestamp` | When the call occurred (UTC) |
| `server_name` | MCP server that handled the call |
| `tool_name` | Name of the tool called |
| `tool_args` | Arguments passed to the tool (JSON) |
| `result` | Tool result (JSON, if captured) |
| `blocked` | Whether the call was blocked by a rule |
| `block_reason` | Why the call was blocked |
| `duration_ms` | Execution time in milliseconds |

## Viewing Logs

### Web Dashboard

```bash
mcp-guard serve
```

Navigate to the Audit Log section.

### API

```bash
# List recent entries
curl http://localhost:9191/api/audit

# Filter by server
curl "http://localhost:9191/api/audit?server=filesystem"

# Filter by tool
curl "http://localhost:9191/api/audit?tool=read_file"

# Show only blocked calls
curl "http://localhost:9191/api/audit?blocked=true"
```

## Storage

Audit logs are stored in `~/.mcp-guard/mcp-guard.db` (SQLite).

## Retention

By default, all logs are retained. Future versions may add automatic cleanup policies.
