# Claude Desktop Setup Guide

This guide walks you through setting up Ralph as an MCP (Model Context Protocol) server for use with Claude Desktop.

## Overview

Ralph can run as an MCP server, allowing Claude Desktop to interact with your PRD (Product Requirements Document) and manage autonomous code generation tasks. Through MCP, Claude can:

- List and filter user stories from your PRD
- Check Ralph's execution status
- Load PRD files
- Execute individual user stories
- Stop running executions
- Read current PRD and status via resources

## Prerequisites

Before setting up Ralph with Claude Desktop, ensure you have:

1. **Ralph installed** - Built from source or installed via release binary
2. **Claude Desktop** - Installed and configured on your system
3. **A valid PRD file** - JSON file following Ralph's PRD schema

## Installation

### Building Ralph

```bash
# Clone the repository
git clone https://github.com/kcirtapfromspace/ralph.git
cd ralph

# Build in release mode
cargo build --release

# The binary will be at ./target/release/ralph
```

### Locating the Binary

After building, note the full path to the Ralph binary. You'll need this for the configuration:

```bash
# On macOS/Linux
/path/to/ralph/target/release/ralph

# On Windows
C:\path\to\ralph\target\release\ralph.exe
```

## Configuration

### Claude Desktop Configuration

Claude Desktop uses a JSON configuration file to define MCP servers. The location varies by operating system:

| OS | Configuration Path |
|----|--------------------|
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |
| Linux | `~/.config/Claude/claude_desktop_config.json` |

### Example Configuration

Add Ralph to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "/path/to/ralph/target/release/ralph",
      "args": ["mcp-server"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Configuration with Pre-loaded PRD

To automatically load a PRD file when Ralph starts:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "/path/to/ralph/target/release/ralph",
      "args": ["mcp-server", "--prd", "/path/to/your/prd.json"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Full Configuration Example

Here's a complete example with multiple settings:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "/Users/developer/projects/ralph/target/release/ralph",
      "args": ["mcp-server", "--prd", "/Users/developer/projects/myapp/prd.json"],
      "env": {
        "RUST_LOG": "ralph=debug,rmcp=info",
        "RALPH_PROFILES__STANDARD__TESTING__COVERAGE_THRESHOLD": "80"
      }
    }
  }
}
```

## Environment Variables

Ralph supports the following environment variables for configuration:

### Logging

| Variable | Description | Example Values |
|----------|-------------|----------------|
| `RUST_LOG` | Controls log verbosity | `error`, `warn`, `info`, `debug`, `trace` |

You can set granular logging levels:

```bash
# General info, debug for Ralph
RUST_LOG=info,ralph=debug

# Trace MCP communication
RUST_LOG=ralph=info,rmcp=trace
```

### Quality Profile Overrides

Ralph's quality profiles can be overridden via environment variables using the `RALPH_` prefix and `__` as a separator for nested keys:

| Variable Pattern | Description |
|-----------------|-------------|
| `RALPH_PROFILES__<PROFILE>__<SECTION>__<KEY>` | Override specific profile settings |

**Examples:**

```bash
# Set coverage threshold for standard profile to 80%
RALPH_PROFILES__STANDARD__TESTING__COVERAGE_THRESHOLD=80

# Disable lint check for minimal profile
RALPH_PROFILES__MINIMAL__CI__LINT_CHECK=false

# Enable cargo-audit for minimal profile
RALPH_PROFILES__MINIMAL__SECURITY__CARGO_AUDIT=true
```

### Integration Settings

When using project management integrations:

| Variable | Description |
|----------|-------------|
| `GITHUB_TOKEN` | GitHub personal access token for GitHub Projects integration |
| `LINEAR_API_KEY` | Linear API key for Linear integration |

## Available MCP Tools

Once configured, Claude Desktop can use the following Ralph tools:

| Tool | Description |
|------|-------------|
| `list_stories` | List user stories from the loaded PRD (optional: filter by status) |
| `get_status` | Get current Ralph execution status |
| `load_prd` | Load a PRD file from the specified path |
| `run_story` | Execute a specific user story by ID |
| `stop_execution` | Cancel the currently running execution |

## Available MCP Resources

Ralph exposes the following resources:

| Resource URI | Description | MIME Type |
|--------------|-------------|-----------|
| `ralph://prd/current` | Contents of the currently loaded PRD | `application/json` |
| `ralph://status` | Current execution status | `application/json` |

## Usage Examples

### Basic Workflow

1. **Start Claude Desktop** - Ralph MCP server starts automatically

2. **Load a PRD** (if not pre-loaded):
   > "Load the PRD file at /path/to/project/prd.json"

3. **List stories**:
   > "Show me all the user stories"
   > "List only the failing stories"

4. **Execute a story**:
   > "Run story US-001"
   > "Execute US-003 with a maximum of 5 iterations"

5. **Check status**:
   > "What's the current execution status?"

6. **Stop if needed**:
   > "Stop the current execution"

## Troubleshooting

### Server Not Starting

**Symptom:** Claude Desktop shows Ralph as disconnected or unavailable.

**Solutions:**
1. Verify the path to the Ralph binary is correct and absolute
2. Ensure the binary has execute permissions: `chmod +x /path/to/ralph`
3. Test Ralph runs manually: `/path/to/ralph mcp-server`
4. Check Claude Desktop logs for error messages

### PRD Not Loading

**Symptom:** "No PRD loaded" error when trying to list stories or run.

**Solutions:**
1. Verify the PRD path exists and is readable
2. Ensure the PRD is valid JSON
3. Check the PRD follows Ralph's schema (has `project`, `branchName`, `userStories`)
4. Try loading with absolute path instead of relative

### Permission Denied Errors

**Symptom:** "Permission denied" or file access errors.

**Solutions:**
1. Ensure Ralph has read access to PRD files
2. Check file permissions: `ls -la /path/to/prd.json`
3. On macOS, you may need to grant Claude Desktop file access in System Preferences

### Connection Timeout

**Symptom:** Claude Desktop times out connecting to Ralph.

**Solutions:**
1. Increase Claude Desktop's MCP timeout if configurable
2. Check for slow startup in Ralph (e.g., loading large configurations)
3. Verify no antivirus is blocking the connection

### Logging for Debugging

Enable verbose logging to diagnose issues:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "/path/to/ralph",
      "args": ["mcp-server"],
      "env": {
        "RUST_LOG": "debug"
      }
    }
  }
}
```

View logs in Claude Desktop's developer tools or check stderr output.

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "No PRD loaded" | PRD file not loaded | Use `load_prd` tool or add `--prd` flag |
| "Story not found" | Invalid story ID | Check available stories with `list_stories` |
| "Already running" | Execution in progress | Wait for completion or use `stop_execution` |
| "File not found" | Invalid PRD path | Verify path exists and is accessible |

### Getting Help

If you continue to experience issues:

1. Check the [Ralph GitHub Issues](https://github.com/kcirtapfromspace/ralph/issues)
2. Enable debug logging and capture output
3. Create an issue with reproduction steps and logs

## Next Steps

- Read the [Quality Profiles Guide](./quality-profiles.md) to configure quality gates
- Set up [Project Management Integration](./integrations.md) with GitHub Projects or Linear
- Review [Architecture Documentation](../architecture/diagrams/system-overview.mmd)
