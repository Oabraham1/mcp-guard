//! mcp-scanner: Security scanner and proxy for MCP servers.

mod api;
mod cli;
mod db;
mod discovery;
mod error;
mod protocol;
mod proxy;
mod scanner;
mod ui;

use clap::Parser;
use cli::{Cli, Commands, OutputFormat};
use colored::Colorize;
use discovery::{discover_all, discover_from_client, ServerConfig};
use error::Result;
use scanner::{ScanResult, Scanner, Severity};
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("mcp_guard=debug")
    } else {
        EnvFilter::new("mcp_guard=info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Commands::Scan {
            client,
            server,
            config,
            timeout,
        } => {
            cmd_scan(client, server, config, timeout, cli.output).await?;
        }
        Commands::Watch { clients } => {
            cmd_watch(clients).await?;
        }
        Commands::Proxy { server, config } => {
            cmd_proxy(server, config).await?;
        }
        Commands::Serve {
            port,
            bind,
            headless,
        } => {
            cmd_serve(port, bind, headless).await?;
        }
        Commands::Init { force } => {
            cmd_init(force)?;
        }
        Commands::Completions { shell } => {
            cmd_completions(shell);
        }
        Commands::List { client } => {
            cmd_list(client)?;
        }
    }

    Ok(())
}

async fn cmd_scan(
    client: Option<String>,
    server: Option<String>,
    config: Option<std::path::PathBuf>,
    timeout: u64,
    output: OutputFormat,
) -> Result<()> {
    let servers = if let Some(server_cmd) = server {
        // Parse server command: "npx -y @modelcontextprotocol/server-filesystem /"
        let parts: Vec<String> = shell_words::split(&server_cmd)
            .map_err(|e| error::Error::Other(format!("Invalid server command: {}", e)))?;

        if parts.is_empty() {
            return Err(error::Error::Other("Empty server command".to_string()));
        }

        vec![ServerConfig::new("manual", &parts[0]).with_args(parts[1..].to_vec())]
    } else if let Some(config_path) = config {
        discovery::clients::GenericDiscovery::new(config_path).parse_file()?
    } else if let Some(client_name) = client {
        discover_from_client(&client_name)?
    } else {
        discover_all()?
    };

    if servers.is_empty() {
        println!("{}", "No MCP servers found.".yellow());
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} server(s), scanning...\n", servers.len()).cyan()
    );

    let scanner = Scanner::new().with_timeout(Duration::from_secs(timeout));
    let mut all_results = Vec::new();

    for server in &servers {
        match scanner.scan(server).await {
            Ok(result) => all_results.push(result),
            Err(e) => {
                eprintln!(
                    "{} {}: {}",
                    "✗".red(),
                    server.name.bold(),
                    e.to_string().red()
                );
            }
        }
    }

    match output {
        OutputFormat::Table => print_table_output(&all_results),
        OutputFormat::Json => print_json_output(&all_results)?,
        OutputFormat::Sarif => print_sarif_output(&all_results)?,
    }

    // Exit with error code if any critical/high threats found
    let has_critical = all_results
        .iter()
        .any(|r| r.threats.iter().any(|t| t.severity <= Severity::High));

    if has_critical {
        std::process::exit(1);
    }

    Ok(())
}

fn print_table_output(results: &[ScanResult]) {
    for result in results {
        let threat_summary = summarize_threats(&result.threats);
        let status = if result.threats.is_empty() {
            "✓".green()
        } else if result
            .threats
            .iter()
            .any(|t| t.severity == Severity::Critical)
        {
            "✗".red()
        } else if result.threats.iter().any(|t| t.severity == Severity::High) {
            "!".red()
        } else {
            "⚠".yellow()
        };

        println!(
            "{} {} ({} tools, {})",
            status,
            result.server.name.bold(),
            result.tools.len(),
            threat_summary
        );

        for threat in &result.threats {
            let severity_str = match threat.severity {
                Severity::Critical => format!("[{}]", threat.severity).red().bold(),
                Severity::High => format!("[{}]", threat.severity).red(),
                Severity::Medium => format!("[{}]", threat.severity).yellow(),
                Severity::Low => format!("[{}]", threat.severity).blue(),
                Severity::Info => format!("[{}]", threat.severity).dimmed(),
            };

            println!("  {} {}", severity_str, threat.title);

            if !threat.message.is_empty() {
                println!("    {}", threat.message.dimmed());
            }

            if !threat.evidence.is_empty() {
                println!("    Evidence: {}", threat.evidence.dimmed());
            }
        }

        if !result.threats.is_empty() {
            println!();
        }
    }

    // Print summary
    let total_threats: usize = results.iter().map(|r| r.threats.len()).sum();
    let total_critical = results
        .iter()
        .flat_map(|r| &r.threats)
        .filter(|t| t.severity == Severity::Critical)
        .count();
    let total_high = results
        .iter()
        .flat_map(|r| &r.threats)
        .filter(|t| t.severity == Severity::High)
        .count();

    println!("\n{}", "─".repeat(50));
    println!(
        "Scanned {} servers, found {} threats",
        results.len().to_string().bold(),
        total_threats.to_string().bold()
    );

    if total_critical > 0 || total_high > 0 {
        println!(
            "  {} critical, {} high severity",
            total_critical.to_string().red(),
            total_high.to_string().red()
        );
    }
}

