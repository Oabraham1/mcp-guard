# Introduction

**mcp-guard** is a security scanner and proxy for MCP (Model Context Protocol) servers.

MCP servers provide tools to AI assistants like Claude, and a compromised or malicious server can inject instructions into the AI's context, exfiltrate data, or execute arbitrary code. mcp-guard helps you detect and mitigate these risks.

## What is MCP?

The Model Context Protocol (MCP) is a standard for AI assistants to interact with external tools and data sources. MCP servers provide:

- **Tools**: Functions the AI can call (read files, search databases, call APIs)
- **Resources**: Data the AI can access (file contents, database records)
- **Prompts**: Pre-defined conversation templates

## Why mcp-guard?

As MCP adoption grows, so do the security risks:

1. **Supply chain attacks**: A malicious npm package update could inject prompt injection into tool descriptions
2. **Privilege escalation**: A compromised server could use overly broad permissions to access sensitive data
3. **Description drift**: Legitimate servers may be compromised, with changes going unnoticed
4. **Tool shadowing**: Malicious servers can register tools with names that shadow legitimate ones

mcp-guard provides:

- **Automated discovery** of MCP servers across all your AI tools
- **Security scanning** for common vulnerabilities and misconfigurations
- **Proxy mode** for runtime filtering and audit logging
- **Snapshot tracking** to detect changes over time

## Quick Example

```bash
# Scan all your MCP servers
mcp-guard scan

# Example output:
# Server: @modelcontextprotocol/server-filesystem
#   [CRITICAL] Description contains prompt injection pattern
#   [HIGH] Tool has root filesystem access
#
# Server: custom-database-mcp
#   [INFO] No authentication configured
```

## Getting Started

1. [Install mcp-guard](./installation.md)
2. [Run your first scan](./quickstart.md)
3. [Configure threat detection](./configuration.md)
