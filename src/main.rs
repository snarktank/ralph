use clap::Parser;
use rmcp::{transport::stdio, ServiceExt};
use std::path::PathBuf;

mod integrations;
mod mcp;
mod quality;

use mcp::RalphMcpServer;

#[derive(Parser, Debug)]
#[command(name = "ralph")]
#[command(version)]
#[command(about = "Enterprise-ready autonomous AI agent framework")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Run quality checks
    Quality,
    /// Start MCP server mode for integration with AI assistants
    McpServer {
        /// Path to PRD file to preload (optional)
        #[arg(long)]
        prd: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Quality) => {
            // Initialize logging to stdout for quality checks
            tracing_subscriber::fmt::init();
            println!("Running quality checks...");
        }
        Some(Commands::McpServer { prd }) => {
            // Configure logging to stderr only for MCP server mode
            // (stdout is reserved for MCP protocol communication)
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .init();

            // Create the server, optionally with a preloaded PRD
            let server = match prd {
                Some(path) => {
                    tracing::info!("Starting MCP server with preloaded PRD: {:?}", path);
                    RalphMcpServer::with_prd(path)
                }
                None => {
                    tracing::info!("Starting MCP server");
                    RalphMcpServer::new()
                }
            };

            // Start the MCP server using stdio transport
            let service = server
                .serve(stdio())
                .await
                .inspect_err(|e| tracing::error!("Error starting MCP server: {}", e))?;

            // Wait for the service to complete
            service.waiting().await?;
        }
        None => {
            // Initialize logging to stdout for default mode
            tracing_subscriber::fmt::init();
            println!("Ralph - Enterprise-ready autonomous AI agent framework");
            println!("Use --help for available commands");
        }
    }

    Ok(())
}
