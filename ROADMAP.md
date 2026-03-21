# Ralph Roadmap

## ✅ Done

### v1.0 — Basic NVIDIA LLM Integration
- Agent loop z NVIDIA API (OpenAI-compatible)
- Retry logic (3 próby z 5s delay)
- Action execution: `read_file`, `edit_file`, `create_file`, `run_command`, `finish`, `list_files`
- PRD-driven workflow (stories z `passes: true/false`)
- Git auto-commit przed edycją `.sh` plików
- Progress logging do `progress.txt`
- Archive poprzednich runów

### v1.1 — Bugfixes (marzec 2026)
- Fix: usunięto zduplikowany blok kodu (~96 linii)
- Fix: pipe → process substitution (subshell problem z `return 0`)
- Fix: COMPLETE signal re-emit na stdout
- Fix: usunięto zduplikowany debug output
- Upgrade: `max_tokens` 4096 → 8192

### v1.2 — Staging Sandbox (marzec 2026)
- Staging copy pattern: edycje `ralph.sh` idą do `.staging/ralph.sh`
- Automatyczna walidacja `bash -n` przed merge
- Fix COMPLETE false-positive w JSON fallbacku
- Sandbox info w system prompt

---

## 🔜 Planned

### v1.3 — Smart File Chunking (SI-008)
- Pliki >200 linii: head 50 + tail 50 + `[... N lines omitted ...]`
- `read_file` z opcjonalnymi `line_start`/`line_end`
- Dokumentacja w system prompt

### v1.4 — UX & Observability (SI-005, SI-006)
- Clean status lines: `[iter 3/40] read_file: ralph.sh (18KB) ... OK`
- `RALPH_DEBUG=1` do verbose output
- Execution timer per action/iteration
- Summary: stories done/remaining, total actions, API calls

### v1.5 — Conversation History Management
- Sliding window (max N wiadomości)
- Token budget guard
- Summarization starszych wiadomości

---

## 🔮 Future Ideas

### Docker Sandbox
- Pełna izolacja zmian w kontenerze
- Agent edytuje, testuje, commituje w kontenerze
- Merge do host'a po pełnej weryfikacji
- Wymaga Docker Desktop (dostępny)
- Pozwala na `npm test`, `pytest`, pełne CI w sandbox

### Multi-Agent Architecture
- Agent "coder" — implementuje zmiany
- Agent "reviewer" — code review przed merge
- Agent "tester" — uruchamia testy w sandbox
- Koordynator decyduje o merge

### Model Auto-Selection
- Benchmark modeli per task type
- Szybki model do read/plan, duży do code generation
- Fallback chain: primary → backup → local

### Web Dashboard
- Real-time monitoring Ralph'a przez browser
- Historia iteracji z diff'ami
- PRD management UI
- Flowchart wykonania na żywo
