# syntax=docker/dockerfile:1

# Ralph MCP Server - Optimized multi-stage build with caching
# This Dockerfile builds the Ralph MCP server for use with Docker MCP toolkit
#
# Build Optimizations (US-001 through US-005):
#   - cargo-chef: Caches Rust dependencies between builds
#   - sccache: Compiler-level caching for faster rebuilds
#   - BuildKit cache mounts: Persists cargo registry and target directories
#
# NOTE: This is a deployment repository. The Dockerfile clones the Ralph source
# from GitHub during build. Source code lives at: https://github.com/kcirtapfromspace/ralph
#
# Build Arguments:
#   VERSION     - Semantic version for the build (e.g., "1.0.0")
#   COMMIT_SHA  - Git commit SHA for traceability
#   RALPH_REPO  - Git repository URL (default: https://github.com/kcirtapfromspace/ralph.git)
#   RALPH_REF   - Git branch/tag to build from (default: main)
#
# Usage:
#   docker build -t ralph .
#   docker build --build-arg RALPH_REF=v1.0.0 -t ralph:1.0.0 .
#   docker build --build-arg VERSION=1.0.0 --build-arg COMMIT_SHA=$(git rev-parse HEAD) -t ralph .
#
# These build args are embedded as labels for version tracking and debugging.

# Build arguments for version tracking
ARG VERSION=dev
ARG COMMIT_SHA=unknown

# Stage 1: Install cargo-chef for dependency caching (US-001)
FROM rust:1.85-slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Stage 2: Plan dependencies (cargo-chef creates a recipe)
FROM chef AS planner
# Clone the Ralph source repository
ARG RALPH_REPO=https://github.com/kcirtapfromspace/ralph.git
ARG RALPH_REF=main
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*
RUN git clone --depth 1 --branch ${RALPH_REF} ${RALPH_REPO} .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (cached layer)
FROM chef AS builder

# Install build dependencies and sccache (US-002)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install sccache for compiler-level caching
RUN cargo install sccache
ENV RUSTC_WRAPPER=/usr/local/cargo/bin/sccache
ENV SCCACHE_DIR=/sccache

# Copy the dependency recipe from planner
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies only (cached unless Cargo.toml/Cargo.lock change)
# Uses BuildKit cache mounts for cargo registry and sccache (US-003)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/sccache \
    cargo chef cook --release --recipe-path recipe.json

# Clone the full source and build the application
ARG RALPH_REPO=https://github.com/kcirtapfromspace/ralph.git
ARG RALPH_REF=main
RUN git clone --depth 1 --branch ${RALPH_REF} ${RALPH_REPO} .

# Build the release binary with cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/sccache \
    cargo build --release --bin ralph

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim AS runtime

# Re-declare build args in runtime stage (required for multi-stage builds)
ARG VERSION
ARG COMMIT_SHA

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false ralph

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/ralph /usr/local/bin/ralph

# Copy quality configuration (needed for quality checks)
COPY --from=builder /app/quality ./quality

# Set ownership
RUN chown -R ralph:ralph /app

# Switch to non-root user
USER ralph

# Health check verifies the ralph binary is functional
# For stdio-based MCP servers, we check that the binary responds to --help
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["ralph", "--help"]

# MCP server uses stdio for communication
# The entrypoint runs the MCP server mode
ENTRYPOINT ["ralph", "mcp-server"]

# Default: no PRD preloaded (can be overridden with --prd flag)
CMD []

# Labels for Docker MCP toolkit compatibility and version tracking
LABEL org.opencontainers.image.title="Ralph MCP Server"
LABEL org.opencontainers.image.description="Enterprise-ready autonomous AI agent framework with MCP server"
LABEL org.opencontainers.image.vendor="kcirtapfromspace"
LABEL org.opencontainers.image.source="https://github.com/kcirtapfromspace/ralph"
LABEL org.opencontainers.image.version="${VERSION}"
LABEL org.opencontainers.image.revision="${COMMIT_SHA}"
LABEL mcp.server="true"
LABEL mcp.transport="stdio"
