#!/bin/bash

# Source the ralph.sh functions
source ./ralph.sh

# Set up test environment
SCRIPT_DIR="$(pwd)"
PRD_FILE="$SCRIPT_DIR/test_prd.json"

# Create a test PRD with a story that should pass verification
cat > "$PRD_FILE" << 'EOF'
{
  "project": "Test",
  "branchName": "test",
  "description": "Test verification",
  "userStories": [
    {
      "id": "TEST-001",
      "title": "Create passing script",
      "verificationCommand": "bash $SCRIPT_DIR/hello.sh | grep -qF 'Hello from Ralph TDD!'",
      "passes": false
    }
  ]
}
EOF

# Create the hello.sh that should pass
echo "#!/bin/bash" > "$SCRIPT_DIR/hello.sh"
echo "echo \"Hello from Ralph TDD!\"" >> "$SCRIPT_DIR/hello.sh"
chmod +x "$SCRIPT_DIR/hello.sh"

# Initialize rejection counter
REJECTION_FILE="$SCRIPT_DIR/.rejection_count"
echo 0 > "$REJECTION_FILE"

echo "=== Testing finish block logic ==="
echo "PRD content:"
cat "$PRD_FILE"
echo

# Simulate what happens in the finish block
echo "Simulating finish block execution..."

# Get current failing story (this should work)
_sid=$(jq -r '[.userStories[] | select(.passes == false)][0].id // empty' "$PRD_FILE" 2>/dev/null)
echo "Story ID: '$_sid'"

if [[ -n "$_sid" && "$_sid" != "null" ]]; then
  echo "Found failing story: $_sid"
  
  # Check if story has verificationCommand
  _verification_cmd=$(jq -r --arg id "$_sid" '.userStories[] | select(.id == $id) | .verificationCommand // empty' "$PRD_FILE" 2>/dev/null)
  echo "Verification command: '$_verification_cmd'"
  
  if [[ -n "$_verification_cmd" && "$_verification_cmd" != "null" ]]; then
    echo "Running verification command..."
    if eval "$_verification_cmd"; then
      echo "Verification PASSED - would mark story as complete"
      # In real code, this would mark the story as passes: true
    else
      echo "Verification FAILED"
    fi
  else
    echo "No verification command found"
  fi
else
  echo "No failing stories found"
fi

# Cleanup
rm -f "$PRD_FILE" "$SCRIPT_DIR/hello.sh" "$REJECTION_FILE"
echo "Test completed."