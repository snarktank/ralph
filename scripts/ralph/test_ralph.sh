#!/bin/bash
# Test suite for Ralph runner with Cursor support
# Tests run without requiring real Amp or real Cursor (use stub binaries)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$SCRIPT_DIR/test-tmp"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# These are set per variant (root canonical vs template)
CURRENT_VARIANT_NAME=""
CURRENT_LAYOUT=""
CURRENT_SOURCE_DIR=""

RALPH_SCRIPT=""
RALPH_WORK_DIR=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Setup test environment
setup_test_env() {
  rm -rf "$TEST_DIR"
  mkdir -p "$TEST_DIR/project"

  local project_dir="$TEST_DIR/project"
  local runner_dir=""

  if [[ "$CURRENT_LAYOUT" == "wrapper-root" ]]; then
    # Root wrapper entrypoints live at repo root, but canonical implementation lives in scripts/ralph/.
    runner_dir="$project_dir"

    # Copy root wrappers
    cp "$REPO_ROOT/ralph.sh" "$runner_dir/ralph.sh"
    cp "$REPO_ROOT/convert-to-prd-json.sh" "$runner_dir/convert-to-prd-json.sh"
    chmod +x "$runner_dir/ralph.sh"
    chmod +x "$runner_dir/convert-to-prd-json.sh"

    # Copy canonical scripts/ralph/ implementation into the project
    mkdir -p "$project_dir/scripts/ralph"
    cp "$CURRENT_SOURCE_DIR/ralph.sh" "$project_dir/scripts/ralph/ralph.sh"
    cp "$CURRENT_SOURCE_DIR/prompt.md" "$project_dir/scripts/ralph/prompt.md"
    cp "$CURRENT_SOURCE_DIR/prompt.cursor.md" "$project_dir/scripts/ralph/prompt.cursor.md"
    cp "$CURRENT_SOURCE_DIR/prompt.convert-to-prd-json.md" "$project_dir/scripts/ralph/prompt.convert-to-prd-json.md"
    cp "$CURRENT_SOURCE_DIR/prd.json.example" "$project_dir/scripts/ralph/prd.json.example"
    cp "$CURRENT_SOURCE_DIR/convert-to-prd-json.sh" "$project_dir/scripts/ralph/convert-to-prd-json.sh"
    chmod +x "$project_dir/scripts/ralph/ralph.sh"
    chmod +x "$project_dir/scripts/ralph/convert-to-prd-json.sh"

    # Run via root wrapper, but create PRD/progress files where canonical runner expects them.
    RALPH_SCRIPT="$runner_dir/ralph.sh"
    RALPH_WORK_DIR="$project_dir/scripts/ralph"
  elif [[ "$CURRENT_LAYOUT" == "scripts" ]]; then
    runner_dir="$project_dir/scripts/ralph"
    mkdir -p "$runner_dir"
    cp "$CURRENT_SOURCE_DIR/ralph.sh" "$runner_dir/ralph.sh"
    cp "$CURRENT_SOURCE_DIR/prompt.md" "$runner_dir/prompt.md"
    cp "$CURRENT_SOURCE_DIR/prompt.cursor.md" "$runner_dir/prompt.cursor.md"
    cp "$CURRENT_SOURCE_DIR/prompt.convert-to-prd-json.md" "$runner_dir/prompt.convert-to-prd-json.md"
    cp "$CURRENT_SOURCE_DIR/prd.json.example" "$runner_dir/prd.json.example"
    cp "$CURRENT_SOURCE_DIR/convert-to-prd-json.sh" "$runner_dir/convert-to-prd-json.sh"
    chmod +x "$runner_dir/ralph.sh"
    chmod +x "$runner_dir/convert-to-prd-json.sh"
    RALPH_SCRIPT="$runner_dir/ralph.sh"
    RALPH_WORK_DIR="$runner_dir"
  else
    echo "Invalid CURRENT_LAYOUT: $CURRENT_LAYOUT" >&2
    exit 1
  fi

  cd "$project_dir"
  
  # Create stub binaries
  mkdir -p "$project_dir/bin"
  export PATH="$project_dir/bin:$PATH"
  
  # Create stub amp binary
  cat > "$project_dir/bin/amp" << 'EOF'
#!/bin/bash
# Stub amp binary for testing
echo "Stub amp executed with args: $@"
if [ -t 0 ]; then
  echo "Stub amp: stdin is a TTY"
else
  echo "Stub amp: stdin is not a TTY"
fi
# Simulate output
echo "Some amp output"
echo "<promise>COMPLETE</promise>"
EOF
  chmod +x "$project_dir/bin/amp"
  
  # Create stub cursor binary
  cat > "$project_dir/bin/cursor" << 'EOF'
#!/bin/bash
# Stub cursor binary for testing
echo "Stub cursor executed with args: $@"
if [ -t 0 ]; then
  echo "Stub cursor: stdin is a TTY"
else
  echo "Stub cursor: stdin is not a TTY"
fi
# Check for required flags (model can vary)
if [[ "$*" == *"--model"* ]] && [[ "$*" == *"--print"* ]] && [[ "$*" == *"--force"* ]] && [[ "$*" == *"--approve-mcps"* ]]; then
  echo "Stub cursor: all required flags present"
else
  echo "Stub cursor: WARNING - missing required flags" >&2
fi
# Simulate output (no COMPLETE by default)
echo "Some cursor output"
EOF
  chmod +x "$project_dir/bin/cursor"
  
  # Create test prd.json
  cat > "$RALPH_WORK_DIR/prd.json" << 'EOF'
{
  "project": "TestProject",
  "branchName": "ralph/test",
  "description": "Test feature",
  "userStories": [
    {
      "id": "US-001",
      "title": "Test story",
      "description": "Test description",
      "acceptanceCriteria": ["Test criterion"],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}
EOF
  
  # Create test progress.txt
  echo "# Ralph Progress Log" > "$RALPH_WORK_DIR/progress.txt"
  echo "Started: $(date)" >> "$RALPH_WORK_DIR/progress.txt"
  echo "---" >> "$RALPH_WORK_DIR/progress.txt"
}

# Cleanup test environment
cleanup_test_env() {
  cd "$SCRIPT_DIR" || true
  rm -rf "$TEST_DIR"
}

# Test helper: capture command output
test_command() {
  local test_name="$1"
  local command="$2"
  local expected_pattern="$3"
  
  echo -n "Testing: $test_name... "
  
  if eval "$command" 2>&1 | grep -q "$expected_pattern"; then
    echo -e "${GREEN}PASS${NC}"
    return 0
  else
    echo -e "${RED}FAIL${NC}"
    echo "  Command: $command"
    echo "  Expected pattern: $expected_pattern"
    return 1
  fi
}

# Test 1: Default worker is Amp when no worker is specified
test_default_worker_amp() {
  setup_test_env
  
  # Run ralph with no worker specified
  OUTPUT=$(bash "$RALPH_SCRIPT" 1 2>&1 || true)
  
  if echo "$OUTPUT" | grep -q "Stub amp executed"; then
    echo -e "${GREEN}PASS${NC}: Default worker is Amp"
  else
    echo -e "${RED}FAIL${NC}: Default worker is not Amp"
    echo "Output: $OUTPUT"
    cleanup_test_env
    return 1
  fi
  
  cleanup_test_env
}

# Test 2: Cursor worker is used only when explicitly selected
test_cursor_worker_explicit() {
  setup_test_env
  
  # Test with --worker cursor
  OUTPUT=$(bash "$RALPH_SCRIPT" 1 --worker cursor 2>&1 || true)
  
  if echo "$OUTPUT" | grep -q "Stub cursor executed"; then
    echo -e "${GREEN}PASS${NC}: Cursor worker used when explicitly selected"
  else
    echo -e "${RED}FAIL${NC}: Cursor worker not used when selected"
    echo "Output: $OUTPUT"
    cleanup_test_env
    return 1
  fi
  
  # Test with RALPH_WORKER env var
  OUTPUT=$(RALPH_WORKER=cursor bash "$RALPH_SCRIPT" 1 2>&1 || true)
  
  if echo "$OUTPUT" | grep -q "Stub cursor executed"; then
    echo -e "${GREEN}PASS${NC}: Cursor worker used with RALPH_WORKER env var"
  else
    echo -e "${RED}FAIL${NC}: Cursor worker not used with RALPH_WORKER"
    echo "Output: $OUTPUT"
    cleanup_test_env
    return 1
  fi
  
  cleanup_test_env
}

# Test 3: Cursor command includes required flags
test_cursor_invocation_flags() {
  setup_test_env
  
  OUTPUT=$(bash "$RALPH_SCRIPT" 1 --worker cursor 2>&1 || true)
  
  if echo "$OUTPUT" | grep -q "all required flags present"; then
    echo -e "${GREEN}PASS${NC}: Cursor command includes all required flags"
  else
    echo -e "${RED}FAIL${NC}: Cursor command missing required flags"
    echo "Output: $OUTPUT"
    cleanup_test_env
    return 1
  fi
  
  cleanup_test_env
}

# Test 4b: PRD->prd.json conversion script can override model
test_convert_prd_json_model_override() {
  setup_test_env

  # Create a dummy PRD markdown file
  mkdir -p "$TEST_DIR/project/tasks"
  echo "# PRD: Example" > "$TEST_DIR/project/tasks/prd-example.md"

  local convert_script="$RALPH_WORK_DIR/convert-to-prd-json.sh"
  if [[ ! -f "$convert_script" ]]; then
    echo -e "${RED}FAIL${NC}: convert-to-prd-json.sh not found"
    cleanup_test_env
    return 1
  fi

  OUTPUT=$(bash "$convert_script" "$TEST_DIR/project/tasks/prd-example.md" --model "gpt-4.1" 2>&1 || true)

  if echo "$OUTPUT" | grep -qF -- "--model gpt-4.1"; then
    echo -e "${GREEN}PASS${NC}: convert-to-prd-json.sh forwards --model override"
  else
    echo -e "${RED}FAIL${NC}: convert-to-prd-json.sh did not forward --model override"
    echo "Output: $OUTPUT"
    cleanup_test_env
    return 1
  fi

  cleanup_test_env
}

# Test 4: Cursor invocation uses normal spawn (no PTY)
test_cursor_no_pty() {
  setup_test_env
  
  OUTPUT=$(bash "$RALPH_SCRIPT" 1 --worker cursor 2>&1 || true)
  
  if echo "$OUTPUT" | grep -q "stdin is not a TTY"; then
    echo -e "${GREEN}PASS${NC}: Cursor invocation uses normal spawn (no PTY)"
  else
    echo -e "${RED}FAIL${NC}: Cursor invocation may be using PTY"
    echo "Output: $OUTPUT"
    cleanup_test_env
    return 1
  fi
  
  cleanup_test_env
}

# Test 5: Stop condition - COMPLETE signal exits loop
test_stop_condition_complete() {
  setup_test_env
  
  # Modify stub amp to output COMPLETE
  cat > "$TEST_DIR/project/bin/amp" << 'EOF'
#!/bin/bash
echo "Iteration output"
echo "<promise>COMPLETE</promise>"
EOF
  chmod +x "$TEST_DIR/project/bin/amp"
  
  OUTPUT=$(bash "$RALPH_SCRIPT" 10 2>&1 || true)
  
  if echo "$OUTPUT" | grep -q "Ralph completed all tasks"; then
    echo -e "${GREEN}PASS${NC}: Loop exits on COMPLETE signal"
  else
    echo -e "${RED}FAIL${NC}: Loop does not exit on COMPLETE signal"
    echo "Output: $OUTPUT"
    cleanup_test_env
    return 1
  fi
  
  cleanup_test_env
}

# Test 6: Stop condition - no COMPLETE continues loop
test_stop_condition_no_complete() {
  setup_test_env
  
  # Modify stub amp to NOT output COMPLETE
  cat > "$TEST_DIR/project/bin/amp" << 'EOF'
#!/bin/bash
echo "Iteration output without COMPLETE"
EOF
  chmod +x "$TEST_DIR/project/bin/amp"
  
  OUTPUT=$(bash "$RALPH_SCRIPT" 2 2>&1 || true)
  
  if echo "$OUTPUT" | grep -q "Iteration 2 of 2"; then
    echo -e "${GREEN}PASS${NC}: Loop continues when no COMPLETE signal"
  else
    echo -e "${RED}FAIL${NC}: Loop does not continue without COMPLETE"
    echo "Output: $OUTPUT"
    cleanup_test_env
    return 1
  fi
  
  cleanup_test_env
}

# Test 7: progress.txt is append-only
test_progress_append_only() {
  setup_test_env
  
  ORIGINAL_CONTENT=$(cat "$RALPH_WORK_DIR/progress.txt")
  
  # Run one iteration
  bash "$RALPH_SCRIPT" 1 >/dev/null 2>&1 || true
  
  NEW_CONTENT=$(cat "$RALPH_WORK_DIR/progress.txt")
  
  if [[ "$NEW_CONTENT" == "$ORIGINAL_CONTENT"* ]]; then
    echo -e "${GREEN}PASS${NC}: progress.txt is append-only"
  else
    echo -e "${RED}FAIL${NC}: progress.txt was overwritten"
    echo "Original: $ORIGINAL_CONTENT"
    echo "New: $NEW_CONTENT"
    cleanup_test_env
    return 1
  fi
  
  cleanup_test_env
}

# Test 8: prd.json parsing failures don't crash runner
test_prd_json_parsing_failure() {
  setup_test_env
  
  # Create invalid prd.json
  echo "invalid json content" > "$RALPH_WORK_DIR/prd.json"
  
  # Runner should not crash
  if bash "$RALPH_SCRIPT" 1 >/dev/null 2>&1; then
    echo -e "${GREEN}PASS${NC}: Runner handles invalid prd.json gracefully"
  else
    echo -e "${RED}FAIL${NC}: Runner crashes on invalid prd.json"
    cleanup_test_env
    return 1
  fi
  
  # Test missing prd.json
  rm -f "$RALPH_WORK_DIR/prd.json"
  
  if bash "$RALPH_SCRIPT" 1 >/dev/null 2>&1; then
    echo -e "${GREEN}PASS${NC}: Runner handles missing prd.json gracefully"
  else
    echo -e "${RED}FAIL${NC}: Runner crashes on missing prd.json"
    cleanup_test_env
    return 1
  fi
  
  cleanup_test_env
}

run_variant() {
  local variant_name="$1"
  local layout="$2"
  local source_dir="$3"

  CURRENT_VARIANT_NAME="$variant_name"
  CURRENT_LAYOUT="$layout"
  CURRENT_SOURCE_DIR="$source_dir"

  echo "Running Ralph test suite (${CURRENT_VARIANT_NAME})..."
  echo ""

  local tests_passed=0
  local tests_failed=0

  if test_default_worker_amp; then ((tests_passed+=1)); else ((tests_failed+=1)); fi
  if test_cursor_worker_explicit; then ((tests_passed+=1)); else ((tests_failed+=1)); fi
  if test_cursor_invocation_flags; then ((tests_passed+=1)); else ((tests_failed+=1)); fi
  if test_cursor_no_pty; then ((tests_passed+=1)); else ((tests_failed+=1)); fi
  if test_convert_prd_json_model_override; then ((tests_passed+=1)); else ((tests_failed+=1)); fi
  if test_stop_condition_complete; then ((tests_passed+=1)); else ((tests_failed+=1)); fi
  if test_stop_condition_no_complete; then ((tests_passed+=1)); else ((tests_failed+=1)); fi
  if test_progress_append_only; then ((tests_passed+=1)); else ((tests_failed+=1)); fi
  if test_prd_json_parsing_failure; then ((tests_passed+=1)); else ((tests_failed+=1)); fi

  echo ""
  echo "========================================="
  echo "Variant: $CURRENT_VARIANT_NAME"
  echo "Tests passed: $tests_passed"
  echo "Tests failed: $tests_failed"
  echo "========================================="

  if [ $tests_failed -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    return 0
  else
    echo -e "${RED}Some tests failed!${NC}"
    return 1
  fi
}

# Run all tests (canonical + template)
main() {
  local overall_failed=0

  # Root wrappers (ralph/ralph.sh delegates into scripts/ralph/)
  if ! run_variant "wrapper-root" "wrapper-root" "$REPO_ROOT/scripts/ralph"; then
    overall_failed=1
  fi

  echo ""

  # Canonical runner (scripts/ralph/ralph.sh)
  if ! run_variant "template-scripts" "scripts" "$REPO_ROOT/scripts/ralph"; then
    overall_failed=1
  fi

  exit $overall_failed
}

# Run tests
main
