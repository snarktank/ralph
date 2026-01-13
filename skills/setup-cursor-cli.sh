#!/bin/bash

# Setup script for Ralph Skills in Cursor CLI
# This script copies rules and commands to ~/.cursor for global installation

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Setting up Ralph Skills for Cursor CLI...${NC}"

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Check if cursor-cli directory exists
if [ ! -d "$SCRIPT_DIR/cursor-cli" ]; then
    echo -e "${YELLOW}Error: cursor-cli directory not found.${NC}"
    echo "Make sure you're running this script from the skills repository root."
    exit 1
fi

# Create ~/.cursor directory structure
CURSOR_DIR="$HOME/.cursor"
echo "Creating ~/.cursor directory structure..."
mkdir -p "$CURSOR_DIR/rules"
mkdir -p "$CURSOR_DIR/commands"

# Copy rules
echo "Copying rules..."
if [ -d "$SCRIPT_DIR/cursor-cli/rules" ]; then
    cp -r "$SCRIPT_DIR/cursor-cli/rules"/* "$CURSOR_DIR/rules/"
    echo -e "${GREEN}✓ Rules copied to ~/.cursor/rules/${NC}"
else
    echo -e "${YELLOW}Warning: cursor-cli/rules directory not found${NC}"
fi

# Copy commands
echo "Copying commands..."
if [ -d "$SCRIPT_DIR/cursor-cli/commands" ]; then
    cp -r "$SCRIPT_DIR/cursor-cli/commands"/* "$CURSOR_DIR/commands/"
    echo -e "${GREEN}✓ Commands copied to ~/.cursor/commands/${NC}"
else
    echo -e "${YELLOW}Warning: cursor-cli/commands directory not found${NC}"
fi

echo ""
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo "Ralph Skills are now installed globally for Cursor CLI."
echo ""
echo "Next steps:"
echo "1. Use '/commands' in Cursor CLI to set up the /prd and /ralph commands"
echo "2. Or use natural language triggers like 'create a PRD for...' or 'run ralph'"
echo ""
echo "Rules are automatically loaded from ~/.cursor/rules/"
echo "Browser verification uses cursor-ide-browser MCP (automatically available)"
