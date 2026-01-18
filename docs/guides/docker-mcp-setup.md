# Docker MCP Setup Guide

This guide walks you through setting up Ralph as an MCP server using Docker and the Docker MCP toolkit, enabling seamless integration with Claude Desktop and other MCP clients via containerized deployment.

## Overview

Running Ralph via Docker provides several advantages:
- **No local Rust installation required** - Just Docker
- **Consistent environment** - Same container image across all machines
- **Easy updates** - Pull the latest image to update
- **Multi-platform support** - Works on amd64 and arm64 (Apple Silicon)

## Prerequisites

Before setting up Ralph with Docker MCP toolkit, ensure you have:

### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| [Docker](https://docs.docker.com/get-docker/) | 20.10+ | Container runtime |
| [Claude Desktop](https://claude.ai/download) | Latest | MCP client application |

### Optional Software

| Software | Purpose |
|----------|---------|
| [Docker Compose](https://docs.docker.com/compose/install/) | Local development and testing |

### Verify Docker Installation

```bash
# Check Docker is installed and running
docker --version
docker info

# Test Docker can pull images
docker pull hello-world
docker run --rm hello-world
```

## Quick Start

The fastest way to get Ralph running with Claude Desktop:

1. **Pull the Ralph Docker image:**
   ```bash
   docker pull ghcr.io/kcirtapfromspace/ralph:latest
   ```

2. **Add Ralph to Claude Desktop config:**

   Open your Claude Desktop configuration file:
   - **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
   - **Linux**: `~/.config/claude/claude_desktop_config.json`

3. **Add the Ralph MCP server configuration:**
   ```json
   {
     "mcpServers": {
       "ralph": {
         "command": "docker",
         "args": [
           "run",
           "-i",
           "--rm",
           "ghcr.io/kcirtapfromspace/ralph:latest"
         ],
         "env": {
           "RUST_LOG": "info"
         },
         "transport": "stdio"
       }
     }
   }
   ```

4. **Restart Claude Desktop** to load the new configuration.

## Step-by-Step Configuration

### Step 1: Pull the Docker Image

```bash
# Pull the latest stable image
docker pull ghcr.io/kcirtapfromspace/ralph:latest

# Or pull a specific version
docker pull ghcr.io/kcirtapfromspace/ralph:v1.0.0

# Verify the image
docker images ghcr.io/kcirtapfromspace/ralph
```

### Step 2: Test Ralph Locally

Before configuring Claude Desktop, verify Ralph works:

```bash
# Run Ralph and check it starts correctly
docker run --rm ghcr.io/kcirtapfromspace/ralph:latest --help

# Test the MCP server mode (will wait for input)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  docker run -i --rm ghcr.io/kcirtapfromspace/ralph:latest
```

### Step 3: Configure Claude Desktop

Create or edit the Claude Desktop configuration file.

**macOS:**
```bash
mkdir -p ~/Library/Application\ Support/Claude
nano ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

**Windows (PowerShell):**
```powershell
New-Item -ItemType Directory -Force -Path $env:APPDATA\Claude
notepad $env:APPDATA\Claude\claude_desktop_config.json
```

**Linux:**
```bash
mkdir -p ~/.config/claude
nano ~/.config/claude/claude_desktop_config.json
```

### Step 4: Add MCP Server Configuration

Add Ralph to your configuration. If you have an existing file, merge the `mcpServers` section:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "docker",
      "args": [
        "run",
        "-i",
        "--rm",
        "ghcr.io/kcirtapfromspace/ralph:latest"
      ],
      "env": {
        "RUST_LOG": "info"
      },
      "transport": "stdio"
    }
  }
}
```

### Step 5: Restart Claude Desktop

Completely quit and restart Claude Desktop. Ralph should now appear as an available MCP server.

## Configuration Options

### Basic Configuration

The minimal configuration to run Ralph:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "ghcr.io/kcirtapfromspace/ralph:latest"],
      "transport": "stdio"
    }
  }
}
```

### With Pre-loaded PRD File

Mount a PRD file from your host machine:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "docker",
      "args": [
        "run",
        "-i",
        "--rm",
        "-v", "/path/to/your/prd.json:/app/prd.json:ro",
        "ghcr.io/kcirtapfromspace/ralph:latest",
        "--prd", "/app/prd.json"
      ],
      "env": {
        "RUST_LOG": "info"
      },
      "transport": "stdio"
    }
  }
}
```

### With Debug Logging

Enable verbose logging for debugging:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "docker",
      "args": [
        "run",
        "-i",
        "--rm",
        "-e", "RUST_LOG=debug",
        "ghcr.io/kcirtapfromspace/ralph:latest"
      ],
      "transport": "stdio"
    }
  }
}
```

### With Custom Quality Profile

Mount a custom quality configuration:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "docker",
      "args": [
        "run",
        "-i",
        "--rm",
        "-v", "/path/to/quality:/app/quality:ro",
        "ghcr.io/kcirtapfromspace/ralph:latest"
      ],
      "transport": "stdio"
    }
  }
}
```

## Docker Run Examples

### Basic Usage

```bash
# Run Ralph MCP server (interactive mode for MCP communication)
docker run -i --rm ghcr.io/kcirtapfromspace/ralph:latest

# Show Ralph help
docker run --rm ghcr.io/kcirtapfromspace/ralph:latest --help
```

### With PRD File

```bash
# Mount and load a PRD file
docker run -i --rm \
  -v /path/to/prd.json:/app/prd.json:ro \
  ghcr.io/kcirtapfromspace/ralph:latest \
  --prd /app/prd.json
```

### With Debug Logging

