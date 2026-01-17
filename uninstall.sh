#!/bin/bash
# Ralph uninstaller

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

INSTALL_DIR="${1:-/usr/local/bin}"
TARGET="$INSTALL_DIR/ralph"

echo ""
echo "Ralph Uninstaller"
echo ""

if [ -L "$TARGET" ] || [ -f "$TARGET" ]; then
  rm "$TARGET"
  echo -e "${GREEN}✓ Removed${NC} $TARGET"
else
  echo -e "${RED}Not found:${NC} $TARGET"
fi

echo ""
