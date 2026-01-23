---
name: mir-to-prd
description: "Convert a Master Intelligence Report (MIR) into actionable PRDs. Extracts technical builds for software PRDs and business tasks for Business PRDs. Triggers on: convert mir, mir to prd, extract tasks from mir, turn mir into prd."
---

# MIR to PRD Converter

Converts Master Intelligence Reports (MIRs) from the FORGE-MIR framework into actionable PRDs for execution.

---

## What is an MIR?

A **Master Intelligence Report** is a consolidated strategic document created by the FORGE-MIR framework. It typically contains:

- Executive summary with key decisions
- Market positioning and competitive analysis
- Strategic recommendations
- Action plans and timelines
- Revenue projections and metrics
- Technical stack recommendations

MIRs are research outputs, not execution documents. This skill converts them into actionable work.

---

## The Job

1. Read the MIR document
2. Identify and categorize extractable items:
   - **Technical builds** → Standard software PRDs
   - **Business/marketing tasks** → Business PRDs
3. Generate appropriate PRD type for each category
4. Optionally export to Asana format
5. Provide summary of what was extracted

**Output Files:**
- `tasks/prd-[technical-feature].md` (for each technical build)
- `tasks/business-prd-[initiative].md` (for business tasks)
- `tasks/asana-import-[name].json` (optional)

---

## Step 1: MIR Analysis

When given an MIR, scan for these patterns:

### Technical Builds (→ Software PRD)

Look for:
- Automation workflows (n8n, Make.com, Zapier)
- Code to write (scripts, integrations, APIs)
- Tools to configure (CRM setup, platform integrations)
- Landing pages or web assets to build

**Indicators:**
- "Build a workflow that..."
- "Create an automation for..."
- "Set up [tool] to..."
- "n8n/Make.com/Zapier"
- "API integration"
- Technical tool names

### Business Tasks (→ Business PRD)

Look for:
- Marketing campaigns
- Content creation plans
- Community building activities
- Launch sequences
- Outreach strategies
- Pricing/packaging decisions

**Indicators:**
- "Launch on [date]"
- "Create content for..."
- "Post on [platform]"
- "Reach out to..."
- "Set up pricing at..."
- Action verbs without technical implementation

---

## Step 2: Extraction Process

For each identified item:

### Technical Build Extraction

```
1. Identify the build (e.g., "LinkedIn viral giveaway automation")
2. Determine scope:
   - What does it need to do?
   - What tools are involved?
   - What are the inputs/outputs?
3. Break into user stories (one context window each)
4. Generate standard PRD using /prd skill format
5. Save to tasks/prd-[feature-name].md
```

### Business Task Extraction

```
1. Identify the initiative (e.g., "Pre-launch community seeding")
2. Determine scope:
   - What's the goal?
   - What actions are needed?
   - What's the timeline?
3. Break into action items
4. Generate Business PRD using /business-prd skill format
5. Save to tasks/business-prd-[name].md
```

---

## Step 3: Output Generation

### For Technical PRDs

Use the standard PRD format (see `/prd` skill):

```markdown
# PRD: [Feature Name]

## Introduction
[Extracted from MIR context]

## Goals
[Extracted from MIR recommendations]

## User Stories
### US-001: [Title]
**Description:** As a [user], I want [feature] so that [benefit].
**Acceptance Criteria:**
- [ ] Specific criterion
- [ ] Typecheck passes
- [ ] [For UI] Verify in browser using dev-browser skill

[... more stories ...]

## Functional Requirements
[Derived from MIR technical specifications]

## Non-Goals
[Explicitly out of scope]

## Technical Considerations
[From MIR tech stack section]
```

### For Business PRDs

Use the Business PRD format (see `/business-prd` skill):

```markdown
# Business PRD: [Initiative Name]

## Overview
[Extracted from MIR context]

## Goals
[Extracted from MIR recommendations]

## Success Metrics
[From MIR metrics/projections section]

## Action Items
### AI-001: [Title]
**Description:** [What and why]
**Owner:** [From MIR or ask user]
**Done Criteria:**
- [ ] Specific verifiable outcome
- [ ] Verified in [platform]
**Due:** [From MIR timeline or ask user]

[... more items ...]

## Dependencies
[From MIR prerequisites]

## Non-Goals
[Explicitly out of scope]
```

---

## Step 4: Asana Export (Optional)

Prompt user: "Would you like to export tasks to Asana format? (Yes/No)"

If yes, combine all action items from Business PRDs into one import file:

```json
{
  "project_id": "[ask user or use .asana-config.json]",
  "source_mir": "[MIR filename]",
  "generated": "[ISO timestamp]",
  "tasks": [
    {
      "name": "AI-001: [Title]",
      "notes": "[Description]\n\nDone Criteria:\n[criteria list]",
      "due_on": "YYYY-MM-DD",
      "assignee": "me"
    }
  ]
}
```

Save to: `tasks/asana-import-[mir-name].json`

---

## Extraction Examples

### Example MIR Section → Technical PRD

