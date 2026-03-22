📄 RESEARCH PAPER: Dogłębna Analiza Repozytoriów, Frameworków i Wzorców Architektonicznych dla Samodoskonalącego się Agenta Kodującego
Wersja: 2.0 Data: 2026-03-22 Charakter dokumentu: Referencyjny research paper — kompletna baza wiedzy

SPIS TREŚCI
Ekosystem Ralph Loop — pełna mapa repozytoriów
Dogłębna analiza: mini-swe-agent (Princeton/Stanford)
Dogłębna analiza: ralphy (michaelshimeles)
Dogłębna analiza: ralph (iannuttall)
Dogłębna analiza: snarktank/ralph
Dogłębna analiza: ralph-orchestrator (mikeyobrien)
Dogłębna analiza: ralph-claude-code (frankbria)
Dogłębna analiza: vercel-labs/ralph-loop-agent
Dogłębna analiza: PageAI-Pro/ralph-loop
Deep Agents (LangChain) — architektura harness'u
Artykuł naukowy: SICA — A Self-Improving Coding Agent (ICLR 2025)
Artykuł naukowy: EvoMAC — Self-Evolving Multi-Agent (ICLR 2025)
Artykuł naukowy: SWE-Search — MCTS dla Software Engineering (ICLR 2025)
Artykuł naukowy: AFlow — Automated Agentic Workflow Optimization (ICLR 2025)
Survey: Awesome-Self-Evolving-Agents (EvoAgentX)
Harness Engineering — wnioski z LangChain
Awesome-Ralph — pełna lista ekosystemu
Wzorce implementacyjne wyekstrahowane z kodu
Zestawienie porównawcze architektur
Indeks linków

1. EKOSYSTEM RALPH LOOP — PEŁNA MAPA
   20
   Ralph to technika kodowania AI, która uruchamia agentów kodujących w automatycznych pętlach aż specyfikacje zostaną spełnione. Ekosystem jest rozległy i aktywnie rozwijany.
   14
   Ralph Wiggum technique to metodologia rozwoju oparta na ciągłych pętlach agentów AI. W swojej istocie jest elegancko prosta: keep feeding an AI agent a task until the job is done. Jak Geoffrey Huntley to opisuje: „Ralph is a Bash loop." Nazwana od wytrwałego Ralpha Wigguma z The Simpsons, ta technika preferuje iteracyjne doskonalenie nad jednorazową perfekcją. Tam gdzie tradycyjne agentic workflows zatrzymują się gdy LLM skończy wywoływać narzędzia, Ralph idzie dalej — weryfikuje ukończenie, dostarcza feedback i uruchamia kolejną iterację aż zadanie faktycznie się powiedzie. Można o tym myśleć jako o `while (true)` dla autonomii AI.
   Kluczowa obserwacja architektoniczna:

18
Model nie trzyma stanu we własnej pamięci. Stan jest przechowywany na zewnątrz — w plikach, outputach, testach, lub środowisku. Każde uruchomienie startuje od zera, ale może nadal korzystać z wyników poprzedniego uruchomienia. To sprawia, że cały system jest zaskakująco odporny i łatwy do zrozumienia.
1.1 Warianty implementacyjne (z awesome-ralph)
20
choo-choo-ralph — 5-fazowy workflow z Beads, ze strukturyzowanymi specyfikacjami, zweryfikowanymi implementacjami i kumulującą się wiedzą. ralph-starter — CLI z integracjami GitHub, Linear, Notion; wspiera wielu agentów, interaktywny wizard, presety workflow i śledzenie kosztów. iannuttall/ralph — minimalna plikowa pętla agenta wspierająca codex/claude/droid/opencode. smart-ralph — spec-driven development workflow, transformuje feature requesty w strukturyzowane specyfikacje. ralph-wiggum-bdd — samodzielny skrypt Bash dla Behavior-Driven Development z Ralph Wiggum Loop. ralph-orchestrator — Rust orchestrator z 7 backendami AI, Hat System dla wyspecjalizowanych person i interaktywnym TUI.
20
pentoai/ml-ralph — autonomiczny agent ML do uruchamiania eksperymentów za pomocą Claude lub Codex. oh-my-ralph — opiniotwórcza pętla Ralph Wiggum zaimplementowana w Pythonie. ralph-wiggum-cursor — implementacja Cursor z śledzeniem tokenów, rotacją kontekstu na 80k tokenów i interaktywnym setupem. opencode-ralph-wiggum — implementacja OpenCode z wyświetlaniem statusu w czasie rzeczywistym, wstrzykiwaniem kontekstu w trakcie pętli i detekcją „utknięcia" agenta. ralph (GitHub Copilot) — rozszerzenie VS Code z wizualnym Control Panel, Progress Timeline i Fresh Chat Mode. ralph-tui — Terminal UI orchestrator łączący się z task trackerami z interaktywnym tworzeniem PRD. Goose Ralph Loop Tutorial — implementacja Block's Goose z cross-model review i recipe-based workflows. ralph-loop-agent — Vercel TypeScript SDK wrapper z callbackami weryfikacji i sumaryzacją kontekstu. multi-agent-ralph-loop — orchestracja wielu agentów dla złożonych projektów wymagających równoległych strumieni pracy. 2. DOGŁĘBNA ANALIZA: mini-swe-agent (Princeton/Stanford)
Repozytorium: github.com/SWE-agent/mini-swe-agent Licencja: MIT Język: Python Rozmiar kodu rdzenia: ~100 linii klasy agenta

2.1 Filozofia
22
Pytanie postawione przez twórców: Co jeśli SWE-agent byłby 100x mniejszy i nadal działał prawie równie dobrze? Minimalny: zaledwie 100 linii Pythona (plus ok. 100 łącznie na środowisko, model, skrypt) — bez wymyślnych zależności! Wydajny: rozwiązuje 65% GitHub issues w benchmarku SWE-bench verified (z Claude Sonnet 4). Wygodny: posiada interfejsy UI. Wdrażalny: oprócz lokalnych envów, obsługuje docker, podman, singularity, apptainer i inne.
Nowsza wersja (v2) osiąga jeszcze wyższe wyniki:
26
agent w 100 liniach osiągający >74% na SWE-bench verified. Performant: startuje znacznie szybciej niż Claude Code. Widely adopted: w użyciu przez Meta, NVIDIA, Essential AI, Anyscale i innych.

2.2 Kluczowe decyzje architektoniczne
22
Mini-SWE-agent nie posiada żadnych narzędzi oprócz basha — nawet nie używa interfejsu tool-calling LLM-ów. Zamiast implementować customowe narzędzia dla każdej specyficznej rzeczy, skupia się w pełni na tym, by LM wykorzystywał shell do pełni potencjału. Chcesz żeby zrobił coś konkretnego jak otwarcie PR? Po prostu powiedz LM-owi żeby sam to wykombinował zamiast spędzać czas na implementowaniu w agencie. Wykonuje akcje przez subprocess.run — każda akcja jest kompletnie niezależna (w przeciwieństwie do utrzymywania stanowej sesji shella). To jest wielka sprawa dla stabilności agenta. Posiada kompletnie liniową historię — każdy krok agenta po prostu dołącza do wiadomości które są przekazywane LM-owi w następnym kroku i to tyle. To jest świetne do debugowania i rozumienia czym LM jest promptowany.
2.3 Struktura kodu rdzenia (default.py)
Z analizy pliku src/minisweagent/agents/default.py:

