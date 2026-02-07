# Installation

## From Crates.io

```bash
cargo install mcp-scanner
```

## From Source

```bash
git clone https://github.com/oabraham1/mcp-scanner
cd mcp-scanner
cargo build --release
```

The binary will be at `target/release/mcp-scanner`.

## Requirements

- Rust 1.75 or later (for building from source)
- SQLite (bundled, no separate installation needed)

## Verify Installation

```bash
mcp-scanner --version
mcp-scanner --help
```

## Shell Completions

Generate completions for your shell:

```bash
# Bash
mcp-scanner completions --shell bash >> ~/.bashrc

# Zsh
mcp-scanner completions --shell zsh >> ~/.zshrc

# Fish
mcp-scanner completions --shell fish > ~/.config/fish/completions/mcp-scanner.fish

# PowerShell
mcp-scanner completions --shell powershell >> $PROFILE
```
