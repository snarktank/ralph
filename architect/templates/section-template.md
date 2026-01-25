# Section XXX: [Section Name]

## Summary

[1-2 sentence high-level description of what this section accomplishes]

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**
[Describe the exact feature/component being built]

**Visual/Functional Outcome:**
[What does success look like? What will the user see/experience?]

**Inputs:**
- [Input 1: source and type]
- [Input 2: source and type]

**Outputs:**
- [Output 1: what gets created/modified]
- [Output 2: what gets created/modified]

**User Interaction:**
[How does the user trigger or interact with this feature?]

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | [Step 1 description] | 15min | `src/path/file.ts` | TypeScript compiles |
| 2 | [Step 2 description] | 20min | `src/path/file.tsx` | Component renders |
| 3 | [Step 3 description] | 15min | `src/path/file.ts` | Unit test passes |

**Detailed Breakdown:**

**Step 1: [Title]**
- Create: [new files/functions/components]
- Modify: [existing files if any]
- Test: [specific verification]

**Step 2: [Title]**
- Create: [new files/functions/components]
- Modify: [existing files if any]
- Test: [specific verification]

---

### 3. Verification Plan

**Unit Tests:**
```bash
npm run test -- [test pattern]
```
Expected:
- [ ] [Specific assertion 1]
- [ ] [Specific assertion 2]

**Build Verification:**
```bash
npm run typecheck  # No errors
npm run lint       # No warnings
npm run build      # Successful
```

**Visual Verification:**
- URL: http://localhost:5173/[path]
- Steps:
  1. [Action to take]
  2. [What to observe]
  3. [Expected result]
- Screenshot: `screenshots/section-XXX-[name].png`

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| [Risk 1] | [What would go wrong] | [How to notice] | [How to fix] |
| [Risk 2] | [What would go wrong] | [How to notice] | [How to fix] |

**Critical Gotchas:**
- [ ] [Important thing to remember]
- [ ] [Common mistake to avoid]

---

### 5. Dependencies

**Requires (must be done first):**
- `task-XXX`: [Dependency description]
- `task-YYY`: [Dependency description]

**Enables (unlocks after completion):**
- `section-AAA`: [What this enables]
- `section-BBB`: [What this enables]

**Shared State/Data:**
- [Store/context that this section uses]
- [Store/context that this section modifies]

---

## Status Checklist

- [ ] Deep thinking analysis complete
- [ ] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

---

## Notes

[Any additional context, decisions made, alternatives considered]