30
Podstawowa klasa agenta. Posiada `__init__(self, model: Model, env: Environment, *, config_class: type = AgentConfig, **kwargs)`. Metoda `run()` — uruchamia `step()` aż agent zakończy. Zwraca słownik z kluczami exit_status i submission. Ustawia `extra_template_vars` z task i kwargs. Formatuje wiadomości: system (render_template z system_template), user (render_template z instance_template). Wykonuje: `outputs = [self.env.execute(action) for action in message.get("extra", {}).get("actions", [])]`. Serializacja: `recursive_merge(agent_data, self.model.serialize(), self.env.serialize(), *extra_dicts)`. Zapisuje trajektorię agenta do pliku.
Pseudokod fundamentalnej pętli:

text

class DefaultAgent:
def **init**(self, model, env, config):
self.model = model
self.env = env
self.messages = []

    def run(self, task):
        # Inicjalizacja z system prompt + task
        self.messages.append(system_message(self.config.system_template))
        self.messages.append(user_message(task))

        while not self.is_finished():
            # Krok 1: LLM generuje odpowiedź
            response = self.model.query(self.messages)

            # Krok 2: Wyekstrahuj akcje (bash commands)
            actions = response.get("extra", {}).get("actions", [])

            # Krok 3: Wykonaj każdą akcję niezależnie (subprocess.run)
            outputs = [self.env.execute(action) for action in actions]

            # Krok 4: Dodaj response + outputs do historii
            self.messages.append(response)
            self.messages.extend(format_observations(outputs))

        return {"exit_status": ..., "submission": ...}

2.4 Dlaczego subprocess.run zamiast persistent shell
26
Wykonuje akcje przez subprocess.run — każda akcja jest kompletnie niezależna (w przeciwieństwie do utrzymywania stanowej sesji shella). „This is a big deal for the stability of the agent, trust me."
Ta decyzja projektowa jest kluczowa: stanowy shell może się zawisnąć, zgromadzić brudny stan (env vars, working directory), a subprocess.run zawsze startuje czysto. Tradeoff: nie ma persystentnego stanu między komendami (np. cd nie przenosi się), ale agent musi po prostu używać pełnych ścieżek.

2.5 Model integration
27
Obsługuje wszystkie modele przez litellm, openrouter, portkey i więcej. Wsparcie dla endpointów /completion i /response, interleaved thinking itd.
Użycie LitellmModel jako wrappera pozwala przełączać się między dostawcami bez zmiany kodu agenta:

Python

agent = DefaultAgent(
LitellmModel(model_name="provider/model-name"),
LocalEnvironment(),
)
agent.run("Fix the bug in auth module")
2.6 Wnioski dla implementacji self-improving agenta
Lekcja z mini-swe-agent:
28
Liniowa historia i prosty control flow czynią mini-SWE-agent szczególnie dobrze nadającym się do fine-tuningu i reinforcement learning. Przy trenowaniu modeli do pracy z agentami AI, chcesz unikać overfittingu do specyficznych zachowań frameworka. Z mini-SWE-agent, sygnał treningowy pochodzi bezpośrednio z zadania a nie z nawigowania złożonym frameworkiem agenta. To prowadzi do modeli, które generalizują lepiej między środowiskami i zadaniami. Efektywność pochodzi ze skupienia na tym co naprawdę ważne: interakcji między modelem językowym a środowiskiem deweloperskim. Przez usunięcie niepotrzebnych abstrakcji i wykorzystanie istniejącej wiedzy modelu o komendach shell.

3. DOGŁĘBNA ANALIZA: ralphy (michaelshimeles)
   Repozytorium: github.com/michaelshimeles/ralphy Licencja: MIT Język: Bash + npm CLI Stars: ~2.5k (stan: marzec 2026)

3.1 Architektura
3
Autonomiczny skrypt bashowy uruchamiający Claude Code, Codex, OpenCode, Cursor agent, Qwen i Droid w pętli aż PRD jest kompletny.
Obsługiwane silniki:

7
ralphy (Claude Code domyślny), --opencode (OpenCode), --cursor (Cursor), --codex (Codex), --qwen (Qwen-Code), --droid (Factory Droid), --copilot (GitHub Copilot), --gemini (Gemini CLI). Obsługuje przekazywanie argumentów specyficznych dla silnika przez separator `--`, np. `ralphy --copilot --model "claude-opus-4.5" --prd PRD.md -- --allow-all-tools`.
3.2 System konfiguracji (.ralphy/config.yaml)
2
Konfiguracja Ralphy obejmuje: project (name, language, framework, description) — auto-wykrywane. Commands (test, lint, build) — auto-wykrywane z package.json/pyproject.toml. Rules — instrukcje które AI MUSI przestrzegać, wstrzykiwane do każdego promptu.
3
Przykładowe rules: „use server actions not API routes", „follow error pattern in src/utils/errors.ts". Boundaries → never_touch: lista plików/wzorców których agent NIE MOŻE modyfikować (np. „src/legacy/\*_", „_.lock"). Rules aplikują się do wszystkich tasków.
3.3 Parallel Execution
1
Agent 1 → /tmp/xxx/agent-1 → ralphy/agent-1-create-auth. Agent 2 → /tmp/xxx/agent-2 → ralphy/agent-2-add-dashboard. Agent 3 → /tmp/xxx/agent-3 → ralphy/agent-3-build-api. Bez --create-pr: auto-merge z powrotem do base branch, AI rozwiązuje konflikty. Z --create-pr: zachowuje branche, tworzy PR. Z --no-merge: zachowuje branche bez mergowania.
1
Taski mogą mieć parallel_group (np. group: 1 = runs together, group: 2 = runs after group 1). --branch-per-task tworzy branch per task. Agent-browser do automatyzacji interakcji przeglądarkowych.
3.4 Ochrona plików i stabilność
3
Agenty nie powinny modyfikować plików PRD, .ralphy/progress.txt, .ralphy-worktrees, ani .ralphy-sandboxes.
3
Gdy silnik zwraca niezerowy exit code, ralphy dołącza ostatnie linie outputu CLI do komunikatu błędu. Naprawiono nieskończoną pętlę retry: taski teraz prawidłowo przerywają się przy fatalnych błędach konfiguracji/uwierzytelniania.
3.5 PRD jako źródło prawdy
4
To jest przykładowy PRD (Product Requirements Document) w formacie Markdown. Ralphy wykonuje każdy niezaznaczony task sekwencyjnie używając wybranego silnika AI. Dziel duże zadania na mikro-zadania. Mniejsze zadania = lepsza jakość kodu.
3.6 Webhook notifications i sandbox mode
1
Prompty jawnie zabraniają edycji PRD i plików .ralphy. Sandbox mode: lekka izolacja używająca symlinków dla zależności (szybsza niż worktrees). Ulepszenia wydajności: cachowanie tasków, równoległa analiza merge, inteligentna kolejność branchy. Webhook notifications: Discord, Slack i custom webhooks dla zakończenia sesji (konfiguracja w .ralphy/config.yaml). Engine-specific arguments przez separator `--`.
3.7 Integracyjne branche (z kodu ralphy.sh)
2
Tworzy integracyjne branche per parallel_group (np. `ralphy/integration-group-$group`). Przy niepowodzeniu merge'a, loguje ostrzeżenie i kontynuuje od bieżącego BASE_BRANCH. 4. DOGŁĘBNA ANALIZA: ralph (iannuttall)
Repozytorium: github.com/iannuttall/ralph Licencja: MIT Język: Node.js (npm)

