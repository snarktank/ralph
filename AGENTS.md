# Ralph Agent Instructions

## Overview

Ralph is an autonomous AI agent loop that runs Claude Code repeatedly until all PRD items are complete. Each iteration is a fresh Claude Code instance with clean context.

## Commands

```bash
# Install ralph globally
./install.sh

# Initialize a project
ralph --init

# Run Ralph in current directory
ralph

# Run with max 20 iterations
ralph 20

# Run in a specific directory
ralph -d /path/to/project

# Show help
ralph --help

# Run the flowchart dev server
cd flowchart && npm run dev
```

## Key Files

- `bin/ralph` - Global CLI binary
- `install.sh` - Installer script
- `prompt.md` - Instructions given to each Claude Code instance
- `prd.json.example` - Example PRD format
- `flowchart/` - Interactive React Flow diagram explaining how Ralph works

## Flowchart

The `flowchart/` directory contains an interactive visualization built with React Flow. It's designed for presentations - click through to reveal each step with animations.

To run locally:
```bash
cd flowchart
npm install
npm run dev
```

## Patterns

- Each iteration spawns a fresh Claude Code instance with clean context
- Memory persists via git history, `progress.txt`, and `prd.json`
- Stories should be small enough to complete in one context window
- Always update AGENTS.md with discovered patterns for future iterations

---

## Terminal UX Enhancement Branch

This section documents the `ralph/terminal-ux-enhancement` branch for creating a best-in-class terminal UX.

### Directory Structure

```
<project-root>/                    # Rust source code
├── src/
│   ├── main.rs                    # CLI entry point
│   ├── mcp/                       # MCP server implementation
│   │   ├── server.rs              # RalphMcpServer struct
│   │   ├── tools/                 # MCP tools
│   │   └── resources/             # MCP resources
│   ├── quality/                   # Quality framework
│   ├── integrations/              # External integrations
│   └── ui/                        # Terminal UI
│       ├── colors.rs
│       ├── display.rs
│       ├── spinner.rs
│       ├── story_view.rs
│       ├── quality_gates.rs
│       ├── summary.rs
│       ├── interrupt.rs
│       ├── ghostty.rs
│       └── help.rs
├── Cargo.toml
├── prd.json                       # User stories
├── progress.txt                   # Implementation log
├── AGENTS.md                      # This file
└── tests/
```

### Coding Standards

#### Rust Conventions
- Run `cargo fmt` after every file change
- Run `cargo clippy -- -D warnings` before committing
- Run `cargo check` for quick type checking
- Use `#![allow(dead_code)]` for scaffolding code

#### Module Organization
- Module files at `src/{module}/mod.rs`
- Export public types in mod.rs
- One struct/enum per file for major types

### Terminal UI Implementation Guidelines

#### Crates
- `indicatif` - Progress bars, spinners
- `console` - Terminal detection
- `owo-colors` - 24-bit RGB colors
- `crossterm` - Advanced terminal control

#### Color Scheme (24-bit RGB)
```rust
success:     (34, 197, 94)   // Green
error:       (239, 68, 68)   // Red
warning:     (234, 179, 8)   // Yellow
in_progress: (59, 130, 246)  // Blue
muted:       (107, 114, 128) // Gray
story_id:    (34, 211, 238)  // Cyan
```

#### Box Drawing Characters
- Rounded corners: `╭ ╮ ╰ ╯`
- Lines: `─ │`
- Status Icons: `✓` passed, `✗` failed, `○` pending

#### Ghostty-Specific Features
```rust
// OSC 8 Hyperlink
"\x1b]8;;{url}\x07{text}\x1b]8;;\x07"

// Title Update
"\x1b]0;{title}\x07"

// Synchronized Output
"\x1b[?2026h"  // Begin
"\x1b[?2026l"  // End
```

### Commit Message Format

```
feat: [US-XXX] - Story title

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
```
