use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "ralph")]
#[command(author, version, about = "Autonomous AI agent loop using Claude Code")]
#[command(after_help = "Examples:
  ralph                  Run in current directory with defaults
  ralph 20               Run with 20 max iterations
  ralph -d ./my-project  Run in specified directory
  ralph init             Create prd.json template

For more info: https://github.com/kcirtapfromspace/ralph")]
struct Cli {
    /// Working directory (default: current directory)
    #[arg(short, long, value_name = "PATH")]
    dir: Option<PathBuf>,

    /// Custom prompt file
    #[arg(short, long, value_name = "FILE")]
    prompt: Option<PathBuf>,

    /// Max iterations (default: 10)
    #[arg(short = 'n', long, value_name = "N", default_value = "10")]
    iterations: u32,

    /// Max iterations (positional, alternative to -n)
    #[arg(value_name = "MAX_ITERATIONS")]
    max_iterations: Option<u32>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize project with prd.json template
    Init,
    /// Show Ralph installation directory
    Home,
}

#[derive(Debug, Serialize, Deserialize)]
struct Prd {
    project: String,
    #[serde(rename = "branchName")]
    branch_name: String,
    description: String,
    #[serde(rename = "userStories")]
    user_stories: Vec<UserStory>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserStory {
    id: String,
    title: String,
    description: String,
    #[serde(rename = "acceptanceCriteria")]
    acceptance_criteria: Vec<String>,
    priority: u32,
    passes: bool,
    notes: String,
}

fn get_ralph_home() -> Result<PathBuf> {
    // First check RALPH_HOME env var
    if let Ok(home) = env::var("RALPH_HOME") {
        return Ok(PathBuf::from(home));
    }

    // Try to find based on executable location
    let exe_path = env::current_exe().context("Failed to get executable path")?;
    let exe_dir = exe_path
        .parent()
        .context("Failed to get executable directory")?;

    // Check if prompt.md exists relative to exe
    // Structure: ralph_home/bin/ralph or ralph_home/target/release/ralph
    for ancestor in exe_dir.ancestors().take(4) {
        let prompt_path = ancestor.join("prompt.md");
        if prompt_path.exists() {
            return Ok(ancestor.to_path_buf());
        }
    }

    // Fall back to ~/.ralph
    if let Some(home) = dirs::home_dir() {
        let ralph_home = home.join(".ralph");
        if ralph_home.exists() {
            return Ok(ralph_home);
        }
    }

    anyhow::bail!("Could not find Ralph home directory. Set RALPH_HOME or install to ~/.ralph")
}

fn init_project(work_dir: &Path, ralph_home: &Path) -> Result<()> {
    let prd_file = work_dir.join("prd.json");
    let example_file = ralph_home.join("prd.json.example");

    if prd_file.exists() {
        println!(
            "{} prd.json already exists in {}",
            "Warning:".yellow(),
            work_dir.display()
        );
        print!("Overwrite? [y/N] ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    if example_file.exists() {
        fs::copy(&example_file, &prd_file).context("Failed to copy prd.json.example")?;
    } else {
        // Create a minimal template
        let template = Prd {
            project: "MyProject".to_string(),
            branch_name: "ralph/my-feature".to_string(),
            description: "Description of your feature".to_string(),
            user_stories: vec![UserStory {
                id: "US-001".to_string(),
                title: "First user story".to_string(),
                description: "As a user, I want X so that Y".to_string(),
                acceptance_criteria: vec![
                    "Criterion 1".to_string(),
                    "Typecheck passes".to_string(),
                ],
                priority: 1,
                passes: false,
                notes: String::new(),
            }],
        };
        let json = serde_json::to_string_pretty(&template)?;
        fs::write(&prd_file, json)?;
    }

    println!("{} {}", "Created".green(), prd_file.display());
    println!();
    println!("Next steps:");
    println!("  1. Edit prd.json with your user stories");
    println!("  2. Run 'ralph' to start the agent loop");

    Ok(())
}

fn archive_previous_run(work_dir: &Path, _current_branch: &str, last_branch: &str) -> Result<()> {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let folder_name = last_branch.strip_prefix("ralph/").unwrap_or(last_branch);
    let archive_dir = work_dir
        .join("archive")
        .join(format!("{}-{}", date, folder_name));

    println!("{} {}", "Archiving previous run:".blue(), last_branch);
    fs::create_dir_all(&archive_dir)?;

    let prd_file = work_dir.join("prd.json");
    let progress_file = work_dir.join("progress.txt");

    if prd_file.exists() {
        fs::copy(&prd_file, archive_dir.join("prd.json"))?;
    }
    if progress_file.exists() {
        fs::copy(&progress_file, archive_dir.join("progress.txt"))?;
    }

    println!("  Archived to: {}", archive_dir.display());

    // Reset progress file
    let mut f = fs::File::create(&progress_file)?;
    writeln!(f, "# Ralph Progress Log")?;
    writeln!(f, "Started: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(f, "---")?;

    Ok(())
}

fn run_iteration(prompt_file: &Path, work_dir: &Path) -> Result<(bool, String)> {
    let prompt_content = fs::read_to_string(prompt_file).context(format!(
        "Failed to read prompt file: {}",
        prompt_file.display()
    ))?;

    let mut child = Command::new("claude")
        .args(["--dangerously-skip-permissions", "--print"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .current_dir(work_dir)
        .spawn()
        .context("Failed to start claude. Is Claude Code CLI installed?")?;

    // Write prompt to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt_content.as_bytes())?;
    }

    // Stream output in real-time while capturing it
    let mut output = String::new();
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let mut buffer = [0u8; 1024];

        loop {
            match std::io::Read::read(&mut reader, &mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    print!("{}", chunk);
                    std::io::stdout().flush().ok();
                    output.push_str(&chunk);
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
    }

    child.wait()?;

    let complete = output.contains("<promise>COMPLETE</promise>");
    Ok((complete, output))
}

fn run_ralph(cli: &Cli) -> Result<()> {
    let ralph_home = get_ralph_home()?;

    let work_dir = cli
        .dir
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let work_dir = work_dir.canonicalize().unwrap_or(work_dir);

    let prompt_file = cli
        .prompt
        .clone()
        .unwrap_or_else(|| ralph_home.join("prompt.md"));

    let max_iterations = cli.max_iterations.unwrap_or(cli.iterations);

    // Validate
    if !work_dir.is_dir() {
        anyhow::bail!("Directory not found: {}", work_dir.display());
    }
    if !prompt_file.exists() {
        anyhow::bail!("Prompt file not found: {}", prompt_file.display());
    }

    let prd_file = work_dir.join("prd.json");
    let progress_file = work_dir.join("progress.txt");
    let last_branch_file = work_dir.join(".last-branch");

    // Check for prd.json
    if !prd_file.exists() {
        println!(
            "{} No prd.json found in {}",
            "Error:".red(),
            work_dir.display()
        );
        println!();
        println!("To get started:");
        println!("  ralph init    Create a prd.json template");
        println!();
        println!(
            "Or create prd.json manually. See: {}",
            ralph_home.join("prd.json.example").display()
        );
        return Ok(());
    }

    // Read PRD
    let prd_content = fs::read_to_string(&prd_file)?;
    let prd: Prd = serde_json::from_str(&prd_content).context("Failed to parse prd.json")?;
    let current_branch = &prd.branch_name;

    // Check for branch change and archive if needed
    if last_branch_file.exists() {
        let last_branch = fs::read_to_string(&last_branch_file)?.trim().to_string();
        if !last_branch.is_empty() && &last_branch != current_branch {
            archive_previous_run(&work_dir, current_branch, &last_branch)?;
        }
    }

    // Track current branch
    fs::write(&last_branch_file, current_branch)?;

    // Initialize progress file if needed
    if !progress_file.exists() {
        let mut f = fs::File::create(&progress_file)?;
        writeln!(f, "# Ralph Progress Log")?;
        writeln!(f, "Started: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(f, "---")?;
    }

    // Header
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════╗".green()
    );
    println!(
        "{}                    {}                       {}",
        "║".green(),
        format!("Ralph v{}", VERSION).blue(),
        "║".green()
    );
    println!(
        "{}          Autonomous AI Agent Loop                    {}",
        "║".green(),
        "║".green()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════╝".green()
    );
    println!();
    println!("  {}   {}", "Directory:".blue(), work_dir.display());
    println!("  {}  {} max", "Iterations:".blue(), max_iterations);
    println!("  {}         {}", "PRD:".blue(), prd_file.display());
    println!();

    // Main loop
    for i in 1..=max_iterations {
        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════════".yellow()
        );
        println!("  {} {} of {}", "Iteration".yellow(), i, max_iterations);
        println!(
            "{}",
            "═══════════════════════════════════════════════════════".yellow()
        );

        let (complete, _output) = run_iteration(&prompt_file, &work_dir)?;

        if complete {
            println!();
            println!("{} Ralph completed all tasks!", "✓".green());
            println!("  Finished at iteration {} of {}", i, max_iterations);
            return Ok(());
        }

        println!();
        println!("Iteration {} complete. Continuing...", i);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    println!();
    println!(
        "{}",
        format!(
            "Ralph reached max iterations ({}) without completing all tasks.",
            max_iterations
        )
        .yellow()
    );
    println!("Check {} for status.", progress_file.display());

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init) => {
            let ralph_home = get_ralph_home()?;
            let work_dir = cli
                .dir
                .clone()
                .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            init_project(&work_dir, &ralph_home)?;
        }
        Some(Commands::Home) => {
            let ralph_home = get_ralph_home()?;
            println!("{}", ralph_home.display());
        }
        None => {
            run_ralph(&cli)?;
        }
    }

    Ok(())
}