4.1 Filozofia — minimalizm
11
Ralph to minimalna, plikowa pętla agenta dla autonomicznego kodowania. Każda iteracja startuje od zera, czyta ten sam stan z dysku i commituje pracę dla jednej story na raz. Ralph traktuje pliki i git jako pamięć, nie kontekst modelu.
4.2 Struktura stanu
11
PRD (JSON) definiuje stories, gates i status. Loop wykonuje jedną story per iterację. Stan persistuje w .ralph/.
Instalacja i użycie:

11
Instalacja globalna: `npm i -g @iannuttall/ralph`. Komendy: `ralph prd` (uruchamia interaktywny prompt), `ralph build 1` (jedno uruchomienie Ralpha). Ralph szuka szablonów w kolejności: .agents/ralph/ w bieżącym projekcie → bundlowane domyślne szablony. `ralph install` — tworzy .agents/ralph/ w bieżącym repo do customizacji promptów i zachowania pętli.
4.3 Mechanizm wznowienia po crash'u
11
Jeśli pętla crashnie i story zostaje w stanie in_progress, można ustawić STALE_SECONDS w .agents/ralph/config.sh, aby Ralph automatycznie reopenował zatrzymane stories.
4.4 Wieloagentowość
11
Konfiguracja agentów: `AGENT_CMD="codex exec --yolo -"`, `AGENT_CMD="claude -p --dangerously-skip-permissions \"$(cat {prompt})\""`, `AGENT_CMD="droid exec --skip-permissions-unsafe -f {prompt}"`, `AGENT_CMD="opencode run \"$(cat {prompt})\""`. Flagi per uruchomienie: `ralph build 1 --agent=codex`, `ralph build 1 --agent=claude`.
11
OpenCode server mode: dla szybszej wydajności z OpenCode, uruchom `opencode serve` w osobnym terminalu i odkomentuj linie AGENT_OPENCODE_CMD w .agents/ralph/agents.sh aby użyć --attach http://localhost:4096.
4.5 Skills system
11
Skills instalowane: commit, dev-browser, prd. Jeśli pominięto skills podczas ralph install, można uruchomić `ralph install --skills` w dowolnym momencie. 5. DOGŁĘBNA ANALIZA: snarktank/ralph
Repozytorium: github.com/snarktank/ralph

5.1 Architektura
12
Ralph to autonomiczna pętla agenta AI uruchamiająca AI coding tools (Amp lub Claude Code) wielokrotnie aż wszystkie elementy PRD są kompletne. Każda iteracja to świeża instancja z czystym kontekstem. Pamięć persistuje przez git history, progress.txt i prd.json. Bazuje na wzorcu Ralpha Geoffreya Huntleya.
5.2 Automatyczny handoff kontekstu
12
Umożliwia automatyczny handoff gdy kontekst się zapełni, pozwalając Ralphowi obsługiwać duże stories które przekraczają jedno okno kontekstu.
5.3 Mechanizm zakończenia
12
Gdy wszystkie stories mają passes: true, Ralph outputuje `<promise>COMPLETE</promise>` i pętla się kończy.
5.4 Uczenie się między iteracjami (AGENTS.md)
12
Po każdej iteracji, Ralph aktualizuje odpowiednie pliki AGENTS.md z learningami. To jest kluczowe, ponieważ AI coding tools automatycznie czytają te pliki, więc przyszłe iteracje (i przyszli ludzcy deweloperzy) korzystają z odkrytych wzorców, gotchas i konwencji.
5.5 Kanały pamięci
12
Jedyną pamięcią między iteracjami jest: [progress.txt + prd.json + git history + AGENTS.md]. Każdy element PRD powinien być wystarczająco mały by zakończyć się w jednym oknie kontekstu. Jeśli task jest zbyt duży, LLM wyczerpuje kontekst przed zakończeniem i produkuje słaby kod. 6. DOGŁĘBNA ANALIZA: ralph-orchestrator (mikeyobrien)
Repozytorium: github.com/mikeyobrien/ralph-orchestrator Język: Rust (backend API) + TypeScript (frontend)

6.1 Architektura
16
Orkiestrator oparty na „hat system" utrzymujący agentów AI w pętli aż zadanie jest wykonane. Ulepszona implementacja techniki Ralph Wiggum dla autonomicznej orchestracji agentów AI.
6.2 MCP Server i multi-workspace
16
Dla użycia multi-repo, uruchom jedną instancję MCP serwera na repo/workspace. Obecne API control-plane Ralpha persistują konfigurację, taski, pętle, sesje planowania i kolekcje pod jednym workspace root, więc server-per-workspace to deterministyczny model.
6.3 Telegram bot — human-in-the-loop
16
RObot: Telegram bot integration. Agent questions — agenty emitują eventy human.interact; pętla blokuje się aż odpowiedź nadejdzie lub timeout. Proactive guidance — wysyłaj wiadomości w dowolnym momencie by kierować agentem w trakcie pętli. Parallel loop routing — wiadomości routowane przez reply-to, @loop-id prefix, lub domyślnie do primary. Komendy Telegram: /status, /tasks, /restart dla widoczności pętli w czasie rzeczywistym.
6.4 Stack technologiczny
16
Rust RPC API (port 3000), frontend (port 5173). Ralph może działać jako MCP server over stdio dla klientów kompatybilnych z MCP. 7. DOGŁĘBNA ANALIZA: ralph-claude-code (frankbria)
Repozytorium: github.com/frankbria/ralph-claude-code

