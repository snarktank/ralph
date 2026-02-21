#!/usr/bin/env bash
set -euo pipefail
#
# ralph-parallel.sh — Parallel mode orchestrator for Ralph.
#
# Launches N containerized Claude Code agents that work on prd.json stories
# simultaneously. Each agent runs in a Docker container with network restrictions,
# resource limits, and no host access.
#
# Usage: ./parallel/ralph-parallel.sh [options] [max_iterations]
#
# Options:
#   --project DIR     Project directory containing prd.json (default: current dir)
#   --image IMAGE     Custom Docker image (default: ralph-agent:latest, auto-built)
#   --agents N        Number of builder agents (default: 2)
#   --researcher N    Number of researcher agents with full internet (default: 0)
#   --model MODEL     Claude model to use (default: claude-sonnet-4-5-20250929)
#   --memory SIZE     Per-container memory limit (default: 4g)
#   --cpus N          Per-container CPU limit (default: 2)
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RALPH_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Source library scripts
source "$SCRIPT_DIR/lib/logging.sh"
source "$SCRIPT_DIR/lib/auth.sh"
source "$SCRIPT_DIR/lib/network-setup.sh"
source "$SCRIPT_DIR/lib/docker-helpers.sh"

# --- Defaults ---
NUM_BUILDERS=2
NUM_RESEARCHERS=0
NUM_VERIFIERS=0
CLAUDE_MODEL="claude-sonnet-4-5-20250929"
CONTAINER_MEMORY="4g"
CONTAINER_CPUS="2"
MAX_ITERATIONS=0
STALE_CLAIM_MINUTES=30
PROJECT_DIR=""
CUSTOM_IMAGE=""
declare -a EXTRA_DOMAINS=()

# --- Parse arguments ---
while [[ $# -gt 0 ]]; do
    case $1 in
        --project)
            PROJECT_DIR="$2"
            shift 2
            ;;
        --project=*)
            PROJECT_DIR="${1#*=}"
            shift
            ;;
        --image)
            CUSTOM_IMAGE="$2"
            shift 2
            ;;
        --image=*)
            CUSTOM_IMAGE="${1#*=}"
            shift
            ;;
        --agents)
            NUM_BUILDERS="$2"
            shift 2
            ;;
        --agents=*)
            NUM_BUILDERS="${1#*=}"
            shift
            ;;
        --researcher)
            NUM_RESEARCHERS="$2"
            shift 2
            ;;
        --researcher=*)
            NUM_RESEARCHERS="${1#*=}"
            shift
            ;;
        --verifier)
            NUM_VERIFIERS="$2"
            shift 2
            ;;
        --verifier=*)
            NUM_VERIFIERS="${1#*=}"
            shift
            ;;
        --model)
            CLAUDE_MODEL="$2"
            shift 2
            ;;
        --model=*)
            CLAUDE_MODEL="${1#*=}"
            shift
            ;;
        --memory)
            CONTAINER_MEMORY="$2"
            shift 2
            ;;
        --memory=*)
            CONTAINER_MEMORY="${1#*=}"
            shift
            ;;
        --cpus)
            CONTAINER_CPUS="$2"
            shift 2
            ;;
        --cpus=*)
            CONTAINER_CPUS="${1#*=}"
            shift
            ;;
        --allow-domain)
            EXTRA_DOMAINS+=("$2")
            shift 2
            ;;
        --allow-domain=*)
            EXTRA_DOMAINS+=("${1#*=}")
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [options] [max_iterations]"
            echo ""
            echo "Options:"
            echo "  --project DIR     Project directory with prd.json (default: current dir)"
            echo "  --image IMAGE     Custom Docker image (default: ralph-agent:latest)"
            echo "  --agents N        Number of builder agents (default: 2)"
            echo "  --researcher N    Number of researcher agents (default: 0)"
            echo "  --verifier N      Number of verifier agents (default: 0)"
            echo "  --model MODEL     Claude model (default: claude-sonnet-4-5-20250929)"
            echo "  --memory SIZE     Per-container memory limit (default: 4g)"
            echo "  --cpus N          Per-container CPU limit (default: 2)"
            echo "  --allow-domain D  Extra domain to whitelist in firewall (repeatable)"
            echo ""
            echo "Arguments:"
            echo "  max_iterations    Per-agent iteration cap (default: 0 = until PRD complete)"
            exit 0
            ;;
        *)
            if [[ "$1" =~ ^[0-9]+$ ]]; then
                MAX_ITERATIONS="$1"
            else
                log_error "Unknown option: $1"
                exit 1
            fi
            shift
            ;;
    esac
