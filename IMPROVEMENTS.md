# Szczegółowy Plan Implementacji Poprawek dla Projektu Ralph

## 1. Opis Projektu

Ralph to autonomiczny agent AI do programowania, który działa w pętli, uruchamiając narzędzia kodujące AI (Amp, Claude Code lub bezpośrednie API NVIDIA) wielokrotnie, aż wszystkie elementy PRD zostaną ukończone. Każda iteracja to nowa instancja z czystym kontekstem, a pamięć utrzymuje się poprzez historię git, plik `progress.txt` i `prd.json`.

## 2. Identyfikacja Problemów

Na podstawie analizy kodu i dokumentacji, zidentyfikowano następujące kluczowe problemy:

1. **Błąd składniowy w funkcji execute_nvidia_actions** - duplikaty case statements
2. **Nieefektywne zarządzanie kontekstem** - nieograniczone rosnące historie rozmów
3. **Brak optymalizacji dla dużych plików** - czytanie całych plików bez względu na rozmiar
4. **Brak monitorowania zużycia kontekstu** - brak widoczności zużycia tokenów
5. **Niekompletne wykrywanie sygnału zakończenia** - problemy z propagacją sygnału COMPLETE

## 3. Szczegółowy Plan Implementacji Poprawek

### Faza 0: Natychmiastowe Poprawki Krytyczne (Priorytet: Krytyczny)

#### 3.1 Naprawa Błędu Składniowego

**Problem**: Duplikaty case statements w funkcji execute_nvidia_actions (linie 366-456)

**Zadania**:

- [ ] Usunięcie linii 366-456 z duplikatami case handlers
- [ ] Weryfikacja składni poleceniem `bash -n ralph.sh`
- [ ] Test podstawowej funkcjonalności z minimalnym PRD

**Szacowany czas**: 30 minut

#### 3.2 Uzupełnienie Brakujących Terminatorów

**Problem**: Brakujące `;;` w niektórych case handlers

**Zadania**:

- [ ] Dodanie brakujących `;;` terminatorów
- [ ] Weryfikacja składni poleceniem `bash -n ralph.sh`

**Szacowany czas**: 15 minut

### Faza 1: Zarządzanie Kontekstem (Priorytet: Wysoki)

#### 1.1 Implementacja Truncate Historii Rozmów

**Problem**: Nieograniczone rosnące historie rozmów prowadzące do przekroczenia kontekstu

**Zadania**:

- [ ] Implementacja funkcji truncate_conversation_history
- [ ] Zachowanie ostatnich N interakcji (N=5-10)
- [ ] Podsumowanie starszych interakcji w pojedynczą wiadomość
- [ ] Testowanie z różnymi długościami historii

**Szacowany czas**: 4 godziny

#### 1.2 Inteligentne Czytanie Plików

**Problem**: Czytanie całych dużych plików zużywa niepotrzebnie tokeny

**Zadania**:

- [ ] Rozszerzenie akcji read_file o opcjonalne parametry:
  - `start_line`: początkowa linia do odczytu (1-indexed)
  - `end_line`: końcowa linia do odczytu (włącznie)
  - `max_lines`: maksymalna liczba linii do odczytu od początku
- [ ] Domyślne zachowanie: zachowanie obecnej funkcjonalności
- [ ] Testowanie z różnymi rozmiarami plików

**Szacowany czas**: 3 godziny

#### 1.3 Monitorowanie Zużycia Kontekstu

**Problem**: Brak widoczności zużycia tokenów podczas wykonania

**Zadania**:

- [ ] Implementacja estymatora zużycia tokenów (~4 znaki/token)
- [ ] Dodanie ostrzeżeń przy przekroczeniu 70% kontekstu
- [ ] Proaktywne truncate przed osiągnięciem limitów

**Szacowany czas**: 2 godziny

### Faza 2: Optymalizacja Modelu (Priorytet: Średni)

#### 2.1 Aktualizacja Konfiguracji Modelu

**Problem**: Obecny model może być niewystarczający dla złożonych PRD

**Zadania**:

- [ ] Aktualizacja .env z rekomendowanymi opcjami modeli:
  - microsoft/phi-3-medium-128k-instruct (128K kontekst)
  - microsoft/phi-3-mini-128k-instruct (128K kontekst)
  - nvidia/nemotron-4-340b-instruct (jeśli dostępny)