7.1 Inteligentna detekcja zakończenia (dual-condition exit gate)
17
566 testów, 100% pass rate. Autonomiczne pętle rozwojowe z inteligentną detekcją zakończenia. Dual-condition exit gate: wymaga ZARÓWNO completion indicators JAK I jawnego EXIT_SIGNAL. Rate limiting z resetem godzinowym (100 calls/hour, konfigurowalne). Circuit breaker z zaawansowaną detekcją błędów (zapobiega runaway loops). Response analyzer z semantycznym rozumieniem i dwustopniowym filtrowaniem błędów.
7.2 Flow detekcji zakończenia — szczegółowy
17
Przykład: Loop 5: Claude outputuje „Phase complete, moving to next feature" → completion_indicators: 3 (high confidence from patterns) → EXIT_SIGNAL: false (Claude mówi że jest więcej pracy) → Result: CONTINUE (respektuje jawny intent Claude'a). Loop 8: Claude outputuje „All tasks complete, project ready" → completion_indicators: 4 → EXIT_SIGNAL: true (Claude potwierdza done) → Result: EXIT z „project_complete".
7.3 Struktura plików
17
Struktura: .ralph/ → PROMPT.md (główne instrukcje), fix_plan.md (priorytetyzowana lista tasków), AGENT.md (instrukcje build/run), specs/ (specyfikacje), specs/stdlib/ (standardowa biblioteka specyfikacji), examples/, logs/, docs/generated/. .ralphrc (plik konfiguracyjny — tool permissions, settings). Migracja ze starej płaskiej struktury: `ralph-migrate`.
7.4 Hierarchia dokumentów
17
PROMPT.md (cele wysokopoziomowe) → specs/ (szczegółowe wymagania gdy potrzebne) → fix_plan.md (konkretne taski które Ralph wykonuje) → AGENT.md (komendy build/test — auto-utrzymywane). Proste projekty: PROMPT.md + fix_plan.md zazwyczaj wystarczy. Złożone features: dodaj specs/feature-name.md. Konwencje zespołu: dodaj specs/stdlib/convention-name.md dla reużywalnych wzorców. 8. DOGŁĘBNA ANALIZA: vercel-labs/ralph-loop-agent
Repozytorium: github.com/vercel-labs/ralph-loop-agent Język: TypeScript Runtime: Vercel AI SDK

8.1 Architektura dwupętlowa (inner + outer loop)
14
Architektura: Ralph Loop (outer) zawiera AI SDK Tool Loop (inner): LLM ↔ tools ↔ LLM ↔ tools ... until done. Po zakończeniu inner loop → verifyCompletion: „Is the TASK actually complete?" Nie? → Inject feedback → Run another iteration. Tak? → Return final result. Standardowe pętle narzędzi AI SDK zatrzymują się gdy model skończy tool calls.
8.2 API i konfiguracja
14
Feedback loops: prowadzą agenta na podstawie real-world checks. Long-running tasks: migracje, refaktoryzacje, zmiany wieloplikowe. Ralph owija generateText z AI SDK w outer loop który iteruje aż verifyCompletion potwierdzi sukces — lub zostanie osiągnięty limit bezpieczeństwa. Cechy: Iteracyjne zakończenie (aż verifyCompletion mówi done), pełna kompatybilność AI SDK, elastyczne warunki stopu (limit iteracji, tokenów lub kosztów), zarządzanie kontekstem (wbudowana sumaryzacja dla długich pętli), streaming support, feedback injection (nieudane weryfikacje prowadzą następną próbę).
8.3 Przykład kodu TypeScript
14
Inicjalizacja: `const agent = new RalphLoopAgent({ model: 'anthropic/claude-opus-4.5', instructions: 'You are a helpful coding assistant.', stopWhen: iterationCountIs(10), verifyCompletion: async ({ result }) => ({ complete: result.text.includes('DONE'), reason: 'Task completed successfully' }) })`. Uruchomienie: `const { text, iterations, completionReason } = await agent.loop({ prompt: 'Create a function that calculates fibonacci numbers' })`. 9. DOGŁĘBNA ANALIZA: PageAI-Pro/ralph-loop
Repozytorium: github.com/PageAI-Pro/ralph-loop

9.1 Struktura plików — najbardziej rozbudowana
13
.agent/ → PROMPT.md (Prompt wysyłany do agenta w każdej iteracji), tasks.json (lookup table tasków, wymagany), tasks/ (indywidualne specyfikacje TASK-{ID}.json), prd/ → PRD.md (dokument wymagań) + SUMMARY.md (krótki przegląd projektu wysyłany do agenta w każdej iteracji), logs/ → LOG.md (log progresu, auto-tworzony), history/ (logi outputu per iteracja), skills/ (współdzielone skille).
9.2 Weryfikacja — wymóg testów
13
Żeby AI faktycznie weryfikowało swoją implementację i pętla działała, potrzebujesz sposobu weryfikacji. Minimum: framework testów end-to-end i framework testów jednostkowych. Przykład: `npm i @playwright/test vitest jsdom typescript eslint prettier -D`.
9.3 STEERING.md — sterowanie w trakcie pętli
13
Zapewnia obserwowalność i śledzenie pracy agenta, pokazując strumień outputu i pełne logi historyczne per iteracja. W niektórych przypadkach agent może mieć problemy lub utknąć. Podczas gdy pętla działa, można edytować plik .agent/STEERING.md aby dodać krytyczną pracę do wykonania przed kontynuowaniem pętli.
9.4 Docker sandbox
13
Zamień `docker sandbox run claude .` na swoje ulubione CLI. Docker obecnie wspiera: claude, codex, opencode, copilot, gemini, cagent, kiro i więcej.
9.5 Skalowalne zadania
13
Steering — pozwala priorytetyzować krytyczną pracę. Używa task lookup table z indywidualnymi szczegółowymi krokami → bardziej skalowalne przy setkach wykonanych tasków. 10. DEEP AGENTS (LangChain) — ARCHITEKTURA HARNESS'u
Repozytorium: github.com/langchain-ai/deepagents Język: Python Licencja: MIT Instalacja: pip install deepagents

