use clap::{ArgAction, Parser, ValueEnum};
use rmcp::{transport::stdio, ServiceExt};
use std::path::PathBuf;
use std::process::ExitCode;

use ralphmacchio::audit;
use ralphmacchio::checkpoint::{CheckpointManager, PauseReason};
use ralphmacchio::logging::{init_logging, LoggingConfig};
use ralphmacchio::mcp::RalphMcpServer;
use ralphmacchio::runner::{Runner, RunnerConfig};
use ralphmacchio::ui::{DisplayOptions, HelpRenderer, UiMode};

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

/// Output format for audit reports
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum AuditOutputFormat {
    /// JSON structured output
    #[default]
    Json,
    /// Human-readable markdown report
    Markdown,
    /// Agent context format for AI assistants
    Context,
    /// Generate all output formats
    All,
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

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(long, short, action = ArgAction::Count, conflicts_with = "quiet")]
    verbose: u8,

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

    /// Enable parallel story execution
    #[arg(long)]
    parallel: bool,

    /// Max concurrent stories (0 = unlimited)
    #[arg(long, default_value = "3")]
    max_concurrency: usize,

    /// Resume from checkpoint if available
    #[arg(long)]
    resume: bool,

    /// Skip checkpoint prompt (do not resume)
    #[arg(long, conflicts_with = "resume")]
    no_resume: bool,

    /// Agent timeout in seconds (overrides default)
    #[arg(long, value_name = "SECONDS")]
    timeout: Option<u64>,

    /// Disable checkpointing
    #[arg(long)]
    no_checkpoint: bool,

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

        /// Enable parallel story execution
        #[arg(long)]
        parallel: bool,

        /// Max concurrent stories (0 = unlimited)
        #[arg(long, default_value = "3")]
        max_concurrency: usize,

        /// Resume from checkpoint if available
        #[arg(long)]
        resume: bool,

        /// Skip checkpoint prompt (do not resume)
        #[arg(long, conflicts_with = "resume")]
        no_resume: bool,

        /// Agent timeout in seconds (overrides default)
        #[arg(long, value_name = "SECONDS")]
        timeout: Option<u64>,

        /// Disable checkpointing
        #[arg(long)]
        no_checkpoint: bool,

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
    /// Audit a codebase for structure, patterns, and opportunities
    Audit {
        /// Target directory to audit
        #[arg(long, short = 'd', default_value = ".")]
        dir: PathBuf,

        /// Output format (json, markdown, context, all)
        #[arg(long, short = 'f', default_value = "json", value_enum)]
        format: AuditOutputFormat,

        /// Output file path (defaults to stdout for single format, or audit-report.{ext} for 'all')
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Quality profile for analysis thresholds
        #[arg(long, default_value = "default")]
        profile: String,

        /// Enable smart Q&A mode for interactive analysis
        #[arg(long)]
        smart: bool,

        /// Skip interactive Q&A prompts
        #[arg(long)]
        no_interactive: bool,

        /// Auto-generate PRD from audit findings
        #[arg(long)]
        generate_prd: bool,

        /// Print help information
        #[arg(long, short)]
        help: bool,
    },
    /// Check execution state without starting a run
    Status {
        /// Working directory (where .ralph directory is located)
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,

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
        .with_verbosity(cli.verbose)
        .with_streaming(true) // Streaming is now default
        .with_expand_details(cli.verbose >= 1) // Expand details at -v or higher
}

/// Exit codes for the status command
mod exit_codes {
    use std::process::ExitCode;

    /// System is idle (no checkpoint, no active execution)
    pub fn idle() -> ExitCode {
        ExitCode::from(0)
    }

    /// Last execution failed
    pub fn failed() -> ExitCode {
        ExitCode::from(1)
    }

    /// Execution is paused (checkpoint exists)
    pub fn paused() -> ExitCode {
        ExitCode::from(75)
    }
}