```bash
# Enable debug logging
docker run -i --rm \
  -e RUST_LOG=debug \
  ghcr.io/kcirtapfromspace/ralph:latest

# Enable trace logging for MCP communication
docker run -i --rm \
  -e RUST_LOG=ralph=debug,rmcp=trace \
  ghcr.io/kcirtapfromspace/ralph:latest
```

### With Volume Mounts

```bash
# Mount PRD and quality configuration
docker run -i --rm \
  -v /path/to/prd.json:/app/prd.json:ro \
  -v /path/to/quality:/app/quality:ro \
  ghcr.io/kcirtapfromspace/ralph:latest \
  --prd /app/prd.json
```

### Specific Image Version

```bash
# Use a specific version
docker run -i --rm ghcr.io/kcirtapfromspace/ralph:v1.0.0

# Use image by git SHA
docker run -i --rm ghcr.io/kcirtapfromspace/ralph:abc123
```

## Local Development with Docker Compose

For local testing and development, use Docker Compose:

```bash
# Build and run locally
docker compose up --build

# Run with a preloaded PRD
docker compose --profile with-prd up --build

# View logs
docker compose logs -f

# Stop and clean up
docker compose down
```

## Available MCP Tools

Once Ralph is configured, Claude Desktop can use these tools:

| Tool | Description |
|------|-------------|
| `list_stories` | List user stories from the loaded PRD (optional: filter by status) |
| `get_status` | Get current Ralph execution status |
| `load_prd` | Load a PRD file from the specified path |
| `run_story` | Execute a specific user story by ID |
| `stop_execution` | Cancel the currently running execution |

## Available MCP Resources

Ralph exposes these resources:

| Resource URI | Description | MIME Type |
|--------------|-------------|-----------|
| `ralph://prd/current` | Contents of the currently loaded PRD | `application/json` |
| `ralph://status` | Current execution status | `application/json` |

## Troubleshooting

### Docker Not Found

**Symptom:** "docker: command not found" or similar errors.

**Solutions:**
1. Verify Docker is installed: `docker --version`
2. Ensure Docker Desktop is running (macOS/Windows)
3. Check Docker is in your PATH
4. On Linux, ensure your user is in the docker group: `sudo usermod -aG docker $USER`

### Image Not Found

**Symptom:** "Unable to find image" or "manifest not found" errors.

**Solutions:**
1. Check image name is correct: `ghcr.io/kcirtapfromspace/ralph`
2. Pull the image explicitly: `docker pull ghcr.io/kcirtapfromspace/ralph:latest`
3. Verify internet connectivity
4. Check GitHub Container Registry status

### Permission Denied

**Symptom:** "Permission denied" when accessing mounted files.

**Solutions:**
1. Ensure mounted files are readable: `ls -la /path/to/prd.json`
2. On macOS, grant Docker file access in System Preferences > Privacy & Security > Files and Folders
3. On Linux, check file permissions and SELinux/AppArmor policies

### Container Exits Immediately

**Symptom:** Container starts and immediately exits.

**Solutions:**
1. Check container logs: `docker logs <container_id>`
2. Ensure you're running with `-i` flag (interactive mode required for stdio)
3. Test the image manually: `docker run --rm ghcr.io/kcirtapfromspace/ralph:latest --help`

### Claude Desktop Not Connecting

**Symptom:** Ralph doesn't appear or shows as disconnected in Claude Desktop.

**Solutions:**
1. Verify configuration JSON syntax: `cat config.json | python3 -m json.tool`
2. Check the full path to docker command: `which docker`
3. Restart Claude Desktop completely (quit and reopen)
4. Check Claude Desktop logs for errors
5. Enable debug logging in Ralph to see connection attempts

### Volume Mount Issues

**Symptom:** "File not found" errors for mounted PRD files.

**Solutions:**
1. Use absolute paths, not relative paths
2. Verify the host file exists: `ls -la /path/to/prd.json`
3. Check mount syntax: `-v /host/path:/container/path:ro`
4. On Windows, ensure the drive is shared with Docker Desktop

### Slow Startup

**Symptom:** Ralph takes a long time to start or times out.

**Solutions:**
1. Pre-pull the image: `docker pull ghcr.io/kcirtapfromspace/ralph:latest`
2. Check available disk space: `docker system df`
3. Prune unused images: `docker image prune`

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "No PRD loaded" | PRD file not loaded | Use `load_prd` tool or add `--prd` to args |
| "Story not found" | Invalid story ID | Check available stories with `list_stories` |
| "Already running" | Execution in progress | Wait for completion or use `stop_execution` |
| "connect ENOENT" | Docker daemon not running | Start Docker Desktop or docker service |

### Getting Debug Logs

To collect logs for debugging:

```bash
# Run with debug logging and capture output
docker run -i --rm \
  -e RUST_LOG=debug \
  ghcr.io/kcirtapfromspace/ralph:latest 2>&1 | tee ralph-debug.log
```

### Getting Help

If you continue to experience issues:

1. Check the [Ralph GitHub Issues](https://github.com/kcirtapfromspace/ralph/issues)
2. Enable debug logging and capture output
3. Create an issue with:
   - Docker version (`docker --version`)
   - Operating system and version
   - Full configuration (redact sensitive paths)
   - Error messages and logs
   - Steps to reproduce

## Related Documentation

- [Claude Desktop Setup Guide](https://github.com/kcirtapfromspace/ralph/blob/main/docs/guides/claude-desktop-setup.md) - Native binary setup (alternative to Docker)
- [Examples README](../../examples/README.md) - Additional configuration examples
- [Ralph README](../../README.md) - Main project documentation
