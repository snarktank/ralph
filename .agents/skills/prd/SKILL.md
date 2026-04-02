---
name: prd
description: "Codex-native PRD generator for Ralph. Use when planning a feature, starting a new project, or when asked to create a PRD. Invoke with $prd."
user-invocable: true
---

# PRD Generator

Use this skill in Codex by typing `$prd`.

Create a detailed Product Requirements Document that is clear, actionable, and suitable for later conversion into Ralph's `prd.json` format.

## The Job

1. Receive a feature description from the user.
2. Ask 3-5 essential clarifying questions only when the scope is ambiguous.
3. Generate a structured PRD in markdown.
4. Save it to `tasks/prd-[feature-name].md`.

Do not implement the feature. This skill only produces the PRD.

## Clarifying Questions

Ask only the highest-value questions. Prefer lettered options so the user can answer quickly.

Focus on:

- primary problem or goal
- target user
- core functionality
- scope boundaries
- success criteria

Use a compact format like:

```text
1. What is the primary goal?
   A. Improve onboarding
   B. Increase retention
   C. Reduce support load
   D. Other
```

## PRD Structure

Generate these sections:

1. Introduction / Overview
2. Goals
3. User Stories
4. Functional Requirements
5. Non-Goals
6. Design Considerations (optional)
7. Technical Considerations (optional)
8. Success Metrics
9. Open Questions

## User Story Requirements

Each story must include:

- a short title
- a description in the form `As a [user], I want [feature] so that [benefit]`
- verifiable acceptance criteria

Stories should be small enough to implement in one focused Ralph iteration.

Use this format:

```markdown
### US-001: [Title]
**Description:** As a [user], I want [feature] so that [benefit].

**Acceptance Criteria:**
- [ ] Specific, verifiable criterion
- [ ] Another criterion
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill
```

Only include the browser verification line for UI stories.

## Writing Rules

- Be explicit and unambiguous.
- Prefer concrete behavior over vague goals.
- Number requirements for easy reference.
- Keep scope realistic for downstream implementation.
- Use examples when they remove ambiguity.

## Output

- Format: markdown
- Location: `tasks/`
- Filename: `prd-[feature-name].md`

After saving the file, tell the user the path and recommend using `$ralph` next to convert it into `./prd.json`.