10.1 Geneza i filozofia
41
Projekt inspirowany głównie Claude Code — początkowo próba zrozumienia co czyni Claude Code uniwersalnym i uczynienia tego jeszcze bardziej uniwersalnym. Deep Agents podąża za modelem „trust the LLM". Agent może robić wszystko na co pozwalają narzędzia. Egzekwuj granice na poziomie narzędzi/sandboxa, nie oczekując że model sam się policjonuje.
10.2 Co to jest „Deep Agent"
42
Użycie LLM do wywoływania narzędzi w pętli to najprostsza forma agenta. Ta architektura jednak może produkować agenty które są „płytkie" i nie radzą sobie z planowaniem i działaniem przy dłuższych, bardziej złożonych zadaniach.
49
Użycie LLM do wywoływania narzędzi w pętli to najprostsza forma agenta. Ta architektura może produkować agentów „płytkich", którzy nie potrafią planować długoterminowo. Aplikacje jak „Deep Research", „Manus" i „Claude Code" obeszły to ograniczenie implementując kombinację czterech rzeczy: planning tool, sub agents, dostęp do file system i szczegółowy prompt. deepagents to pakiet Pythona implementujący to w sposób ogólnego przeznaczenia.
10.3 Cztery filary architektury
Planning Tool (write_todos):
42
Claude Code używa Todo list tool. Zabawne jest to, że to nic nie robi! To w zasadzie no-op. To strategia context engineering żeby utrzymać agenta na kursie. Deep agents radzą sobie lepiej z wykonywaniem złożonych zadań w dłuższych horyzontach czasowych. Planowanie (nawet przez no-op tool call) jest dużym komponentem tego.
43
deepagents zawiera wbudowane narzędzie write_todos umożliwiające agentom rozbijanie złożonych zadań na dyskretne kroki, śledzenie progresu i adaptowanie planów gdy pojawiają się nowe informacje.
File System:
42
Claude Code ma dostęp do systemu plików i może modyfikować pliki tam, nie tylko do wykonania zadania ale też do notowania. Działa też jako współdzielona przestrzeń robocza dla wszystkich agentów (i subagentów) do współpracy. Manus to kolejny przykład deep agenta znacząco wykorzystującego system plików jako „pamięć". Deep agents działają przez długie okresy i akumulują dużo kontekstu. Posiadanie systemu plików do przechowywania (i późniejszego odczytywania) pomaga w zarządzaniu kontekstem.
43
Narzędzia systemu plików (ls, read_file, write_file, edit_file) pozwalają agentom odciążyć duży kontekst do pamięci lub systemu plików, zapobiegając przepełnieniu okna kontekstu i umożliwiając pracę ze zmienno-długościowymi wynikami narzędzi.
Sub-agents:
42
Claude Code może spawnować subagentów. To pozwala dzielić zadania. Można też tworzyć customowe subagenty dla większej kontroli.
Detailed System Prompt:
42
Zrekonstruowane system prompty Claude Code są długie. Zawierają szczegółowe instrukcje użycia narzędzi. Zawierają examples (few shot prompts) jak zachowywać się w pewnych sytuacjach. Claude Code nie jest anomalią — większość najlepszych coding lub deep research agentów ma dość złożone system prompty. Bez tych system promptów, agenty nie byłyby nawet w przybliżeniu tak „głębokie". Prompting nadal ma znaczenie!
10.4 Context Management
45
Zarządzanie kontekstem jest krytyczne dla długo działających agentów i trudne do prawidłowego wykonania. Deep Agents zawiera middleware pomagające agentom kompresować historię konwersacji, odciążać duże wyniki narzędzi, izolować kontekst z subagentami i używać prompt caching do redukcji latencji i kosztów.
10.5 Minimalny kod do uruchomienia
43
Dwie linie do startu: `from deepagents import create_deep_agent`, `agent = create_deep_agent(tools=[get_weather], system_prompt="You are a helpful assistant")`. Uruchomienie: `agent.invoke({"messages": [{"role": "user", "content": "what is the weather in sf"}]})`.
44
DeepAgents to po prostu skompilowany graf LangGraph. To oznacza pełną kompatybilność z LangGraph checkpointerami, streamingiem, human-in-the-loop.
10.6 Custom subagents (zaawansowane)
49
Dla bardziej złożonych use case'ów, można dostarczyć własny, pre-built graf LangGraph jako subagenta. Domyślnie deepagents używa „claude-sonnet-4-20250514". Można customizować przekazując dowolny obiekt modelu LangChain.
10.7 Persistent Memory
43
Rozszerzanie agentów o persystentną pamięć między wątkami za pomocą LangGraph Memory Store. Agenty mogą zapisywać i odczytywać informacje z poprzednich konwersacji. 11. ARTYKUŁ NAUKOWY: SICA — A Self-Improving Coding Agent (ICLR 2025)
Źródło: arxiv.org/abs/2504.15228 Autorzy: Maxime Robeyns et al. Venue: ICLR 2025 Workshop SSI-FM

11.1 Teza główna
34

40
LLM-y są często wdrażane w systemach agentowych: kodzie który orchestruje wywołania LLM i dostarcza im narzędzia. Wykazano że system agentowy, wyposażony w podstawowe narzędzia kodowania, może autonomicznie edytować samego siebie, i przez to poprawić swoją wydajność na zadaniach benchmarkowych. Odnotowano wzrost wydajności od 17% do 53% na losowym podzbiorze SWE Bench Verified.
11.2 Konfiguracja eksperymentalna
31
Bazowy system agentowy na iteracji 0 używał Sonnet 3.5 z „reasoning" subagenta na o3-mini. Sam o3-mini osiąga 87% i 79% na AIME i GPQA Diamond z „high" reasoning effort, podczas gdy system agentowy jako całość osiągał inne wyniki.
11.3 Kluczowe odkrycia — ewolucja narzędzi
31
Dla benchmarku edycji plików, agent był początkowo wyposażony jedynie w możliwość nadpisywania plików co jest wolne i nieefektywne tokenowo, i szybko zaimplementował „inteligentne" narzędzia edycji oparte na diffach i zakresach. Dla zadania nawigacji symboli, agent zaimplementował prosty lokalizator symboli oparty na AST na iteracji 9 (prawdopodobnie eksploatując fakt że benchmarki były głównie Pythonowe), co przyniosło dobre ulepszenia widoczne też w innych zadaniach.
11.4 Ograniczenia i bezpieczeństwo
31
Wydaje się że istnieją limity tego, co można osiągnąć z czysto scaffolding-owymi systemami, które nie modyfikują wag modelu. Gdyby system aktualizował wagi modelu jak i scaffolding, oczekiwalibyśmy że ryzyko rekurencyjnie samodoskonalących się systemów byłoby wyższe. Kluczowa mitygacja bezpieczeństwa: obserwowalność — ludzie mogą i nadzorują wszystkie aspekty potencjalnie samodoskonalących się systemów. SICA została zaprojektowana z wyrafinowanymi lecz przyjaznymi mechanizmami obserwowalności, dostarczającymi bogaty wgląd w chain-of-thought, akcje i wywołania subagentów na każdym kroku. Obejmuje interaktywny interfejs webowy oraz asynchronicznego overseera opartego na LLM.
11.5 Interferencja reasoning models
31
Dla „reasoning models" takich jak o1, o3-mini czy DeepSeek R1, podejrzewamy że włączenie prymitywnych komponentów mających indukować zachowanie rozumowania (takie jak te dołączone w iteracjach 4 i 6) może w rzeczywistości przerywać łańcuch rozumowania modelu. 12. ARTYKUŁ NAUKOWY: EvoMAC — Self-Evolving Multi-Agent (ICLR 2025)
Venue: ICLR 2025 (Conference Paper)