done

TOTAL_AGENTS=$((NUM_BUILDERS + NUM_RESEARCHERS + NUM_VERIFIERS))

if [ "$TOTAL_AGENTS" -eq 0 ]; then
    log_error "No agents configured. Use --agents N and/or --researcher N."
    exit 1
fi

# --- Validate project directory ---
# Default to current working directory if --project not specified
if [ -z "$PROJECT_DIR" ]; then
    PROJECT_DIR="$(pwd)"
fi
# Resolve to absolute path
PROJECT_DIR="$(cd "$PROJECT_DIR" && pwd)"
PRD_FILE="$PROJECT_DIR/prd.json"
# CLAUDE-parallel.md lives in the ralph repo, not the project
PARALLEL_PROMPT="$SCRIPT_DIR/CLAUDE-parallel.md"
VERIFIER_PROMPT="$SCRIPT_DIR/CLAUDE-verifier.md"

if [ ! -f "$PRD_FILE" ]; then
    log_error "No prd.json found in $PROJECT_DIR"
    log_error "Create a prd.json first (see prd.json.example)."
    exit 1
fi

if [ ! -f "$PARALLEL_PROMPT" ]; then
    log_error "Missing parallel/CLAUDE-parallel.md prompt file"
    exit 1
fi

if [ "$NUM_VERIFIERS" -gt 0 ] && [ ! -f "$VERIFIER_PROMPT" ]; then
    log_error "Missing parallel/CLAUDE-verifier.md prompt file (required when --verifier > 0)"
    exit 1
fi

if [ ! -d "$PROJECT_DIR/.git" ]; then
    log_error "$PROJECT_DIR is not a git repository"
    exit 1
fi

# --- Display config ---
PROJECT_NAME=$(jq -r '.project // "unknown"' "$PRD_FILE" 2>/dev/null || echo "unknown")
BRANCH_NAME=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
TOTAL_STORIES=$(jq '.userStories | length' "$PRD_FILE" 2>/dev/null || echo "?")
DONE_STORIES=$(jq '[.userStories[] | select(.passes == true)] | length' "$PRD_FILE" 2>/dev/null || echo "?")

