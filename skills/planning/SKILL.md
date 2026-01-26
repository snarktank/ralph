---
name: planning
description: "Deep requirements exploration before creating a PRD. Use when starting any new feature to ensure requirements are fully understood. Triggers on: plan this feature, explore requirements, planning session, before I write a prd."
---

# Planning Skill

Forces deep requirements exploration through 5 mandatory question rounds before you can create a PRD. This prevents under-specified features and wasted implementation cycles.

---

## The Job

1. Conduct 5 rounds of questions with the user
2. Document answers in a planning summary
3. Save output to `tasks/planning-[feature].md`
4. Only then can you proceed to PRD creation

**Important:** You cannot skip rounds. All 5 rounds must be completed before moving to PRD.

---

## Completion Gate

**You MUST complete all 5 rounds before this skill is considered complete.**

After each round, explicitly state:
```
Round [N] complete. [5-N] rounds remaining.
```

Do NOT proceed to PRD creation until you have stated:
```
Round 5 complete. Planning session finished.
```

If the user asks to skip rounds or rush to implementation, remind them:
> "Planning requires all 5 rounds. Skipping leads to incomplete requirements and rework. Which question should we tackle next?"

---

## Round 1: Problem Understanding

**Goal:** Understand WHAT problem we're solving and WHY it matters.

Ask questions about:
- What problem does this solve?
- Who experiences this problem?
- What happens today without this feature?
- What pain points does this address?
- Why is this important now?

### Example Questions:

```
1. What specific problem are we trying to solve?
   A. Users cannot do X at all
   B. Users can do X but it's slow/painful
   C. Users frequently make mistakes doing X
   D. Other: [please specify]

2. Who experiences this problem most acutely?
   A. New users during onboarding
   B. Power users doing advanced tasks
   C. All users equally
   D. Internal team members

3. What happens today when users encounter this problem?
   A. They work around it manually
   B. They contact support
   C. They abandon the task
   D. They use a competitor
```

After gathering answers, summarize:
```
## Round 1 Summary: Problem Understanding
- Problem: [concise problem statement]
- Affected users: [who]
- Current workaround: [what they do now]
- Impact: [why it matters]
```

**Round 1 complete. 4 rounds remaining.**

---

## Round 2: Scope Definition

**Goal:** Define the boundaries of what we WILL and WON'T build.

Ask questions about:
- What is the minimum viable solution?
- What would a full-featured version include?
- What is explicitly out of scope?
- What are the must-haves vs nice-to-haves?
- What adjacent features should we NOT touch?

### Example Questions:

```
1. What is the minimum viable version of this feature?
   A. Just the core functionality, no polish
   B. Core + basic UI polish
   C. Full feature set with advanced options
   D. Let me describe: [specify]

2. Which of these are must-haves vs nice-to-haves?
   [List potential features, ask user to categorize]

3. What should this feature explicitly NOT do?
   A. No integration with external services
   B. No admin configuration options
   C. No mobile-specific features
   D. Other: [specify]

4. Are there adjacent features we should leave alone?
   A. Yes: [list them]
   B. No, we can modify anything needed
```

After gathering answers, summarize:
```
## Round 2 Summary: Scope Definition
- MVP includes: [list]
- Nice-to-haves (not MVP): [list]
- Explicitly out of scope: [list]
- Do not touch: [list of adjacent features to avoid]
```

**Round 2 complete. 3 rounds remaining.**

---

## Round 3: Technical Constraints

**Goal:** Identify technical limitations, dependencies, and architecture requirements.

Ask questions about:
- What existing systems does this touch?
- What database changes are needed?
- What API changes are needed?
- Are there performance requirements?
- Are there security considerations?
- What dependencies exist?

### Example Questions:

```
1. What existing systems will this feature interact with?
   A. Database only
   B. Database + existing API endpoints
   C. Database + API + external services
   D. Let me list: [specify]

2. Are there performance requirements?
   A. Must handle X requests per second
   B. Must respond within X milliseconds
   C. No specific requirements
   D. Other: [specify]

3. Are there security considerations?
   A. Handles sensitive user data
   B. Requires authentication checks
   C. Needs rate limiting
   D. No special security needs

4. What existing code patterns should we follow?
   A. Follow existing patterns in [module]
   B. This is a new pattern for the codebase
   C. Not sure, needs investigation
```

