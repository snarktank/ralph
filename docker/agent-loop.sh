#!/usr/bin/env bash
set -euo pipefail
#
# agent-loop.sh — Container entrypoint for Ralph parallel agents.
#
# Clones from a bind-mounted project directory, claims stories from prd.json
# via git atomic push, runs Claude Code per iteration, and pushes results.
#
# Expected environment variables:
#   AGENT_ID                 - Unique agent identifier (e.g., "agent-1")
#   AGENT_ROLE               - One of: builder, researcher
#   MAX_ITERATIONS           - Max loop iterations (0 = infinite, default: 0)
#   CLAUDE_MODEL             - Model to use (default: claude-sonnet-4-5-20250929)
#
# Auth: Claude credentials are mounted via Docker volume at /home/agent/.claude
#
# Exit codes:
#   0 - Clean exit (all stories complete, stop requested, or iteration limit)
#   1 - General failure (missing credentials, prompt error, etc.)
#   2 - Auth failure (expired/invalid credentials after MAX_AUTH_FAILURES retries)
#       The orchestrator treats exit 2 as a signal to halt all agents.
#

AGENT_ID="${AGENT_ID:?AGENT_ID is required}"
AGENT_ROLE="${AGENT_ROLE:?AGENT_ROLE is required}"
MAX_ITERATIONS="${MAX_ITERATIONS:-0}"
CLAUDE_MODEL="${CLAUDE_MODEL:-claude-sonnet-4-5-20250929}"

REPO_PATH="/repo.git"
WORKSPACE="/workspace"
PROMPT_DIR="/parallel-prompt"
PROMPT_FILE="$PROMPT_DIR/CLAUDE-parallel.md"
STOP_FILE="/harness-state/stop_requested"
LOG_DIR="/agent-logs"
ITERATION=0

echo "[$AGENT_ID] Starting agent loop (role=$AGENT_ROLE, model=$CLAUDE_MODEL, max_iterations=$MAX_ITERATIONS)"

# --- Step 1: Initialize firewall based on role ---
echo "[$AGENT_ID] Initializing firewall for role: $AGENT_ROLE"
case "$AGENT_ROLE" in
    researcher)
        sudo /opt/ralph/init-firewall-researcher.sh
        ;;
    *)
        sudo RALPH_EXTRA_DOMAINS="${RALPH_EXTRA_DOMAINS:-}" /opt/ralph/init-firewall-builder.sh
        ;;
esac

# --- Step 2: Copy Claude auth credentials from mounted volume ---
if [ ! -f /claude-auth/.credentials.json ]; then
    echo "[$AGENT_ID] ERROR: No Claude credentials found at /claude-auth/.credentials.json"
    echo "[$AGENT_ID] Ensure the ralph-claude-auth volume is mounted."
    exit 1
fi
mkdir -p ~/.claude
cp /claude-auth/.credentials.json ~/.claude/.credentials.json
chmod 600 ~/.claude/.credentials.json
echo "[$AGENT_ID] Claude credentials copied"

# --- Step 3: Clone or update workspace from bare repo ---
setup_workspace() {
    if [ -d "$WORKSPACE/.git" ]; then
        echo "[$AGENT_ID] Fetching latest changes"
        cd "$WORKSPACE"
        git fetch origin
        # Reset to current branch's remote tracking, not hard-coded main
        local current_branch
        current_branch=$(git branch --show-current 2>/dev/null || echo "")
        if [ -n "$current_branch" ]; then
            git reset --hard "origin/$current_branch" 2>/dev/null || true
        else
            git reset --hard origin/main 2>/dev/null || git reset --hard origin/master 2>/dev/null || true
        fi
    else
        echo "[$AGENT_ID] Cloning bare repo into workspace"
        git clone "$REPO_PATH" "$WORKSPACE"
        cd "$WORKSPACE"
    fi
}

# --- Step 4: Set git identity ---
setup_git_identity() {
    if [ -n "${GIT_AUTHOR_NAME_OVERRIDE:-}" ] && [ -n "${GIT_AUTHOR_EMAIL_OVERRIDE:-}" ]; then
        git config user.name "$GIT_AUTHOR_NAME_OVERRIDE"
        git config user.email "$GIT_AUTHOR_EMAIL_OVERRIDE"
        echo "[$AGENT_ID] Committing as: $GIT_AUTHOR_NAME_OVERRIDE <$GIT_AUTHOR_EMAIL_OVERRIDE>"
    else
        git config user.name "$AGENT_ID"
        git config user.email "${AGENT_ID}@ralph-agent.local"
    fi
    git config pull.rebase true
    git config push.autoSetupRemote true
}

