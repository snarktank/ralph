---
name: add-feature
description: "Append a new feature to an in-progress Ralph project mid-run. Use when you already have a prd.json and want to add incremental work without re-authoring the whole spec. Triggers on: add a feature, add-feature, append to prd, new feature for ralph, add work to the loop."
user-invocable: true
---

# Add Feature

Append one or more new items to an **existing** Ralph `prd.json` mid-project,
low-ceremony. This is the incremental sibling of `/prd` + `/ralph`: those author
a project from scratch; this drops a feature into a project the loop is already
working. **Append is immediate — no inbox, no approval step, no `pending` state
(E.4).**

---

## The Job

Given a free-text feature description:

1. Generate one or more structured items (MVP-scoped, see below).
2. Append them to `prd.json` `items` — **without touching any existing item**
   (existing V1 items keep `passes:true`; you only add) (E.2).
3. Append a human-readable entry to `features.md` (E.3).

Do **not** start implementing. The loop picks the new items up on its next
iteration.

---

## Step 1 — MVP scoping (inherit /prd discipline) (E.5)

Apply the same aggressive-MVP discipline `/prd` uses:

- Collapse the request to the **2–3 highest-impact** items. Reject speculative
  complexity, "while we're here" additions, and gold-plating.
- Each item must be completable in **one Ralph iteration** (one context window).
  If you can't describe the change in 2–3 sentences, split it.
- Order items by dependency (schema → backend → UI). A new item must not depend
  on a later new item.

If the description is genuinely ambiguous on scope, ask **at most one** lettered
clarifying question, then proceed. Bias toward appending over interrogating.

---

## Step 2 — Item schema

Match the fork's `{meta, items}` item schema exactly. New items are always
`passes:false` and carry the three control fields at their defaults:

```json
{
  "id": "FEAT-001",
  "block": "FEAT",
  "blockName": "<short feature group name>",
  "description": "<verifiable, one-line 'add X so that Y'>",
  "passes": false,
  "attempts": 0,
  "blocked": false,
  "blockReason": ""
}
```

Rules:

- **IDs:** use a `FEAT-NNN` series (or continue an existing feature series).
  Never reuse or renumber an existing item's id.
- **block / blockName:** group related appended items under a shared block label
  so they render together in the audit. Use `FEAT` (or a descriptive variant)
  to keep them distinct from the original A–I blocks.
- **description:** verifiable, not vague. "Filter has options All|Active|Done"
  not "filtering works well". Fold quality gates ("typecheck passes",
  "tests pass", and for UI: "verify in browser") into the description or as the
  closing clause, consistent with how existing items read.

---

## Step 3 — Append to prd.json (never overwrite) (E.2)

1. Read the current `prd.json`.
2. Append the new item(s) to the end of the `items` array.
3. Leave `meta`, and every existing item, **byte-for-byte unchanged**.
4. Write the file back with the same 2-space indentation.

After writing, sanity-check: the item count increased by exactly the number you
added, and no existing id changed its `passes` value.

---

## Step 4 — Append to features.md (E.3)

Append (never replace) a dated entry. Create `features.md` with a header if it
doesn't exist:

```markdown
## YYYY-MM-DD — <Feature title>
<one-paragraph description of what was requested and why>
Items added: FEAT-001, FEAT-002
---
```

---

## Checklist

- [ ] Scoped to 2–3 high-impact, single-iteration items (rejected speculative complexity)
- [ ] Items match the `{meta, items}` item schema, `passes:false`, defaults set
- [ ] Appended to `prd.json` `items`; no existing item altered; meta untouched
- [ ] Dated entry appended to `features.md`
- [ ] Did NOT implement anything or wait for approval — append is immediate
