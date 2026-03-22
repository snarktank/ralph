# Ralph Roadmap

## ✅ Ukończone

### v1.0 — Basic NVIDIA LLM Integration
- Agent loop z NVIDIA API (OpenAI-compatible)
- Action execution: `read_file`, `edit_file`, `create_file`, `run_command`, `finish`, `list_files`
- PRD-driven workflow (stories z `passes: true/false`)
- Git auto-commit przed edycją `.sh` plików
- Progress logging do `progress.txt`
- Archive poprzednich runów

### v1.1 — Bugfixes
- Fix: usunięto zduplikowany blok kodu (~96 linii)
- Fix: pipe → process substitution (subshell problem z `return 0`)
- Fix: COMPLETE signal re-emit na stdout
- Upgrade: `max_tokens` 4096 → 8192

### v1.2 — Staging Sandbox
- Staging copy pattern: edycje `ralph.sh` → `.staging/ralph.sh`
- Automatyczna walidacja `bash -n` przed merge
- Fix COMPLETE false-positive w JSON fallbacku

### v1.3 — Smart File Chunking
- Pliki >200 linii: head 50 + tail 50 + `[... N lines omitted ...]`
- `read_file` z opcjonalnymi `line_start`/`line_end`

### v1.4 — UX & Observability
- `debug_log()` — debug output ukryty za `RALPH_DEBUG=1`
- `status_line()` — czytelne statusy z emoji per action
- `print_summary()` — end-of-run summary (stories, iteracje, API calls, czas)
- Startup banner z wersją i konfiguracją
- Auto-cleanup po COMPLETE (archiwizacja plików roboczych)

### v1.5 — Context Management & Model Optimization
- Sliding window (10 msg, adaptacyjne 6 msg przy wysokim kontekście)
- Token budget guard — estymator tokenów, ostrzeżenia przy 70%, shrink przy 80%
- Exponential backoff w retry (5s → 15s → 45s)
- Walidacja API error field
- Fallback model chain (primary → backup → micro)
- Verification/TDD mode z `verificationCommand` w PRD

---

## 🔜 Planowane

### Ocena i Priorytety

| Feature | Łatwość | Ważność | Sensowność | Rekomendacja |
|---|:---:|:---:|:---:|---|
| **v1.6 — Checkpoint & Resume** | 🟡 Średnia | 🔴 Wysoka | ✅ Tak | ⭐ **Następny** |
| **v1.7 — Intelligent Error Recovery** | 🟡 Średnia | 🔴 Wysoka | ✅ Tak | ⭐ **Następny** |
| **v1.8 — `--dry-run` Mode** | 🟢 Łatwa | 🟡 Średnia | ✅ Tak | 📌 Szybki win |
| **v1.9 — Parallel Story Detection** | 🔴 Trudna | 🟡 Średnia | 🤔 Ew. | 📅 Później |
| **v2.0 — Docker Sandbox** | 🔴 Trudna | 🔴 Wysoka | ✅ Tak | 📅 Później |
| **v2.1 — Multi-Agent (coder + reviewer)** | 🔴 Trudna | 🟡 Średnia | 🤔 Ew. | 📅 Opcjonalnie |
| **v2.2 — Web Dashboard** | 🔴 Trudna | 🟢 Niska | ❌ Nie teraz | 🗑️ Odpuścić |

---

### v1.6 — Checkpoint & Resume ⭐

**Problem:** Gdy Ralph się crashuje lub kończy max iteracje, trzeba zaczynać od nowa.

**Rozwiązanie:**
- `--resume` flag — wznawia od ostatniego checkpointu
- Automatyczny zapis stanu (conversation history, current story, iteration count) do `.ralph-checkpoint.json`
- Na starcie: sprawdzenie czy istnieje checkpoint i pytanie o wznowienie

| | |
|---|---|
| Łatwość | 🟡 Średnia (~3 godz.) |
| Ważność | 🔴 Wysoka — oszczędza czas i tokeny |
| Ryzyko | Niskie — backward-compatible |

---

### v1.7 — Intelligent Error Recovery ⭐

**Problem:** Gdy LLM zwraca niepoprawny JSON 3x z rzędu, pętla się mieli bez postępu.

**Rozwiązanie:**
- Licznik consecutive JSON parse failures
- Po 3 failures: automatyczny "hint" prompt z ostatnim błędem
- Po 5 failures: skip story, log do progress.txt, przejdź do następnej
- `--strict` mode: fail fast zamiast retry

| | |
|---|---|
| Łatwość | 🟡 Średnia (~2 godz.) |
| Ważność | 🔴 Wysoka — zapobiega marnowaniu tokenów |
| Ryzyko | Niskie |

---

### v1.8 — `--dry-run` Mode

**Problem:** Brak sposobu na preview co Ralph zrobi bez faktycznej implementacji.

**Rozwiązanie:**
- `./ralph.sh --dry-run` — Ralph planuje ale nie wykonuje akcji
- Wyświetla: jakie story by zabrał, jakie pliki by czytał, jaki prompt by wysłał
- Estymowany koszt tokenów

| | |
|---|---|
| Łatwość | 🟢 Łatwa (~1 godz.) |
| Ważność | 🟡 Średnia — przydatne do debugowania |
| Ryzyko | Brak |

---

### v1.9 — Parallel Story Detection

**Problem:** Stories z niekolidującymi plikami mogłyby być implementowane równolegle.

**Rozwiązanie:**
- Analiza dependency graph między stories
- Detekcja stories operujących na różnych plikach
- Opcjonalne równoległe uruchomienie agentów

| | |
|---|---|
| Łatwość | 🔴 Trudna (~8 godz.) |
| Ważność | 🟡 Średnia — speedup tylko przy dużych PRD |
| Ryzyko | Wysokie — merge conflicts, racing conditions |

---

### v2.0 — Docker Sandbox

**Problem:** Agent uruchamia komendy bezpośrednio na hoście — brak izolacji.

**Rozwiązanie:**
- Pełna izolacja zmian w kontenerze Docker
- Agent edytuje, testuje, commituje w kontenerze
- Merge do hosta po pełnej weryfikacji
- Wymaga Docker Desktop

| | |
|---|---|
| Łatwość | 🔴 Trudna (~12 godz.) |
| Ważność | 🔴 Wysoka — bezpieczeństwo i powtarzalność |
| Ryzyko | Średnie — wymaga Docker knowledge |

---

### v2.1 — Multi-Agent Architecture

**Problem:** Jeden agent robi wszystko — brak code review, brak testowania.

**Rozwiązanie:**
- Agent "coder" — implementuje zmiany
- Agent "reviewer" — code review przed merge
- Agent "tester" — uruchamia testy
- Koordynator decyduje o merge

| | |
|---|---|
| Łatwość | 🔴 Trudna (~20 godz.) |
| Ważność | 🟡 Średnia — jakość kodu |
| Ryzyko | Wysokie — fundamentalna przebudowa |

---

### ~~v2.2 — Web Dashboard~~ (Odpuścić)

Real-time monitoring przez browser, historia z diffami, PRD management UI.

**Dlaczego nie:** Flowchart już istnieje. Dashboard to osobny projekt, nie core Ralpha. `print_summary` (v1.4) pokrywa 80% potrzeb obserwability. Koszt vs zysk nieopłacalny.
