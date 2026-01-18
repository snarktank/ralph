use clap::{Parser, ValueEnum};
use rmcp::{transport::stdio, ServiceExt};
use std::path::PathBuf;

mod integrations;
mod mcp;
mod quality;
mod ui;

use mcp::RalphMcpServer;
use ui::{DisplayOptions, HelpRenderer, UiMode};

/// UI mode for terminal display
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum CliUiMode {
    /// Auto-detect based on terminal capabilities
    #[default]
    Auto,
    /// Force enable rich terminal UI
    Enabled,
    /// Force disable rich terminal UI (plain text only)
    Disabled,
}

impl From<CliUiMode> for UiMode {
    fn from(mode: CliUiMode) -> Self {
        match mode {
            CliUiMode::Auto => UiMode::Auto,
            CliUiMode::Enabled => UiMode::Enabled,
            CliUiMode::Disabled => UiMode::Disabled,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "ralph")]
#[command(version)]
#[command(about = "Enterprise-ready autonomous AI agent framework")]
#[command(disable_help_flag = true)]
#[command(disable_version_flag = true)]
struct Cli {
    /// UI mode: auto (default), enabled, or disabled
    #[arg(long, default_value = "auto", value_enum)]
    ui: CliUiMode,

    /// Disable colors (also respects NO_COLOR environment variable)
    #[arg(long)]
    no_color: bool,

    /// Suppress all output except errors
    #[arg(long, short)]
    quiet: bool,

    /// Print help information with styled output
    #[arg(long, short)]
    help: bool,

    /// Print version information with build details
    #[arg(long, short = 'V')]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
#[command(subcommand_negates_reqs = true)]
enum Commands {
    /// Run quality checks
    Quality {
        /// Print help information
        #[arg(long, short)]
        help: bool,
    },
    /// Start MCP server mode for integration with AI assistants
    McpServer {
        /// Path to PRD file to preload (optional)
        #[arg(long)]
        prd: Option<PathBuf>,

        /// Print help information
        #[arg(long, short)]
        help: bool,
    },
}

/// Build display options from CLI arguments
fn build_display_options(cli: &Cli) -> DisplayOptions {
    DisplayOptions::new()
        .with_ui_mode(cli.ui.into())
        .with_color(!cli.no_color)
        .with_quiet(cli.quiet)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Build display options from CLI flags
    let display_options = build_display_options(&cli);

    // Create help renderer with color settings
    let help_renderer =
        HelpRenderer::new().with_color(!cli.no_color && std::env::var("NO_COLOR").is_err());

    // Handle --help flag with styled output
    if cli.help {
        print!("{}", help_renderer.render_help());
        return Ok(());
    }

    // Handle --version flag with styled output
    if cli.version {
        print!("{}", help_renderer.render_version());
        return Ok(());
    }

    match cli.command {
        Some(Commands::Quality { help: true }) => {
            println!("Run quality checks (typecheck, lint, test)");
            println!();
            println!("Usage: ralph quality");
            println!();
            println!("Options:");
            println!("  -h, --help  Print help information");
            return Ok(());
        }
        Some(Commands::Quality { help: false }) => {
            // Initialize logging to stdout for quality checks (unless quiet)
            if !cli.quiet {
                tracing_subscriber::fmt::init();
                println!("Running quality checks...");
            }
        }
        Some(Commands::McpServer { help: true, .. }) => {
            println!("Start MCP server mode for integration with AI assistants");
            println!();
            println!("Usage: ralph mcp-server [OPTIONS]");
            println!();
            println!("Options:");
            println!("  --prd <FILE>  Path to PRD file to preload (optional)");
            println!("  -h, --help    Print help information");
            return Ok(());
        }
        Some(Commands::McpServer { prd, help: false }) => {
            // Configure logging to stderr only for MCP server mode
            // (stdout is reserved for MCP protocol communication)
            if !cli.quiet {
                tracing_subscriber::fmt()
                    .with_writer(std::io::stderr)
                    .init();
            }

            // Create the server, optionally with a preloaded PRD
            let server = match prd {
                Some(path) => {
                    if !cli.quiet {
                        tracing::info!("Starting MCP server with preloaded PRD: {:?}", path);
                    }
                    RalphMcpServer::with_prd_and_display(path, display_options)
                }
                None => {
                    if !cli.quiet {
                        tracing::info!("Starting MCP server");
                    }
                    RalphMcpServer::with_display(display_options)
                }
            };

            // Start the MCP server using stdio transport
            let service = server.serve(stdio()).await.map_err(|e| {
                tracing::error!("Error starting MCP server: {}", e);
                e
            })?;

            // Wait for the service to complete
            service.waiting().await?;
        }
        None => {
            // Initialize logging to stdout for default mode (unless quiet)
            if !cli.quiet {
                tracing_subscriber::fmt::init();
                println!("Ralph - Enterprise-ready autonomous AI agent framework");
                println!("Use --help for available commands");
            }
        }
    }

    Ok(())
}
