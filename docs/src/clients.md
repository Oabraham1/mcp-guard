# Supported Clients

mcp-scanner automatically discovers MCP server configurations from these AI clients.

## Claude Desktop

**Config path:**
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

**Format:**
```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/"]
    }
  }
}
```

## Cursor

**Config path:** `~/.cursor/mcp.json`

**Format:** Same as Claude Desktop.

## Windsurf

**Config path:** `~/.codeium/windsurf/mcp_config.json`

**Format:** Same as Claude Desktop.

## Zed

**Config path:** `~/.config/zed/settings.json`

**Format:**
```json
{
  "context_servers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/"]
    }
  }
}
```

## Cline

**Config path:** `~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json`

**Format:** Same as Claude Desktop.

## Continue

**Config path:** `~/.continue/config.json`

**Format:**
```json
{
  "mcpServers": [
    {
      "name": "server-name",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/"]
    }
  ]
}
```

## VS Code

**Config path:** `.vscode/mcp.json` (per-workspace)

**Format:** Same as Claude Desktop.

## Roo Code

**Config path:** `~/.config/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/mcp_settings.json`

**Format:** Same as Claude Desktop.

## Claude Code

**Config paths:**
- Global: `~/.claude/settings.json`
- Project: `.mcp.json`

**Format:** Same as Claude Desktop.

## Adding Custom Configs

Use `--config` to scan a custom configuration file:

```bash
mcp-scanner scan --config /path/to/mcp.json
```

The file should use the Claude Desktop format.
