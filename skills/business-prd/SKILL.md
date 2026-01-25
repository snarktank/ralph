---
name: business-prd
description: "Generate a Business PRD for marketing, launch, or operational initiatives. Use for non-software tasks that require human execution. Triggers on: create business prd, marketing plan, launch plan, business requirements."
---

# Business PRD Generator

Create structured Business Product Requirements Documents for marketing campaigns, community launches, operational initiatives, and other non-software projects.

---

## The Job

1. Receive an initiative description from the user (or extract from an MIR)
2. Ask 3-5 clarifying questions (with lettered options)
3. Generate a structured Business PRD
4. Save to `tasks/business-prd-[name].md`
5. Optionally export to Asana-compatible JSON

**Important:** This is for planning and task breakdown. Execution is manual or via other tools.

---

## Step 1: Clarifying Questions

Ask only critical questions where the initial prompt is ambiguous. Focus on:

- **Goal/Outcome:** What does success look like?
- **Timeline:** When must this be complete?
- **Resources:** Who is responsible? What tools are needed?
- **Scope/Boundaries:** What is explicitly out of scope?

### Format Questions Like This:

```
1. What is the primary goal of this initiative?
   A. Launch a new product/service
   B. Grow audience/community
   C. Generate leads/sales
   D. Improve operations/processes
   E. Other: [please specify]

2. What is the timeline?
   A. This week (urgent)
   B. Within 2 weeks
   C. Within 1 month
   D. Within 3 months
   E. Ongoing/no fixed deadline

3. Who is the primary owner?
   A. Me (solo execution)
   B. My team
   C. External contractor/agency
   D. Shared responsibility
```

This lets users respond with "1B, 2C, 3A" for quick iteration.

---

## Step 2: Business PRD Structure

Generate the Business PRD with these sections:

### 1. Overview
Brief description of the initiative and the business problem it solves.

### 2. Goals
Specific, measurable objectives (bullet list).
- Use SMART format where applicable
- Include target metrics

### 3. Success Metrics
How will success be measured? Include:
- Quantitative metrics (numbers, percentages)
- Qualitative indicators (feedback, engagement)
- Timeline for measurement

### 4. Action Items
Each action item needs:
- **ID:** Sequential (AI-001, AI-002, etc.)
- **Title:** Short descriptive name
- **Description:** What needs to be done and why
- **Owner:** Person or role responsible
- **Done Criteria:** Verifiable checklist of what "done" means
- **Due:** Date or milestone

**Format:**
```markdown
### AI-001: [Title]
**Description:** [What to do and why it matters]

**Owner:** [Person/Role]

**Done Criteria:**
- [ ] Specific verifiable outcome
- [ ] Deliverable exists at [location]
- [ ] Metric: [X achieved]
- [ ] Verified in [platform/tool]

**Due:** [Date or milestone]
```

**Important:**
- Done criteria must be verifiable, not vague
- "Done well" is bad. "10 posts published" is good
- Include where to verify (Skool, LinkedIn, Asana, etc.)

### 5. Dependencies
What must happen first? List prerequisites and blockers.

### 6. Resources Required
- Tools/platforms needed
- Budget (if applicable)
- Time commitment estimates
- External help needed

### 7. Non-Goals (Out of Scope)
What this initiative will NOT include. Critical for managing scope creep.

### 8. Risks & Mitigations
Potential problems and how to address them.

### 9. Open Questions
Remaining questions or areas needing clarification.

---

## Step 3: Asana Export (Optional)

If the user requests Asana export, also generate a JSON file:

**Prompt:** "Would you like me to generate an Asana-compatible task list? (Yes/No)"

If yes, create `tasks/asana-import-[name].json`:

```json
{
  "project_id": "[from user or .asana-config.json]",
  "tasks": [
    {
      "name": "AI-001: [Title]",
      "notes": "[Description]\n\nDone Criteria:\n- [ ] Criterion 1\n- [ ] Criterion 2",
      "due_on": "YYYY-MM-DD",
      "assignee": "me"
    }
  ]
}
```

**Asana Field Mapping:**
| Business PRD | Asana Task |
|--------------|------------|
| Title | name (prefixed with ID) |
| Description + Done Criteria | notes |
| Due | due_on (YYYY-MM-DD format) |
| Owner | assignee ("me" or user ID) |