# --- Step 5: Check out the correct branch from prd.json ---
checkout_prd_branch() {
    if [ ! -f "$WORKSPACE/prd.json" ]; then
        echo "[$AGENT_ID] WARNING: No prd.json found in workspace"
        return 1
    fi

    local branch_name
    branch_name=$(jq -r '.branchName // empty' "$WORKSPACE/prd.json" 2>/dev/null || echo "")

    if [ -z "$branch_name" ]; then
        echo "[$AGENT_ID] No branchName in prd.json, staying on current branch"
        return 0
    fi

    local current_branch
    current_branch=$(git branch --show-current 2>/dev/null || echo "")

    if [ "$current_branch" = "$branch_name" ]; then
        echo "[$AGENT_ID] Already on branch: $branch_name"
        return 0
    fi

    echo "[$AGENT_ID] Checking out branch: $branch_name"
    if git show-ref --verify --quiet "refs/heads/$branch_name" 2>/dev/null; then
        git checkout "$branch_name"
    elif git show-ref --verify --quiet "refs/remotes/origin/$branch_name" 2>/dev/null; then
        git checkout -b "$branch_name" "origin/$branch_name"
    else
        git checkout -b "$branch_name"
    fi
}

# --- Step 5b: Check if this agent already owns an incomplete story ---
# Returns 0 and prints story ID if found, returns 1 if no active claim
check_existing_claim() {
    cd "$WORKSPACE"

    # Pull latest prd.json
    local current_branch
    current_branch=$(git branch --show-current 2>/dev/null || echo "")
    # Always fetch first so we discover remote branches created by other agents
    git fetch origin >&2 2>&1 || true
    if git rev-parse --verify "origin/$current_branch" >/dev/null 2>&1; then
        git pull --rebase >&2 2>&1 || {
            git rebase --abort >/dev/null 2>&1 || true
            git fetch origin >&2 2>&1
            git reset --hard "origin/$current_branch" >&2 2>&1
        }
    fi

    if [ ! -f prd.json ]; then
        return 1
    fi

    local story_id
    story_id=$(jq -r --arg agent "$AGENT_ID" '
        .userStories
        | map(select(.passes == false and .claimed_by == $agent))
        | first
        | .id // empty
    ' prd.json 2>/dev/null || echo "")

    if [ -n "$story_id" ]; then
        echo "[$AGENT_ID] Already owns incomplete story: $story_id" >&2
        echo "$story_id"
        return 0
    fi
    return 1
}

# --- Step 5c: Release a claim after Claude failure ---
release_claim() {
    local story_id="$1"
    cd "$WORKSPACE"

    if [ ! -f prd.json ]; then
        return 1
    fi

    echo "[$AGENT_ID] Releasing claim on $story_id" >&2

    jq --arg sid "$story_id" '
        .userStories |= map(
            if .id == $sid then
                .claimed_by = null | .claimed_at = null
            else . end
        )
    ' prd.json > prd.json.tmp && mv prd.json.tmp prd.json

    git add prd.json >&2 2>&1
    git commit -m "[$AGENT_ID] Release: $story_id (Claude failure)" >&2 2>&1 || {
        git checkout -- prd.json >/dev/null 2>&1 || true
        return 1
    }
    git push >&2 2>&1 || {
        echo "[$AGENT_ID] Failed to push release for $story_id" >&2
        git reset --hard HEAD~1 >&2 2>&1
        return 1
    }
    return 0
}

# --- Step 6: Claim a story in prd.json ---
# Returns 0 and prints story ID if claimed, returns 1 if no stories available
claim_story() {
    cd "$WORKSPACE"

    # Pull latest prd.json (all git output to stderr to keep stdout clean for return value)
    # On a new branch with no remote tracking yet, pull will fail — that's fine, we continue
    local current_branch
    current_branch=$(git branch --show-current 2>/dev/null || echo "")
    # Always fetch first so we discover remote branches created by other agents
    git fetch origin >&2 2>&1 || true
    if git rev-parse --verify "origin/$current_branch" >/dev/null 2>&1; then
        git pull --rebase >&2 2>&1 || {
            git rebase --abort >/dev/null 2>&1 || true
            git fetch origin >&2 2>&1
            git reset --hard "origin/$current_branch" >&2 2>&1
        }
    fi

    if [ ! -f prd.json ]; then
        echo "[$AGENT_ID] No prd.json found" >&2
        return 1
    fi

    # Find highest-priority unclaimed story whose dependencies are all satisfied
    local story_id
    story_id=$(jq -r '
        . as $prd |
        ($prd.userStories | map(select(.passes == true)) | map(.id)) as $passed |
        $prd.userStories
        | map(select(
            .passes == false
            and (.claimed_by == null or .claimed_by == "")
            and ((.dependsOn // []) | all(. as $dep | $passed | any(. == $dep)))
        ))
        | sort_by(.priority)
        | first
        | .id // empty
    ' prd.json 2>/dev/null || echo "")

    if [ -z "$story_id" ]; then
        echo "[$AGENT_ID] No unclaimed stories available" >&2
        return 1
    fi

    # Claim it by setting claimed_by and claimed_at
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    jq --arg agent "$AGENT_ID" --arg ts "$timestamp" --arg sid "$story_id" '
        .userStories |= map(
            if .id == $sid then
                .claimed_by = $agent | .claimed_at = $ts
            else . end
        )
    ' prd.json > prd.json.tmp && mv prd.json.tmp prd.json

    git add prd.json >&2 2>&1
    git commit -m "[$AGENT_ID] Claim: $story_id" >&2 2>&1 || {
        echo "[$AGENT_ID] Failed to commit claim for $story_id" >&2
        git checkout -- prd.json >/dev/null 2>&1 || true
        return 1
    }

    # Atomic push — if this fails, another agent claimed something concurrently
    if git push >&2 2>&1; then
        echo "$story_id"
        return 0
    else
        echo "[$AGENT_ID] Push failed (concurrent claim). Resetting and retrying..." >&2
        git reset --hard HEAD~1 >&2 2>&1
        local retry_branch
        retry_branch=$(git branch --show-current 2>/dev/null || echo "")
        # Fetch first to discover remote branches created by other agents
        git fetch origin >&2 2>&1 || true
        if git rev-parse --verify "origin/$retry_branch" >/dev/null 2>&1; then
            git pull --rebase >&2 2>&1 || {
                git rebase --abort >/dev/null 2>&1 || true
                git fetch origin >&2 2>&1
                git reset --hard "origin/$retry_branch" >&2 2>&1
            }
        fi
        return 1
    fi
}

# --- Step 6b: Claim a story for verification ---
# Returns 0 and prints story ID if claimed, returns 1 if no stories ready for verification
claim_verification() {
    cd "$WORKSPACE"

    # Pull latest prd.json
    local current_branch
    current_branch=$(git branch --show-current 2>/dev/null || echo "")
    # Always fetch first so we discover remote branches created by other agents
    git fetch origin >&2 2>&1 || true
    if git rev-parse --verify "origin/$current_branch" >/dev/null 2>&1; then
        git pull --rebase >&2 2>&1 || {
            git rebase --abort >/dev/null 2>&1 || true
            git fetch origin >&2 2>&1
            git reset --hard "origin/$current_branch" >&2 2>&1
        }
    fi

    if [ ! -f prd.json ]; then
        echo "[$AGENT_ID] No prd.json found" >&2
        return 1
    fi

    # Find stories with passes=true, verified!=true, and no verified_by claim
    local story_id
    story_id=$(jq -r '
        .userStories
        | map(select(.passes == true and .verified != true and (.verified_by == null or .verified_by == "")))
        | first
        | .id // empty
    ' prd.json 2>/dev/null || echo "")

    if [ -z "$story_id" ]; then
        echo "[$AGENT_ID] No stories ready for verification" >&2
        return 1
    fi

    # Claim it by setting verified_by and verified_at
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    jq --arg agent "$AGENT_ID" --arg ts "$timestamp" --arg sid "$story_id" '
        .userStories |= map(
            if .id == $sid then
                .verified_by = $agent | .verified_at = $ts
            else . end
        )
    ' prd.json > prd.json.tmp && mv prd.json.tmp prd.json

    git add prd.json >&2 2>&1
    git commit -m "[$AGENT_ID] Verify claim: $story_id" >&2 2>&1 || {
        echo "[$AGENT_ID] Failed to commit verification claim for $story_id" >&2
        git checkout -- prd.json >/dev/null 2>&1 || true
        return 1
    }

    # Atomic push
    if git push >&2 2>&1; then
        echo "$story_id"
        return 0
    else
        echo "[$AGENT_ID] Push failed (concurrent claim). Resetting and retrying..." >&2
        git reset --hard HEAD~1 >&2 2>&1
        local retry_branch
        retry_branch=$(git branch --show-current 2>/dev/null || echo "")
        # Fetch first to discover remote branches created by other agents
        git fetch origin >&2 2>&1 || true
        if git rev-parse --verify "origin/$retry_branch" >/dev/null 2>&1; then
            git pull --rebase >&2 2>&1 || {
                git rebase --abort >/dev/null 2>&1 || true
                git fetch origin >&2 2>&1
                git reset --hard "origin/$retry_branch" >&2 2>&1
            }
        fi
        return 1
    fi
}

# --- Step 7: Check if all stories are complete ---
all_stories_complete() {
    if [ ! -f "$WORKSPACE/prd.json" ]; then
        return 1
    fi

    if [ "$AGENT_ROLE" = "verifier" ]; then
        # Verifiers check that all stories are both passing AND verified
        local incomplete
        incomplete=$(jq '[.userStories[] | select(.passes == false or .verified != true)] | length' "$WORKSPACE/prd.json" 2>/dev/null || echo "1")
        [ "$incomplete" -eq 0 ]
    else
        local incomplete
        incomplete=$(jq '[.userStories[] | select(.passes == false)] | length' "$WORKSPACE/prd.json" 2>/dev/null || echo "1")
        [ "$incomplete" -eq 0 ]
    fi
}

# --- Step 8: Push changes with retry ---
push_with_retry() {
    local max_attempts=3
    local attempt=0
    local branch
    branch=$(git branch --show-current 2>/dev/null || echo "main")

    while [ $attempt -lt $max_attempts ]; do
        if git push origin "$branch" 2>&1; then
            echo "[$AGENT_ID] Push successful."
            return 0
        fi
        attempt=$((attempt + 1))
        echo "[$AGENT_ID] Push failed (attempt $attempt/$max_attempts). Rebasing..."
        git pull --rebase origin "$branch" 2>&1 || {
            echo "[$AGENT_ID] Rebase conflict. Aborting rebase and resetting."
            git rebase --abort 2>/dev/null || true
            git fetch origin
            git reset --hard "origin/$branch"
            return 1
        }
    done

    echo "[$AGENT_ID] Failed to push after $max_attempts attempts."
    return 1
}

# --- Step 9: Prepare the prompt with agent identity ---
prepare_prompt() {
    if [ ! -f "$PROMPT_FILE" ]; then
        echo "[$AGENT_ID] ERROR: Prompt file not found at $PROMPT_FILE"
        return 1
    fi

    # Inject agent identity and claimed story into prompt
    sed -e "s/{{AGENT_ID}}/$AGENT_ID/g" -e "s/{{CLAIMED_STORY}}/$CLAIMED_STORY/g" "$PROMPT_FILE"
}

# --- Main loop ---
setup_workspace
setup_git_identity
checkout_prd_branch

echo "[$AGENT_ID] Entering main loop"

AUTH_FAILURES=0
MAX_AUTH_FAILURES=5

while true; do
    # Check stop signal
    if [ -s "$STOP_FILE" ]; then
        echo "[$AGENT_ID] Stop requested. Exiting gracefully."
        exit 0
    fi

    # Check iteration limit
    if [ "$MAX_ITERATIONS" -gt 0 ] && [ "$ITERATION" -ge "$MAX_ITERATIONS" ]; then
        echo "[$AGENT_ID] Reached max iterations ($MAX_ITERATIONS). Exiting."
        exit 0
    fi

    # Check if all stories are done
    if all_stories_complete; then
        echo "[$AGENT_ID] All stories complete. Exiting."
        exit 0
    fi

    # Clean any unstaged changes from previous iteration to prevent rebase failures
    cd "$WORKSPACE"
    git checkout -- . 2>/dev/null || true
    git clean -fd 2>/dev/null || true

    ITERATION=$((ITERATION + 1))
    COMMIT=$(git rev-parse --short=6 HEAD 2>/dev/null || echo "000000")
    LOGFILE="${LOG_DIR}/${AGENT_ID}_iter${ITERATION}_${COMMIT}.log"
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    echo "[$AGENT_ID] === Iteration $ITERATION (commit: $COMMIT) at $TIMESTAMP ==="

    # First check if this agent already owns an incomplete story (from a failed iteration)
    CLAIMED_STORY=""

    if [ "$AGENT_ROLE" = "verifier" ]; then
        CLAIM_FUNC="claim_verification"
    else
        # Check for existing claim before trying to grab a new one
        if CLAIMED_STORY=$(check_existing_claim); then
            echo "[$AGENT_ID] Resuming existing claim: $CLAIMED_STORY"
        fi
        CLAIM_FUNC="claim_story"
    fi

    # If no existing claim, attempt to claim a new story (retry up to 3 times)
    CLAIM_ATTEMPTS=0
    while [ $CLAIM_ATTEMPTS -lt 3 ] && [ -z "$CLAIMED_STORY" ]; do
        # Use if to prevent set -e from killing the script on claim failure
        if CLAIMED_STORY=$($CLAIM_FUNC); then
            break
        else
            CLAIMED_STORY=""
            CLAIM_ATTEMPTS=$((CLAIM_ATTEMPTS + 1))
            sleep 2
        fi
    done

    if [ -z "$CLAIMED_STORY" ]; then
        if [ "$AGENT_ROLE" = "verifier" ]; then
            echo "[$AGENT_ID] No stories ready for verification. Checking if all complete..."
            if all_stories_complete; then
                echo "[$AGENT_ID] All stories verified. Exiting."
                exit 0
            fi
            echo "[$AGENT_ID] Builders still working. Waiting 30s..."
            sleep 30
            continue
        else
            echo "[$AGENT_ID] Could not claim any story. Checking if all complete..."
            if all_stories_complete; then
                echo "[$AGENT_ID] All stories complete. Exiting."
                exit 0
            fi
            echo "[$AGENT_ID] Stories exist but couldn't claim. Waiting 30s..."
            sleep 30
            continue
        fi
    fi

    echo "[$AGENT_ID] Claimed story: $CLAIMED_STORY"

    # Prepare prompt
    PROMPT=$(prepare_prompt) || {
        echo "[$AGENT_ID] Failed to prepare prompt. Sleeping 10s..."
        sleep 10
        continue
    }

    # Run Claude
    echo "[$AGENT_ID] Running Claude (model: $CLAUDE_MODEL) for story: $CLAIMED_STORY"
    CLAUDE_EXIT=0
    claude --dangerously-skip-permissions \
        --print \
        --model "$CLAUDE_MODEL" \
        -p "$PROMPT" \
        &> "$LOGFILE" || CLAUDE_EXIT=$?

    # Reset auth failure counter on successful invocation
    if [ $CLAUDE_EXIT -eq 0 ]; then
        AUTH_FAILURES=0
    fi

    # Detect hard failures (auth errors, crashes) — release claim so other agents can take it
    if [ $CLAUDE_EXIT -ne 0 ] && [ -f "$LOGFILE" ]; then
        if grep -q "authentication_error\|OAuth token has expired\|Failed to authenticate" "$LOGFILE"; then
            AUTH_FAILURES=$((AUTH_FAILURES + 1))
            echo "[$AGENT_ID] Claude auth failure #$AUTH_FAILURES/$MAX_AUTH_FAILURES. Releasing claim on $CLAIMED_STORY."
            echo "[$AGENT_ID] Error details from log:"
            grep -i "error\|rate\|limit\|auth" "$LOGFILE" | tail -5 || true
            release_claim "$CLAIMED_STORY" || true
            if [ "$AUTH_FAILURES" -ge "$MAX_AUTH_FAILURES" ]; then
                echo "[$AGENT_ID] Reached max auth failures ($MAX_AUTH_FAILURES). Exiting to avoid infinite loop."
                exit 2
            fi
            # Exponential backoff: 60, 120, 240, 480, 480 (capped)
            BACKOFF=$((60 * (1 << (AUTH_FAILURES - 1))))
            [ "$BACKOFF" -gt 480 ] && BACKOFF=480
            echo "[$AGENT_ID] Waiting ${BACKOFF}s before retrying (exponential backoff)..."
            sleep "$BACKOFF"
            continue
        fi
        echo "[$AGENT_ID] Claude exited with error (code: $CLAUDE_EXIT). Check log: $LOGFILE"
    fi

    echo "[$AGENT_ID] Claude session complete. Pushing changes..."

    # Stage and commit any remaining unstaged changes
    if ! git diff --quiet || ! git diff --cached --quiet; then
        git add -A
        if ! git diff --cached --quiet; then
            git commit -m "[$AGENT_ID] Iteration $ITERATION: $CLAIMED_STORY" || true
        fi
    fi

    # Push with retry
    push_with_retry

    # Write per-agent progress
    {
        echo "## $TIMESTAMP - $CLAIMED_STORY (Iteration $ITERATION)"
        echo "- Agent: $AGENT_ID"
        echo "- Commit: $COMMIT"
        echo "---"
    } >> "$WORKSPACE/progress-${AGENT_ID}.txt"

    # Check for completion sentinel in output
    if [ -f "$LOGFILE" ] && grep -q "<promise>COMPLETE</promise>" "$LOGFILE"; then
        echo "[$AGENT_ID] Completion sentinel detected. Verifying all stories..."
        git pull --rebase 2>/dev/null || true
        if all_stories_complete; then
            echo "[$AGENT_ID] All stories confirmed complete. Exiting."
            exit 0
        fi
    fi

    echo "[$AGENT_ID] Iteration $ITERATION complete. Sleeping 5s..."
    sleep 5
done
