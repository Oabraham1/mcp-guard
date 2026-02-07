# API Reference

The mcp-guard web server exposes a REST API for programmatic access.

## Base URL

Default: `http://localhost:9191`

## Endpoints

### Health Check

```
GET /api/health
```

Returns server health status.

**Response:**
```json
{
  "status": "ok"
}
```

### List Servers

```
GET /api/servers
```

List all discovered MCP servers.

**Response:**
```json
{
  "servers": [
    {
      "name": "filesystem",
      "client": "claude",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/"],
      "transport": "stdio"
    }
  ]
}
```

### Run Scan

```
POST /api/scan
```

Scan discovered servers for security threats.

**Request Body (optional):**
```json
{
  "client": "claude",
  "server": "filesystem"
}
```

**Response:**
```json
{
  "results": [
    {
      "server": "filesystem",
      "threats": [
        {
          "id": "PERM-EXEC-shell",
          "severity": "high",
          "category": "permission_scope",
          "title": "Code execution capability",
          "message": "Tool 'shell' can execute arbitrary code",
          "remediation": "Limit command execution to specific commands"
        }
      ],
      "tools": [
        {
          "name": "read_file",
          "description": "Read a file from disk"
        }
      ]
    }
  ]
}
```

### List Audit Entries

```
GET /api/audit
```

**Query Parameters:**
- `limit` - Max entries to return (default: 100)
- `offset` - Pagination offset
- `server` - Filter by server name
- `tool` - Filter by tool name
- `blocked` - Filter by blocked status (true/false)

**Response:**
```json
{
  "entries": [
    {
      "id": 1,
      "timestamp": "2024-01-15T12:00:00Z",
      "server_name": "filesystem",
      "tool_name": "read_file",
      "tool_args": {"path": "/tmp/test.txt"},
      "result": {"content": "Hello, world!"},
      "blocked": false,
      "duration_ms": 15
    }
  ],
  "total": 150
}
```

### List Rules

```
GET /api/rules
```

**Response:**
```json
{
  "rules": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "rule_type": "block",
      "pattern": "delete_*",
      "reason": "Prevent deletions",
      "priority": 10,
      "enabled": true
    }
  ]
}
```

### Create Rule

```
POST /api/rules
```

**Request Body:**
```json
{
  "rule_type": "block",
  "pattern": "delete_*",
  "reason": "Prevent destructive operations",
  "priority": 10
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Update Rule

```
PUT /api/rules/:id
```

**Request Body:**
```json
{
  "enabled": false
}
```

### Delete Rule

```
DELETE /api/rules/:id
```

## Error Responses

```json
{
  "error": "Not found",
  "message": "Rule with ID xxx not found"
}
```

HTTP status codes:
- `400` - Bad request
- `404` - Not found
- `500` - Internal server error
