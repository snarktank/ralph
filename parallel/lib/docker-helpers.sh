#!/usr/bin/env bash
#
# docker-helpers.sh — Container launch and management helpers for Ralph parallel mode.
#

RALPH_IMAGE="${RALPH_IMAGE:-ralph-agent:latest}"
CLAUDE_AUTH_VOLUME="ralph-claude-auth"

build_image() {
    local docker_dir="$1"
    local dockerfile="${2:-}"

    log_info "Building Ralph agent image..."
    if [ -n "$dockerfile" ]; then
        docker build -t "$RALPH_IMAGE" -f "$dockerfile" "$docker_dir"
    else
        docker build -t "$RALPH_IMAGE" "$docker_dir"
    fi
    log_info "Image built: $RALPH_IMAGE"
}

# Verify the shared Claude auth volume exists and has credentials
check_auth_volume() {
    if ! docker volume inspect "$CLAUDE_AUTH_VOLUME" &> /dev/null; then
        log_error "Claude auth volume '$CLAUDE_AUTH_VOLUME' not found."
        log_error "Run the auth setup first:"
        log_error "  docker run -it --entrypoint bash \\"
        log_error "    -v $CLAUDE_AUTH_VOLUME:/home/agent/.claude \\"
        log_error "    $RALPH_IMAGE"
        log_error "  Then inside: claude login"
        return 1
    fi

    # Quick check that credentials exist on the volume
    local has_creds
    has_creds=$(docker run --rm --entrypoint test \
        -v "$CLAUDE_AUTH_VOLUME":/claude-auth:ro \
        "$RALPH_IMAGE" -f /claude-auth/.credentials.json && echo "yes" || echo "no")

    if [ "$has_creds" != "yes" ]; then
        log_error "No credentials found on auth volume. Run 'claude login' in a container first."
        return 1
    fi

    return 0
}

launch_agent() {
    local agent_id="$1"
    local agent_role="$2"
    local project_dir="$3"
    local claude_model="$4"
    local max_iterations="$5"
    local container_memory="${6:-4g}"
    local container_cpus="${7:-2}"
    local git_author_name="${8:-}"
    local git_author_email="${9:-}"
    local ralph_branch="${10:-}"

    # Determine network based on role (verifiers use builder network — no internet needed)
    local network
    case "$agent_role" in
        researcher) network="$RESEARCHER_NETWORK" ;;
        *)          network="$BUILDER_NETWORK" ;;
    esac

    local container_name="ralph-${agent_id}"
    local project_dir_abs
    project_dir_abs="$(cd "$project_dir" && pwd)"

    # Select prompt based on role: verifiers get CLAUDE-verifier.md, others get CLAUDE-parallel.md
    local prompt_path
    if [ "$agent_role" = "verifier" ]; then
        prompt_path="${VERIFIER_PROMPT:-$project_dir_abs/parallel/CLAUDE-verifier.md}"
    else
        prompt_path="${PARALLEL_PROMPT:-$project_dir_abs/parallel/CLAUDE-parallel.md}"
    fi

    log_info "Launching container: $container_name (role=$agent_role, network=$network)"

    # Ensure log and state directories exist, and stop signal file is present
    # (Docker errors on bind-mounting a nonexistent file)
    mkdir -p "$project_dir_abs/agent_logs" "$project_dir_abs/.ralph"
    touch "$project_dir_abs/.ralph/stop_requested"

    docker run -d \
        --name "$container_name" \
        --network "$network" \
        --memory="$container_memory" \
        --cpus="$container_cpus" \
        --pids-limit=256 \
        --cap-add=NET_ADMIN \
        --cap-add=NET_RAW \
        --label "ralph.role=$agent_role" \
        --label "ralph.agent_id=$agent_id" \
        --label "ralph.project_dir=$project_dir_abs" \
        -e "AGENT_ID=$agent_id" \
        -e "AGENT_ROLE=$agent_role" \
        -e "CLAUDE_MODEL=$claude_model" \
        -e "MAX_ITERATIONS=$max_iterations" \
        -e "GIT_AUTHOR_NAME_OVERRIDE=${git_author_name}" \
        -e "GIT_AUTHOR_EMAIL_OVERRIDE=${git_author_email}" \
        -e "RALPH_EXTRA_DOMAINS=${RALPH_EXTRA_DOMAINS:-}" \
        -e "RALPH_BRANCH=${ralph_branch}" \
        -v "$CLAUDE_AUTH_VOLUME:/claude-auth:ro" \
        -v "$project_dir_abs/.ralph/repo.git:/repo.git:rw" \
        -v "$prompt_path:/parallel-prompt/CLAUDE-parallel.md:ro" \
        -v "$project_dir_abs/agent_logs:/agent-logs:rw" \
        -v "$project_dir_abs/.ralph/stop_requested:/harness-state/stop_requested:ro" \
        "$RALPH_IMAGE"

    log_info "Container $container_name started"
}

stop_agent() {
    local container_name="$1"
    local timeout="${2:-30}"

    log_info "Stopping container: $container_name (timeout=${timeout}s)"
    docker stop -t "$timeout" "$container_name" 2>/dev/null || true
    docker rm "$container_name" 2>/dev/null || true
}

is_agent_running() {
    local container_name="$1"
    docker inspect -f '{{.State.Running}}' "$container_name" 2>/dev/null | grep -q "true"
}

restart_agent() {
    local container_name="$1"

    log_info "Restarting container: $container_name"
    docker restart "$container_name" 2>/dev/null || {
        log_error "Could not restart $container_name"
        return 1
    }
    log_info "Container $container_name restarted"
}
