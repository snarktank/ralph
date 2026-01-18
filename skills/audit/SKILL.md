---
name: audit
description: "Run an interactive codebase audit to analyze project structure, detect issues, and generate improvement PRDs. Use when you want to understand a codebase, find technical debt, or plan improvements. Triggers on: audit this codebase, analyze this project, find technical debt, what needs improvement."
---

# Codebase Audit

Run an interactive audit of a codebase to understand its structure, detect issues, identify opportunities, and optionally generate a PRD for improvements.

---

## The Job

1. Analyze the codebase structure and patterns
2. Ask 4 clarifying questions (with lettered options)
3. Present findings organized by severity
4. Offer to generate a PRD from findings
5. Offer to convert the PRD to prd.json for Ralph execution

**Important:** This is an interactive skill. Wait for user responses before proceeding to the next step.

---

## Step 1: Codebase Analysis

Before asking questions, gather information about the codebase:

### File Inventory
- Count files by extension
- Identify key files (README, Cargo.toml, package.json, etc.)
- Calculate total lines of code
- Detect project root markers

### Language Detection
- Detect primary language from manifest files and extensions
- Identify secondary languages
- Calculate language percentages

### Dependency Analysis
- Parse package manifests (Cargo.toml, package.json, etc.)
- List direct dependencies
- Identify dev dependencies

### Pattern Detection
- Detect naming conventions (snake_case, camelCase, PascalCase)
- Identify module organization patterns
- Detect error handling patterns
- Identify async patterns if present

### Architecture Analysis
- Detect architecture patterns (layered, modular, hexagonal, etc.)
- Identify module boundaries
- Detect coupling between modules

### API Inventory
- Detect HTTP endpoints
- Detect CLI commands
- Detect MCP tools if present

### Test Coverage
- Count test files and functions
- Identify untested modules
- Detect test patterns (unit, integration, e2e)

### Documentation Gaps
- Check README completeness
- Detect missing doc comments on public items
- Identify undocumented APIs

---

## Step 2: Interactive Q&A

Ask the user 4 questions to understand their goals and priorities. This helps prioritize findings appropriately.

### Format Questions Like This:

```
I've completed the initial analysis. Let me ask a few questions to better prioritize the findings:

1. What is the primary purpose of this project?
   A. Internal tool for our team/organization
   B. Customer-facing product or service
   C. Open source library or framework
   D. Prototype or proof of concept

2. What is your main priority for this codebase?
   A. Speed of development (move fast)
   B. Code quality and maintainability
   C. Security and compliance
   D. Performance and scalability

3. Who are the primary users of this software?
   A. Developers or technical users
   B. Non-technical end users
   C. Enterprise customers
   D. Mixed audience (all of the above)

4. What is the current stage of this project?
   A. New project, just getting started
   B. Active development with regular releases
   C. Maintenance mode (bug fixes only)
   D. Legacy system (needs modernization)

You can answer all at once (e.g., "1A 2B 3C 4A") or one by one.
```

### Parse User Responses

Accept various formats:
- "1A 2B 3C 4A" (structured)
- "1A, 2B, 3C, 4A" (with commas)
- "ABCA" (just letters, assumes sequential)
- "A B C A" (spaced letters)

### How Answers Affect Findings

**By Priority:**
- Security priority: Elevate security-related findings
- Quality priority: Elevate code quality and tech debt findings
- Performance priority: Elevate performance-related findings
- Speed priority: De-emphasize minor findings

**By Purpose:**
- Customer-facing/Open source: Elevate documentation findings
- Prototype: Lower severity for non-critical issues
- Internal tool: Documentation less critical

**By Stage:**
- Legacy: Prioritize modernization issues
- Maintenance: Prioritize stability concerns
- New/Active: Standard prioritization

---

## Step 3: Present Findings

After Q&A, present findings organized by severity and category.

### Format Findings Like This:

```
## Audit Findings

Based on your answers, here are the prioritized findings:

### Critical Issues (0)
[None found - or list critical issues]

### High Priority (2)

**ARCH-001: Missing abstraction layer** [High]
- Category: Architecture
- Affected: src/api.rs, src/db.rs
- Description: No service layer between API and database
- Recommendation: Add a service layer to separate concerns

**SEC-001: Hardcoded credentials** [High]
- Category: Security
- Affected: src/config.rs
- Description: API key hardcoded in source
- Recommendation: Use environment variables for secrets

### Medium Priority (3)
[List medium findings...]

### Low Priority (5)
[List low findings...]

---

## Feature Opportunities

Based on the analysis, here are opportunities for improvement:

**FEAT-001: Add health check endpoint** [Low Complexity]
- Rationale: API exists but lacks health check for monitoring
- Suggested: Add GET /health endpoint returning service status

**FEAT-002: Add structured logging** [Medium Complexity]
- Rationale: Currently using println! for output
- Suggested: Implement tracing with log levels
```

---

## Step 4: Offer PRD Generation

After presenting findings, ask if the user wants to generate a PRD.

```
---

Would you like me to generate a PRD from these findings?

This will:
1. Convert high/medium findings to user stories
2. Convert opportunities to user stories
3. Create a structured PRD in tasks/prd-[project]-improvements.md

Generate PRD? [Y/n]:
```

### If User Confirms

Generate PRD following the `/prd` skill format:

1. **Introduction**: Summary of audit and what the PRD addresses
2. **Goals**: Based on findings and opportunities
3. **User Stories**:
   - Findings become "Address: [finding title]" stories
   - Opportunities become "Implement: [opportunity title]" stories
   - Include severity/complexity badges
   - All stories have verifiable acceptance criteria
   - Include "Typecheck passes" as criterion
4. **Functional Requirements**: Numbered list from stories
5. **Non-Goals**: Changes outside identified scope
6. **Technical Considerations**: Key file types and patterns found
7. **Success Metrics**: Based on findings addressed

Save to: `tasks/prd-[project-name]-improvements.md`

---

## Step 5: Offer prd.json Conversion

After generating the PRD, ask if the user wants to convert it for Ralph.

```
PRD generated: tasks/prd-myproject-improvements.md

Would you like to convert this to prd.json for Ralph execution?

This will:
1. Parse the user stories from the PRD
2. Generate prd.json with all stories set to passes: false
3. Create branch name: ralph/[project]-improvements

Convert to prd.json? [Y/n]:
```

### If User Confirms

Convert following the `/ralph` skill format:

```json
{
  "project": "[Project Name]",
  "branchName": "ralph/[project-name]-improvements",
  "description": "[Project] Improvements - Address audit findings and implement opportunities",
  "userStories": [
    {
      "id": "US-001",
      "title": "[Story title]",
      "description": "As a [user], I want [feature] so that [benefit]",
      "acceptanceCriteria": [
        "Criterion 1",
        "Criterion 2",
        "Typecheck passes"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}
```

Save to: `prd.json` in working directory

---

## Summary Output

After completion, provide a summary:

```
## Audit Complete

**Analyzed:** /path/to/project
**Findings:** 10 total (0 critical, 2 high, 3 medium, 5 low)
**Opportunities:** 3 identified

**Generated:**
- PRD: tasks/prd-myproject-improvements.md (8 user stories)
- prd.json: Ready for Ralph execution

**Next Steps:**
1. Review the generated PRD
2. Run `ralph` to execute the improvements
3. Or run individual stories manually
```

---

## CLI Equivalent

This skill mirrors the behavior of `ralph audit` CLI command:

```bash
# Full interactive audit
ralph audit

# With specific directory
ralph audit -d /path/to/project

# Skip Q&A and auto-generate PRD
ralph audit --no-interactive --generate-prd

# Smart mode: only ask questions when confidence is low
ralph audit --smart
```

---

## Checklist

Before completing the audit:

- [ ] Scanned file inventory
- [ ] Detected languages and dependencies
- [ ] Analyzed code patterns and architecture
- [ ] Asked 4 clarifying questions
- [ ] Refined findings based on user priorities
- [ ] Presented findings by severity
- [ ] Presented feature opportunities
- [ ] Offered PRD generation (if findings exist)
- [ ] Offered prd.json conversion (if PRD generated)
- [ ] Provided summary with next steps
