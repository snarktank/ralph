#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMP_DIR"
}

trap cleanup EXIT

pass() {
  echo "PASS: $1"
}

fail() {
  echo "FAIL: $1" >&2
  exit 1
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local message="$3"

  if [[ "$haystack" != *"$needle"* ]]; then
    fail "$message"
  fi
}

assert_file_contains() {
  local file="$1"
  local needle="$2"
  local message="$3"

  if ! grep -Fq "$needle" "$file"; then
    fail "$message"
  fi
}

setup_fixture() {
  local fixture_dir="$1"

  mkdir -p "$fixture_dir/lib" "$fixture_dir/bin"
  cp "$SCRIPT_DIR/ralph.sh" "$fixture_dir/"
  cp "$SCRIPT_DIR/prompt.md" "$fixture_dir/"
  cp "$SCRIPT_DIR/CLAUDE.md" "$fixture_dir/"
  cp "$SCRIPT_DIR/lib/rate-limit.sh" "$fixture_dir/lib/"

  cat > "$fixture_dir/progress.txt" <<'EOF'
# Ralph Progress Log
Started: smoke test
---
EOF

  cat > "$fixture_dir/prd.json" <<'EOF'
{"branchName":"ralph/test-rate-limit","userStories":[{"id":"US-001","title":"Smoke test story","passes":false}]}
EOF
}

run_claude_parseable_test() {
  local fixture_dir="$TMP_DIR/claude-parseable"
  local output=""

  setup_fixture "$fixture_dir"

  cat > "$fixture_dir/bin/claude" <<EOF
#!/bin/bash
state_file="$fixture_dir/claude-state"
count=0
if [ -f "\$state_file" ]; then
  count=\$(cat "\$state_file")
fi
count=\$((count + 1))
printf '%s' "\$count" > "\$state_file"
if [ "\$count" -eq 1 ]; then
  echo "Claude usage limit reached. Your limit will reset at 7pm (Asia/Tokyo)."
else
  echo "<promise>COMPLETE</promise>"
fi
EOF

  cat > "$fixture_dir/bin/sleep" <<EOF
#!/bin/bash
echo "sleep:\$1" >> "$fixture_dir/sleep.log"
EOF

  chmod +x "$fixture_dir/bin/claude" "$fixture_dir/bin/sleep"

  output=$(PATH="$fixture_dir/bin:$PATH" bash "$fixture_dir/ralph.sh" --tool claude 1 2>&1)

  assert_contains "$output" "Claude hit a rate limit." "Expected Claude rate-limit detection message"
  assert_contains "$output" "Detected reset time: 7pm (Asia/Tokyo)." "Expected reset-time parsing message"
  assert_contains "$output" "Ralph completed all tasks!" "Expected Ralph to complete after retry"
  assert_file_contains "$fixture_dir/claude-state" "2" "Expected Claude to be invoked twice for the same iteration"

  local first_sleep
  first_sleep=$(head -n 1 "$fixture_dir/sleep.log")
  if [[ "$first_sleep" == "sleep:2" ]]; then
    fail "Expected the first sleep to be the computed quota wait, not the normal 2-second pause"
  fi

  pass "Claude parseable reset message retries the same iteration"
}

run_claude_fallback_test() {
  local fixture_dir="$TMP_DIR/claude-fallback"
  local output=""

  setup_fixture "$fixture_dir"

  cat > "$fixture_dir/bin/claude" <<EOF
#!/bin/bash
state_file="$fixture_dir/claude-state"
count=0
if [ -f "\$state_file" ]; then
  count=\$(cat "\$state_file")
fi
count=\$((count + 1))
printf '%s' "\$count" > "\$state_file"
if [ "\$count" -eq 1 ]; then
  echo "Rate limit encountered. Please try again later."
else
  echo "<promise>COMPLETE</promise>"
fi
EOF

  cat > "$fixture_dir/bin/sleep" <<EOF
#!/bin/bash
echo "sleep:\$1" >> "$fixture_dir/sleep.log"
EOF

  chmod +x "$fixture_dir/bin/claude" "$fixture_dir/bin/sleep"

  output=$(PATH="$fixture_dir/bin:$PATH" bash "$fixture_dir/ralph.sh" --tool claude 1 2>&1)

  assert_contains "$output" "Couldn't parse reset details from Claude output. Falling back to 5h 0m 0s." "Expected 5-hour fallback message"
  assert_contains "$output" "Ralph completed all tasks!" "Expected Ralph to complete after fallback retry"
  assert_file_contains "$fixture_dir/sleep.log" "sleep:18000" "Expected fallback sleep duration to be 18000 seconds"

  pass "Claude unparseable rate-limit message falls back to 5 hours"
}

