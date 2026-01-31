# Ralph MCP Integration 🚀

Ralph now supports **FastMCP 3.0** with both server and client capabilities!

## Features

### MCP Server Mode
Ralph exposes its autonomous agent capabilities as an MCP server with:

**Tools:**
- `run_ralph_iteration(agent, max_iterations, prd_path)` - Run Ralph for N iterations
- `get_ralph_status()` - Get current execution status from progress.txt
- `get_prd_status(prd_path)` - Get PRD completion metrics

**Resources:**
- `ralph://prd` - Current PRD JSON content
- `ralph://progress` - Current progress.txt log

### Usage

**As CLI (default):**
```bash
./ralph.sh --agent codex 3
# or
uv run --script ralphython.py --agent codex 3
```

**As MCP Server (HTTP):**
```bash
uv run --script ralphython.py --mcp --transport http --port 8000
```

**As MCP Server (stdio):**
```bash
uv run --script ralphython.py --mcp --transport stdio
```

### Client Example
```python
import asyncio
from fastmcp import Client

async def check_ralph():
    async with Client("http://localhost:8000/mcp") as client:
        # Get PRD status
        status = await client.call_tool("get_prd_status", {})
        print(f"Completed: {status['completed_stories']}/{status['total_stories']}")
        
        # Run Ralph for 1 iteration
        result = await client.call_tool("run_ralph_iteration", {
            "agent": "codex",
            "max_iterations": 1
        })
        print(f"Exit code: {result['exit_code']}")

asyncio.run(check_ralph())
```

### MCP Client Config
Add to your MCP client settings (e.g., Claude Desktop):

```json
{
  "mcpServers": {
    "ralph": {
      "command": "uv",
      "args": ["run", "--script", "/path/to/ralphython.py", "--mcp"]
    }
  }
}
```

## Implementation Details

- FastMCP 3.0.0b1 (beta)
- PEP 723 inline script dependencies
- Supports both CLI and MCP modes via `--mcp` flag
- Model: gpt-5.2-codex (configurable via `CODEX_MODEL` env var)

