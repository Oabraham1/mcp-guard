//! Command-line interface definitions using clap.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mcp-guard")]
#[command(author, version, about = "Security scanner and proxy for MCP servers")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format
    #[arg(short, long, global = true, default_value = "table")]
    pub output: OutputFormat,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan MCP servers for security vulnerabilities
    Scan {
        /// Scan only servers from a specific client
        #[arg(long)]
        client: Option<String>,

        /// Scan a specific server command directly
        #[arg(long)]
        server: Option<String>,

        /// Scan servers from a config file
        #[arg(long)]
        config: Option<PathBuf>,

        /// Timeout in seconds for each server
        #[arg(long, default_value = "30")]
        timeout: u64,
    },

    /// Watch for config changes and re-scan automatically
    Watch {
        /// Clients to watch (comma-separated, or all if not specified)
        #[arg(long)]
        clients: Option<String>,
    },

    /// Start a proxy between client and MCP server
    Proxy {
        /// Server command to proxy
        #[arg(long, required_unless_present = "config")]
        server: Option<String>,

        /// Proxy rules config file
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// Start the web UI and API server
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "9191")]
        port: u16,

        /// Address to bind to
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,

        /// Don't open browser automatically
        #[arg(long)]
        headless: bool,
    },

    /// Initialize mcp-guard configuration
    Init {
        /// Force overwrite existing config
        #[arg(long)]
        force: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(long)]
        shell: Shell,
    },

    /// List discovered MCP servers
    List {
        /// Only list servers from a specific client
        #[arg(long)]
        client: Option<String>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Sarif,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    #[value(name = "powershell")]
    Pwsh,
}

impl Shell {
    pub fn to_clap_shell(self) -> clap_complete::Shell {
        match self {
            Shell::Bash => clap_complete::Shell::Bash,
            Shell::Zsh => clap_complete::Shell::Zsh,
            Shell::Fish => clap_complete::Shell::Fish,
            Shell::Pwsh => clap_complete::Shell::PowerShell,
        }
    }
}
