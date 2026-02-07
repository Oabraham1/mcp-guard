# CLI Reference

## Global Options

```
--help, -h     Show help information
--version, -V  Show version
```

## Commands

### `mcp-guard scan`

Scan MCP servers for security vulnerabilities.

```bash
mcp-guard scan [OPTIONS]
```

**Options:**
- `--client <NAME>` - Only scan servers from this client (claude, cursor, windsurf, etc.)
- `--server <COMMAND>` - Scan a specific server command
- `--config <PATH>` - Load servers from a config file
- `--output <FORMAT>` - Output format: table (default), json, sarif
- `--timeout <SECONDS>` - Per-server timeout (default: 30)

**Examples:**
```bash
mcp-guard scan
mcp-guard scan --client claude
mcp-guard scan --server "npx server.js"
mcp-guard scan --output sarif > results.sarif
```

### `mcp-guard list`

List discovered MCP servers.

```bash
mcp-guard list [OPTIONS]
```

**Options:**
- `--client <NAME>` - Only list servers from this client

**Examples:**
```bash
mcp-guard list
mcp-guard list --client cursor
```

### `mcp-guard serve`

Start the web dashboard and API server.

```bash
mcp-guard serve [OPTIONS]
```

**Options:**
- `--port <PORT>` - Port to listen on (default: 9191)
- `--headless` - Don't open browser automatically

**Examples:**
```bash
mcp-guard serve
mcp-guard serve --port 8080
mcp-guard serve --headless
```

### `mcp-guard proxy`

Proxy an MCP server with filtering and audit logging.

```bash
mcp-guard proxy --server <COMMAND>
```

**Options:**
- `--server <COMMAND>` - Server command to proxy (required)

**Examples:**
```bash
mcp-guard proxy --server "npx -y @modelcontextprotocol/server-filesystem /"
```

### `mcp-guard init`

Create default configuration file.

```bash
mcp-guard init [OPTIONS]
```

**Options:**
- `--force` - Overwrite existing configuration

**Examples:**
```bash
mcp-guard init
mcp-guard init --force
```

### `mcp-guard completions`

Generate shell completions.

```bash
mcp-guard completions --shell <SHELL>
```

**Options:**
- `--shell <SHELL>` - Shell to generate for: bash, zsh, fish, powershell

**Examples:**
```bash
mcp-guard completions --shell bash
mcp-guard completions --shell zsh
```