12.1 Architektura — trzy zespoły
32
Według ogólnego paradygmatu, EvoMAC dla software development obejmuje trzy zasadnicze komponenty: i) adaptowalna sieć MAC (Multi-Agent Collaboration) oparta na zespole kodującym, generującym kod przez feed-forward; ii) specjalnie zaprojektowany testing team tworzący unit test cases służące jako target proxy i weryfikujący generowany kod w kompilatorze do produkcji obiektywnego feedbacku; iii) updating team używający algorytmu textual back-propagation do aktualizacji coding teamu.
12.2 Tekstowa Back-Propagation
32
Rozwiązanie oparte na tekstowej backpropagacji. Rozwiązanie self-evolution iteracyjnie aktualizuje sieć MAC używając algorytmu tekstowej backpropagacji, prowadzonej przez feedback środowiskowy. Główna idea to analiza wpływu każdego agenta w sieci MAC na końcowy feedback środowiskowy i użycie tych analiz do aktualizacji promptów agenta i agentic workflow.
12.3 Dlaczego nie self-critique
32
Inny sposób: strategia self-critique, która używa agenta krytyka do bezpośredniej oceny outputu. To podejście ma dwa inherentne ograniczenia: i) krytyka może być subiektywna i stronnicza, ii) agent krytyk może mieć halucynacje, powodując niespójności i błędy. Te ograniczenia mogą powodować że sieć MAC stanie się okopana we własnych preferencjach lub ewoluuje w złym kierunku, szczególnie przy wielokrotnej iteracji. W porównaniu, podejście EvoMAC wykorzystuje executor środowiskowy (kompilator) do dostarczania obiektywnego feedbacku, zapobiegając stronniczości i halucynacjom.
Kluczowy wniosek dla implementacji: Weryfikacja przez kompilator/testy > weryfikacja przez LLM-as-judge. Obiektywny feedback (testy przechodzą / nie przechodzą) jest bardziej niezawodny niż subiektywna ocena innego LLM-a.

13. ARTYKUŁ NAUKOWY: SWE-Search — MCTS dla SE (ICLR 2025)
    Venue: ICLR 2025 (Conference Paper)

13.1 Architektura — trzy agenty + MCTS
35
Value (Function) Agent: do przybliżania użyteczności każdej obserwacji, używają value function opartej na LLM, która oprócz outputowania wartości generuje też wyjaśnienie w języku naturalnym. To wyjaśnienie może być wykorzystane do poprawienia następnych akcji z parent nodes, umożliwiając iteracyjne samodoskonalenie procesu wyszukiwania. Search Algorithm: rdzeń strategii eksploracji SWE-Search oparty na Monte Carlo Tree Search (MCTS) z heurystycznym procesem selekcji podobnym do AlphaZero, specjalnie dostosowanym do zadań software engineering. Balansuje eksplorację i eksploatację. Discriminator Agent: w końcowej fazie ocenia rozwiązania wygenerowane przez proces wyszukiwania.
13.2 Ewolucja stanów repozytorium
35
Dynamika środowiska zdefiniowane przez context-dependent transition function P: S × A × C → Δ(S), modelującą ewolucję stanu repozytorium po każdej akcji. Rozkład stanów początkowych p0: C → Δ(S), policy π: S × C → Δ(A) — selekcjonuje akcje na podstawie bieżącego stanu i kontekstu.
13.3 Moatless-tools — revert capability
35
Moatless-Adapted to rozszerzona wersja biblioteki moatless-tools ze wsparciem dla struktury drzewiastej, możliwością revertowania do wcześniejszych wersji codebase i zdolnością uruchamiania testów. Standardowa implementacja moatless-tools oparta na maszynie stanowej gdzie stan trzyma informację o kontekście pliku i właściwościach. 14. HARNESS ENGINEERING — WNIOSKI Z LANGCHAIN
Źródło: blog.langchain.com/improving-deep-agents-with-harness-engineering/ (luty 2026)

14.1 Wyniki — 13.7-punktowy skok
48
Użyto prostego przepisu do iteracyjnego ulepszenia deepagents-cli (coding agent) o 13.7 punktów z 52.8 do 66.5 na Terminal Bench 2.0. Jedynie modyfikowano harness, model pozostał stały (gpt-5.2-codex). Terminal Bench 2.0 to standardowy benchmark do ewaluacji agentycznego kodowania — 89 zadań z ML, debuggingu, biologii. Harbor orchestruje uruchomienia — spinuje sandboxes (Daytona), interaguje z pętlą agenta, uruchamia weryfikację + scoring. Każda akcja agenta przechowywana w LangSmith z metrykami: latencja, tokeny, koszty.
14.2 Trzy kluczowe pokrętła optymalizacji
48
Harness agenta ma wiele pokręteł: system prompts, tools, hooks/middleware, skills, delegacja do subagentów, systemy pamięci i więcej. Celowo kompresują przestrzeń optymalizacji i skupiają się na trzech: System Prompt, Tools i Middleware (ich termin na hooks wokół wywołań modelu i narzędzi).
14.3 Zasady ogólne
48
Context Engineering w Imieniu Agentów. Kompilowanie kontekstu nadal jest trudne dla agentów, szczególnie w nieznanych środowiskach. Onboarding modeli z kontekstem takim jak struktury katalogów, dostępne narzędzia, najlepsze praktyki kodowania i strategie rozwiązywania problemów pomaga zmniejszyć powierzchnię błędów. Pomóż agentom samoweryfikować swoją pracę. Modele są stronnicze ku pierwszemu wiarygodnemu rozwiązaniu.
14.4 Multi-model systems i przyszłość
48
Jest więcej otwartego researchu do zrobienia w projektowaniu harness'ów. Interesujące kierunki obejmują systemy wielomodelowe (Codex, Gemini i Claude razem), prymitywy pamięciowe dla ciągłego uczenia żeby agenty mogły autonomicznie poprawiać się na zadaniach, oraz mierzenie zmian harness'a między modelami. Dla outer loop ulepszania agentów, badają metody jak RLMs do efektywniejszego miningu traces.
48
Naturalnym podejściem dla modeli jest Adaptive Reasoning — model decyduje ile compute poświęcić na rozumowanie. W multi-model harness, balansowanie budżetów rozumowania mogłoby wyglądać jak użycie dużego modelu do planowania i przekazanie mniejszemu modelowi implementacji. Przestrzeń projektowa agentów jest duża.
14.5 Tailoring do modelu
48
Test z Claude Opus 4.6 osiągnął 59.6% z wcześniejszą wersją harness'a, konkurencyjny ale gorszy niż Codex bo nie uruchamiano tego samego Improvement Loop z Claude. Wiele zasad generalizuje się jak dobra preparacja kontekstu i fokus na weryfikacji, ale uruchomienie kilku rund iteracji harness'a dla danego zadania pomaga zmaksymalizować wydajność agenta. 15. SURVEY: Awesome-Self-Evolving-Agents (EvoAgentX)
Repozytorium: github.com/EvoAgentX/Awesome-Self-Evolving-Agents Artykuł: arxiv.org/abs/2508.07407

