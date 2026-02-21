FROM node:20-slim

# System deps for networking, git, and general tooling
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    iptables \
    ipset \
    iproute2 \
    dnsutils \
    jq \
    curl \
    sudo \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install claude-code globally
RUN npm install -g @anthropic-ai/claude-code

# Create non-root agent user (UID 1001 since node:20-slim uses 1000 for 'node')
RUN useradd -m -s /bin/bash -u 1001 agent

# Allow agent to run firewall init scripts via sudo (no password)
RUN echo "agent ALL=(root) NOPASSWD:SETENV: /opt/ralph/init-firewall-builder.sh, /opt/ralph/init-firewall-researcher.sh" \
    > /etc/sudoers.d/agent-firewall && chmod 0440 /etc/sudoers.d/agent-firewall

# Copy scripts
COPY agent-loop.sh /opt/ralph/agent-loop.sh
COPY init-firewall-builder.sh /opt/ralph/init-firewall-builder.sh
COPY init-firewall-researcher.sh /opt/ralph/init-firewall-researcher.sh
RUN chmod +x /opt/ralph/agent-loop.sh /opt/ralph/init-firewall-builder.sh /opt/ralph/init-firewall-researcher.sh

# Workspace for cloned repo
RUN mkdir -p /workspace && chown agent:agent /workspace

# Claude config directory
RUN mkdir -p /home/agent/.claude && chown agent:agent /home/agent/.claude

USER agent
WORKDIR /workspace

ENTRYPOINT ["/opt/ralph/agent-loop.sh"]
