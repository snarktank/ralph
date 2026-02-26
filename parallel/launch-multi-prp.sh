#!/usr/bin/env bash
set -euo pipefail
#
# launch-multi-prp.sh — Launch one containerized agent per PRP file.
#
# Each PRP gets its own feature branch and agent container. Agents work
# independently on their branches. On completion, branches are fetched
# back to the project and gh pr create commands are printed.
#
# Usage:
#   ./parallel/launch-multi-prp.sh \
#     --project /path/to/repo \
#     --prp prps/prp-03.json \
#     --prp prps/prp-04.json \
#     [--model claude-sonnet-4-5-20250929] \
#     [--memory 4g] [--cpus 2] [--allow-domain example.com]
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RALPH_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Source library scripts
source "$SCRIPT_DIR/lib/logging.sh"
source "$SCRIPT_DIR/lib/auth.sh"
source "$SCRIPT_DIR/lib/network-setup.sh"
source "$SCRIPT_DIR/lib/docker-helpers.sh"

# --- Defaults ---
CLAUDE_MODEL="claude-sonnet-4-5-20250929"
CONTAINER_MEMORY="4g"
CONTAINER_CPUS="2"
MAX_ITERATIONS=0
STALE_CLAIM_MINUTES=30
PROJECT_DIR=""
declare -a PRP_FILES=()
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
        --prp)
            PRP_FILES+=("$2")
            shift 2
            ;;
        --prp=*)
            PRP_FILES+=("${1#*=}")
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
        --max-iterations)
            MAX_ITERATIONS="$2"
            shift 2
            ;;
        --max-iterations=*)
            MAX_ITERATIONS="${1#*=}"
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
            echo "Usage: $0 --project DIR --prp FILE [--prp FILE ...] [options]"
            echo ""
            echo "Launch one containerized agent per PRP file. Each gets its own branch."
            echo ""
            echo "Required:"
            echo "  --project DIR     Project git repository"
            echo "  --prp FILE        PRP JSON file (relative to project dir, repeatable)"
            echo ""
            echo "Options:"
            echo "  --model MODEL     Claude model (default: claude-sonnet-4-5-20250929)"
            echo "  --memory SIZE     Per-container memory (default: 4g)"
            echo "  --cpus N          Per-container CPUs (default: 2)"
            echo "  --max-iterations N  Per-agent iteration cap (default: 0 = until done)"
            echo "  --allow-domain D  Extra domain to whitelist (repeatable)"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# --- Validate inputs ---
if [ -z "$PROJECT_DIR" ]; then
    log_error "Missing --project DIR"
    exit 1
fi
PROJECT_DIR="$(cd "$PROJECT_DIR" && pwd)"