33
Wizualna taksonomia technik ewolucji i optymalizacji agentów AI, skategoryzowana w trzy główne kierunki: optymalizacja single-agent, optymalizacja multi-agent i optymalizacja domain-specific. Struktura drzewiasta ilustruje rozwój tych podejść od 2023 do 2025.
15.1 Kluczowe projekty z survey
33
Narzędzia i frameworki: EvoAgentX (EMNLP'25 Demo) — Automated Framework for Evolving Agentic Workflows. MASLab — Unified Codebase for LLM-based Multi-Agent Systems. ToRA (ICLR'24) — tool-integrated reasoning agent for mathematical problem solving. STaR (NeurIPS'22) — Bootstrapping reasoning with reasoning.
33
Nowe paradygmaty: Alita — Generalist Agent Enabling Scalable Agentic Reasoning with Minimal Predefinition and Maximal Self-Evolution. Building Self-Evolving Agents via Experience-Driven Lifelong Learning. EvoAgent — Self-evolving Agent with Continual World Model for Long-Horizon Tasks. MetaAgent — Automatically Constructing Multi-Agent Systems Based on Finite State Machines. MCP-Zero — Active Tool Discovery for Autonomous LLM Agents. 16. WZORCE IMPLEMENTACYJNE WYEKSTRAHOWANE Z KODU
Na podstawie analizy kodu wszystkich powyższych repozytoriów, wyekstrahowane zostały następujące wzorce implementacyjne:

Wzorzec 1: Liniowa Historia (mini-swe-agent)
text

messages = [system_prompt, user_task]
while not done:
response = llm(messages)
actions = extract_actions(response)
outputs = [subprocess.run(a) for a in actions]
messages.append(response)
messages.append(format_outputs(outputs))
Zalety: debugowalność, przewidywalność, brak ukrytego stanu.

Wzorzec 2: Outer/Inner Loop (vercel ralph-loop-agent)
text

outer_iteration = 0
while outer_iteration < max_iterations: # Inner loop: LLM ↔ tools ↔ LLM ↔ tools (AI SDK)
result = generateText(model, tools, task + feedback)

    # Verify
    verification = verifyCompletion(result)
    if verification.complete:
        return result
    else:
        feedback = verification.reason
        outer_iteration += 1

Zalety: separacja koncepcji generowania i weryfikacji.

Wzorzec 3: File-Based Memory (iannuttall/ralph)
text

# Stan w plikach, nie w kontekście modelu

progress = read_json("prd.json")
for story in progress.stories:
if story.status == "pending":
story.status = "in_progress"
save_json("prd.json", progress)

        result = run_agent(story)

        if result.passes:
            story.status = "done"
            git_commit(story.title)
        else:
            story.status = "failed"
            log_learning(story, result.error)

Wzorzec 4: Dual-Condition Exit Gate (frankbria/ralph-claude-code)
text

# Nie wystarczy jedno — wymagane OBA warunki

completion_indicators = count_completion_patterns(output)
exit_signal = parse_explicit_exit(output)

if completion_indicators >= threshold AND exit_signal == True:
exit("project_complete")
elif completion_indicators >= threshold AND exit_signal == False:
continue # Respektuj jawny intent LLM
else:
continue
Wzorzec 5: STEERING.md — Human Override w Trakcie Pętli (PageAI-Pro)
text

# W każdej iteracji:

steering = read_file(".agent/STEERING.md")
if steering.has_content():
prepend_to_prompt(steering) # Agent priorytetyzuje steering nad normalnym taskiem
Wzorzec 6: AGENTS.md jako Cross-Iteration Learning (snarktank/ralph)
text

# Po każdej iteracji:

learnings = extract_learnings(result, test_output, errors)
append_to_file("AGENTS.md", learnings)

# W każdej iteracji (automatycznie przez CLI):

# Claude/Amp czytają AGENTS.md przed startem

# → Kumulacja wiedzy między iteracjami

Wzorzec 7: Textual Back-Propagation (EvoMAC)
text

# Zamiast LLM-as-Judge (halucynacje, bias):

code = coding_team.generate(task)
tests = testing_team.generate_tests(task)
feedback = compiler.verify(code, tests) # OBIEKTYWNY feedback

if not feedback.passes: # Analiza: który agent w sieci spowodował błąd?
blame = analyze_influence(coding_team, feedback)
updating_team.update_prompts(coding_team, blame, feedback) # → Następna iteracja ze zaktualizowanymi promptami
Wzorzec 8: Deep Agent z write_todos no-op (LangChain)
text

@tool
def write_todos(todos: list[str]) -> str:
"""Plan your tasks.""" # To narzędzie NIC NIE ROBI — jest no-op! # Ale wymusza na LLM-ie artykułowanie planu # co poprawia jakość execution
return "Todos recorded."

# LLM wywołuje write_todos PRZED implementacją

# → „Plan" ląduje w kontekście → lepsze wyniki

17. ZESTAWIENIE PORÓWNAWCZE ARCHITEKTUR
    Cecha mini-swe-agent ralphy iannuttall/ralph snarktank/ralph ralph-orchestrator frankbria/ralph vercel/ralph-loop PageAI-Pro/ralph Deep Agents
    Język Python Bash/npm Node.js Bash Rust+TS Bash TypeScript Bash Python
    LOC rdzenia ~100 ~2000 ~500 ~300 ~5000+ ~1000 ~300 ~500 ~1000
    Parallel execution ❌ ✅ Worktrees+Sandbox ❌ ❌ ✅ Parallel loops ❌ ❌ ❌ ✅ Subagents
    Checkpoint/Resume ❌ ✅ progress.txt ✅ STALE_SECONDS ✅ progress.txt+prd.json ✅ MCP state ❌ (circuit breaker) ❌ ✅ LOG.md ✅ LangGraph checkpointer
    Error Recovery Liniowa historia ✅ Auth/rate limit detection ✅ STALE_SECONDS reopen ❌ ✅ Telegram alerts ✅ Circuit breaker ✅ Feedback injection ✅ STEERING.md ✅ Middleware
    Exit Detection Agent decides Checkbox w Markdown Story status JSON <promise>COMPLETE</promise> Hat system Dual-condition gate verifyCompletion() Task status JSON Agent decides
    Learning Memory ❌ ❌ ❌ ✅ AGENTS.md ✅ Persistent state ❌ ❌ ✅ LOG.md ✅ Memory Store
    Test Verification Bash Deleguje do CLI Deleguje do CLI Deleguje do CLI Deleguje do CLI Deleguje do CLI verifyCompletion() Wymaga Playwright/Vitest Custom tools
    Human-in-the-loop ❌ ❌ ❌ ❌ ✅ Telegram ❌ ❌ ✅ STEERING.md ✅ LangGraph HITL
    Multi-model ✅ LiteLLM ✅ 7+ engines ✅ 4 engines ✅ 2 engines ✅ Multiple backends ❌ (Claude only) ✅ AI SDK ✅ Docker backends ✅ LangChain models
    Self-improvement ❌ ❌ ❌ Partial (AGENTS.md) ❌ ❌ ❌ ❌ ❌ (ale architektura wspiera)
18. SURVEY LITERATURY NAUKOWEJ — PEŁNA LISTA
    Na podstawie Awesome-Self-Evolving-Agents (EvoAgentX):

Artykuł Venue Kluczowy Wkład
SICA: A Self-Improving Coding Agent ICLR 2025 Workshop Scaffolding self-improvement, SWE-bench +17-53%
EvoMAC: Self-Evolving MAC for Software Dev ICLR 2025 Textual back-propagation, coding+testing+updating teams
SWE-Search: MCTS for Software Engineering ICLR 2025 AlphaZero-style MCTS dla SE, value function + discriminator
AFlow: Automated Agentic Workflow Optimization ICLR 2025 Automatyczna generacja i optymalizacja workflow agentów
Promptbreeder: Self-referential self-improvement ICML Prompt evolution through self-referential mutation
Voyager: Open-ended embodied agent NeurIPS LLM jako kontroler z otwartym eksploracja
MetaGPT: Meta Programming for Multi-Agent ICLR 2024 Firma software'owa jako multi-agent system
STaR: Bootstrapping Reasoning with Reasoning NeurIPS 2022 Self-improvement reasoning przez bootstrapping
AutoCoderover: Autonomous program improvement 2024 Autonomiczne ulepszanie programów
Gödel Agent: Recursive self-improvement 2025 LangGraph + self-modification via writer-critic
MCP-Zero: Active Tool Discovery 2025 Autonomiczne odkrywanie narzędzi przez agenta
EvoAgent: Self-evolving with Continual World Model 2025 Ciągły world model dla long-horizon tasks
GEPA: Genetic-Pareto optimization OpenAI Cookbook Ewolucja promptów przez selekcję Pareto 19. INDEKS LINKÓW
Repozytoria Ralph Loop:
github.com/michaelshimeles/ralphy — główna implementacja, bash+npm, 2.5k stars
github.com/iannuttall/ralph — minimalna plikowa pętla, npm
github.com/snarktank/ralph — Amp/Claude Code, AGENTS.md learning
github.com/mikeyobrien/ralph-orchestrator — Rust orchestrator, MCP, Telegram
github.com/frankbria/ralph-claude-code — dual-condition exit gate, 566 testów
github.com/vercel-labs/ralph-loop-agent — TypeScript SDK, Vercel AI SDK
github.com/PageAI-Pro/ralph-loop — STEERING.md, Docker sandbox, task lookup
github.com/snwfdhmp/awesome-ralph — curated lista wszystkich implementacji Ralph
Agent Frameworks:
github.com/SWE-agent/mini-swe-agent — 100-line agent, >74% SWE-bench
github.com/SWE-agent/SWE-agent — pełna wersja, NeurIPS 2024
github.com/langchain-ai/deepagents — Deep Agents harness, LangGraph
github.com/langchain-ai/langgraph — graf stanowy runtime
Artykuły naukowe:
arxiv.org/abs/2504.15228 — SICA: A Self-Improving Coding Agent
proceedings.iclr.cc — EvoMAC, SWE-Search, AFlow (ICLR 2025)
github.com/EvoAgentX/Awesome-Self-Evolving-Agents — survey z taksonomią
Blogi i tutoriale:
blog.langchain.com/deep-agents/ — Deep Agents announcement
blog.langchain.com/improving-deep-agents-with-harness-engineering/ — Harness engineering
www.aihero.dev/getting-started-with-ralph — tutorial Ralph
www.thetoolnerd.com — tutorial Ralph Loop z Claude Code 20. KLUCZOWE WNIOSKI SYNTETYCZNE
Obserwacja 1 — Minimalizm wygrywa z złożonością: Mini-swe-agent udowadnia, że 100 linii Pythona z subprocess.run i liniową historią osiąga >74% na SWE-bench. Złożone frameworki nie są konieczne dla efektywności — to model językowy robi ciężką robotę. Złożoność powinna być dodawana tylko gdy rozwiązuje konkretny problem (checkpoint, parallel execution, safety).

Obserwacja 2 — Obiektywna weryfikacja > LLM-as-Judge: EvoMAC (ICLR 2025) formalnie wykazał, że self-critique przez LLM ma dwa inherentne problemy: subiektywność/stronniczość i halucynacje. Kompilator/tester jako weryfikator daje obiektywny, nie-halucinujący feedback. Testy jednostkowe i E2E powinny być pierwszorzędnymi obywatelami w pętli self-improvement.

Obserwacja 3 — Scaffolding-only self-improvement ma limity ale działa: SICA (ICLR 2025) wykazał +17-53% na SWE-bench przez samomodyfikację scaffoldingu (bez zmiany wag modelu). Agent samodzielnie wymyślił inteligentne narzędzia edycji (diff-based zamiast overwrite) i nawigator symboli (AST). Ale reasoning models (o1, R1) mogą być zakłócane przez prymitywne komponenty scaffoldingu próbujące indukować rozumowanie.

Obserwacja 4 — Cztery kanały pamięci to minimum: Wielokrotnie potwierdzane: git history + progress file + learnings/AGENTS.md + system prompt/PRD = minimalna trwała pamięć dla stabilnej pętli. snarktank/ralph, iannuttall/ralph i PageAI-Pro/ralph-loop niezależnie konwergowały na tym wzorcu.

Obserwacja 5 — write_todos jako no-op jest genialny: Deep Agents (LangChain) odkryło, że Claude Code używa narzędzia Todo List które NIC NIE ROBI — jest pure context engineering trick. Wymusza na LLM-ie artykułowanie planu w kontekście, co drastycznie poprawia jakość execution. To jest wzorzec wart replikacji.

Obserwacja 6 — Harness engineering > model switching: LangChain wykazał +13.7 punktów na Terminal Bench 2.0 przez samą modyfikację harness'a (prompts, tools, middleware) przy stałym modelu. Implikacja: zanim zmienisz model, najpierw optymalizuj harness. Ale harness powinien być tailorowany do modelu — różne modele wymagają różnego promptowania.

Obserwacja 7 — STEERING.md — live human override: PageAI-Pro/ralph-loop wprowadził wzorzec edytowalnego pliku STEERING.md czytelnego przez agenta w każdej iteracji. Pozwala na ludzką interwencję bez zatrzymywania pętli — agent priorytetyzuje instrukcje steeringowe nad normalnymi taskami.

Obserwacja 8 — Subprocess.run > persistent shell: Mini-swe-agent podkreśla: każda komenda shell powinna być niezależną egzekucją (subprocess.run), nie częścią stanowej sesji. Stanowy shell akumuluje brudny stan i jest podatny na zawieszenie. Niezależne subprocess'y są drastycznie stabilniejsze.

Koniec dokumentu. Wszystkie informacje pochodzą z publicznie dostępnych repozytoriów GitHub, artykułów ICLR 2025, blogów LangChain i Vercel Labs, zweryfikowanych w marcu 2026. Linki prowadzą do oryginalnych źródeł i mogą być użyte do dalszej eksploracji.