- [ ] Dodanie skryptu walidacji modelu i kontekstu
- [ ] Implementacja mechanizmu fallback do mniejszych modeli
- [ ] Dokumentacja procedury zmiany modelu

**Szacowany czas**: 3 godziny

### Faza 3: Zaawansowane Funkcje (Priorytet: Niski)

#### 3.1 Kompresja Historii Bazująca na Podsumowaniach

**Zadania**:

- [ ] Implementacja kompresji historii bazującej na podsumowaniach
- [ ] Dodanie dashboardu zużycia kontekstu
- [ ] Tworzenie adaptacyjnego truncate bazującego na ważności treści

**Szacowany czas**: 6 godzin

## 4. Plan Testowania i Walidacji

### Testy Jednostkowe

- [ ] Test składni execute_nvidia_actions
- [ ] Test truncate historii zachowujący istotne informacje
- [ ] Test inteligentnego czytania plików z różnymi parametrami

### Testy Integracyjne

- [ ] Uruchomienie pętli agenta z małym PRD (2-3 stories)
- [ ] Weryfikacja ograniczonego rozmiaru historii
- [ ] Potwierdzenie działania wykrywania zakończenia
- [ ] Test czytania dużych plików z zakresami linii

### Benchmarki Wydajności

- [ ] Pomiar zużycia tokenów przez 10 iteracji
- [ ] Weryfikacja <80% zużycia kontekstu dla typowych obciążeń
- [ ] Test z plikami o różnej wielkości (1K, 10K, 100K linii)

## 5. Plan Wdrażania i Mitigacji Ryzyka

### Kompatybilność Wsteczna

- [ ] Wszystkie zmiany zachowują kompatybilność wsteczną
- [ ] Istniejące PRD i workflow kontynuują działanie bez zmian
- [ ] Nowe funkcje są opcjonalne i nie wymagają zmian w istniejącym kodzie

### Mechanizmy Bezpieczeństwa

- [ ] Truncate historii zachowuje akcje zakończenia (sygnały COMPLETE)
- [ ] Czytanie plików domyślnie pełne jeśli nie podano zakresu
- [ ] Walidacja modelu zapobiega wyborowi niekompatybilnych modeli

### Procedury Rollbacku

- [ ] Tworzenie tagów git przed każdą fazą
- [ ] Jasna dokumentacja jak cofnąć zmiany
- [ ] Zachowanie oryginalnego ralph.sh jako backup

## 6. Szacunkowy Harmonogram

| Faza        | Zadania                | Szacowany czas          |
| ----------- | ---------------------- | ----------------------- |
| Faza 0      | Poprawki krytyczne     | 45 minut                |
| Faza 1      | Zarządzanie kontekstem | 9 godzin                |
| Faza 2      | Optymalizacja modelu   | 3 godziny               |
| Faza 3      | Zaawansowane funkcje   | 6 godzin                |
| Testowanie  | Wszystkie testy        | 4 godziny               |
| **Łącznie** |                        | **22 godziny 45 minut** |

## 7. Kryteria Sukcesu

1. **Błędy składniowe naprawione** - `bash -n ralph.sh` nie zwraca błędów
2. **Zarządzanie kontekstem działa** - historia rozmów nie przekracza 80% kontekstu
3. **Inteligentne czytanie plików działa** - duże pliki czytane selektywnie
4. **Monitorowanie kontekstu działa** - ostrzeżenia przy 70% zużycia
5. **Wykrywanie zakończenia działa** - sygnał COMPLETE propagowany poprawnie
6. **Testy przechodzą** - wszystkie testy jednostkowe i integracyjne zielone
7. **Kompatybilność wsteczna zachowana** - istniejące PRD nadal działają

## 8. Następne Kroki

1. Przegląd i zatwierdzenie tego planu
2. Rozpoczęcie implementacji Fazy 0 (poprawki krytyczne)
3. Weryfikacja podstawowej funkcjonalności
4. Rozpoczęcie implementacji Fazy 1 (zarządzanie kontekstem)
5. Testowanie i iteracja aż do osiągnięcia kryteriów sukcesu

GLM 5

## 1. FLOWCHART - CO TO JEST?

