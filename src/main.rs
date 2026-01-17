use clap::Parser;

mod integrations;
mod mcp;
mod quality;

#[derive(Parser, Debug)]
#[command(name = "ralph")]
#[command(about = "Enterprise-ready autonomous AI agent framework")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Run quality checks
    Quality,
    /// Start MCP server
    McpServer,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    tracing_subscriber::fmt::init();

    match cli.command {
        Some(Commands::Quality) => {
            println!("Running quality checks...");
        }
        Some(Commands::McpServer) => {
            println!("Starting MCP server...");
        }
        None => {
            println!("Ralph - Enterprise-ready autonomous AI agent framework");
            println!("Use --help for available commands");
        }
    }

    Ok(())
}
