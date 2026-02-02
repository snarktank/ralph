# Structured Workflow for Ralph

This directory contains optional examples and templates that help structure work more effectively in Ralph, while preserving Ralph's minimal philosophy.

**Everything here is optional and additive.** You can use Ralph perfectly fine without any of this. These examples exist to help you think about how to break work into tasks that fit naturally into Ralph's one-agent, one-task, one-iteration loop.

## Philosophy

Ralph executes tasks. It doesn't orchestrate workflows. The structure comes from how you define tasks, not from complex logic in the execution loop.

When tasks are well-structured:
- Each iteration has clear focus
- Dependencies are obvious from task order
- The agent knows exactly what "done" means
- Progress is easy to track

When tasks are poorly structured:
- The agent gets confused about scope
- Context runs out before completion
- Dependencies cause failures
- Progress is unclear

## The Approach

Break work into phases that naturally flow from one to the next:

1. **Foundation** - Schema, data structures, core types
2. **Logic** - Business rules, server actions, API endpoints
3. **Interface** - UI components, forms, displays
4. **Integration** - Connecting pieces, end-to-end flows
5. **Polish** - Edge cases, error handling, refinement

Each phase contains multiple small tasks. Tasks within a phase can often be done in parallel (Ralph picks by priority), but phases should generally be ordered.

## Files in This Directory

- **`prd.json.example`** - Example PRD with optional metadata fields (`stage`, `focus`, `responsibility`) that help organize tasks without changing how Ralph runs
- **`prompt-templates/`** - Optional prompt snippets you can reference when customizing `prompt.md` for different task types
- **`example-feature/`** - A complete end-to-end example showing how structured tasks improve clarity

## Using These Examples

1. **Read the example PRD** to see how metadata can help organize tasks
2. **Review the prompt templates** if you want to customize behavior by task type
3. **Study the example feature** to see the pattern in practice

Then adapt these ideas to your own work. The metadata fields are optional - Ralph will work with or without them. The prompt templates are suggestions - use them, modify them, or ignore them.

## Key Principle

**Ralph remains a task executor.** These examples show how to structure tasks better, not how to change Ralph's execution model. If you find yourself wanting to modify `ralph.sh` or add orchestration logic, step back and reconsider your task structure instead.