**Flowchart/** to interaktywna wizualizacja React Flow pokazująca jak działa Ralph:

- Prezentacja krokowa - klikasz "Next" i odkrywasz kolejne etapy
- Opublikowana na GitHub Pages: https://snarktank.github.io/ralph/
- Używana do prezentacji jak działa Ralph agent loop
- Deployowana automatycznie przez GitHub Actions (.github/workflows/deploy.yml)
  **Werdykt:** ZOSTAWIĆ - to część produktu, używana do prezentacji.

---

## 2. KATALOGOWANIE PLIKÓW WG PRZYDATNOŚCI

### A. PLIKI KRYTYCZNE (MUSZĄ ZOSTAĆ)

| Plik                           | Rola                       | Uwagi                               |
| ------------------------------ | -------------------------- | ----------------------------------- |
| `ralph.sh`                     | Główny skrypt              | 683 linii, BŁĄD SYNTAX na linii 366 |
| `prd.json.example`             | Template PRD               | Referencyjny                        |
| `AGENTS.md`                    | Instrukcje dla agentów     | Wczytywane przez AI                 |
| `prompt.md`                    | Instrukcje dla Amp         | Wczytywane przez Amp                |
| `CLAUDE.md`                    | Instrukcje dla Claude Code | Wczytywane przez Claude             |
| `.gitignore`                   | Ignorowanie plików         | Krytyczne                           |
| `README.md`                    | Dokumentacja               | Krytyczna                           |
| `skills/prd/SKILL.md`          | Skill generowania PRD      | Funkcjonalny                        |
| `skills/ralph/SKILL.md`        | Skill konwersji PRD        | Funkcjonalny                        |
| `.github/workflows/deploy.yml` | CI/CD dla flowchart        | Działa                              |
| `flowchart/`                   | Interaktywna wizualizacja  | Prezentacja                         |

### B. PLIKI AKTYWNE (PRACA W TOKGU)

| Plik           | Rola            | Uwagi                              |
| -------------- | --------------- | ---------------------------------- |
| `prd.json`     | Aktualny PRD    | Self-improve-v2, 3 stories pending |
| `progress.txt` | Log postępu     | 12 linii, aktywne                  |
| `.env`         | Konfiguracja    | **NIGDY nie commitować!**          |
| `.last-branch` | Branch tracking | Generowany                         |

### C. PLIKI DO USUNIĘCIA (REDUNDANTNE)

#### DUPLIKATY PLANÓW (4 sztuki):

| Plik                      | Powód usunięcia                           |
| ------------------------- | ----------------------------------------- |
| `PLAN_LLM_INTEGRATION.md` | Zastąpiony przez `PLAN_LLM_NVIDIA.md`     |
| `PLAN_LLM_NVIDIA.md`      | Zastąpiony przez `PLAN_NVIDIA_LLM.md`     |
| `PLAN_NVIDIA_LLM.md`      | Zastąpiony przez implementację w ralph.sh |
| `PLAN.md`                 | Historyczny, nieaktualny                  |

#### DUPLIKATY POPRAWEK (3 sztuki):

| Plik                        | Powód usunięcia                              |
| --------------------------- | -------------------------------------------- |
| `IMPROVEMENTS.md`           | Zastąpiony przez `RALPH_IMPROVEMENT_PLAN.md` |
| `RALPH_IMPROVEMENT_PLAN.md` | Duplikuje `ROADMAP.md`                       |
| `PROJECT_AUDIT_PLAN.md`     | Ten plik jest wynikiem audytu                |

#### PLIKI TESTOWE (5 sztuk):

| Plik                   | Powód usunięcia                              |
| ---------------------- | -------------------------------------------- |
| `test-nvidia-prd.md`   | Testowy PRD, już niepotrzebny                |
| `test-prd.md`          | Testowy PRD, duplikuje                       |
| `test-simple-prd.md`   | Testowy PRD, duplikuje                       |
| `test-nvidia.sh`       | Skrypt testowy - został włączony do ralph.sh |
| `test_llm_response.sh` | Debug script, niepotrzebny                   |
| `debug-nvidia.sh`      | Debug script, niepotrzebny                   |
| `debug-agent.sh`       | Debug script, niepotrzebny                   |

#### PLIKI INSTRUKCJI TESTOWYCH (2 sztuki):

| Plik                   | Powód usunięcia                        |
| ---------------------- | -------------------------------------- |
| `TEST_NVIDIA.md`       | Instrukcje testowe - już przetestowano |
| `TEST_INSTRUCTIONS.md` | Instrukcje testowe - duplikat          |

#### PLIKI BADAWCZE (1 sztuka):

| Plik                    | Powód usunięcia                   |
| ----------------------- | --------------------------------- |
| `INVESTIGATION_PLAN.md` | Historyczne badanie, zrealizowane |

#### PLIKI DEBUG/LOG (2 sztuki):

| Plik                 | Powód usunięcia                            |
| -------------------- | ------------------------------------------ |
| `ralph-debug.log`    | 238 linii logów - powinno być w .gitignore |
| `.chat_history.json` | Historia czatu - tymczasowa                |

### D. PLIKI DO KONSOLIDACJI

| Plik         | Akcja                                 |
| ------------ | ------------------------------------- |
| `ROADMAP.md` | ZOSTAWIĆ - wersja obecna jest dobra   |
| `.env`       | Stworzyć `.env.example` jako template |

---

## 3. PROBLEMY DO NAPRAWIENIA

### A. BŁĄD SYNTAX W ralph.sh

- **Linia 366:** Orphan case statement po błędnym merge
- **Rozwiązanie:** Usunąć linie 366-475 (duplikaty)

### B. BRAK .env.example

- Stworzyć template z bezpiecznymi wartościami:
  ```
  NVIDIA_API_KEY=your-api-key-here
  LLM_MODEL=qwen/qwen2.5-coder-32b-instruct
  LLM_BASE_URL=https://integrate.api.nvidia.com/v1
  LLM_TEMPERATURE=0.3
  ```

### C. ralph-debug.log NIE JEST W .gitignore

- Dodać do .gitignore (już jest linia 19, ale sprawdzić)

---

## 4. PROponOWANA NOWA STRUKTURA

```
ralph/
├── ralph.sh              # Główny skrypt
├── .env.example          # Template konfiguracji (NOWY)
├── .gitignore
├── README.md             # Główna dokumentacja
├── ROADMAP.md            # Plan rozwoju
├── LICENSE
├── prd.json.example      # Template PRD
│
├── docs/                 # (OPCJONALNIE - jeśli wolisz separację)
│   └── AGENTS.md         # Instrukcje dla agentów
│
├── skills/
│   ├── prd/SKILL.md
│   └── ralph/SKILL.md
│
├── flowchart/            # Interaktywna wizualizacja
│   └── ...
│
├── .github/
│   └── workflows/
│       └── deploy.yml
│
└── archive/              # Historia runów (ignorowane w git)
```

---

## 5. PLIKI DO USUNIĘCIA (LISTA KOŃCOWA)

**Do usunięcia (22 pliki):**

1. `PLAN.md`
2. `PLAN_LLM_INTEGRATION.md`
3. `PLAN_LLM_NVIDIA.md`
4. `PLAN_NVIDIA_LLM.md`
5. `IMPROVEMENTS.md`
6. `RALPH_IMPROVEMENT_PLAN.md`
7. `PROJECT_AUDIT_PLAN.md`
8. `INVESTIGATION_PLAN.md`
9. `TEST_NVIDIA.md`
10. `TEST_INSTRUCTIONS.md`
11. `test-nvidia-prd.md`
12. `test-prd.md`
13. `test-simple-prd.md`
14. `test-nvidia.sh`
15. `test_llm_response.sh`
16. `debug-nvidia.sh`
17. `debug-agent.sh`
18. `ralph-debug.log`
19. `.chat_history.json`
20. `.last-branch` (tymczasowy)
    21-22. `undefined/` katalog (artefakt)

---

## 6. KOLEJNOŚĆ DZIAŁAŃ

1. **NAJPIERW:** Naprawić syntax error w ralph.sh (linie 366-475)
2. **POTEM:** Stworzyć `.env.example`
3. **POTEM:** Usunąć pliki redundantne
4. **NA KONIEC:** Zaktualizować README.md z nową strukturą

---

## 7. UWAGI O ORGANIZACJI PRACY

### Obecne problemy:

1. Zbyt dużo plików "plan" które się duplikują
2. Brak .env.example
3. Pliki testowe zaśmiecają root
4. Historyczne pliki badawcze nie są archiwizowane

### Rekomendacje:

1. **JEDEN plik planu** = ROADMAP.md
2. **Template .env.example** wymagany
3. **Pliki testowe** do katalogu `tests/` lub usunąć po użyciu
4. **Pliki badawcze** do `docs/archive/` lub usunąć

---

NEMOTRON 3 SUPER

## EXECUTIVE SUMMARY

This plan outlines a comprehensive audit of the Ralph project infrastructure to identify useful vs. outdated files, assess organizational structure, and provide recommendations for better file organization and workflow efficiency. The audit will be conducted in phases to ensure thorough analysis while maintaining project functionality.

## AUDIT OBJECTIVES

1. Identify critical files essential for Ralph operation
2. Detect outdated, redundant, or temporary files
3. Evaluate documentation relevance and accuracy
4. Assess skills and extension organization
5. Provide actionable recommendations for file restructuring
6. Improve maintainability and onboarding experience

## PHASED AUDIT APPROACH

### PHASE 1: CORE INFRASTRUCTURE AUDIT (Days 1-2)

**Focus**: Files essential for Ralph's basic operation
**Files to Audit**:

- `ralph.sh` - Main execution script
- `prd.json` - Requirements database
- `progress.txt` - Persistent learning
- `.env` - Environment configuration
- `AGENTS.md` / `prompt.md` / `CLAUDE.md` - Agent instructions
  **Audit Criteria**:
- Is the file required for basic Ralph execution?
- Does it contain executable logic vs. documentation?
- How frequently is it modified during normal operation?
- Are there duplicate or conflicting versions?
  **Deliverable**: Core Infrastructure Assessment Report

### PHASE 2: DOCUMENTATION AUDIT (Days 3-4)

**Focus**: All markdown documentation files
**Files to Audit**:

- All `*.md` files in root directory
- Documentation in subdirectories (flowchart/, skills/)
- README files in subdirectories
  **Audit Criteria**:
- Accuracy: Does documentation match current implementation?
- Completeness: Does it cover all necessary information?
- Redundancy: Is same information duplicated elsewhere?
- Accessibility: Is it easy for new contributors to find?
- Maintenance burden: Is it kept up-to-date?
  **Deliverable**: Documentation Quality Report with prioritization

### PHASE 3: SKILLS AND EXTENSIONS AUDIT (Days 5-6)

**Focus**: Skills directory and related extensions
**Files to Audit**:

- `/skills/prd/` - PRD generation skill
- `/skills/ralph/` - Ralph conversion skill
- Skill metadata files
- Any custom skill implementations
  **Audit Criteria**:
- Functional correctness: Do skills work as intended?
- Integration quality: How well do they integrate with Ralph?
- Documentation: Are skills properly documented?
- Usage patterns: Which skills are actually used?
  **Deliverable**: Skills Effectiveness Analysis

### PHASE 4: TEMPORARY AND BUILD ARTIFACTS AUDIT (Days 7-8)

**Focus**: Files that should be temporary or generated
**Files to Audit**:

- Debug files (`debug-*.log`, `debug-*.sh`)
- Large logs (`ralph-debug.log`)
- Temporary JSON files (`.chat_history.json`, `.last-branch`)
- Build artifacts and caches
- Archived runs in `/archive/`
  **Audit Criteria**:
- Should this be temporary?
- Is it safe to delete/ignore?
- Does it serve a persistent purpose?
- Can it be better organized?
  **Deliverable**: Temporary Files Management Plan

### PHASE 5: DEPENDENCIES AND CONFIGURATION AUDIT (Days 9-10)

**Focus**: Project configuration and dependencies
**Files to Audit**:

- `package.json` and lock files
- Configuration files (`.gitignore`, `.eslintrc*`, etc.)
- Scripts and tooling
- Dependency relevance and versions
  **Audit Criteria**:
- Are dependencies necessary and up-to-date?
- Is configuration optimal for development/CI?
- Are there security vulnerabilities?
- Can build processes be improved?
  **Deliverable**: Dependencies Health Check

## DETAILED FILE-BY-FILE AUDIT PLAN

### TIER 1: CRITICAL OPERATIONAL FILES (Must be preserved and maintained)

| File           | Purpose                        | Audit Focus                               | Risk if Mismanaged              |
| -------------- | ------------------------------ | ----------------------------------------- | ------------------------------- |
| `ralph.sh`     | Main execution script          | Syntax correctness, logic flow, security  | Complete Ralph failure          |
| `prd.json`     | Requirements database          | JSON validity, schema consistency         | Agent loop malfunction          |
| `progress.txt` | Learning persistence           | Format consistency, growth management     | Loss of institutional knowledge |
| `.env`         | Environment configuration      | Security (API keys), variable correctness | Authentication failures         |
| `AGENTS.md`    | Claude Code agent instructions | Accuracy, completeness                    | Agent misbehavior               |
| `prompt.md`    | Amp agent instructions         | Accuracy, completeness                    | Agent misbehavior               |
| `CLAUDE.md`    | Claude Code agent instructions | Accuracy, completeness                    | Agent misbehavior               |

### TIER 2: IMPORTANT SUPPORT FILES (Should be well-maintained)

| File                   | Purpose                 | Audit Focus                                 | Risk if Mismanaged             |
| ---------------------- | ----------------------- | ------------------------------------------- | ------------------------------ |
| `README.md`            | Project introduction    | Clarity, accuracy, getting started guidance | Poor onboarding                |
| `ROADMAP.md`           | Future direction        | Relevance, accuracy, strategic alignment    | Misguided development          |
| `IMPROVEMENTS.md`      | Tracking enhancements   | Actionability, progress tracking            | Lost improvement opportunities |
| `TEST_INSTRUCTIONS.md` | Testing guidelines      | Clarity, completeness, effectiveness        | Reduced quality assurance      |
| `TEST_NVIDIA.md`       | NVIDIA-specific tests   | Relevance to current config, effectiveness  | Undetected regressions         |
| `/skills/` directory   | Extension functionality | Correctness, documentation, integration     | Broken extensibility           |

### TIER 3: CONTEXTUAL/HISTORICAL FILES (Review for archival or deletion)

| File                         | Purpose                   | Audit Focus                      | Risk if Mismanaged        |
| ---------------------------- | ------------------------- | -------------------------------- | ------------------------- |
| `INVESTIGATION_PLAN.md`      | Past investigation docs   | Relevance to current work        | Confusion, clutter        |
| `PLAN*.md` files             | Historical planning       | Superseded status, relevance     | Documentation debt        |
| `test-*-prd.md` files        | Test requirements         | Current usefulness, duplication  | Test maintenance overhead |
| Flowchart files              | Interactive visualization | Accuracy, functionality, size    | Misleading representation |
| Debug scripts (`debug-*.sh`) | Debugging helpers         | Current usefulness, alternatives | Clutter, confusion        |

### TIER 4: TEMPORARY/BUILD ARTIFACTS (Should be cleaned or ignored)

| File/Pattern                 | Purpose         | Audit Focus                    | Risk if Mismanaged                        |
| ---------------------------- | --------------- | ------------------------------ | ----------------------------------------- |
| `.chat_history.json`         | Chat history    | Should be ephemeral            | Privacy/security concerns                 |
| `.last-branch`               | Branch tracking | Should be transient            | Confusion in branch management            |
| `debug-*.log` files          | Debug output    | Should be rotated/deleted      | Disk space waste, sensitive data exposure |
| `ralph-debug.log`            | Large debug log | Size management, rotation      | Disk exhaustion                           |
| `/archive/` directory        | Past runs       | Organization, retention policy | Disorganization, recovery difficulty      |
| `node_modules/`              | JS dependencies | Standard management            | Standard concerns                         |
| Build artifacts (flowchart/) | Compiled assets | Standard build hygiene         | Standard concerns                         |

## SPECIFIC AUDIT TASKS

### TASK 1: Core Script Analysis

- Execute `bash -n ralph.sh` to check syntax
- Identify all external dependencies called
- Map execution flow and decision points
- Document environment variable requirements
- Assess security considerations (input validation, etc.)

### TASK 2: PRD and Progress File Analysis

- Validate `prd.json` JSON schema
- Analyze story structure and prioritization mechanism
- Examine `progress.txt` format and growth patterns
- Check for consistency between files
- Verify completion detection logic

### TASK 3: Documentation Accuracy Verification

- Cross-reference `README.md` claims with actual implementation
- Verify installation/instructions in documentation work
- Check if agent instructions match actual script behavior
- Validate flowchart accuracy against current code
- Review skill documentation for correctness

### TASK 4: Skills Functionality Test

- Test PRD skill with sample input
- Test Ralph skill with sample PRD conversion
- Check for skill integration points in main script
- Verify skill documentation matches implementation
- Assess skill reusability across projects

### TASK 5: Temporary File Lifecycle Analysis

- Identify what creates each temporary file type
- Determine appropriate retention periods
- Recommend cleanup/archiving strategies
- Check for PII or sensitive data in logs
- Suggest .gitignore improvements

### TASK 6: Dependency and Build System Review

- Audit `package.json` for necessity and versions
- Check for known vulnerabilities in dependencies
- Review build scripts for flowchart directory
- Validate that build outputs match source
- Assess CI/CD readiness

## ORGANIZATION RECOMMENDATIONS

### PROPOSED DIRECTORY STRUCTURE

```
ralph/
├── ralph.sh                 # Main executable
├── prd.json                 # Requirements database
├── progress.txt             # Learning persistence
├── .env                     # Environment (gitignored template)
├── docs/                    # Consolidated documentation
│   ├── README.md            # Main documentation
│   ├── ROADMAP.md           # Project direction
│   ├── CONTRIBUTING.md      # Contribution guidelines
│   ├── ARCHITECTURE.md      # System design
│   ├── API_REFERENCE.md     # Internal APIs
│   └── TROUBLESHOOTING.md   # Common issues
├── skills/                  # Extension skills
│   ├── prd/                 # PRD generation
│   └── ralph/               # Ralph conversion
├── config/                  # Configuration files
│   ├── agent-templates/     # AGENTS.md, prompt.md, CLAUDE.md templates
│   └── logs/                # Log rotation configuration
├── tests/                   # Test suite
│   ├── unit/                # Unit tests
│   ├── integration/         # Integration tests
│   └── fixtures/            # Test data
├── scripts/                 # Helper scripts
│   ├── debug/               # Debugging utilities
│   ├── maintenance/         # Cleanup and maintenance
│   └── deployment/          # Deployment helpers
├── archive/                 # Historical runs (organized by date)
└── temp/                    # Temporary files (gitignored)
```

### FILE NAMING AND ORGANIZATION PRINCIPLES

1. **Clear separation of concerns**: Code vs. documentation vs. configuration
2. **Consistent naming**: kebab-case for files, descriptive names
3. **Logical grouping**: Related files in shared directories
4. **Explicit temporariness**: Temporary files in clearly marked locations
5. **Documentation consolidation**: Related docs in docs/ hierarchy
6. **Configuration isolation**: Environment-specific config separated

## RISK ASSESSMENT AND MITIGATION

### HIGH RISK ITEMS

- Modifying `ralph.sh` without proper testing
- Corrupting `prd.json` or `progress.txt`
- Exposing API keys in logs or version control
- Breaking skill integration mechanisms

### MITIGATION STRATEGIES

- Use backup before any file modifications
- Implement validation checks for critical files
- Add pre-commit hooks to prevent API key commits
- Create test suite for core functionality
- Document rollback procedures

## SUCCESS METRICS

1. **Core functionality preserved**: Ralph executes successfully after audit
2. **Documentation accuracy**: >90% of documentation claims verified true
3. **File organization**: Clear structure with intuitive file location
4. **Maintenance reduction**: Less time spent searching for files
5. **Onboarding improvement**: New contributors productive faster
6. **Risk reduction**: Fewer accidental data exposures or losses

## DELIVERABLES

1. **Audit Reports** for each phase (findings, recommendations)
2. **Updated File Inventory** with usefulness ratings
3. **Reorganization Plan** with step-by-step instructions
4. **Implementation Guidelines** for maintaining organization
5. **Risk Mitigation Checklist** for ongoing safety
6. **Template Files** for standardized documentation

## TIMELINE AND RESOURCES

- **Total Duration**: Approximately 2 weeks
- **Primary Resource**: One dedicated auditor
- **Secondary Resources**: Occasional developer consultation for validation
- **Dependencies**: Access to repository, basic Unix tools

## NEXT STEPS

1. Review and approve this audit plan
2. Begin Phase 1 (Core Infrastructure Audit)
3. Schedule regular check-ins to discuss findings
4. Adjust plan based on discovered realities
5. Implement recommendations in priority order

---

_This plan is designed to be executed while maintaining project functionality. All recommendations will prioritize backward compatibility and minimal disruption to ongoing work._

## QWEN 3

2. Krok po kroku – analiza
   2.1. Przegląd struktury katalogów

- Działanie: Użyj ls -R lub glob("\*_/_") do wylistowania wszystkich plików i katalogów.
- Cel: Zbudować mapę projektu. Szczególnie zwrócić uwagę na:
  - src/ – kod źródłowy
  - tests/ lub **tests**/ – testy
  - docs/ – dokumentacja
  - config/ – ustawienia
  - scripts/ – skrypty build/test/deploy
  - \*.md – dokumentacja, testy, specyfikacje
    2.2. Wyszukanie plików .md związanych z testami API NVIDIA