#[tokio::main]
async fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
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
        return Ok(ExitCode::SUCCESS);
    }

    // Handle --version flag with styled output and mascot
    if cli.version {
        print!("{}", help_renderer.render_version());
        return Ok(ExitCode::SUCCESS);
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
            println!("  --parallel               Enable parallel story execution");
            println!(
                "  --max-concurrency <N>    Max concurrent stories (0 = unlimited) [default: 3]"
            );
            println!("  --resume                 Resume from checkpoint if available");
            println!("  --no-resume              Skip checkpoint prompt (do not resume)");
            println!("  --timeout <SECONDS>      Agent timeout in seconds (overrides default)");
            println!("  --no-checkpoint          Disable checkpointing");
            println!("  -h, --help               Print help information");
            return Ok(ExitCode::SUCCESS);
        }
        Some(Commands::Run {
            ref prd,
            ref dir,
            max_iterations,
            parallel,
            max_concurrency,
            resume,
            no_resume,
            timeout,
            no_checkpoint,
            help: false,
        }) => {
            run_stories(
                &cli,
                prd.clone(),
                dir.clone(),
                max_iterations,
                parallel,
                max_concurrency,
                resume,
                no_resume,
                timeout,
                no_checkpoint,
            )
            .await?;
        }
        Some(Commands::Quality { help: true }) => {
            println!("Run quality checks (typecheck, lint, test)");
            println!();
            println!("Usage: ralph quality");
            println!();
            println!("Options:");
            println!("  -h, --help  Print help information");
            return Ok(ExitCode::SUCCESS);
        }
        Some(Commands::Quality { help: false }) => {
            // Initialize logging for quality checks (unless quiet)
            if !cli.quiet {
                init_logging(LoggingConfig::from_verbosity(cli.verbose));
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
            return Ok(ExitCode::SUCCESS);
        }
        Some(Commands::McpServer { prd, help: false }) => {
            // Configure logging to stderr only for MCP server mode
            // (stdout is reserved for MCP protocol communication)
            if !cli.quiet {
                init_logging(LoggingConfig::from_verbosity(cli.verbose));
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
        Some(Commands::Audit { help: true, .. }) => {
            println!("Audit a codebase for structure, patterns, and opportunities");
            println!();
            println!("Usage: ralph audit [OPTIONS]");
            println!();
            println!("Options:");
            println!("  -d, --dir <DIR>        Target directory to audit [default: .]");
            println!("  -f, --format <FORMAT>  Output format: json, markdown, context, all [default: json]");
            println!("  -o, --output <FILE>    Output file path (stdout if not specified)");
            println!("  --profile <NAME>       Quality profile for analysis [default: default]");
            println!("  --smart                Enable smart Q&A mode for interactive analysis");
            println!("  --no-interactive       Skip interactive Q&A prompts");
            println!("  --generate-prd         Auto-generate PRD from audit findings");
            println!("  -h, --help             Print help information");
            return Ok(ExitCode::SUCCESS);
        }
        Some(Commands::Audit {
            ref dir,
            format,
            ref output,
            ref profile,
            smart,
            no_interactive,
            generate_prd,
            help: false,
        }) => {
            run_audit(
                &cli,
                dir.clone(),
                format,
                output.clone(),
                profile.clone(),
                smart,
                no_interactive,
                generate_prd,
            )
            .await?;
        }
        Some(Commands::Status { help: true, .. }) => {
            println!("Check execution state without starting a run");
            println!();
            println!("Usage: ralph status [OPTIONS]");
            println!();
            println!("Options:");
            println!("  -d, --dir <DIR>  Working directory [default: .]");
            println!("  -h, --help       Print help information");
            println!();
            println!("Exit codes:");
            println!("  0   Idle (no checkpoint, no active execution)");
            println!("  1   Failed (last execution failed)");
            println!("  75  Paused (checkpoint exists, execution paused)");
            return Ok(ExitCode::SUCCESS);
        }
        Some(Commands::Status {
            ref dir,
            help: false,
        }) => {
            return run_status(dir.clone(), cli.quiet);
        }
        None => {
            // Default: run stories if prd.json exists, otherwise show help
            // Check multiple locations: prd.json, ralph/prd.json
            let prd_path = find_prd_file(&cli.prd);
            if let Some(prd) = prd_path {
                run_stories(
                    &cli,
                    prd,
                    cli.dir.clone(),
                    cli.max_iterations,
                    cli.parallel,
                    cli.max_concurrency,
                    cli.resume,
                    cli.no_resume,
                    cli.timeout,
                    cli.no_checkpoint,
                )
                .await?;
            } else {
                print!("{}", help_renderer.render_help());
            }
        }
    }

    Ok(ExitCode::SUCCESS)
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
#[allow(clippy::too_many_arguments)]
async fn run_stories(
    cli: &Cli,
    prd: PathBuf,
    dir: Option<PathBuf>,
    max_iterations: u32,
    parallel: bool,
    max_concurrency: usize,
    resume: bool,
    no_resume: bool,
    timeout: Option<u64>,
    no_checkpoint: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use ralphmacchio::parallel::scheduler::ParallelRunnerConfig;

    let working_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let display_options = build_display_options(cli);

    // Build parallel config with the specified max_concurrency
    // 0 means unlimited, which we represent with usize::MAX
    let parallel_config = ParallelRunnerConfig {
        max_concurrency: if max_concurrency == 0 {
            u32::MAX
        } else {
            max_concurrency as u32
        },
        ..Default::default()
    };

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
        display_options,
        parallel,
        parallel_config: Some(parallel_config),
        resume,
        no_resume,
        timeout_seconds: timeout,
        no_checkpoint,
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

/// Run the codebase audit
#[allow(clippy::too_many_arguments)]
async fn run_audit(
    cli: &Cli,
    dir: PathBuf,
    format: AuditOutputFormat,
    output: Option<PathBuf>,
    _profile: String,
    _smart: bool,
    _no_interactive: bool,
    generate_prd: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use audit::{
        AgentContext, AgentContextWriter, AuditReport, InventoryScanner, JsonReportWriter,
        MarkdownReportWriter, PrdConverter, PrdConverterConfig, PrdGenerator, PrdGeneratorConfig,
    };
    use std::time::Instant;

    let start_time = Instant::now();

    // Resolve the target directory
    let target_dir = if dir.is_absolute() {
        dir
    } else {
        std::env::current_dir()?.join(&dir)
    };

    if !target_dir.exists() {
        return Err(format!("Directory not found: {}", target_dir.display()).into());
    }

    if !cli.quiet {
        eprintln!("Auditing codebase at: {}", target_dir.display());
    }

    // Create the audit report
    let mut report = AuditReport::new(target_dir.clone());

    // Run inventory scan
    let scanner = InventoryScanner::new(target_dir.clone());
    report.inventory = scanner.scan()?;

    // Update metadata with duration
    report.metadata.duration_ms = start_time.elapsed().as_millis() as u64;

    // Generate output based on format
    match format {
        AuditOutputFormat::Json => {
            let json_output = JsonReportWriter::to_json_string(&report)?;
            write_output(&output, &json_output)?;
        }
        AuditOutputFormat::Markdown => {
            let md_output = MarkdownReportWriter::to_markdown_string(&report);
            write_output(&output, &md_output)?;
        }
        AuditOutputFormat::Context => {
            // Create a minimal AgentContext from the report
            let context = AgentContext::new();
            let ctx_output = AgentContextWriter::generate_patterns_section(&context);
            write_output(&output, &ctx_output)?;
        }
        AuditOutputFormat::All => {
            // For 'all' format, write to files with appropriate extensions
            let base_path = output.unwrap_or_else(|| PathBuf::from("audit-report"));
            let base_stem = base_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("audit-report");
            let base_dir = base_path.parent().unwrap_or(std::path::Path::new("."));

            // Write JSON
            let json_path = base_dir.join(format!("{}.json", base_stem));
            let json_output = JsonReportWriter::to_json_string(&report)?;
            std::fs::write(&json_path, &json_output)?;
            if !cli.quiet {
                eprintln!("Wrote JSON report to: {}", json_path.display());
            }

            // Write Markdown
            let md_path = base_dir.join(format!("{}.md", base_stem));
            let md_output = MarkdownReportWriter::to_markdown_string(&report);
            std::fs::write(&md_path, &md_output)?;
            if !cli.quiet {
                eprintln!("Wrote Markdown report to: {}", md_path.display());
            }

            // Write Context
            let ctx_path = base_dir.join(format!("{}.context.md", base_stem));
            let context = AgentContext::new();
            let ctx_output = AgentContextWriter::generate_patterns_section(&context);
            std::fs::write(&ctx_path, &ctx_output)?;
            if !cli.quiet {
                eprintln!("Wrote agent context to: {}", ctx_path.display());
            }
        }
    }

    if !cli.quiet {
        eprintln!("Audit completed in {}ms", start_time.elapsed().as_millis());
    }

    // Handle PRD generation
    if generate_prd || should_prompt_for_prd(&report, cli.quiet) {
        let prd_config = PrdGeneratorConfig::new()
            .with_skip_prompt(generate_prd) // Skip prompt if --generate-prd flag is set
            .with_output_dir(target_dir.join("tasks"));

        let generator = PrdGenerator::with_config(prd_config);

        // Prompt user unless --generate-prd flag is set
        let should_generate = if generate_prd {
            true
        } else {
            generator.prompt_user_confirmation()?
        };

        if should_generate {
            let result = generator.generate(&report)?;
            if !cli.quiet {
                eprintln!(
                    "Generated PRD with {} user stories at: {}",
                    result.story_count,
                    result.prd_path.display()
                );
                eprintln!(
                    "  - {} from findings, {} from opportunities",
                    result.findings_converted, result.opportunities_converted
                );
            }

            // Convert PRD to prd.json
            let converter_config = PrdConverterConfig::new()
                .with_skip_prompt(generate_prd) // Skip prompt if --generate-prd flag is set
                .with_output_dir(target_dir.clone());

            let converter = PrdConverter::with_config(converter_config);

            // Prompt user unless --generate-prd flag is set
            let should_convert = if generate_prd {
                true
            } else {
                converter.prompt_user_confirmation()?
            };

            if should_convert {
                let convert_result = converter.convert(&result.prd_path)?;
                if !cli.quiet {
                    eprintln!(
                        "Converted PRD to prd.json with {} stories at: {}",
                        convert_result.story_count,
                        convert_result.prd_json_path.display()
                    );
                    eprintln!(
                        "  - Project: {}, Branch: {}",
                        convert_result.project_name, convert_result.branch_name
                    );
                    eprintln!("You can now run 'ralph run' to execute the user stories.");
                }
            }
        }
    }

    Ok(())
}

/// Determine if we should prompt the user about PRD generation
fn should_prompt_for_prd(report: &audit::AuditReport, quiet: bool) -> bool {
    // Don't prompt in quiet mode
    if quiet {
        return false;
    }

    // Prompt if there are actionable findings or opportunities
    let has_findings = report
        .findings
        .iter()
        .any(|f| f.severity >= audit::Severity::Medium);
    let has_opportunities = !report.opportunities.is_empty();

    has_findings || has_opportunities
}

/// Write output to file or stdout
fn write_output(path: &Option<PathBuf>, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    match path {
        Some(p) => {
            std::fs::write(p, content)?;
        }
        None => {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(content.as_bytes())?;
        }
    }
    Ok(())
}

/// Run the status command to check execution state
fn run_status(dir: Option<PathBuf>, quiet: bool) -> Result<ExitCode, Box<dyn std::error::Error>> {
    use chrono::Utc;

    let working_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // Try to load checkpoint
    let manager = match CheckpointManager::new(&working_dir) {
        Ok(m) => m,
        Err(e) => {
            if !quiet {
                eprintln!("Error accessing checkpoint directory: {}", e);
            }
            return Ok(exit_codes::failed());
        }
    };

    match manager.load() {
        Ok(Some(checkpoint)) => {
            // Checkpoint exists - execution is paused
            if !quiet {
                println!("Status: PAUSED");
                println!();

                // Display checkpoint info
                if let Some(ref story) = checkpoint.current_story {
                    println!("Current Story: {}", story.story_id);
                    println!("  Iteration: {}/{}", story.iteration, story.max_iterations);
                }

                // Display pause reason
                let reason_str = match &checkpoint.pause_reason {
                    PauseReason::UsageLimitExceeded => "Usage limit exceeded".to_string(),
                    PauseReason::RateLimited => "Rate limited".to_string(),
                    PauseReason::UserRequested => "User requested".to_string(),
                    PauseReason::Timeout => "Timeout".to_string(),
                    PauseReason::IterationBoundary => "Iteration boundary".to_string(),
                    PauseReason::Error(msg) => format!("Error: {}", msg),
                };
                println!("Pause Reason: {}", reason_str);

                // Calculate and display age
                let now = Utc::now();
                let age = now.signed_duration_since(checkpoint.created_at);
                let age_str = format_duration(age);
                println!("Checkpoint Age: {}", age_str);

                // Display uncommitted files if any
                if !checkpoint.uncommitted_files.is_empty() {
                    println!();
                    println!("Uncommitted Files:");
                    for file in &checkpoint.uncommitted_files {
                        println!("  - {}", file);
                    }
                }

                // Display suggested action
                println!();
                println!("Suggested Action:");
                match &checkpoint.pause_reason {
                    PauseReason::UsageLimitExceeded => {
                        println!(
                            "  Wait for usage limits to reset, then run 'ralph run' to resume."
                        );
                    }
                    PauseReason::RateLimited => {
                        println!(
                            "  Wait for rate limits to clear, then run 'ralph run' to resume."
                        );
                    }
                    PauseReason::UserRequested => {
                        println!("  Run 'ralph run' to resume execution.");
                    }
                    PauseReason::Timeout => {
                        println!("  Check for stuck processes, then run 'ralph run' to resume.");
                    }
                    PauseReason::IterationBoundary => {
                        println!("  Run 'ralph run' to resume execution.");
                    }
                    PauseReason::Error(_) => {
                        println!(
                            "  Review the error above, fix any issues, then run 'ralph run' to resume."
                        );
                    }
                }
            }
            Ok(exit_codes::paused())
        }
        Ok(None) => {
            // No checkpoint - system is idle
            if !quiet {
                println!("Status: IDLE");
                println!();
                println!("No active execution or checkpoint found.");
                println!();
                println!("Suggested Action:");
                println!("  Run 'ralph run' to start executing user stories.");
            }
            Ok(exit_codes::idle())
        }
        Err(e) => {
            // Error loading checkpoint - treat as failed
            if !quiet {
                eprintln!("Status: ERROR");
                eprintln!();
                eprintln!("Failed to load checkpoint: {}", e);
                eprintln!();
                eprintln!("Suggested Action:");
                eprintln!(
                    "  Check .ralph/checkpoint.json for corruption or remove it to start fresh."
                );
            }
            Ok(exit_codes::failed())
        }
    }
}

/// Format a duration in a human-readable way
fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds().unsigned_abs();

    if total_seconds < 60 {
        format!("{} seconds ago", total_seconds)
    } else if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        format!(
            "{} minute{} ago",
            minutes,
            if minutes == 1 { "" } else { "s" }
        )
    } else if total_seconds < 86400 {
        let hours = total_seconds / 3600;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else {
        let days = total_seconds / 86400;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    }
}