fn summarize_threats(threats: &[scanner::Threat]) -> String {
    if threats.is_empty() {
        return "no threats".green().to_string();
    }

    let by_severity: std::collections::HashMap<Severity, usize> =
        threats
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, t| {
                *acc.entry(t.severity).or_insert(0) += 1;
                acc
            });

    let mut parts = Vec::new();
    if let Some(&n) = by_severity.get(&Severity::Critical) {
        parts.push(format!("{} critical", n).red().to_string());
    }
    if let Some(&n) = by_severity.get(&Severity::High) {
        parts.push(format!("{} high", n).red().to_string());
    }
    if let Some(&n) = by_severity.get(&Severity::Medium) {
        parts.push(format!("{} medium", n).yellow().to_string());
    }
    if let Some(&n) = by_severity.get(&Severity::Low) {
        parts.push(format!("{} low", n).blue().to_string());
    }
    if let Some(&n) = by_severity.get(&Severity::Info) {
        parts.push(format!("{} info", n).dimmed().to_string());
    }

    parts.join(", ")
}

fn print_json_output(results: &[ScanResult]) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    println!("{}", json);
    Ok(())
}

fn print_sarif_output(results: &[ScanResult]) -> Result<()> {
    // Basic SARIF 2.1.0 output
    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "mcp-scanner",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/oabraham1/mcp-scanner"
                }
            },
            "results": results.iter().flat_map(|r| {
                r.threats.iter().map(|t| {
                    serde_json::json!({
                        "ruleId": t.id,
                        "level": match t.severity {
                            Severity::Critical | Severity::High => "error",
                            Severity::Medium => "warning",
                            _ => "note"
                        },
                        "message": { "text": t.message },
                        "locations": [{
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": r.server.name.clone()
                                }
                            }
                        }]
                    })
                })
            }).collect::<Vec<_>>()
        }]
    });

    let json = serde_json::to_string_pretty(&sarif)?;
    println!("{}", json);
    Ok(())
}

async fn cmd_watch(clients: Option<String>) -> Result<()> {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::collections::HashSet;
    use std::sync::mpsc::channel;

    println!("{}", "Starting watch mode...".cyan());
    println!("{}", "Press Ctrl+C to stop.\n".dimmed());

    let servers = if let Some(ref client_name) = clients {
        discover_from_client(client_name)?
    } else {
        discover_all()?
    };

    if servers.is_empty() {
        println!("{}", "No MCP servers found to watch.".yellow());
        return Ok(());
    }

    let config_paths: HashSet<std::path::PathBuf> =
        servers.iter().filter_map(|s| s.config_path()).collect();

    if config_paths.is_empty() {
        println!("{}", "No config files found to watch.".yellow());
        return Ok(());
    }

    println!("Watching {} config file(s):", config_paths.len());
    for path in &config_paths {
        println!("  {}", path.display().to_string().dimmed());
    }
    println!();

    let scanner = Scanner::new();

    println!("{}", "Running initial scan...".cyan());
    run_watch_scan(&scanner, &servers).await;

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| error::Error::Other(format!("Failed to create watcher: {}", e)))?;

    for path in &config_paths {
        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .map_err(|e| {
                error::Error::Other(format!("Failed to watch {}: {}", path.display(), e))
            })?;
    }

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if event.kind.is_modify() || event.kind.is_create() {
                    println!("\n{}", "Config changed, re-scanning...".cyan());

                    let new_servers = if let Some(ref client_name) = clients {
                        discover_from_client(client_name).unwrap_or_default()
                    } else {
                        discover_all().unwrap_or_default()
                    };

                    run_watch_scan(&scanner, &new_servers).await;
                }
            }
            Ok(Err(e)) => {
                eprintln!("{}", format!("Watch error: {}", e).red());
            }
            Err(e) => {
                eprintln!("{}", format!("Channel error: {}", e).red());
                break;
            }
        }
    }

    Ok(())
}

