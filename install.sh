#!/bin/bash
# Ralph installer
# Builds the Rust binary and installs it to your PATH

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Find the directory where this script lives
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default install location
INSTALL_DIR="${1:-/usr/local/bin}"

echo ""
echo -e "${BLUE}Ralph Installer${NC}"
echo ""

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
  echo -e "${RED}Error:${NC} Cargo (Rust) not found"
  echo ""
  echo "Install Rust first: https://rustup.rs"
  echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo ""
  exit 1
fi

# Build the release binary
echo -e "${BLUE}Building ralph...${NC}"
cd "$SCRIPT_DIR/cli"
cargo build --release --quiet

RALPH_BIN="$SCRIPT_DIR/cli/target/release/ralph"

# Check if binary was built
if [ ! -f "$RALPH_BIN" ]; then
  echo -e "${RED}Error:${NC} Build failed - binary not found"
  exit 1
fi

echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Check if install directory exists
if [ ! -d "$INSTALL_DIR" ]; then
  echo -e "${YELLOW}Creating directory:${NC} $INSTALL_DIR"
  mkdir -p "$INSTALL_DIR" 2>/dev/null || {
    echo -e "${RED}Error:${NC} Cannot create $INSTALL_DIR"
    echo "Try: sudo ./install.sh"
    exit 1
  }
fi

# Check write permissions
if [ ! -w "$INSTALL_DIR" ]; then
  echo -e "${RED}Error:${NC} Cannot write to $INSTALL_DIR"
  echo ""
  echo "Options:"
  echo "  1. Run with sudo: sudo ./install.sh"
  echo "  2. Install to ~/bin: ./install.sh ~/bin"
  echo ""
  exit 1
fi

# Remove existing file
TARGET="$INSTALL_DIR/ralph"
if [ -L "$TARGET" ] || [ -f "$TARGET" ]; then
  echo -e "${YELLOW}Removing existing:${NC} $TARGET"
  rm "$TARGET"
fi

# Copy the binary (not symlink, since we need RALPH_HOME to work)
cp "$RALPH_BIN" "$TARGET"
chmod +x "$TARGET"

# Also copy the binary to bin/ for backward compatibility
mkdir -p "$SCRIPT_DIR/bin"
cp "$RALPH_BIN" "$SCRIPT_DIR/bin/ralph"
chmod +x "$SCRIPT_DIR/bin/ralph"

echo -e "${GREEN}✓ Installed${NC} ralph to $TARGET"
echo ""

# Set RALPH_HOME hint
echo -e "${BLUE}Note:${NC} Set RALPH_HOME for ralph to find prompt.md and skills:"
echo ""
echo "  export RALPH_HOME=\"$SCRIPT_DIR\""
echo ""
echo "Add this to your shell config (~/.bashrc, ~/.zshrc, etc.)"
echo ""

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo -e "${YELLOW}Note:${NC} $INSTALL_DIR is not in your PATH"
  echo ""
  echo "Add to your shell config:"
  echo ""
  echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
  echo ""
fi

echo "Usage:"
echo "  ralph --help      Show help"
echo "  ralph init        Initialize project with prd.json"
echo "  ralph             Run agent loop in current directory"
echo ""
