# Architecture

This document describes the internal architecture of mcp-scanner.

## Module Overview

```
src/
├── main.rs          # Entry point and CLI
├── cli.rs           # Command definitions
├── error.rs         # Error types
├── discovery/       # MCP server discovery
│   ├── clients/     # Per-client parsers
│   └── config.rs    # Server configuration
├── scanner/         # Security scanning
│   ├── threats/     # Threat detectors
│   ├── snapshot.rs  # Description drift tracking
│   └── report.rs    # Scan results
├── proxy/           # STDIO proxy
│   ├── interceptor.rs
│   └── rules.rs
├── protocol/        # MCP protocol
│   ├── jsonrpc.rs   # JSON-RPC 2.0
│   ├── mcp.rs       # MCP types
│   └── transport/   # Transport implementations
├── db/              # SQLite storage
│   ├── audit.rs     # Audit logging
│   └── migrations.rs
└── web/             # Web dashboard
    ├── api.rs       # REST API
    └── ui.rs        # htmx UI
```

## Discovery

The discovery module finds MCP servers across AI clients:

1. `McpClientParser` trait defines how to parse client configs
2. Each client has a parser in `discovery/clients/`
3. `all_clients()` returns all available parsers
4. `discover_all()` finds and parses all configs

## Scanning

The scanner connects to MCP servers and analyzes them:

1. `Scanner` manages the scanning process
2. `ThreatDetector` trait defines threat detection
3. Each detector in `scanner/threats/` checks for specific issues
4. Results are collected into `ScanResult`

## Proxy

The proxy intercepts tool calls:

1. Client connects to mcp-scanner via STDIO
2. mcp-scanner spawns the real server
3. JSON-RPC messages are intercepted
4. Rules are applied to tool calls
5. Audit log entries are created

## Protocol

MCP communication uses JSON-RPC 2.0:

1. `Message` enum represents requests/responses/notifications
2. `StdioTransport` handles STDIO communication
3. MCP-specific types in `mcp.rs`

## Database

SQLite stores persistent data:

1. `r2d2` connection pool
2. Migrations run on startup
3. `AuditLog` for tool call history
4. Snapshots stored as JSON files (not in DB)

## Web Dashboard

htmx-powered web interface:

1. Axum web server
2. REST API for data access
3. Server-rendered HTML with htmx for interactivity