- Działanie: Użyj glob("\*_/_.md") + grep -i "nvidia\|api\|test" na zawartości plików .md.
- Cel: Zidentyfikować pliki typu nvidia-api-test.md, api-tests-nvidia.md, nvidia-integration.md itp.
- Uzasadnienie: Jeśli te pliki zawierają testy ręczne lub specyfikacje, ale nie są wykonywane automatycznie, to są nieaktualne. Współczesne projekty wymagają testów jako kodu (np. _.test.js, _.spec.ts), a nie dokumentów .md.
  2.3. Sprawdzenie, czy testy są wykonywane
- Działanie: Przeszukaj package.json, jest.config.js, vite.config.js, cypress.config.js, pytest.ini, tox.ini – czy są w nich włączone pliki .md jako testy.
- Cel: Jeśli .md nie są w żadnym konfiguratorze testowym – to nie są testami, tylko dokumentacją.
- Uzasadnienie: Testy w .md nie są automatyzowane – nie są częścią CI/CD. Są niepewne i niezawodne. Powinny być przeniesione do kodu lub usunięte.
  2.4. Analiza struktury plików testowych
- Działanie: Użyj glob("tests/\*_/_.{js,ts,py}") i grep -l "nvidia" w tych plikach.
- Cel: Znaleźć prawdziwe testy API NVIDIA – czy są one w odpowiednich folderach (tests/api/nvidia/)?
- Uzasadnienie: Jeśli testy są w tests/ ale nie mają jasnej struktury – to nie są skalowalne.
  2.5. Ocena dokumentacji
