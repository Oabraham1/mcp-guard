# CLI Reference

## Global Options

```
--help, -h     Show help information
--version, -V  Show version
```

## Commands

### `mcp-scanner scan`

Scan MCP servers for security vulnerabilities.

```bash
mcp-scanner scan [OPTIONS]
```

**Options:**
- `--client <NAME>` - Only scan servers from this client (claude, cursor, windsurf, etc.)
- `--server <COMMAND>` - Scan a specific server command
- `--config <PATH>` - Load servers from a config file
- `--output <FORMAT>` - Output format: table (default), json, sarif
- `--timeout <SECONDS>` - Per-server timeout (default: 30)

**Examples:**
```bash
mcp-scanner scan
mcp-scanner scan --client claude
mcp-scanner scan --server "npx server.js"
mcp-scanner scan --output sarif > results.sarif
```

### `mcp-scanner list`

List discovered MCP servers.

```bash
mcp-scanner list [OPTIONS]
```

**Options:**
- `--client <NAME>` - Only list servers from this client

**Examples:**
```bash
mcp-scanner list
mcp-scanner list --client cursor
```

### `mcp-scanner serve`

Start the web dashboard and API server.

```bash
mcp-scanner serve [OPTIONS]
```

**Options:**
- `--port <PORT>` - Port to listen on (default: 9191)
- `--headless` - Don't open browser automatically

**Examples:**
```bash
mcp-scanner serve
mcp-scanner serve --port 8080
mcp-scanner serve --headless
```

### `mcp-scanner proxy`

Proxy an MCP server with filtering and audit logging.

```bash
mcp-scanner proxy --server <COMMAND>
```

**Options:**
- `--server <COMMAND>` - Server command to proxy (required)

**Examples:**
```bash
mcp-scanner proxy --server "npx -y @modelcontextprotocol/server-filesystem /"
```

### `mcp-scanner init`

Create default configuration file.

```bash
mcp-scanner init [OPTIONS]
```

**Options:**
- `--force` - Overwrite existing configuration

**Examples:**
```bash
mcp-scanner init
mcp-scanner init --force
```

### `mcp-scanner completions`

Generate shell completions.

```bash
mcp-scanner completions --shell <SHELL>
```

**Options:**
- `--shell <SHELL>` - Shell to generate for: bash, zsh, fish, powershell

**Examples:**
```bash
mcp-scanner completions --shell bash
mcp-scanner completions --shell zsh
```
