# Ralph MCP Server Examples

This directory contains example configurations for running Ralph as an MCP server.

## Docker MCP Toolkit Configuration

The `mcp-toolkit-config.json` file provides a ready-to-use configuration for the Docker MCP toolkit.

### Usage with Claude Desktop

1. Copy the configuration to your Claude Desktop settings:

   **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
   **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
   **Linux**: `~/.config/claude/claude_desktop_config.json`

2. Merge the `mcpServers` section into your existing config, or use it as a starting point:

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

3. Restart Claude Desktop to load the new configuration.

### Configuration Options

- **RUST_LOG**: Set logging level (`error`, `warn`, `info`, `debug`, `trace`)
- **Volume mounts**: Add `-v /path/to/prd.json:/app/prd.json:ro` to args to preload a PRD file

### Example with PRD File

To run Ralph with a preloaded PRD file:

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

### Manual Docker Run

You can also run Ralph directly with Docker:

```bash
# Basic run
docker run -i --rm ghcr.io/kcirtapfromspace/ralph:latest

# With PRD file
docker run -i --rm \
  -v /path/to/prd.json:/app/prd.json:ro \
  ghcr.io/kcirtapfromspace/ralph:latest \
  --prd /app/prd.json

# With debug logging
docker run -i --rm \
  -e RUST_LOG=debug \
  ghcr.io/kcirtapfromspace/ralph:latest
```

## Related Documentation

- [Docker MCP Setup Guide](../docs/guides/docker-mcp-setup.md) - Comprehensive guide for Docker deployment
- [Ralph Main Repository](https://github.com/kcirtapfromspace/ralph) - Main project documentation
