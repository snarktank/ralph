---
name: flick
description: "Clean up Ralph intermediary files (prd.json, progress.txt) after a completed run. Triggers on: flick, flick booger, clean up ralph files, remove prd.json, delete progress.txt, ralph cleanup."
user-invocable: true
---

# Flick — Ralph Cleanup

Finds and removes `prd.json` and `progress.txt` left behind by a Ralph run.

---

## The Job

Search for Ralph intermediary files in the following locations (relative to the current project directory, and at the user level):

1. `{project_root}/`
2. `{project_root}/tasks/`
3. `{project_root}/ralph/`
4. `{project_root}/.agents/ralph/`
5. `{project_root}/.agents/skills/ralph/`
6. `{project_root}/.claude/ralph/`
7. `{project_root}/.claude/skills/ralph/`
8. `~/.agents/ralph/`
9. `~/.agents/skills/ralph/`
10. `~/.claude/ralph/`
11. `~/.claude/skills/ralph/`

Where `{project_root}` is the root of the current project (the working directory).

---

## Steps

1. **Find all instances** of `prd.json` and `progress.txt` across every location above.

2. **If nothing is found**, tell the user: no Ralph files found.

3. **For each location that has files:**
   - Show the location path and which files were found there
   - If a `prd.json` exists at that location, show its first 15 lines
   - Ask the user: "Delete these files? (y/n)"
   - If yes, delete them and confirm each deletion
   - If no, skip and move to the next location

4. **When all locations are processed**, give a brief summary of what was deleted vs skipped.

---

## Notes

- Be precise about paths when reporting — show the full path of each file found and deleted.
- Do not delete any files without confirmation.
- If the user passes `--nuke`, `--all`, or says "nuke it" / "delete everything" / "delete all", skip the per-location prompts and delete all found files immediately.