**Import Instructions:**
```
To import tasks to Asana, use the MCP Asana tools:
- mcp__asana__asana_create_task for each task
- Or batch import via Asana API/CSV
```

---

## Writing for Clear Execution

The PRD reader needs to execute without ambiguity. Therefore:

- Be explicit about what "done" looks like
- Include specific platforms/tools to use
- Provide links or references where helpful
- Number action items for easy tracking
- Include realistic due dates based on complexity

---

## Output

- **Primary:** `tasks/business-prd-[name].md` (kebab-case)
- **Optional:** `tasks/asana-import-[name].json`

---

## Example Business PRD

```markdown
# Business PRD: Skool Community Pre-Launch

## Overview

Prepare the AI Agency Accelerator Skool community for paid launch on February 6, 2026. Focus on seeding initial members, creating foundational content, and building social proof before opening paid subscriptions.

## Goals

- Have 10+ active seed members before paid launch
- Create "Start Here" onboarding content
- Establish weekly content rhythm
- Build initial social proof with introductions and engagement

## Success Metrics

- 10+ seed members joined and introduced themselves
- 3+ community posts with engagement (likes/comments)
- "Start Here" course module completed
- Weekly content calendar established
- 50+ LinkedIn connections from target audience

## Action Items

### AI-001: Seed community with founding contacts
**Description:** Invite 10-15 trusted contacts to join the free Skool community before public launch. These early members provide social proof and initial engagement.

**Owner:** Danny

**Done Criteria:**
- [ ] 10+ invitations sent via personal DM
- [ ] 5+ accepted and posted introduction
- [ ] At least 3 different posts/comments from seed members
- [ ] Verified in Skool member list

**Due:** Feb 5, 2026

### AI-002: Create "Start Here" onboarding module
**Description:** Build a simple onboarding course in Skool Classroom that orients new members on how to get value from the community.

**Owner:** Danny

**Done Criteria:**
- [ ] Course created in Skool Classroom
- [ ] Includes: Welcome video/post, community guidelines, how to ask questions
- [ ] Published and visible to members
- [ ] Verified by visiting as logged-in member

**Due:** Feb 3, 2026

### AI-003: Execute LinkedIn viral giveaway strategy
**Description:** Post content offering a free resource (e.g., "Comment BLUEPRINT to get my AI workflow guide") to build LinkedIn audience and drive Skool sign-ups.

**Owner:** Danny

**Done Criteria:**
- [ ] LinkedIn post published with clear CTA
- [ ] Response workflow ready (manual or automated)
- [ ] 20+ comments received
- [ ] Commenters directed to Skool community
- [ ] Verified: new Skool joins from LinkedIn source

**Due:** Jan 30, 2026

### AI-004: Join 5 relevant Skool communities
**Description:** Join active Skool communities where target audience exists (AI, agency, marketing) and begin providing value through comments and posts.

**Owner:** Danny

**Done Criteria:**
- [ ] 5 communities joined
- [ ] Introduced self in each community
- [ ] Posted 3+ helpful comments/answers
- [ ] Bio links to AI Agency Accelerator
- [ ] Verified in Skool profile "Communities" section

**Due:** Jan 28, 2026

## Dependencies

- Skool community must be created and configured (DONE)
- Categories and basic structure in place (DONE)
- Profile and bio completed (DONE)

## Resources Required

- **Tools:** Skool, LinkedIn, Loom (for screen recordings)
- **Time:** ~2 hours/day for community engagement
- **Budget:** None required for pre-launch

## Non-Goals

- Paid advertising (not until after launch)
- Complex automation (manual responses OK for now)
- Full course library (just "Start Here" for launch)
- Perfect content (MVP quality acceptable)

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Seed members don't engage | Personally message each, ask specific questions |
| LinkedIn post gets low reach | Test different formats, post at optimal times |
| Time constraints | Focus on highest-impact items first |

## Open Questions

- Should we offer founding member discount to seed members?
- What's the minimum viable "Start Here" content?
```

---

## Checklist

Before saving the Business PRD:

- [ ] Asked clarifying questions with lettered options
- [ ] Incorporated user's answers
- [ ] Action items are specific and actionable
- [ ] Done criteria are verifiable (not vague)
- [ ] Due dates are realistic
- [ ] Non-goals section defines clear boundaries
- [ ] Saved to `tasks/business-prd-[name].md`
- [ ] (Optional) Asana export generated if requested
