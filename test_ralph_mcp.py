#!/usr/bin/env python3
"""Quick test of Ralph MCP server capabilities."""

import asyncio
import subprocess
import time
from fastmcp import Client


async def test_ralph_mcp():
    """Test Ralph MCP tools and resources."""
    # Start Ralph MCP server in background
    proc = subprocess.Popen(
        ["uv", "run", "--script", "ralphython.py", "--mcp", "--transport", "http", "--port", "8766"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    
    # Wait for server to start
    print("⏳ Waiting for Ralph MCP server to start...")
    time.sleep(3)
    
    try:
        async with Client("http://localhost:8766/mcp") as client:
            print("✅ Connected to Ralph MCP server\n")
            
            # Test get_prd_status tool
            print("📋 Testing get_prd_status tool...")
            result = await client.call_tool("get_prd_status", {})
            print(f"   PRD Status: {result['project']}")
            print(f"   Completed: {result['completed_stories']}/{result['total_stories']} ({result['completion_percentage']}%)\n")
            
            # Test get_ralph_status tool
            print("📊 Testing get_ralph_status tool...")
            result = await client.call_tool("get_ralph_status", {})
            print(f"   Status: {result['status']}")
            print(f"   Total lines: {result.get('total_lines', 0)}\n")
            
            # Test resources
            print("📂 Testing ralph://prd resource...")
            resources = await client.list_resources()
            prd_resources = [r for r in resources if "prd" in r.uri]
            if prd_resources:
                print(f"   Found resource: {prd_resources[0].uri}\n")
            
            print("✅ All tests passed!")
            
    finally:
        proc.terminate()
        proc.wait(timeout=2)
        print("\n🛑 Server stopped")


if __name__ == "__main__":
    asyncio.run(test_ralph_mcp())