run_amp_smoke_test() {
  local fixture_dir="$TMP_DIR/amp"
  local output=""

  setup_fixture "$fixture_dir"

  cat > "$fixture_dir/bin/amp" <<'EOF'
#!/bin/bash
cat >/dev/null
echo "<promise>COMPLETE</promise>"
EOF

  chmod +x "$fixture_dir/bin/amp"

  output=$(PATH="$fixture_dir/bin:$PATH" bash "$fixture_dir/ralph.sh" --tool amp 1 2>&1)

  assert_contains "$output" "Ralph completed all tasks!" "Expected amp mode to remain unaffected"

  pass "Amp flow still completes normally"
}

run_claude_new_format_test() {
  local fixture_dir="$TMP_DIR/claude-new-format"
  local output=""

  setup_fixture "$fixture_dir"

  # Simulates the new Claude rate-limit message format:
  # "You've hit your usage limit. It will reset at 7pm (Asia/Tokyo)."
  cat > "$fixture_dir/bin/claude" <<EOF
#!/bin/bash
state_file="$fixture_dir/claude-state"
count=0
if [ -f "\$state_file" ]; then
  count=\$(cat "\$state_file")
fi
count=\$((count + 1))
printf '%s' "\$count" > "\$state_file"
if [ "\$count" -eq 1 ]; then
  echo "You've hit your usage limit. It will reset at 7pm (Asia/Tokyo)."
else
  echo "<promise>COMPLETE</promise>"
fi
EOF

  cat > "$fixture_dir/bin/sleep" <<EOF
#!/bin/bash
echo "sleep:\$1" >> "$fixture_dir/sleep.log"
EOF

  chmod +x "$fixture_dir/bin/claude" "$fixture_dir/bin/sleep"

  output=$(PATH="$fixture_dir/bin:$PATH" bash "$fixture_dir/ralph.sh" --tool claude 1 2>&1)

  assert_contains "$output" "Claude hit a rate limit." "Expected Claude rate-limit detection message for new format"
  assert_contains "$output" "Detected reset time: 7pm (Asia/Tokyo)." "Expected reset-time parsing to succeed for new 'It will reset at' format"
  assert_contains "$output" "Ralph completed all tasks!" "Expected Ralph to complete after retry on new format"
  assert_file_contains "$fixture_dir/claude-state" "2" "Expected Claude to be invoked twice for the same iteration"

  local first_sleep
  first_sleep=$(head -n 1 "$fixture_dir/sleep.log")
  if [[ "$first_sleep" == "sleep:2" ]]; then
    fail "Expected the first sleep to be the computed quota wait, not the normal 2-second pause"
  fi

  pass "New-format 'hit your limit' message: detected and reset time parsed correctly (no 5h fallback)"
}

run_claude_resets_format_test() {
  local fixture_dir="$TMP_DIR/claude-resets-format"
  local output=""

  setup_fixture "$fixture_dir"

  # Simulates the Claude rate-limit message format with "resets":
  # "You've hit your usage limit. resets 12:30am (America/Sao_Paulo)."
  cat > "$fixture_dir/bin/claude" <<EOF
#!/bin/bash
state_file="$fixture_dir/claude-state"
count=0
if [ -f "\$state_file" ]; then
  count=\$(cat "\$state_file")
fi
count=\$((count + 1))
printf '%s' "\$count" > "\$state_file"
if [ "\$count" -eq 1 ]; then
  echo "You've hit your usage limit. resets 12:30am (America/Sao_Paulo)."
else
  echo "<promise>COMPLETE</promise>"
fi
EOF

  cat > "$fixture_dir/bin/sleep" <<EOF
#!/bin/bash
echo "sleep:\$1" >> "$fixture_dir/sleep.log"
EOF

  chmod +x "$fixture_dir/bin/claude" "$fixture_dir/bin/sleep"

  output=$(PATH="$fixture_dir/bin:$PATH" bash "$fixture_dir/ralph.sh" --tool claude 1 2>&1)

  assert_contains "$output" "Claude hit a rate limit." "Expected Claude rate-limit detection message for 'resets' format"
  assert_contains "$output" "Detected reset time: 12:30am (America/Sao_Paulo)." "Expected reset-time parsing to succeed for 'resets' format"
  assert_contains "$output" "Ralph completed all tasks!" "Expected Ralph to complete after retry on 'resets' format"
  assert_file_contains "$fixture_dir/claude-state" "2" "Expected Claude to be invoked twice for the same iteration"

  local first_sleep
  first_sleep=$(head -n 1 "$fixture_dir/sleep.log")
  if [[ "$first_sleep" == "sleep:2" ]]; then
    fail "Expected the first sleep to be the computed quota wait, not the normal 2-second pause"
  fi

  pass "New-format 'resets' message: detected and reset time parsed correctly (no 5h fallback)"
}

run_claude_parseable_test
run_claude_fallback_test
run_claude_new_format_test
run_claude_resets_format_test
run_amp_smoke_test

echo "All Ralph smoke tests passed."