log_info "Ralph Parallel Mode"
log_info "===================="
log_info "Project: $PROJECT_NAME"
log_info "Branch: ${BRANCH_NAME:-<not set>}"
log_info "Stories: $DONE_STORIES/$TOTAL_STORIES complete"
log_info "Agents: $NUM_BUILDERS builders, $NUM_RESEARCHERS researchers, $NUM_VERIFIERS verifiers ($TOTAL_AGENTS total)"
log_info "Image: ${CUSTOM_IMAGE:-$RALPH_IMAGE (default)}"
log_info "Model: $CLAUDE_MODEL"
log_info "Memory: $CONTAINER_MEMORY per container"
log_info "CPUs: $CONTAINER_CPUS per container"
log_info "Max iterations: $MAX_ITERATIONS (0=until PRD complete)"
if [ ${#EXTRA_DOMAINS[@]} -gt 0 ]; then
    RALPH_EXTRA_DOMAINS=$(IFS=,; echo "${EXTRA_DOMAINS[*]}")
    export RALPH_EXTRA_DOMAINS
    log_info "Extra domains: $RALPH_EXTRA_DOMAINS"
else
    RALPH_EXTRA_DOMAINS=""
fi
echo ""

# --- Step 1: Build or verify Docker image ---
PROJECT_DOCKERFILE="$PROJECT_DIR/Dockerfile.ralph"

if [ -n "$CUSTOM_IMAGE" ]; then
    # Explicit --image flag takes priority
    export RALPH_IMAGE="$CUSTOM_IMAGE"
    log_info "Using custom image: $RALPH_IMAGE"
    if ! docker image inspect "$RALPH_IMAGE" &> /dev/null; then
        log_error "Custom image '$RALPH_IMAGE' not found. Build it first."
        exit 1
    fi
elif [ -f "$PROJECT_DOCKERFILE" ]; then
    # Project has a Dockerfile.ralph — build a project-specific image
    # Tag includes project name to avoid collisions between projects
    PROJECT_IMAGE_TAG="ralph-agent-${PROJECT_NAME}:latest"
    export RALPH_IMAGE="$PROJECT_IMAGE_TAG"
    log_info "Found Dockerfile.ralph — building project image: $RALPH_IMAGE"

    # Always ensure the base image exists first
    if ! docker image inspect "ralph-agent:latest" &> /dev/null; then
        log_info "Building base image first..."
        build_image "$RALPH_ROOT/docker"
    fi

    build_image "$PROJECT_DIR" "$PROJECT_DOCKERFILE"
else
    log_info "Checking Docker image..."
    if ! docker image inspect "$RALPH_IMAGE" &> /dev/null; then
        build_image "$RALPH_ROOT/docker"
    else
        log_info "Image $RALPH_IMAGE already exists. Use 'docker rmi $RALPH_IMAGE' to force rebuild."
    fi
fi

# --- Step 2: Create Docker networks ---
create_networks

# --- Step 3: Verify Claude auth volume ---
log_info "Checking Claude auth volume..."
if ! check_auth_volume; then
    exit 1
fi
log_info "Claude auth volume verified"

# --- Step 4: Create bare repo for agent coordination ---
# Agents need a shared bare repo to push to — you can't reliably push
# to a non-bare repo's checked-out branch. We create .ralph/repo.git
# as a bare clone of the project, and agents push/pull from this.
BARE_REPO="$PROJECT_DIR/.ralph/repo.git"
if [ ! -d "$BARE_REPO" ]; then
    log_info "Creating bare repo for agent coordination..."
    mkdir -p "$PROJECT_DIR/.ralph"
    git clone --bare --filter=blob:none "file://$PROJECT_DIR" "$BARE_REPO"
    log_info "Bare repo created at $BARE_REPO"
else
    # Update the bare repo from the working directory
    log_info "Updating bare repo from project..."
    cd "$PROJECT_DIR"
    git push --force "$BARE_REPO" --all 2>&1 || {
        log_error "Failed to sync bare repo from project. Manual intervention required."
        exit 1
    }
    # Clean stale feature branches from bare repo to prevent agents from seeing old state
    log_info "Cleaning stale branches from bare repo..."
    git --git-dir="$BARE_REPO" for-each-ref --format='%(refname:short)' refs/heads/ | \
        grep -Ev '^(main|master)$' | \
        xargs -I{} git --git-dir="$BARE_REPO" branch -D {} 2>/dev/null || true
    cd - > /dev/null
fi

# Clear any previous stop signal (truncate to empty; file is kept for Docker bind-mount)
: > "$PROJECT_DIR/.ralph/stop_requested"

# --- Step 6: Launch agent containers ---
AGENT_NUM=0
declare -a CONTAINER_NAMES=()

# Load friend identities for git author spoofing (optional)
FRIENDS_FILE="$PROJECT_DIR/.ralph/friends.json"
declare -a FRIEND_NAMES=()
declare -a FRIEND_EMAILS=()
if [ -f "$FRIENDS_FILE" ]; then
    while IFS= read -r name; do
        FRIEND_NAMES+=("$name")
    done < <(jq -r '.[].name' "$FRIENDS_FILE")
    while IFS= read -r email; do
        FRIEND_EMAILS+=("$email")
    done < <(jq -r '.[].email' "$FRIENDS_FILE")
    log_info "Loaded ${#FRIEND_NAMES[@]} friend identities from friends.json"
fi

launch_agents_for_role() {
    local role="$1"
    local count="$2"

    [ "$count" -le 0 ] && return

    for i in $(seq 1 "$count"); do
        AGENT_NUM=$((AGENT_NUM + 1))
        local agent_id="agent-${AGENT_NUM}"
        local container_name="ralph-${agent_id}"

        # Assign friend identity if available, otherwise use default agent identity
        local git_author_name=""
        local git_author_email=""
        local idx=$((AGENT_NUM - 1))
        if [ ${#FRIEND_NAMES[@]} -gt 0 ] && [ "$idx" -lt ${#FRIEND_NAMES[@]} ]; then
            git_author_name="${FRIEND_NAMES[$idx]}"
            git_author_email="${FRIEND_EMAILS[$idx]}"
            log_info "Agent $agent_id will commit as: $git_author_name <$git_author_email>"
        fi

        # Stop existing container with same name if present
        if docker inspect "$container_name" &> /dev/null; then
            log_warn "Container $container_name already exists. Removing."
            stop_agent "$container_name" 10
        fi

        launch_agent \
            "$agent_id" \
            "$role" \
            "$PROJECT_DIR" \
            "$CLAUDE_MODEL" \
            "$MAX_ITERATIONS" \
            "$CONTAINER_MEMORY" \
            "$CONTAINER_CPUS" \
            "$git_author_name" \
            "$git_author_email"

        CONTAINER_NAMES+=("$container_name")
    done
}

log_info "Launching agents..."
launch_agents_for_role "builder" "$NUM_BUILDERS"
launch_agents_for_role "researcher" "$NUM_RESEARCHERS"
launch_agents_for_role "verifier" "$NUM_VERIFIERS"

log_info "All $TOTAL_AGENTS agents launched."
echo ""

# --- Step 6: Monitor loop ---
MONITOR_INTERVAL=30
log_info "Entering monitor loop (checking every ${MONITOR_INTERVAL}s)."
log_info "Use ./parallel/stop.sh to stop."
log_info "Use ./parallel/status.sh to check status."
echo ""

# Helper: read a file from the bare repo without a full checkout
read_from_bare_repo() {
    local file="$1"
    local branch="${2:-main}"
    git --git-dir="$BARE_REPO" show "${branch}:${file}" 2>/dev/null \
        || git --git-dir="$BARE_REPO" show "master:${file}" 2>/dev/null \
        || echo ""
}

# Helper: check if all stories are complete in the bare repo
# When verifiers are in use, requires passes AND verified for all stories.
check_all_stories_complete() {
    local prd_content
    prd_content=$(read_from_bare_repo "prd.json" "$BRANCH_NAME")
    [ -z "$prd_content" ] && return 1

    if [ "$NUM_VERIFIERS" -gt 0 ]; then
        # Require both passes and verified
        local incomplete
        incomplete=$(echo "$prd_content" | jq '[.userStories[] | select(.passes == false or .verified != true)] | length' 2>/dev/null || echo "1")
        [ "$incomplete" -eq 0 ]
    else
        # Passes-only (backward compat)
        local incomplete
        incomplete=$(echo "$prd_content" | jq '[.userStories[] | select(.passes == false)] | length' 2>/dev/null || echo "1")
        [ "$incomplete" -eq 0 ]
    fi
}

recover_stale_claims() {
    # Read prd.json from the bare repo (agents push there, not to project dir)
    local prd_content
    prd_content=$(read_from_bare_repo "prd.json" "$BRANCH_NAME")
    [ -z "$prd_content" ] && return

    local now_epoch
    now_epoch=$(date +%s)
    local stale_seconds=$((STALE_CLAIM_MINUTES * 60))

    local claims
    claims=$(echo "$prd_content" | jq -r '
        .userStories[]
        | select(.passes == false and .claimed_by != null and .claimed_by != "")
        | "\(.id)|\(.claimed_by)|\(.claimed_at // "")"
    ' 2>/dev/null || echo "")

    [ -z "$claims" ] && return

    local cleared=false
    local updated_prd="$prd_content"
    while IFS='|' read -r story_id agent claimed_at; do
        [ -z "$story_id" ] && continue
        [ -z "$claimed_at" ] && continue

        # Parse claimed_at timestamp (GNU date -d first, macOS date -j fallback)
        local claimed_epoch
        claimed_epoch=$(date -d "$claimed_at" +%s 2>/dev/null \
            || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$claimed_at" +%s 2>/dev/null \
            || echo "0")

        if [ "$claimed_epoch" -eq 0 ]; then
            continue
        fi

        local age=$((now_epoch - claimed_epoch))
        if [ "$age" -gt "$stale_seconds" ]; then
            local container_name="ralph-${agent}"
            if ! is_agent_running "$container_name"; then
                log_warn "Stale claim detected: $story_id by $agent (${age}s old, container not running). Clearing."
                updated_prd=$(echo "$updated_prd" | jq --arg sid "$story_id" '
                    .userStories |= map(
                        if .id == $sid then
                            del(.claimed_by) | del(.claimed_at)
                        else . end
                    )
                ')
                cleared=true
            fi
        fi
    done <<< "$claims"

    if $cleared; then
        # Commit the cleared claims to the bare repo via a temp checkout
        local temp_dir
        temp_dir=$(mktemp -d)
        git clone "$BARE_REPO" "$temp_dir/work" 2>/dev/null
        cd "$temp_dir/work"
        git config user.name "ralph-orchestrator"
        git config user.email "orchestrator@ralph-agent.local"
        if [ -n "$BRANCH_NAME" ]; then
            git checkout "$BRANCH_NAME" 2>/dev/null || true
        fi
        echo "$updated_prd" | jq '.' > prd.json
        git add prd.json
        git commit -m "[orchestrator] Clear stale claims" 2>/dev/null || true
        git push origin 2>/dev/null || true
        cd - > /dev/null
        rm -rf "$temp_dir"
    fi
}

recover_stale_verification_claims() {
    [ "$NUM_VERIFIERS" -eq 0 ] && return

    local prd_content
    prd_content=$(read_from_bare_repo "prd.json" "$BRANCH_NAME")
    [ -z "$prd_content" ] && return

    local now_epoch
    now_epoch=$(date +%s)
    local stale_seconds=$((STALE_CLAIM_MINUTES * 60))

    local vclaims
    vclaims=$(echo "$prd_content" | jq -r '
        .userStories[]
        | select(.passes == true and .verified != true and .verified_by != null and .verified_by != "")
        | "\(.id)|\(.verified_by)|\(.verified_at // "")"
    ' 2>/dev/null || echo "")

    [ -z "$vclaims" ] && return

    local cleared=false
    local updated_prd="$prd_content"
    while IFS='|' read -r story_id agent verified_at; do
        [ -z "$story_id" ] && continue
        [ -z "$verified_at" ] && continue

        local claimed_epoch
        claimed_epoch=$(date -d "$verified_at" +%s 2>/dev/null \
            || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$verified_at" +%s 2>/dev/null \
            || echo "0")

        if [ "$claimed_epoch" -eq 0 ]; then
            continue
        fi

        local age=$((now_epoch - claimed_epoch))
        if [ "$age" -gt "$stale_seconds" ]; then
            local container_name="ralph-${agent}"
            if ! is_agent_running "$container_name"; then
                log_warn "Stale verification claim: $story_id by $agent (${age}s old, container not running). Clearing."
                updated_prd=$(echo "$updated_prd" | jq --arg sid "$story_id" '
                    .userStories |= map(
                        if .id == $sid then
                            .verified_by = null | .verified_at = null
                        else . end
                    )
                ')
                cleared=true
            fi
        fi
    done <<< "$vclaims"

    if $cleared; then
        local temp_dir
        temp_dir=$(mktemp -d)
        git clone "$BARE_REPO" "$temp_dir/work" 2>/dev/null
        cd "$temp_dir/work"
        git config user.name "ralph-orchestrator"
        git config user.email "orchestrator@ralph-agent.local"
        if [ -n "$BRANCH_NAME" ]; then
            git checkout "$BRANCH_NAME" 2>/dev/null || true
        fi
        echo "$updated_prd" | jq '.' > prd.json
        git add prd.json
        git commit -m "[orchestrator] Clear stale verification claims" 2>/dev/null || true
        git push origin 2>/dev/null || true
        cd - > /dev/null
        rm -rf "$temp_dir"
    fi
}

while true; do
    sleep "$MONITOR_INTERVAL"

    # Check if stop was requested
    if [ -s "$PROJECT_DIR/.ralph/stop_requested" ]; then
        log_info "Stop requested. Shutting down all agents..."
        for name in "${CONTAINER_NAMES[@]}"; do
            stop_agent "$name" 30
        done
        teardown_networks
        log_info "All agents stopped. Exiting."
        exit 0
    fi

    # Recover stale claims
    recover_stale_claims
    recover_stale_verification_claims

    # Check container health
    ALL_STOPPED=true
    for name in "${CONTAINER_NAMES[@]}"; do
        if is_agent_running "$name"; then
            ALL_STOPPED=false
        else
            EXIT_CODE=$(docker inspect -f '{{.State.ExitCode}}' "$name" 2>/dev/null || echo "unknown")

            if [ "$EXIT_CODE" = "0" ]; then
                log_info "Container $name exited cleanly (code 0)."
            elif [ "$EXIT_CODE" = "2" ]; then
                log_error "Container $name exited due to auth failure (code 2). Credentials may be expired."
                log_error "Halting all agents."
                echo "auth_failure" > "$PROJECT_DIR/.ralph/stop_requested"
                for stop_name in "${CONTAINER_NAMES[@]}"; do
                    [ "$stop_name" = "$name" ] && continue
                    stop_agent "$stop_name" 10
                done
                teardown_networks
                log_error "All agents stopped. Refresh your Claude credentials and re-run."
                exit 1
            else
                log_warn "Container $name stopped unexpectedly (exit code: $EXIT_CODE). Restarting..."
                restart_agent "$name"
                if is_agent_running "$name"; then
                    ALL_STOPPED=false
                else
                    log_error "Failed to restart $name"
                fi
            fi
        fi
    done

    # Check if all stories are complete (read from bare repo)
    if check_all_stories_complete; then
        log_info "All PRD stories are complete!"
        log_info "Shutting down agents..."
        for name in "${CONTAINER_NAMES[@]}"; do
            stop_agent "$name" 15
        done
        # Sync bare repo back to project working directory
        log_info "Syncing results back to project..."
        cd "$PROJECT_DIR"
        git fetch "$BARE_REPO" 2>/dev/null || true
        git merge FETCH_HEAD 2>/dev/null || true
        teardown_networks
        log_info "Done. All stories passed."
        exit 0
    fi

    if $ALL_STOPPED; then
        log_info "All agents have exited. Cleaning up..."
        for name in "${CONTAINER_NAMES[@]}"; do
            docker rm "$name" 2>/dev/null || true
        done
        teardown_networks
        log_info "Done."
        exit 0
    fi
done
