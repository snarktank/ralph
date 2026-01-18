use clap::{Parser, ValueEnum};
use rmcp::{transport::stdio, ServiceExt};
use std::path::PathBuf;

mod integrations;
mod mcp;
mod quality;
mod runner;
mod ui;

use mcp::RalphMcpServer;
use runner::{Runner, RunnerConfig};
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

    /// Disable startup animations
    #[arg(long)]
    no_animation: bool,

    /// Suppress all output except errors
    #[arg(long, short)]
    quiet: bool,

    /// Print help information with styled output
    #[arg(long, short)]
    help: bool,

    /// Print version information with build details and mascot
    #[arg(long, short = 'V')]
    version: bool,

    /// Path to PRD file (for default run mode)
    #[arg(long, short, default_value = "prd.json")]
    prd: PathBuf,

    /// Working directory
    #[arg(long, short = 'd')]
    dir: Option<PathBuf>,

    /// Maximum iterations per story
    #[arg(long, default_value = "10")]
    max_iterations: u32,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
#[command(subcommand_negates_reqs = true)]
enum Commands {
    /// Run all stories until complete (default behavior if no command given)
    Run {
        /// Path to PRD file
        #[arg(long, short, default_value = "prd.json")]
        prd: PathBuf,

        /// Working directory
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,

        /// Maximum iterations per story
        #[arg(long, default_value = "10")]
        max_iterations: u32,

        /// Print help information
        #[arg(long, short)]
        help: bool,
    },
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

    // Create help renderer with color and animation settings
    let use_color = !cli.no_color && std::env::var("NO_COLOR").is_err();
    let help_renderer = HelpRenderer::new()
        .with_color(use_color)
        .with_animation(!cli.no_animation);

    // Handle --help flag with styled output
    if cli.help {
        print!("{}", help_renderer.render_help());
        return Ok(());
    }

    // Handle --version flag with styled output and mascot
    if cli.version {
        print!("{}", help_renderer.render_version());
        return Ok(());
    }

    match cli.command {
        Some(Commands::Run { help: true, .. }) => {
            println!("Run all stories from PRD until complete");
            println!();
            println!("Usage: ralph run [OPTIONS]");
            println!("       ralph [OPTIONS]  (default if no command given)");
            println!();
            println!("Options:");
            println!("  -p, --prd <FILE>         Path to PRD file [default: prd.json]");
            println!("  -d, --dir <DIR>          Working directory");
            println!("  --max-iterations <N>     Max iterations per story [default: 10]");
            println!("  -h, --help               Print help information");
            return Ok(());
        }
        Some(Commands::Run {
            ref prd,
            ref dir,
            max_iterations,
            help: false,
        }) => {
            run_stories(&cli, prd.clone(), dir.clone(), max_iterations).await?;
        }
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
            // Default: run stories if prd.json exists, otherwise show help
            // Check multiple locations: prd.json, ralph/prd.json
            let prd_path = find_prd_file(&cli.prd);
            if let Some(prd) = prd_path {
                run_stories(&cli, prd, cli.dir.clone(), cli.max_iterations).await?;
            } else {
                print!("{}", help_renderer.render_help());
            }
        }
    }

    Ok(())
}

/// Find the PRD file, checking multiple locations
fn find_prd_file(default_path: &std::path::Path) -> Option<PathBuf> {
    // Check the specified path first
    if default_path.exists() {
        return Some(default_path.to_path_buf());
    }

    // Check ralph/prd.json
    let ralph_prd = PathBuf::from("ralph/prd.json");
    if ralph_prd.exists() {
        return Some(ralph_prd);
    }

    // Check .ralph/prd.json
    let dot_ralph_prd = PathBuf::from(".ralph/prd.json");
    if dot_ralph_prd.exists() {
        return Some(dot_ralph_prd);
    }

    None
}

/// Run stories from the PRD until all pass
async fn run_stories(
    cli: &Cli,
    prd: PathBuf,
    dir: Option<PathBuf>,
    max_iterations: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let working_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let config = RunnerConfig {
        prd_path: if prd.is_absolute() {
            prd
        } else {
            working_dir.join(&prd)
        },
        working_dir: working_dir.clone(),
        max_iterations_per_story: max_iterations,
        max_total_iterations: 0, // unlimited
        agent_command: None,     // auto-detect
        quiet: cli.quiet,
    };

    let runner = Runner::new(config);
    let result = runner.run().await;

    if result.all_passed {
        Ok(())
    } else {
        Err(format!(
            "Failed: {}/{} stories passed. {}",
            result.stories_passed,
            result.total_stories,
            result.error.unwrap_or_default()
        )
        .into())
    }
}