After gathering answers, summarize:
```
## Round 3 Summary: Technical Constraints
- Systems affected: [list]
- Database changes: [yes/no, what]
- API changes: [yes/no, what]
- Performance requirements: [list]
- Security considerations: [list]
- Patterns to follow: [reference]
```

**Round 3 complete. 2 rounds remaining.**

---

## Round 4: Edge Cases

**Goal:** Identify what could go wrong and how to handle it.

Ask questions about:
- What happens when X fails?
- What if the user does Y unexpectedly?
- What about empty states?
- What about error states?
- What about concurrent operations?
- What about data migration for existing users?

### Example Questions:

```
1. What should happen when [primary action] fails?
   A. Show error message and let user retry
   B. Automatically retry X times
   C. Fall back to [alternative behavior]
   D. Other: [specify]

2. What about empty states (no data yet)?
   A. Show helpful empty state with CTA
   B. Show nothing
   C. Show sample/demo data
   D. Other: [specify]

3. What about existing users/data?
   A. Migration needed for existing data
   B. Feature only applies to new data
   C. Backfill existing data automatically
   D. Let users manually migrate

4. What if user does something unexpected?
   [List specific unexpected behaviors and ask how to handle]
```

After gathering answers, summarize:
```
## Round 4 Summary: Edge Cases
- Error handling: [approach]
- Empty states: [approach]
- Data migration: [approach]
- Unexpected user behavior: [list with handling]
- Concurrent operations: [approach]
```

**Round 4 complete. 1 round remaining.**

---

## Round 5: Verification Strategy

**Goal:** Define how we'll know the feature works correctly.

Ask questions about:
- How will we test this feature?
- What manual testing is needed?
- What automated tests should exist?
- How do we verify in production?
- What metrics indicate success?
- What could we monitor for issues?

### Example Questions:

```
1. What automated tests should cover this feature?
   A. Unit tests for core logic
   B. Integration tests for API endpoints
   C. E2E tests for user flows
   D. All of the above

2. What manual testing is required?
   A. Visual inspection of UI changes
   B. Testing edge cases in browser
   C. Testing with different user roles
   D. List specific scenarios: [specify]

3. How do we know this feature is successful in production?
   A. Users complete [action] X% more often
   B. Support tickets about [topic] decrease
   C. Feature adoption reaches X%
   D. Other metrics: [specify]

4. What should we monitor for issues?
   A. Error rates on new endpoints
   B. Performance metrics
   C. User feedback/complaints
   D. All of the above
```

After gathering answers, summarize:
```
## Round 5 Summary: Verification Strategy
- Automated tests: [list]
- Manual testing: [list]
- Success metrics: [list]
- Monitoring: [list]
```

**Round 5 complete. Planning session finished.**

---

## Output Format

After all 5 rounds, compile the summaries into `tasks/planning-[feature].md`:

```markdown
# Planning Summary: [Feature Name]

Generated: [Date]
Status: Ready for PRD

---

## Round 1: Problem Understanding
[Summary from Round 1]

## Round 2: Scope Definition
[Summary from Round 2]

## Round 3: Technical Constraints
[Summary from Round 3]

## Round 4: Edge Cases
[Summary from Round 4]

## Round 5: Verification Strategy
[Summary from Round 5]

---

## Next Steps

1. Create PRD using `/prd` skill
2. Convert to `prd.json` using `/ralph` skill
3. Run Ralph to implement
```

---

## Checklist

Before completing planning:

- [ ] Round 1 complete (Problem Understanding)
- [ ] Round 2 complete (Scope Definition)
- [ ] Round 3 complete (Technical Constraints)
- [ ] Round 4 complete (Edge Cases)
- [ ] Round 5 complete (Verification Strategy)
- [ ] All summaries documented
- [ ] Saved to `tasks/planning-[feature].md`

**Do not proceed to PRD until all boxes are checked.**
