# Ralph Docker MCP Deployment

This repository contains Docker deployment configuration for the [Ralph MCP Server](https://github.com/kcirtapfromspace/ralph).

## Quick Start

```bash
# Pull the Ralph Docker image
docker pull ghcr.io/kcirtapfromspace/ralph:latest

# Run Ralph MCP server
docker run -i --rm ghcr.io/kcirtapfromspace/ralph:latest
```

## Documentation

- [Docker MCP Setup Guide](docs/guides/docker-mcp-setup.md) - Complete setup instructions for Docker deployment
- [Examples](examples/README.md) - Configuration examples for Claude Desktop

## Files

| File | Purpose |
|------|---------|
| `Dockerfile` | Multi-stage build for Ralph MCP server |
| `docker-compose.yml` | Local development and testing |
| `.dockerignore` | Build context optimization |
| `examples/` | MCP toolkit configuration examples |
| `docs/guides/` | Setup and usage documentation |

## Building Locally

```bash
# Build the image
docker build -t ralph-mcp .

# Run with docker-compose
docker compose up --build
```

## CI/CD

The `.github/workflows/docker.yml` workflow automatically builds and publishes images to GitHub Container Registry on:
- Push to `main` branch
- Version tags (`v*`)

Images are available at `ghcr.io/kcirtapfromspace/ralph`.