- Działanie: Przeczytaj README.md, docs/README.md, docs/api.md.
- Cel: Czy dokumentacja jest:
  - Aktualna?
  - Zintegrowana z kodem?
  - Generowana automatycznie (np. z JSDoc/Swagger)?
- Uzasadnienie: Stara dokumentacja = niezawodność = błędy u użytkowników.
  2.6. Sprawdzenie plików konfiguracyjnych
- Działanie: Przegląd package.json, tsconfig.json, .eslintrc, .prettierrc, .gitignore.
- Cel: Czy są one spójne? Czy zawierają niepotrzebne wpisy? Czy .gitignore zawiera pliki .md z testami?
- Uzasadnienie: Nieaktualne konfiguracje prowadzą do błędów buildu lub niezgodności między środowiskami.
  2.7. Analiza historii git
- Działanie: git log --oneline --grep="nvidia" --since="1 year ago"
- Cel: Zrozumieć, jak często i kiedy dodawano pliki .md z testami. Czy były one zastępowane kodem?
- Uzasadnienie: Jeśli pliki .md były dodawane w ostatnich 6 miesiącach – to nie są przestarzałe, ale nie są automatyzowane. Należy je migrować.

---

3. Propozycje poprawy
   3.1. Organizacja plików
   Bieżąca struktura
   tests/nvidia-api-test.md
   docs/api.md (ręczna)
   docs/test-nvidia.md
   scripts/test-nvidia.sh
   3.2. Proces pracy

- ✅ Testy zawsze jako kod, nigdy jako .md
- ✅ Dokumentacja API generowana automatycznie (np. Swagger UI)
- ✅ Każdy nowy endpoint API → dodaje test w tests/api/ + dokumentację w docs/api/
- ✅ CI/CD → wymaga 100% testów API, nie pozwala na commit z niezaimplementowanymi testami
- ✅ Codzienne audyty – skrypt npm run audit → sprawdza:
  - Czy są pliki .md w tests/?
  - Czy są testy w tests/api/?
  - Czy dokumentacja jest aktualna?
    3.3. Automatyzacja audytu
- Utworzyć skrypt scripts/audit-structure.js:
  - Szuka .md w tests/ → ostrzega
  - Sprawdza, czy tests/api/ ma testy
  - Sprawdza, czy docs/api.md jest starszy niż 30 dni
  - Sprawdza, czy package.json ma test:api skrypt

---

4. Podsumowanie

- Nieaktualne: Pliki .md w katalogach testowych, które nie są wykonywane.
- Przydatne: Katalogi src/, tests/ z kodem testowym, docs/ z generowaną dokumentacją.
- Propozycja: Zastąpić wszystkie testy .md kodem testowym, zintegrować dokumentację z API, automatyzować audyt.

---
