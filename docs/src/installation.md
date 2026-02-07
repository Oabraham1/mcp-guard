# Installation

## From Crates.io

```bash
cargo install mcp-guard
```

## From Source

```bash
git clone https://github.com/oabraham1/mcp-guard
cd mcp-guard
cargo build --release
```

The binary will be at `target/release/mcp-guard`.

## Requirements

- Rust 1.75 or later (for building from source)
- SQLite (bundled, no separate installation needed)

## Verify Installation

```bash
mcp-guard --version
mcp-guard --help
```

## Shell Completions

Generate completions for your shell:

```bash
# Bash
mcp-guard completions --shell bash >> ~/.bashrc

# Zsh
mcp-guard completions --shell zsh >> ~/.zshrc

# Fish
mcp-guard completions --shell fish > ~/.config/fish/completions/mcp-guard.fish

# PowerShell
mcp-guard completions --shell powershell >> $PROFILE
```
