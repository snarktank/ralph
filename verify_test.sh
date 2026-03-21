#!/bin/bash

# Test verification logic
SCRIPT_DIR="$(pwd)"
PRD_FILE="$SCRIPT_DIR/test_prd.json"

# Create test PRD
cat > "$PRD_FILE" << 'EOF'
{
  "project": "Test",
  "branchName": "test",
  "description": "Test verification",
  "userStories": [
    {
      "id": "TEST-001",
      "title": "Create failing script",
      "verificationCommand": "bash $SCRIPT_DIR/hello.sh | grep -qF 'Hello from Ralph TDD!'",
      "passes": false
    }
  ]
}
EOF

# Initialize test files
echo "#!/bin/bash" > "$SCRIPT_DIR/hello.sh"
echo "echo \"Wrong output\"" >> "$SCRIPT_DIR/hello.sh"
chmod +x "$SCRIPT_DIR/hello.sh"

# Initialize rejection counter
REJECTION_FILE="$SCRIPT_DIR/.rejection_count"
echo 0 > "$REJECTION_FILE"

echo "=== Test 1: Failing verification ==="
# Test the verification logic from the finish block
_sid=$(jq -r '[.userStories[] | select(.passes == false)][0].id // empty' "$PRD_FILE" 2>/dev/null)
echo "Story ID: $_sid"

if [[ -n "$_sid" && "$_sid" != "null" ]]; then
  _verification_cmd=$(jq -r --arg id "$_sid" '.userStories[] | select(.id == $id) | .verificationCommand // empty' "$PRD_FILE" 2>/dev/null)
  echo "Verification command: $_verification_cmd"
  
  if [[ -n "$_verification_cmd" && "$_verification_cmd" != "null" ]]; then
    echo "Running verification..."
    if eval "$_verification_cmd"; then
      echo "Verification PASSED"
      # Reset rejection counter on success
      echo 0 > "$REJECTION_FILE"
    else
      _exit_code=$?
      echo "Verification FAILED with exit code $_exit_code"
      _current_rejections=$(cat "$REJECTION_FILE")
      _new_rejections=$((_current_rejections + 1))
      echo $_new_rejections > "$REJECTION_FILE"
      echo "Rejection count: $_new_rejections"
      if [[ $_new_rejections -ge 5 ]]; then
        echo "CRITICAL HINT: 5 consecutive failures"
      fi
    fi
  fi
fi

echo "Rejection count: $(cat "$REJECTION_FILE")"

echo -e "\n=== Test 2: Passing verification ==="
# Now make hello.sh pass
echo "#!/bin/bash" > "$SCRIPT_DIR/hello.sh"
echo "echo \"Hello from Ralph TDD!\"" >> "$SCRIPT_DIR/hello.sh"
chmod +x "$SCRIPT_DIR/hello.sh"

# Run verification again
if eval "$_verification_cmd"; then
  echo "Verification PASSED"
  # Reset rejection counter on success
  echo 0 > "$REJECTION_FILE"
else
  _exit_code=$?
  echo "Verification FAILED with exit code $_exit_code"
  _current_rejections=$(cat "$REJECTION_FILE")
  _new_rejections=$((_current_rejections + 1))
  echo $_new_rejections > "$REJECTION_FILE"
  echo "Rejection count: $_new_rejections"
  if [[ $_new_rejections -ge 5 ]]; then
    echo "CRITICAL HINT: 5 consecutive failures"
  fi
fi

echo "Rejection count: $(cat "$REJECTION_FILE")"
# Use single quotes for jq filter to avoid Windows issues
PASSES_FIELD=$(jq -r '.userStories[] | select(.id == "TEST-001") | .passes' "$PRD_FILE")
echo "PRD passes field: $PASSES_FIELD"

# Cleanup
rm -f "$PRD_FILE" "$SCRIPT_DIR/hello.sh" "$REJECTION_FILE"