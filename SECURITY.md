# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in mcp-scanner, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Open a private security advisory on GitHub or email the maintainer with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes (optional)

We will acknowledge receipt within 48 hours and provide a detailed response within 7 days.

## Scope

This policy applies to:
- The mcp-scanner CLI tool
- The mcp-scanner web dashboard and API
- The proxy functionality

## What to Report

- Security vulnerabilities in mcp-scanner code
- Bypasses of threat detection
- Privilege escalation in the proxy
- SQL injection or other injection attacks
- Authentication/authorization issues
- Information disclosure

## What Not to Report

- Vulnerabilities in dependencies (report to the dependency maintainers)
- Vulnerabilities in MCP servers that mcp-scanner scans (not our code)
- Denial of service attacks
- Social engineering

## Safe Harbor

We will not pursue legal action against researchers who:
- Report vulnerabilities in good faith
- Do not exploit vulnerabilities beyond verification
- Do not access, modify, or delete data belonging to others
- Give us reasonable time to address the issue before disclosure

## Security Measures

mcp-scanner implements several security measures:

### Data Protection
- All data stored locally in SQLite
- No data sent to external servers
- Credentials/tokens not logged in audit entries

### Proxy Security
- Tool calls are logged but results are sanitized
- Rule-based blocking prevents unauthorized operations
- All operations are auditable

### Detection
- Prompt injection patterns are regularly updated
- Hash-based drift detection for tool descriptions
- Cross-server tool shadowing detection

## Updating

Keep mcp-scanner updated to get the latest security patches:

```bash
cargo install mcp-scanner --force
```
