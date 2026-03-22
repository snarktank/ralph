# Ralph — Setup & Usage Guide

Pełna instrukcja uruchomienia pętli Ralpha, konfiguracja narzędzi i zarządzanie przebiegami.

## Wymagania

- Jedno z narzędzi AI:
  - [Amp CLI](https://ampcode.com) (domyślne)
  - [Claude Code](https://docs.anthropic.com/en/docs/claude-code) (`npm install -g @anthropic-ai/claude-code`)
  - Lub bezpośrednie API LLM (NVIDIA — patrz niżej)
- `jq` zainstalowany (`brew install jq` na macOS, `apt install jq` na Ubuntu)
- `curl` zainstalowany
- Repozytorium git

## Konfiguracja .env

```bash
cp .env.example .env
# Edytuj .env i ustaw NVIDIA_API_KEY=twój-prawdziwy-klucz
```

> **⚠️ .env nigdy nie jest commitowany.** Jest w `.gitignore`.

## Instalacja

### Opcja 1: Kopiowanie do projektu

```bash
# Z katalogu głównego projektu
mkdir -p scripts/ralph
cp /path/to/ralph/ralph.sh scripts/ralph/
cp /path/to/ralph/prompt.md scripts/ralph/prompt.md    # Dla Amp
# LUB
cp /path/to/ralph/CLAUDE.md scripts/ralph/CLAUDE.md    # Dla Claude Code
chmod +x scripts/ralph/ralph.sh
```

### Opcja 2: Globalna instalacja skills

Dla Amp:
```bash
cp -r skills/prd ~/.config/amp/skills/
cp -r skills/ralph ~/.config/amp/skills/
```

Dla Claude Code:
```bash
cp -r skills/prd ~/.claude/skills/
cp -r skills/ralph ~/.claude/skills/
```

### Opcja 3: Claude Code Marketplace

```bash
/plugin marketplace add snarktank/ralph
/plugin install ralph-skills@ralph-marketplace
```

Dostępne skille: `/prd` (generowanie PRD), `/ralph` (konwersja PRD na JSON).

### Konfiguracja Amp auto-handoff (zalecana)

Dodaj do `~/.config/amp/settings.json`:

```json
{
  "amp.experimental.autoHandoff": { "context": 90 }
}
```

Umożliwia automatyczne przejścia między kontekstami gdy się zapełnią.

## Uruchamianie Pętli

### Krok 1: Stwórz PRD

```
Load the prd skill and create a PRD for [opis feature'a]
```

Odpowiedz na pytania — skill zapisuje wynik do `tasks/prd-[nazwa].md`.

### Krok 2: Skonwertuj PRD na format Ralph

```
Load the ralph skill and convert tasks/prd-[nazwa].md to prd.json
```

Tworzy `prd.json` z user stories gotowymi do autonomicznego wykonania.

### Krok 3: Uruchom Ralpha

**Amp (domyślny):**
```bash
./ralph.sh [max_iteracji]
```

**Claude Code:**
```bash
./ralph.sh --tool claude [max_iteracji]
```

**NVIDIA API (bezpośrednie):**
```bash
./ralph.sh --llm-provider nvidia --llm-api-key TWÓJ_KLUCZ [max_iteracji]

# Opcjonalne parametry:
# --llm-model MODEL           (default: abacusai/dracarys-llama-3.1-70b-instruct)
# --llm-base-url URL          (default: https://integrate.api.nvidia.com/v1)
# --llm-temperature FLOAT     (default: 0.7)
```

Domyślnie: 10 iteracji.

### Co robi Ralph w każdej iteracji

1. Wybiera najwyższy priorytet story z `passes: false`
2. Implementuje to story
3. Uruchamia testy/quality checks
4. Commituje jeśli testy przeszły
5. Oznacza story jako `passes: true` w `prd.json`
6. Dopisuje wnioski do `progress.txt`
7. Powtarza aż wszystkie stories przejdą lub skończą się iteracje

### Sygnał zakończenia

Gdy wszystkie stories mają `passes: true`, Ralph wypisuje `<promise>COMPLETE</promise>` i kończy.

## Pliki Runtime

Te pliki są generowane automatycznie podczas pracy Ralpha. **Nie trzeba ich ręcznie zarządzać.**

| Plik | Opis | Czyszczenie |
|---|---|---|
| `prd.json` | Aktywny PRD z statusem stories | Ralph auto-archiwizuje przy zmianie brancha |
| `progress.txt` | Log postępu z wnioskami | Ralph auto-archiwizuje przy zmianie brancha |
| `.last-branch` | Nazwa ostatniego brancha (tracking) | Nadpisywane automatycznie |
| `.rejection_count` | Licznik nieudanych weryfikacji TDD | Resetowany po sukcesie |
| `ralph-debug.log` | Log debugowania | Nadpisywany przy każdym uruchomieniu |

> **Auto-archiwizacja:** gdy `branchName` w `prd.json` zmieni się od ostatniego uruchomienia, Ralph automatycznie kopiuje `prd.json` i `progress.txt` do `archive/YYYY-MM-DD-nazwa-feature/` i resetuje `progress.txt`.

## Debugowanie

```bash
# Które stories są gotowe?
jq '.userStories[] | {id, title, passes}' prd.json

# Log postępu z poprzednich iteracji
cat progress.txt

# Historia gitowa
git log --oneline -10

# Status PRD (skrypt pomocniczy)
bash parse_stats.sh
```

## Customizacja Promptu

Po skopiowaniu `prompt.md` (Amp) lub `CLAUDE.md` (Claude Code) do projektu, dostosuj:
- Komendy quality check specyficzne dla projektu
- Konwencje codebase'u
- Typowe pułapki dla twojego stacku

## Archiwizacja

Ralph **automatycznie archiwizuje** poprzednie uruchomienia gdy zaczynasz nową feature (inny `branchName`).

Archiwa: `archive/YYYY-MM-DD-nazwa-feature/` — zawierają `prd.json` i `progress.txt` z poprzedniego run'a.

## Referencje

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Amp documentation](https://ampcode.com/manual)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
- [NVIDIA API documentation](https://docs.nvidia.com/nim/large-language-models/latest/)