async fn run_watch_scan(scanner: &Scanner, servers: &[ServerConfig]) {
    for server in servers {
        match scanner.scan(server).await {
            Ok(result) => {
                let status = if result.threats.is_empty() {
                    "✓".green()
                } else if result
                    .threats
                    .iter()
                    .any(|t| t.severity == Severity::Critical)
                {
                    "✗".red()
                } else {
                    "⚠".yellow()
                };

                println!(
                    "{} {} ({} threats)",
                    status,
                    result.server.name.bold(),
                    result.threats.len()
                );

                for threat in &result.threats {
                    let severity_str = match threat.severity {
                        Severity::Critical => format!("[{}]", threat.severity).red().bold(),
                        Severity::High => format!("[{}]", threat.severity).red(),
                        Severity::Medium => format!("[{}]", threat.severity).yellow(),
                        Severity::Low => format!("[{}]", threat.severity).blue(),
                        Severity::Info => format!("[{}]", threat.severity).dimmed(),
                    };
                    println!("  {} {}", severity_str, threat.title);
                }
            }
            Err(e) => {
                eprintln!(
                    "{} {}: {}",
                    "✗".red(),
                    server.name.bold(),
                    e.to_string().red()
                );
            }
        }
    }
}

async fn cmd_proxy(server: Option<String>, _config: Option<std::path::PathBuf>) -> Result<()> {
    let server_cmd = server.ok_or_else(|| {
        error::Error::Other("--server argument required for proxy mode".to_string())
    })?;

    let parts: Vec<String> = shell_words::split(&server_cmd)
        .map_err(|e| error::Error::Other(format!("Invalid server command: {}", e)))?;

    if parts.is_empty() {
        return Err(error::Error::Other("Empty server command".to_string()));
    }

    let command = parts[0].clone();
    let args = parts[1..].to_vec();

    // Set up database for audit logging
    let db_path = db::default_db_path()?;
    let pool = db::create_pool(&db_path)?;

    eprintln!(
        "{}",
        format!("Proxying server: {} {}", command, args.join(" ")).cyan()
    );

    let interceptor = proxy::ProxyInterceptor::new(command, args).with_db(pool);

    interceptor.run().await
}

async fn cmd_serve(port: u16, bind: String, headless: bool) -> Result<()> {
    let db_path = db::default_db_path()?;
    let pool = db::create_pool(&db_path)?;

    let url = format!("http://{}:{}", bind, port);
    println!(
        "{}",
        format!("Starting mcp-scanner server at {}", url).cyan()
    );

    if !headless {
        // Try to open browser
        if let Err(e) = open::that(&url) {
            eprintln!("{}", format!("Failed to open browser: {}", e).yellow());
        }
    }

    api::serve(pool, &bind, port).await
}

fn cmd_init(force: bool) -> Result<()> {
    let home =
        dirs::home_dir().ok_or_else(|| error::Error::Other("No home directory".to_string()))?;
    let config_dir = home.join(".mcp-scanner");
    let config_file = config_dir.join("config.toml");

    if config_file.exists() && !force {
        println!(
            "{}",
            format!(
                "Config already exists at {}. Use --force to overwrite.",
                config_file.display()
            )
            .yellow()
        );
        return Ok(());
    }

    std::fs::create_dir_all(&config_dir)?;

    let default_config = r#"# mcp-scanner configuration

[scan]
timeout = 30  # seconds
# concurrency = 4  # parallel scans

[output]
format = "table"  # table, json, sarif

# [proxy]
# Example proxy rules:
# [[proxy.rules]]
# tool_pattern = "dangerous_*"
# action = "block"
# reason = "Blocked by policy"
"#;

    std::fs::write(&config_file, default_config)?;
    println!(
        "{}",
        format!("Created config at {}", config_file.display()).green()
    );

    Ok(())
}

fn cmd_completions(shell: cli::Shell) {
    use clap::CommandFactory;
    use clap_complete::generate;
    use std::io;

    let mut cmd = Cli::command();
    generate(
        shell.to_clap_shell(),
        &mut cmd,
        "mcp-scanner",
        &mut io::stdout(),
    );
}

fn cmd_list(client: Option<String>) -> Result<()> {
    let servers = if let Some(client_name) = client {
        discover_from_client(&client_name)?
    } else {
        discover_all()?
    };

    if servers.is_empty() {
        println!("{}", "No MCP servers found.".yellow());
        return Ok(());
    }

    println!("{}", format!("Found {} server(s):\n", servers.len()).cyan());

    for server in &servers {
        println!("  {} {}", "•".blue(), server.name.bold());
        println!("    Command: {} {}", server.command, server.args.join(" "));
        println!("    Source: {}", server.display_source().dimmed());

        if !server.env.is_empty() {
            let env_keys: Vec<&str> = server.env.keys().map(|s| s.as_str()).collect();
            println!("    Env: {}", env_keys.join(", ").dimmed());
        }
        println!();
    }

    Ok(())
}
