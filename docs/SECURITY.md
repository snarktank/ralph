# Security Guide

Ralph Wiggum runs as an autonomous agent with significant system access. This document outlines security best practices to prevent credential exposure and unauthorized actions.

## Mandatory Safeguards

Before running Ralph in any environment, ensure these safeguards are in place:

1. **Never expose production credentials** - Ralph should not have access to production databases, cloud accounts, or API keys
2. **Use isolated environments** - Run Ralph in sandboxed containers, VMs, or development environments only
3. **Limit file system access** - Restrict Ralph to the project directory when possible
4. **Review generated code** - Always review commits before merging to protected branches
5. **Monitor token usage** - Set budget limits to prevent runaway API costs

## Pre-Flight Security Checklist

Run through this checklist before starting any Ralph session:

- [ ] **Environment Variables Cleared** - Ensure dangerous environment variables are not set:
  - `AWS_ACCESS_KEY_ID` - AWS credentials could allow cloud resource access
  - `AWS_SECRET_ACCESS_KEY` - AWS credentials could allow cloud resource access
  - `DATABASE_URL` - Database connection strings could expose production data
  - `OPENAI_API_KEY` - Could incur costs on your account
  - `ANTHROPIC_API_KEY` - Could incur costs on your account
  - `GITHUB_TOKEN` - Could push to repositories or access private repos
  - `NPM_TOKEN` - Could publish packages
  - `DOCKER_PASSWORD` - Could push images

- [ ] **Running in Sandbox** - Confirm you're in a sandboxed environment
- [ ] **Git Remote Verified** - Ensure pushes go to correct repository
- [ ] **Branch Protection** - Confirm main/master has branch protection enabled
- [ ] **Budget Set** - API cost limits configured

## Emergency Stop

If Ralph begins behaving unexpectedly, use these methods to stop execution:

### Immediate Stop
```bash
# Kill the ralph.sh process
pkill -f ralph.sh

# Or find and kill specifically
ps aux | grep ralph.sh
kill -9 <PID>
```

### Graceful Stop
```bash
# Create a stop file (if ralph.sh is configured to check for it)
touch .ralph-stop

# Or simply Ctrl+C in the terminal running ralph.sh
```

### Post-Emergency Checklist
1. Review git log for any unexpected commits
2. Check git diff for uncommitted changes
3. Review any files created or modified
4. Check cloud console for any unexpected resources
5. Rotate any credentials that may have been exposed

## Docker Sandboxing

Running Ralph in Docker provides isolation from your host system:

```dockerfile
# Dockerfile.ralph
FROM node:20-slim

# Install required tools
RUN apt-get update && apt-get install -y \
    git \
    jq \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash ralph
USER ralph
WORKDIR /home/ralph/workspace

# Copy only necessary files
COPY --chown=ralph:ralph . .

# Don't include any credentials in the image
# Pass API keys at runtime only
```

```bash
# Build the sandbox
docker build -f Dockerfile.ralph -t ralph-sandbox .

# Run with minimal permissions
docker run -it --rm \
    --network=none \
    --read-only \
    --tmpfs /tmp \
    -v $(pwd):/home/ralph/workspace \
    -e ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY \
    ralph-sandbox \
    ./ralph.sh
```

### Docker Security Options
- `--network=none` - Prevents network access (remove if Ralph needs to fetch dependencies)
- `--read-only` - Makes container filesystem read-only
- `--tmpfs /tmp` - Provides writable temp directory
- Mount only the project directory, not your entire home folder

## Additional Recommendations

1. **Use separate API keys** - Create dedicated API keys for Ralph with lower rate limits
2. **Enable audit logging** - Log all commands Ralph executes for review
3. **Set up alerts** - Configure cost alerts in your cloud provider dashboards
4. **Regular credential rotation** - Rotate any credentials that have been in the environment
5. **Review before merge** - Never auto-merge Ralph's PRs without human review
