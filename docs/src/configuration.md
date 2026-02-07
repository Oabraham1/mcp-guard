# Configuration

## Config File Location

mcp-guard stores its configuration at `~/.mcp-guard/config.toml`.

Create a default config with:

```bash
mcp-guard init
```

## Configuration Options

```toml
[scan]
# Timeout for each server connection (seconds)
timeout = 30

[output]
# Default output format: table, json, sarif
format = "table"
```

## Data Directory

mcp-guard stores data in `~/.mcp-guard/`:

| Path | Description |
|------|-------------|
| `config.toml` | Configuration file |
| `mcp-guard.db` | SQLite database (audit logs, rules) |
| `snapshots/` | Tool description snapshots for drift detection |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `MCP_GUARD_LOG` | Log level (error, warn, info, debug, trace) |
| `MCP_GUARD_PORT` | Default port for `serve` command |
