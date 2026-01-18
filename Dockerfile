# syntax=docker/dockerfile:1

# Ralph MCP Server - Multi-stage build for minimal image size
# This Dockerfile builds the Ralph MCP server for use with Docker MCP toolkit
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

# Stage 1: Build the Rust binary
# Using Rust 1.85+ for Edition 2024 support (required by rmcp crate)
FROM rust:1.85-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Clone the Ralph source repository
# This Dockerfile is for deployment - source lives in the main ralph repo
ARG RALPH_REPO=https://github.com/kcirtapfromspace/ralph.git
ARG RALPH_REF=main
RUN git clone --depth 1 --branch ${RALPH_REF} ${RALPH_REPO} .

# Build the release binary
RUN cargo build --release --bin ralph

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
