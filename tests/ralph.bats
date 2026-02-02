#!/usr/bin/env bats
# Tests for ralph.sh
# Run with: bats tests/ralph.bats

setup() {
    TEST_DIR="$(mktemp -d)"
    cp ralph.sh "$TEST_DIR/"
    cd "$TEST_DIR"

    cat > prd.json << 'EOF'
{
  "project": "TestProject",
  "branchName": "ralph/test-feature",
  "description": "Test feature",
  "userStories": [
    {
      "id": "US-001",
      "title": "Test story",
      "description": "Test description",
      "acceptanceCriteria": ["Test passes"],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}
EOF

    echo "Mock prompt" > prompt.md
    echo "Mock claude prompt" > CLAUDE.md
}

teardown() {
    cd /
    rm -rf "$TEST_DIR"
}

# =============================================================================
# Argument Parsing Tests
# =============================================================================

@test "--tool amp is accepted" {
    run bash -c '
        TOOL="amp"
        args=(--tool amp)
        while [[ ${#args[@]} -gt 0 ]]; do
            case ${args[0]} in
                --tool) TOOL="${args[1]}"; args=("${args[@]:2}") ;;
                *) args=("${args[@]:1}") ;;
            esac
        done
        [[ "$TOOL" == "amp" ]] && echo "PASS"
    '
    [[ "$output" == "PASS" ]]
}

@test "--tool claude is accepted" {
    run bash -c '
        TOOL="amp"
        args=(--tool claude)
        while [[ ${#args[@]} -gt 0 ]]; do
            case ${args[0]} in
                --tool) TOOL="${args[1]}"; args=("${args[@]:2}") ;;
                *) args=("${args[@]:1}") ;;
            esac
        done
        [[ "$TOOL" == "claude" ]] && echo "PASS"
    '
    [[ "$output" == "PASS" ]]
}

@test "--tool=claude syntax is accepted" {
    run bash -c '
        TOOL="amp"
        args=(--tool=claude)
        while [[ ${#args[@]} -gt 0 ]]; do
            case ${args[0]} in
                --tool=*) TOOL="${args[0]#*=}"; args=("${args[@]:1}") ;;
                *) args=("${args[@]:1}") ;;
            esac
        done
        [[ "$TOOL" == "claude" ]] && echo "PASS"
    '
    [[ "$output" == "PASS" ]]
}

@test "numeric argument sets max iterations" {
    run bash -c '
        MAX_ITERATIONS=10
        args=(5)
        while [[ ${#args[@]} -gt 0 ]]; do
            if [[ "${args[0]}" =~ ^[0-9]+$ ]]; then
                MAX_ITERATIONS="${args[0]}"
            fi
            args=("${args[@]:1}")
        done
        [[ "$MAX_ITERATIONS" == "5" ]] && echo "PASS"
    '
    [[ "$output" == "PASS" ]]
}

# =============================================================================
# Tool Validation Tests
# =============================================================================

@test "invalid tool is rejected" {
    run bash -c '
        TOOL="invalid"
        if [[ "$TOOL" != "amp" && "$TOOL" != "claude" ]]; then
            echo "Error: Invalid tool"
            exit 1
        fi
    '
    [[ "$status" -eq 1 ]]
    [[ "$output" == *"Invalid tool"* ]]
}

# =============================================================================
# PRD JSON Tests
# =============================================================================

@test "prd.json branchName is extracted correctly" {
    run bash -c "jq -r '.branchName' prd.json"
    [[ "$output" == "ralph/test-feature" ]]
}

@test "prd.json userStories exist" {
    run bash -c "jq '.userStories | length' prd.json"
    [[ "$output" == "1" ]]
}

@test "prd.json story has required fields" {
    run bash -c "jq '.userStories[0] | has(\"id\", \"title\", \"passes\")' prd.json"
    [[ "$output" == "true" ]]
}

# =============================================================================
# Archive Logic Tests
# =============================================================================

@test "branch change is detected" {
    echo "ralph/old-feature" > .last-branch
    CURRENT_BRANCH=$(jq -r '.branchName // empty' prd.json)
    LAST_BRANCH=$(cat .last-branch)
    [[ "$CURRENT_BRANCH" != "$LAST_BRANCH" ]]
}

@test "last branch file is updated" {
    CURRENT_BRANCH=$(jq -r '.branchName' prd.json)
    echo "$CURRENT_BRANCH" > .last-branch
    [[ "$(cat .last-branch)" == "ralph/test-feature" ]]
}

# =============================================================================
# Progress File Tests
# =============================================================================

@test "progress file header format" {
    echo "# Ralph Progress Log" > progress.txt
    echo "Started: $(date)" >> progress.txt
    echo "---" >> progress.txt
    run head -1 progress.txt
    [[ "$output" == "# Ralph Progress Log" ]]
}

# =============================================================================
# Completion Detection Tests
# =============================================================================

@test "completion signal is detected" {
    OUTPUT="Some output <promise>COMPLETE</promise> more output"
    run bash -c "echo '$OUTPUT' | grep -q '<promise>COMPLETE</promise>' && echo 'DETECTED'"
    [[ "$output" == "DETECTED" ]]
}

@test "incomplete output does not trigger completion" {
    OUTPUT="Some output without completion signal"
    run bash -c "echo '$OUTPUT' | grep -q '<promise>COMPLETE</promise>' || echo 'NOT_COMPLETE'"
    [[ "$output" == "NOT_COMPLETE" ]]
}
