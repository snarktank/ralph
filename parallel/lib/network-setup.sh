#!/usr/bin/env bash
#
# network-setup.sh — Docker network creation and teardown for Ralph parallel mode.
#

BUILDER_NETWORK="ralph-builder"
RESEARCHER_NETWORK="ralph-researcher"

create_networks() {
    log_info "Creating Docker networks..."

    # Builder network: bridge with masquerade (firewall handles restrictions)
    if ! docker network inspect "$BUILDER_NETWORK" &> /dev/null; then
        docker network create "$BUILDER_NETWORK" \
            --driver bridge \
            --opt "com.docker.network.bridge.enable_ip_masquerade=true"
        log_info "Created network: $BUILDER_NETWORK"
    else
        log_info "Network $BUILDER_NETWORK already exists"
    fi

    # Researcher network: standard bridge with full internet
    if ! docker network inspect "$RESEARCHER_NETWORK" &> /dev/null; then
        docker network create "$RESEARCHER_NETWORK" \
            --driver bridge
        log_info "Created network: $RESEARCHER_NETWORK"
    else
        log_info "Network $RESEARCHER_NETWORK already exists"
    fi
}

teardown_networks() {
    log_info "Tearing down Docker networks..."

    docker network rm "$BUILDER_NETWORK" 2>/dev/null && \
        log_info "Removed network: $BUILDER_NETWORK" || \
        log_info "Network $BUILDER_NETWORK not found or in use"

    docker network rm "$RESEARCHER_NETWORK" 2>/dev/null && \
        log_info "Removed network: $RESEARCHER_NETWORK" || \
        log_info "Network $RESEARCHER_NETWORK not found or in use"
}