**MIR Content:**
```
**"Viral Giveaway" Workflow (ChatGPT):**
1. Post: "Comment 'BLUEPRINT' and I'll send it."
2. Connect automation (Waalaxy/n8n) to LinkedIn
3. When user comments "BLUEPRINT" → auto-like + connection request + DM
4. DM links to free resource gated inside Skool
```

**Extracted Technical PRD:**
```markdown
# PRD: LinkedIn Viral Giveaway Automation

## Introduction
Automate the response workflow when LinkedIn users comment on posts requesting resources.

## Goals
- Automatically detect comments containing trigger words
- Send connection request and DM to commenters
- Direct users to gated resource in Skool
- Reduce manual response time from hours to seconds

## User Stories

### US-001: Create n8n webhook for LinkedIn events
**Description:** As an agency owner, I need to receive LinkedIn comment notifications in n8n.
**Acceptance Criteria:**
- [ ] n8n HTTP webhook node created
- [ ] Webhook URL documented
- [ ] Test payload received successfully
- [ ] Typecheck passes

### US-002: Filter comments for trigger word
**Description:** As an agency owner, I want to only respond to comments containing "BLUEPRINT".
**Acceptance Criteria:**
- [ ] IF node checks comment text (case insensitive)
- [ ] Non-matching comments are ignored
- [ ] Matching comments proceed to next step
- [ ] Typecheck passes

### US-003: Send automated DM response
**Description:** As an agency owner, I want to automatically DM users who comment the trigger word.
**Acceptance Criteria:**
- [ ] LinkedIn API node sends DM
- [ ] DM contains link to Skool resource
- [ ] DM is personalized with user's name
- [ ] Rate limiting prevents spam flags
- [ ] Typecheck passes
```

### Example MIR Section → Business PRD

**MIR Content:**
```
### 5.2 Pre-Launch Phase (Jan 23 - Feb 5)

**Weeks 1-2: Infrastructure**
1. Launch **FREE** Skool community immediately (before Feb 6)
2. Set up membership questions to qualify leads
3. Create "Start Here" course explaining FORGE-MIR

**Seed the community:**
- Invite 10-15 trusted colleagues/friends for FREE
- Ask each to post 1 intro and 1 question
```

**Extracted Business PRD:**
```markdown
# Business PRD: Skool Pre-Launch Seeding

## Overview
Prepare the AI Agency Accelerator Skool community for paid launch by seeding with founding members and creating essential onboarding content.

## Goals
- Have active community engagement before paid launch
- Create social proof with real introductions
- Establish content foundation

## Success Metrics
- 10+ seed members joined
- 5+ introduction posts
- "Start Here" module published
- 3+ discussion threads active

## Action Items

### AI-001: Invite seed members
**Description:** Send personal invitations to 10-15 trusted contacts to join the free Skool community.
**Owner:** Danny
**Done Criteria:**
- [ ] 10+ invitations sent via DM/email
- [ ] Track responses in spreadsheet
- [ ] 5+ accepted and joined
- [ ] Verified in Skool member list
**Due:** Jan 28, 2026

### AI-002: Prompt seed member introductions
**Description:** Ask each seed member to post an introduction and one question.
**Owner:** Danny
**Done Criteria:**
- [ ] Personal message sent to each seed member
- [ ] Introduction template/prompt provided
- [ ] 5+ introductions posted
- [ ] At least 3 questions asked
- [ ] Verified in Skool community feed
**Due:** Feb 3, 2026

### AI-003: Create "Start Here" onboarding
**Description:** Build a simple onboarding course in Skool Classroom.
**Owner:** Danny
**Done Criteria:**
- [ ] Course module created in Classroom
- [ ] Includes welcome, guidelines, how-to sections
- [ ] Published and visible to members
- [ ] Tested by logging in as member
**Due:** Feb 3, 2026
```

---

## Workflow Summary

```
┌─────────────────┐
│   Read MIR      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Scan for items  │
│ - Technical     │
│ - Business      │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐ ┌────────┐
│Tech PRD│ │Biz PRD │
│(/prd)  │ │(/biz)  │
└────┬───┘ └────┬───┘
     │         │
     └────┬────┘
          │
          ▼
┌─────────────────┐
│ Asana Export?   │
│ (optional)      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Summary Report  │
└─────────────────┘
```

---

## Final Output Summary

After processing, provide a summary:

```markdown
## MIR Conversion Summary

**Source:** [MIR filename]
**Processed:** [timestamp]

### Technical PRDs Generated (X)
- `tasks/prd-[name-1].md` - [Brief description]
- `tasks/prd-[name-2].md` - [Brief description]

### Business PRDs Generated (X)
- `tasks/business-prd-[name-1].md` - [Brief description]

### Asana Export
- `tasks/asana-import-[name].json` - X tasks ready for import

### Items Skipped (X)
- [Item] - Reason: [too vague / already done / out of scope]

### Manual Follow-up Required
- [Item needing clarification]
```

---

## Checklist

Before completing MIR conversion:

- [ ] MIR fully read and analyzed
- [ ] Technical builds identified and extracted
- [ ] Business tasks identified and extracted
- [ ] Each PRD saved to correct location
- [ ] User stories/action items are appropriately sized
- [ ] Due dates aligned with MIR timeline
- [ ] Asana export generated (if requested)
- [ ] Summary provided to user
