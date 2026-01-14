#!/bin/bash
# Ralph Installer - Sets up Ralph in your project
# Usage: /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/snarktank/ralph/HEAD/install.sh)"

set -e

RALPH_GIT_REPO="https://github.com/snarktank/ralph.git"
RALPH_DIR="scripts/ralph"
AMP_CONFIG_DIR="$HOME/.config/amp"
AMP_SKILLS_DIR="$AMP_CONFIG_DIR/skills"
AMP_SETTINGS="$AMP_CONFIG_DIR/settings.json"
TMP_DIR=$(mktemp -d)

echo "🐕 Installing Ralph..."
echo ""

# Clone the repo to temp directory
echo "📥 Cloning Ralph repository..."
git clone --depth 1 --quiet "$RALPH_GIT_REPO" "$TMP_DIR"
echo "   ✓ Repository cloned"

# Create ralph directory in project
mkdir -p "$RALPH_DIR"

# Copy ralph files
echo ""
echo "� Installing Ralph files..."
cp "$TMP_DIR/ralph.sh" "$RALPH_DIR/"
cp "$TMP_DIR/prompt.md" "$RALPH_DIR/"
cp "$TMP_DIR/prd.json.example" "$RALPH_DIR/"
chmod +x "$RALPH_DIR/ralph.sh"

echo "   ✓ ralph.sh"
echo "   ✓ prompt.md"
echo "   ✓ prd.json.example"

# Create Amp config directory if needed
mkdir -p "$AMP_SKILLS_DIR"

# Copy skills
echo ""
echo "� Installing Amp skills..."
cp -r "$TMP_DIR/skills/prd" "$AMP_SKILLS_DIR/"
cp -r "$TMP_DIR/skills/ralph" "$AMP_SKILLS_DIR/"
echo "   ✓ prd skill"
echo "   ✓ ralph skill"

# Cleanup temp directory
rm -rf "$TMP_DIR"

# Configure Amp auto-handoff
echo ""
echo "⚙️  Configuring Amp settings..."

if [ -f "$AMP_SETTINGS" ]; then
  # Check if autoHandoff already configured
  if grep -q "autoHandoff" "$AMP_SETTINGS" 2>/dev/null; then
    echo "   ✓ autoHandoff already configured"
  else
    # Add autoHandoff to existing settings using jq if available
    if command -v jq &> /dev/null; then
      TMP_FILE=$(mktemp)
      jq '. + {"amp.experimental.autoHandoff": {"context": 90}}' "$AMP_SETTINGS" > "$TMP_FILE"
      mv "$TMP_FILE" "$AMP_SETTINGS"
      echo "   ✓ Added autoHandoff to settings"
    else
      echo "   ⚠ jq not installed - please manually add to $AMP_SETTINGS:"
      echo '     "amp.experimental.autoHandoff": { "context": 90 }'
    fi
  fi
else
  # Create new settings file
  cat > "$AMP_SETTINGS" << 'EOF'
{
  "amp.experimental.autoHandoff": { "context": 90 }
}
EOF
  echo "   ✓ Created settings with autoHandoff"
fi

echo ""
echo "✅ Ralph installed successfully!"
echo ""
echo "Next steps:"
echo "  1. Create a PRD:  Load the prd skill and create a PRD for [your feature]"
echo "  2. Convert to JSON:  Load the ralph skill and convert tasks/prd-*.md to prd.json"
echo "  3. Run Ralph:  $RALPH_DIR/ralph.sh"
echo ""
echo "📚 Docs: https://github.com/snarktank/ralph"