if [ ${#PRP_FILES[@]} -eq 0 ]; then
    log_error "No --prp files specified"
    exit 1
fi

if [ ! -d "$PROJECT_DIR/.git" ]; then
    log_error "$PROJECT_DIR is not a git repository"
    exit 1
fi

# Resolve extra domains
if [ ${#EXTRA_DOMAINS[@]} -gt 0 ]; then
    RALPH_EXTRA_DOMAINS=$(IFS=,; echo "${EXTRA_DOMAINS[*]}")
    export RALPH_EXTRA_DOMAINS
fi

# CLAUDE-parallel.md prompt lives in the ralph repo
PARALLEL_PROMPT="$SCRIPT_DIR/CLAUDE-parallel.md"
if [ ! -f "$PARALLEL_PROMPT" ]; then
    log_error "Missing parallel/CLAUDE-parallel.md prompt file"
    exit 1
fi
export PARALLEL_PROMPT

# --- Validate all PRP files and extract branch names ---
declare -a BRANCH_NAMES=()
declare -a PRP_BASENAMES=()
TOTAL_PRPS=${#PRP_FILES[@]}

log_info "Ralph Multi-PRP Launcher"
log_info "========================"
log_info "Project: $PROJECT_DIR"
log_info "PRPs: $TOTAL_PRPS"
log_info "Model: $CLAUDE_MODEL"
log_info "Memory: $CONTAINER_MEMORY per container"
log_info "CPUs: $CONTAINER_CPUS per container"
echo ""

for prp_file in "${PRP_FILES[@]}"; do
    local_path="$PROJECT_DIR/$prp_file"
    if [ ! -f "$local_path" ]; then
        log_error "PRP file not found: $local_path"
        exit 1
    fi

    branch=$(jq -r '.branchName // empty' "$local_path" 2>/dev/null || echo "")
    if [ -z "$branch" ]; then
        log_error "PRP file missing branchName: $prp_file"
        exit 1
    fi

    project_name=$(jq -r '.project // "unknown"' "$local_path" 2>/dev/null || echo "unknown")
    total_stories=$(jq '.userStories | length' "$local_path" 2>/dev/null || echo "?")

    BRANCH_NAMES+=("$branch")
    PRP_BASENAMES+=("$(basename "$prp_file")")

    log_info "  PRP: $prp_file -> branch: $branch ($total_stories stories)"
done
echo ""

# --- Step 1: Build or verify Docker image ---
log_info "Checking Docker image..."
if ! docker image inspect "$RALPH_IMAGE" &> /dev/null; then
    build_image "$RALPH_ROOT/docker"
else
    log_info "Image $RALPH_IMAGE already exists."
fi

# --- Step 2: Create Docker networks ---
create_networks

# --- Step 3: Verify Claude auth volume ---
log_info "Checking Claude auth volume..."
if ! check_auth_volume; then
    exit 1
fi
log_info "Claude auth volume verified"

# --- Step 4: Create/update bare repo and set up feature branches ---
BARE_REPO="$PROJECT_DIR/.ralph/repo.git"
if [ ! -d "$BARE_REPO" ]; then
    log_info "Creating bare repo for agent coordination..."
    mkdir -p "$PROJECT_DIR/.ralph"
    git clone --bare --filter=blob:none "file://$PROJECT_DIR" "$BARE_REPO"
    log_info "Bare repo created at $BARE_REPO"
else
    log_info "Updating bare repo from project..."
    cd "$PROJECT_DIR"
    git push --force "$BARE_REPO" --all 2>&1 || {
        log_error "Failed to sync bare repo from project."
        exit 1
    }
    cd - > /dev/null
fi

# Clear stop signal
mkdir -p "$PROJECT_DIR/.ralph"
touch "$PROJECT_DIR/.ralph/stop_requested"
: > "$PROJECT_DIR/.ralph/stop_requested"

# Create feature branches and commit each PRP to its branch
TEMP_CLONE=$(mktemp -d)
git clone "$BARE_REPO" "$TEMP_CLONE/work" 2>/dev/null
cd "$TEMP_CLONE/work"
git config user.name "ralph-orchestrator"
git config user.email "orchestrator@ralph-agent.local"

for i in $(seq 0 $((TOTAL_PRPS - 1))); do
    branch="${BRANCH_NAMES[$i]}"
    prp_file="${PRP_FILES[$i]}"
    local_path="$PROJECT_DIR/$prp_file"
    prp_dir=$(dirname "$prp_file")

    log_info "Setting up branch: $branch"

    # Create or checkout the branch
    if git show-ref --verify --quiet "refs/heads/$branch" 2>/dev/null; then
        git checkout "$branch"
    elif git show-ref --verify --quiet "refs/remotes/origin/$branch" 2>/dev/null; then
        git checkout -b "$branch" "origin/$branch"
    else
        # Create from main/master
        git checkout main 2>/dev/null || git checkout master 2>/dev/null || true
        git checkout -b "$branch"
    fi

    # Copy PRP file into the branch
    mkdir -p "$prp_dir"
    cp "$local_path" "$prp_file"

    # Also place as prp.json at root for agent-loop.sh detection
    cp "$local_path" "prp.json"

    git add "$prp_file" "prp.json"
    git commit -m "[orchestrator] Add PRP: $(basename "$prp_file") on $branch" 2>/dev/null || true
    git push origin "$branch" 2>/dev/null || git push --set-upstream origin "$branch" 2>/dev/null
    log_info "  Branch $branch ready with PRP"
done

cd - > /dev/null
rm -rf "$TEMP_CLONE"

# --- Step 5: Load friend identities (optional) ---
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

# --- Step 6: Launch one container per PRP ---
declare -a CONTAINER_NAMES=()
declare -a AGENT_IDS=()
mkdir -p "$PROJECT_DIR/agent_logs"

for i in $(seq 0 $((TOTAL_PRPS - 1))); do
    agent_num=$((i + 1))
    agent_id="agent-${agent_num}"
    container_name="ralph-${agent_id}"
    branch="${BRANCH_NAMES[$i]}"
    prp_basename="${PRP_BASENAMES[$i]}"

    # Assign friend identity if available
    git_author_name=""
    git_author_email=""
    if [ ${#FRIEND_NAMES[@]} -gt 0 ] && [ "$i" -lt ${#FRIEND_NAMES[@]} ]; then
        git_author_name="${FRIEND_NAMES[$i]}"
        git_author_email="${FRIEND_EMAILS[$i]}"
        log_info "Agent $agent_id ($prp_basename) will commit as: $git_author_name <$git_author_email>"
    fi

    # Stop existing container if present
    if docker inspect "$container_name" &> /dev/null; then
        log_warn "Container $container_name already exists. Removing."
        stop_agent "$container_name" 10
    fi

    log_info "Launching $agent_id for $prp_basename on branch $branch"

    launch_agent \
        "$agent_id" \
        "builder" \
        "$PROJECT_DIR" \
        "$CLAUDE_MODEL" \
        "$MAX_ITERATIONS" \
        "$CONTAINER_MEMORY" \
        "$CONTAINER_CPUS" \
        "$git_author_name" \
        "$git_author_email" \
        "$branch"

    CONTAINER_NAMES+=("$container_name")
    AGENT_IDS+=("$agent_id")
done

log_info "All $TOTAL_PRPS agents launched."
echo ""

# --- Step 7: Monitor loop ---
MONITOR_INTERVAL=30
log_info "Entering monitor loop (checking every ${MONITOR_INTERVAL}s)."
log_info "Stop: echo stop > $PROJECT_DIR/.ralph/stop_requested"
echo ""

# Track per-branch completion (use parallel arrays instead of associative array for bash 3.x compat)
declare -a BRANCH_COMPLETE_FLAGS=()
for _i in $(seq 0 $((TOTAL_PRPS - 1))); do
    BRANCH_COMPLETE_FLAGS+=("false")
done

check_branch_complete() {
    local branch="$1"
    local prd_content
    prd_content=$(git --git-dir="$BARE_REPO" show "${branch}:prp.json" 2>/dev/null || echo "")
    [ -z "$prd_content" ] && return 1

    local incomplete
    incomplete=$(echo "$prd_content" | jq '[.userStories[] | select(.passes == false)] | length' 2>/dev/null || echo "1")
    [ "$incomplete" -eq 0 ]
}

recover_stale_claims_for_branch() {
    local branch="$1"
    local prd_content
    prd_content=$(git --git-dir="$BARE_REPO" show "${branch}:prp.json" 2>/dev/null || echo "")
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

        local claimed_epoch
        claimed_epoch=$(date -d "$claimed_at" +%s 2>/dev/null \
            || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$claimed_at" +%s 2>/dev/null \
            || echo "0")
        [ "$claimed_epoch" -eq 0 ] && continue

        local age=$((now_epoch - claimed_epoch))
        if [ "$age" -gt "$stale_seconds" ]; then
            local container_name="ralph-${agent}"
            if ! is_agent_running "$container_name"; then
                log_warn "Stale claim on $branch: $story_id by $agent (${age}s). Clearing."
                updated_prd=$(echo "$updated_prd" | jq --arg sid "$story_id" '
                    .userStories |= map(
                        if .id == $sid then del(.claimed_by) | del(.claimed_at) else . end
                    )
                ')
                cleared=true
            fi
        fi
    done <<< "$claims"

    if $cleared; then
        local temp_dir
        temp_dir=$(mktemp -d)
        git clone "$BARE_REPO" "$temp_dir/work" 2>/dev/null
        cd "$temp_dir/work"
        git config user.name "ralph-orchestrator"
        git config user.email "orchestrator@ralph-agent.local"
        git checkout "$branch" 2>/dev/null || true
        echo "$updated_prd" | jq '.' > "prp.json"
        git add "prp.json"
        git commit -m "[orchestrator] Clear stale claims on $branch" 2>/dev/null || true
        git push origin 2>/dev/null || true
        cd - > /dev/null
        rm -rf "$temp_dir"
    fi
}

while true; do
    sleep "$MONITOR_INTERVAL"

    # Check stop signal
    if [ -s "$PROJECT_DIR/.ralph/stop_requested" ]; then
        log_info "Stop requested. Shutting down all agents..."
        for name in "${CONTAINER_NAMES[@]}"; do
            stop_agent "$name" 30
        done
        teardown_networks
        log_info "All agents stopped."
        exit 0
    fi

    # Per-branch status check and stale claim recovery
    ALL_BRANCHES_DONE=true
    for i in $(seq 0 $((TOTAL_PRPS - 1))); do
        branch="${BRANCH_NAMES[$i]}"
        if [ "${BRANCH_COMPLETE_FLAGS[$i]}" = "true" ]; then
            continue
        fi
        if check_branch_complete "$branch"; then
            log_info "Branch $branch: ALL STORIES COMPLETE"
            BRANCH_COMPLETE_FLAGS[$i]="true"
        else
            ALL_BRANCHES_DONE=false
            recover_stale_claims_for_branch "$branch"
        fi
    done

    # Check container health
    ALL_STOPPED=true
    for i in $(seq 0 $((TOTAL_PRPS - 1))); do
        name="${CONTAINER_NAMES[$i]}"
        branch="${BRANCH_NAMES[$i]}"

        if is_agent_running "$name"; then
            ALL_STOPPED=false
            continue
        fi

        EXIT_CODE=$(docker inspect -f '{{.State.ExitCode}}' "$name" 2>/dev/null || echo "unknown")

        if [ "$EXIT_CODE" = "0" ]; then
            log_info "Container $name exited cleanly (branch: $branch)."
        elif [ "$EXIT_CODE" = "2" ]; then
            log_error "Container $name: AUTH FAILURE (code 2). Halting all agents."
            echo "auth_failure" > "$PROJECT_DIR/.ralph/stop_requested"
            for stop_name in "${CONTAINER_NAMES[@]}"; do
                [ "$stop_name" = "$name" ] && continue
                stop_agent "$stop_name" 10
            done
            teardown_networks
            log_error "All agents stopped. Refresh credentials and re-run."
            exit 1
        else
            # Only restart if the branch isn't complete
            if [ "${BRANCH_COMPLETE_FLAGS[$i]}" != "true" ]; then
                log_warn "Container $name stopped (exit $EXIT_CODE). Restarting..."
                restart_agent "$name"
                if is_agent_running "$name"; then
                    ALL_STOPPED=false
                else
                    log_error "Failed to restart $name"
                fi
            fi
        fi
    done

    # All branches complete?
    if $ALL_BRANCHES_DONE; then
        log_info "ALL BRANCHES COMPLETE!"
        log_info "Shutting down remaining agents..."
        for name in "${CONTAINER_NAMES[@]}"; do
            stop_agent "$name" 15
        done

        # Fetch all branches back to project
        log_info "Syncing branches back to project..."
        cd "$PROJECT_DIR"
        for branch in "${BRANCH_NAMES[@]}"; do
            git fetch "$BARE_REPO" "$branch:$branch" 2>/dev/null || {
                log_warn "Could not fetch branch $branch"
            }
        done
        cd - > /dev/null

        teardown_networks

        echo ""
        log_info "Done! Create PRs with:"
        echo ""
        for i in $(seq 0 $((TOTAL_PRPS - 1))); do
            branch="${BRANCH_NAMES[$i]}"
            prp_file="${PRP_FILES[$i]}"
            desc=$(jq -r '.description // ""' "$PROJECT_DIR/$prp_file" 2>/dev/null || echo "")
            echo "  git push origin $branch && gh pr create --base main --head $branch --title \"${desc:0:70}\" --fill"
        done
        echo ""
        exit 0
    fi

    if $ALL_STOPPED; then
        log_info "All containers exited. Fetching branches..."
        cd "$PROJECT_DIR"
        for branch in "${BRANCH_NAMES[@]}"; do
            git fetch "$BARE_REPO" "$branch:$branch" 2>/dev/null || true
        done
        cd - > /dev/null

        for name in "${CONTAINER_NAMES[@]}"; do
            docker rm "$name" 2>/dev/null || true
        done
        teardown_networks

        echo ""
        log_info "Status per branch:"
        for i in $(seq 0 $((TOTAL_PRPS - 1))); do
            branch="${BRANCH_NAMES[$i]}"
            if [ "${BRANCH_COMPLETE_FLAGS[$i]}" = "true" ]; then
                echo "  $branch: COMPLETE"
            else
                echo "  $branch: INCOMPLETE"
            fi
        done
        echo ""
        exit 0
    fi
done
