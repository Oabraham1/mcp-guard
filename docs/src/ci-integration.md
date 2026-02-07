# CI/CD Integration

mcp-guard can be integrated into your CI/CD pipeline to catch security issues before deployment.

## SARIF Output

mcp-guard supports SARIF (Static Analysis Results Interchange Format), which is compatible with GitHub Code Scanning, Azure DevOps, and other tools.

```bash
mcp-guard scan --output sarif > results.sarif
```

## GitHub Actions

```yaml
name: MCP Security Scan

on:
  push:
    paths:
      - '.vscode/mcp.json'
      - 'mcp.json'
  pull_request:
    paths:
      - '.vscode/mcp.json'
      - 'mcp.json'

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install mcp-guard
        run: cargo install mcp-guard

      - name: Run security scan
        run: mcp-guard scan --config .vscode/mcp.json --output sarif > results.sarif

      - name: Upload SARIF results
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: results.sarif
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No critical or high severity threats |
| 1 | Critical or high severity threats found |
| 2 | Error during scanning |

## Fail on Severity

Use `jq` to fail on specific severities:

```bash
mcp-guard scan --output json | jq -e '.threats | map(select(.severity == "critical" or .severity == "high")) | length == 0'
```

## Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

if [ -f ".vscode/mcp.json" ]; then
  mcp-guard scan --config .vscode/mcp.json
  if [ $? -ne 0 ]; then
    echo "MCP security issues found. Fix them before committing."
    exit 1
  fi
fi
```
