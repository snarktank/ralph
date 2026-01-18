# syntax=docker/dockerfile:1

# Ralph MCP Server - Optimized build with cargo-chef dependency caching
#
# First build: ~15-20 min (compiles deps + installs cargo-chef)
# Subsequent builds: ~2-5 min (deps cached, only app code compiles)
#
# Build Arguments:
#   RALPH_REPO  - Git repository URL (default: https://github.com/kcirtapfromspace/ralph.git)
#   RALPH_REF   - Git branch/tag to build from (default: main)
#   VERSION     - Semantic version for the build
#   COMMIT_SHA  - Git commit SHA for traceability

ARG VERSION=dev
ARG COMMIT_SHA=unknown

# Stage 1: Chef base with ALL build dependencies
FROM rust:1.85-slim-bookworm AS chef
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef
WORKDIR /app

# Stage 2: Generate dependency recipe
FROM chef AS planner
ARG RALPH_REPO=https://github.com/kcirtapfromspace/ralph.git
ARG RALPH_REF=main
RUN git clone --depth 1 --branch ${RALPH_REF} ${RALPH_REPO} .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (this layer is cached)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Clone and build application
ARG RALPH_REPO=https://github.com/kcirtapfromspace/ralph.git
ARG RALPH_REF=main
RUN git clone --depth 1 --branch ${RALPH_REF} ${RALPH_REPO} .
RUN cargo build --release --bin ralph

# Stage 4: Runtime image
FROM debian:bookworm-slim AS runtime
ARG VERSION
ARG COMMIT_SHA

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false ralph

WORKDIR /app
COPY --from=builder /app/target/release/ralph /usr/local/bin/ralph
COPY --from=builder /app/quality ./quality
RUN chown -R ralph:ralph /app

USER ralph
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["ralph", "--help"]
ENTRYPOINT ["ralph", "mcp-server"]
CMD []

LABEL org.opencontainers.image.title="Ralph MCP Server"
LABEL org.opencontainers.image.description="Enterprise-ready autonomous AI agent framework with MCP server"
LABEL org.opencontainers.image.vendor="kcirtapfromspace"
LABEL org.opencontainers.image.source="https://github.com/kcirtapfromspace/ralph"
LABEL org.opencontainers.image.version="${VERSION}"
LABEL org.opencontainers.image.revision="${COMMIT_SHA}"
LABEL mcp.server="true"
LABEL mcp.transport="stdio"
