#!/usr/bin/env bash
#
# auth.sh — Claude auth token retrieval for Ralph parallel mode.
#
# Token retrieval priority:
#   1. RALPH_CLAUDE_TOKEN env var
#   2. .ralph/token file in project directory
#   3. 1Password via `op read` (interactive — only at startup)
#

fetch_claude_token() {
    local project_dir="${1:-}"

    # Priority 1: Environment variable
    if [ -n "${RALPH_CLAUDE_TOKEN:-}" ]; then
        log_info "Using token from RALPH_CLAUDE_TOKEN env var"
        echo "$RALPH_CLAUDE_TOKEN"
        return 0
    fi

    # Priority 2: Token file
    if [ -n "$project_dir" ] && [ -f "$project_dir/.ralph/token" ]; then
        local file_token
        file_token=$(cat "$project_dir/.ralph/token")
        if [ -n "$file_token" ]; then
            log_info "Using token from $project_dir/.ralph/token"
            echo "$file_token"
            return 0
        fi
    fi

    # Priority 3: 1Password (interactive — will prompt for biometric/password)
    log_info "Fetching token from 1Password (may prompt for auth)..."

    if ! command -v op &> /dev/null; then
        log_error "No token available."
        log_error "Provide a token via one of:"
        log_error "  1. RALPH_CLAUDE_TOKEN env var"
        log_error "  2. File at <project>/.ralph/token"
        log_error "  3. Install 1Password CLI: https://developer.1password.com/docs/cli/get-started/"
        return 1
    fi

    local op_ref="${OP_ITEM_REF:-op://Private/Claude Code OAuth/credential}"
    local token
    token=$(op read "$op_ref" 2>&1) || {
        log_error "Failed to read token from 1Password."
        log_error "Reference: $op_ref"
        log_error "Fallback: set RALPH_CLAUDE_TOKEN env var or write token to <project>/.ralph/token"
        return 1
    }

    if [ -z "$token" ]; then
        log_error "1Password returned empty token."
        return 1
    fi

    echo "$token"
}
