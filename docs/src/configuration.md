# Configuration

## Config File Location

mcp-scanner stores its configuration at `~/.mcp-scanner/config.toml`.

Create a default config with:

```bash
mcp-scanner init
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

mcp-scanner stores data in `~/.mcp-scanner/`:

| Path | Description |
|------|-------------|
| `config.toml` | Configuration file |
| `mcp-scanner.db` | SQLite database (audit logs, rules) |
| `snapshots/` | Tool description snapshots for drift detection |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `MCP_GUARD_LOG` | Log level (error, warn, info, debug, trace) |
| `MCP_GUARD_PORT` | Default port for `serve` command |
