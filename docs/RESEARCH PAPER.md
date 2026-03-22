📄 RESEARCH PAPER: Samodoskonalący się Agent Kodujący — Architektura, Frameworki, Wzorce i Plan Implementacji
Wersja: 1.0 Data: 2026-03-22 Przeznaczenie: Dokument referencyjny dla agenta bez dostępu do internetu

SPIS TREŚCI
Streszczenie
Stan wiedzy: Istniejące projekty Self-Improving Agents
Analiza frameworków orchestracji agentów
Wzorce architektoniczne (Patterns)
Istniejące implementacje Ralph Loop
Checkpoint & Resume — wzorce z istniejących projektów
Intelligent Error Recovery — wzorce z istniejących projektów
Self-Improvement Loop — architektura referencyjna
Bezpieczeństwo samomodyfikacji
Plan implementacji (v1.6 – v2.0)
Repozytoria referencyjne (z URLami)

1. STRESZCZENIE
   Niniejszy dokument stanowi kompletny research paper na temat budowy samodoskonalącego się agenta kodującego — systemu, który iteracyjnie ulepsza swój własny kod, scaffolding i prompty za pomocą LLM, w zamkniętej pętli z automatyczną weryfikacją (TDD), checkpointowaniem i error recovery.

Dokument został przygotowany na podstawie przeglądu:

15+ repozytoriów open-source implementujących autonomiczne pętle kodowania
8 frameworków orchestracji agentów (LangGraph, CrewAI, AutoGen, CAMEL/OASIS, itp.)
1 opublikowanego artykułu naukowego (ICLR 2025) bezpośrednio dotyczącego self-improving coding agents
Wiodących artykułów (Addy Osmani, Ryan Carson, Geoffrey Huntley) opisujących „Ralph Loop" 2. STAN WIEDZY: ISTNIEJĄCE PROJEKTY SELF-IMPROVING AGENTS
2.1 Artykuł naukowy: „A Self-Improving Coding Agent" (ICLR 2025 Workshop)
49
Wyniki wykazały, że SWE-bench Verified odnotował zauważalny wzrost wydajności w trakcie iteracji, akumulując ulepszenia frameworka agenta. Wyniki LiveCodeBench wykazały subtelną poprawę, choć nie tak wyraźną jak w innych benchmarkach.
49
Eksperyment obejmował relatywnie długohoryzontowe zadania w zestawie benchmarków, w tym SWE-bench Verified (testujące dekompozycję problemów, nawigację kodu i efektywną edycję plików — z losowego podzbioru 50 pytań), a także 50 losowych pytań z LiveCodeBench, przypominających competitive programming. Dodatkowo zawierał dwa syntetyczne zadania benchmarkowe, zdefiniowane wewnątrz codebase agenta — w tym benchmark edycji plików skonstruowany przez klonowanie repozytoriów SWE-bench, znajdowanie „interesujących" edycji w kolejnych commitach i montowanie repozytorium.
Kluczowe wnioski z artykułu:

49
Framework oferuje zarówno praktyczne podejście do automatyzowanego projektowania systemów agentowych, jak i fundament dla przyszłych prac nad wspólnym treningiem modeli bazowych z ich systemami agentowymi.
49
Należy starannie rozważyć implikacje bezpieczeństwa systemów zaprojektowanych do samodoskonalenia. Chociaż agent wykazuje poprawę w zadaniach agentowych, istnieją ograniczenia tego, co można osiągnąć w systemach opartych wyłącznie na scaffoldingu (bez modyfikacji wag modelu). Gdyby system aktualizował wagi modelu oprócz scaffoldingu, ryzyko rekurencyjnie samodoskonalących się systemów byłoby wyższe.
2.2 „Self-Improving Coding Agents" — Addy Osmani (2025)
22
Osmani opisuje wizję autonomicznych agentów AI, którzy mogą kodować, testować i dostarczać kod podczas snu dewelopera. Artykuł obejmuje konfigurację pętli samodoskonalących się agentów — od orchestracji po pliki kontekstowe, persystencję pamięci, walidację QA, skalowanie, debugowanie i zarządzanie ryzykiem.
22
Sercem tego podejścia jest iteracyjna pętla agenta, często nazywana techniką „Ralph Wiggum" (spopularyzowana przez Geoffreya Huntleya i osoby takie jak Ryan Carson).
Wzorce pamięci między iteracjami (4 kanały):

22
Oprócz AGENTS.md, solidne pętle agentów używają kilku mechanizmów persystencji do zachowania stanu i unikania zapominania między iteracjami. Implementacja Ralph Loop Carsona używa co najmniej czterech kanałów pamięci: Git Commit History — zmiany kodu z każdej iteracji są commitowane, więc następna iteracja może zrobić git diff lub sprawdzić repo.
Debugowanie i samorefleksja:

22
Użycie git log i git diff jest częste. Szybki `git log --oneline -5` pokaże ostatnie commity agenta. Jeśli coś wygląda źle w diffie, można to wychwycić zanim pójdzie za daleko. Można nawet zautomatyzować review diffa — np. przerwać pętlę jeśli diff jest dużo większy niż oczekiwany lub dotyka krytycznych plików poza zakresem zadania.
22
Agent Introspection: Można poprosić agenta o wyjaśnienie siebie. Na przykład, jeśli utknie na zadaniu, można zmodyfikować prompt: „Jeśli testy nadal failują po 3 próbach, wypisz swoje rozumowanie i plan." To może ujawnić wewnętrzny chain-of-thought (tzw. „self-reflection"), co może dać wskazówki.
Model Context Protocol (MCP) do współdzielenia wiedzy:

22
Warto rozważyć współdzielenie wiedzy AGENTS.md między agentami lub przebiegami. Jeśli uruchamiasz wiele pętli (dla różnych projektów lub równoległych zadań), można scentralizować wspólną wiedzę. Serwer MCP Erica Ma jest przykładem systemu do przechowywania i serwowania takiego kontekstu różnym agentom jednolicie. Nawet współdzielony zestaw plików markdown referencyjnych może zapewnić spójność.
2.3 SEA-TS: Self-Evolving Agent for Time Series (2025)
21
SEA-TS (Self-Evolving Agent for Time Series Algorithms) wprowadza autonomiczny framework do generowania, walidacji i optymalizacji kodu algorytmów prognozowania przez iteracyjną pętlę samoewolucji, z MCTS Tree, generatorem kodu LLM, reviewerem kodu LLM, agentem rozumowania LLM, środowiskiem Sandbox, archiwum jakości-różnorodności MAP-Elites i pamięcią uruchomień promptów. Framework wykorzystuje Metric-Advantage Monte Carlo Tree Search (MA-MCTS) do zastąpienia stałych nagród statystycznie znormalizowanymi score'ami przewagi, dając dyskryminacyjne prowadzenie ku trajektoriom algorytmicznym o wysokim potencjale. System integruje automatyczny code review z bieżącą optymalizacją promptów i globalnym sterowalnym rozumowaniem, by trwale kodować wzorce korekcyjne i umożliwić transfer wiedzy między trajektoriami.
2.4 Gödel Agent — Recursive Self-Improvement (LangGraph)
26
LangGraph był już używany do tworzenia pętli writer-critic w autonomicznych agentach, które iteracyjnie udoskonalają swoje outputy. W jednym przypadku projekt „GPT-Newspaper" użył pętli writer ↔ critic, gdzie agent-pisarz redaguje treść, a agent-krytyk ją ocenia, powtarzając aż treść jest wysokiej jakości. Gödel Agent przyjmuje podobne podejście: graf agenta zawiera pętlę samodoskonalenia, gdzie agent proponuje samomodyfikację, analizuje ją (możliwie symulując wydajność) i albo finalizuje, albo rewizuje propozycję. Ta jawna struktura pętli zapewnia wielokrokowe rozumowanie o sobie.
Rollback i CLI:

26
CrewAI wspiera funkcję replay do restartu od konkretnego kroku zadania. W praktyce służy to jako mechanizm rollbacku: jeśli samomodyfikacja produkuje niepożądane zachowanie, można użyć `crewai replay -t <task_id>` do cofnięcia agenta do wcześniejszego kroku. Logowanie każdej iteracji samodoskonalenia wraz z wersjonowaniem kodu agenta pozwala CLI na automatyczny rollback jeśli weryfikacja lub test się nie powiedzie.
2.5 OpenAI Cookbook: Self-Evolving Agents (2025)
28
Zademonstrowano, że pętla samoewolucji działa i prompt może być ulepszany autonomicznie za pomocą Evals. Początkowo polegano na relatywnie prostym, statycznym metaprompcie do ulepszania system promptu. W dalszej części eksplorowano bardziej dynamiczną metodę: Genetic-Pareto (GEPA) — framework, który próbkuje trajektorie agenta, reflektuje nad nimi w języku naturalnym, proponuje rewizje promptów i ewoluuje system przez iteracyjne pętle feedbacku. Metoda GEPA opisana w odpowiednim artykule oferuje przekonujący blueprint dla ciągłej, samodoskonalenia optymalizacji promptów.
Kluczowe zasady:

28
Systemy agentowe często osiągają plateau po proof-of-concept, ponieważ polegają na ludziach do diagnozowania edge case'ów i korekcji błędów.
28
Optymalizacja nie zawsze jest skuteczna, więc zdolność do rollbacku wersji promptu jest kluczowa. Wierność informacji z graderów jest absolutnie kluczowa dla zapewnienia jakości optymalizacji. 3. ANALIZA FRAMEWORKÓW ORCHESTRACJI AGENTÓW
3.1 LangGraph (LangChain)
31
LangGraph używa workflow opartych na grafach dla stanowych, wielokrokowych procesów. Doskonale radzi sobie z workflow wymagającymi szczegółowego zarządzania stanem i iteracyjnych kroków.
34
Zapewnia nadrzędną kontrolę dla złożonych workflow wymagających warunkowego branchowania, wzorców cyklicznych lub precyzyjnego zarządzania stanem. Oferuje kontrolę opartą na maszynach stanowych.
34
Duże przedsiębiorstwa, w tym Klarna, Replit i Elastic, uruchamiają agentów opartych na LangGraph w produkcji.
Deepagents (LangChain) — nowe repozytorium:

48
100% open source (licencja MIT), w pełni rozszerzalny. Provider agnostic — działa z dowolnym dużym modelem językowym wspierającym tool calling, w tym modelami frontier i open. Zbudowany na LangGraph — produkcyjny runtime ze streamingiem, persystencją i checkpointowaniem. Agent harness wyposażony w narzędzie planowania, backend systemu plików i zdolność spawnowania subagentów.
Rekomendacja dla Ralpha: ⭐⭐⭐⭐⭐ — LangGraph jest najbardziej odpowiedni dla self-improving agenta ze względu na jawne zarządzanie stanem, pętle cykliczne i checkpointowanie.

3.2 CrewAI
31
CrewAI przypisuje specyficzne role agentom dla ustrukturyzowanego wykonywania zadań. Z designem opartym na rolach, dobrze działa dla sekwencyjnych, jasno zdefiniowanych procesów.
33
Używa dwuwarstwowej architektury: Crews i Flows. Crews odpowiadają za dynamiczną, opartą na rolach współpracę agentów, podczas gdy Flows zapewniają deterministyczną, event-driven orchestrację zadań.
26
Natywnie wspiera pliki konfiguracyjne YAML do definiowania agentów i ich atrybutów, co jest uważane za najlepszą praktykę. YAML zapewnia czytelny, łatwy w utrzymaniu widok konfiguracji agenta i umożliwia szybkie korekty bez zmiany kodu. Specyfikacja ról, celów i zachowań w YAML z parametrami wypełnianymi w runtime umożliwia dynamiczne zmiany zachowania.
Rekomendacja dla Ralpha: ⭐⭐⭐⭐ — Doskonały do definiowania ról (Coder, Reviewer, Tester) w YAML, ale mniej elastyczny w pętlach cyklicznych niż LangGraph.

3.3 AutoGen (Microsoft)
31
Ułatwia konwersacyjną współpracę między agentami. Idealny dla zadań konwersacyjnych jak brainstorming lub wsparcie klienta.
33
Architektura AutoGen składa się z niskopoziomowego Core do event-driven messaging i orchestracji oraz wysokopoziomowego interfejsu AgentChat do rozwijania agentów konwersacyjnych. Preferuje orchestrację konwersacji nad ustrukturyzowane flowcharty. Asynchroniczna pętla eventów i rozszerzenia RPC wspierają niskooperacyjne, wysokoprzepustowe workflow multi-agentowe. AutoGen udowodnił swoją trwałość w produkcyjnych przypadkach — np. w Novo Nordisk, gdzie napędza orchestrację agentów w środowiskach data science z rygorystyczną zgodnością farmaceutyczną.
46
Microsoft łączy teraz zespoły Semantic Kernel i AutoGen w nową platformę `agent-framework` do budowy agentów kompatybilnych z A2A (Agent-to-Agent), podobnych do magnetic-one.
Rekomendacja dla Ralpha: ⭐⭐⭐ — Dobry do fazy konwersacyjnej (debata między agentami), ale wymaga więcej pracy do zbudowania pętli self-improvement.

3.4 OASIS (camel-ai/oasis)
Uwaga: OASIS to NIE jest framework do parallel agents w kontekście kodowania. Jest to symulator interakcji społecznych.

15
OASIS: Open Agent Social Interaction Simulations with One Million Agents.
16
Zaprojektowany do skalowania do miliona agentów, umożliwiając badania na dużą skalę interakcji społecznych. System używa efektywnych operacji bazodanowych, wielu instancji LLM do load balancingu, ograniczania współbieżnych żądań i akceleracji czasu.
Jednak architektura asynchroniczna jest interesująca:

18
System wykorzystuje mechanizmy asynchroniczne, umożliwiające agentom wysyłanie wielu żądań jednocześnie nawet podczas oczekiwania na odpowiedzi z poprzednich interakcji, a moduł środowiskowy przetwarza przychodzące wiadomości równolegle. Usługi inferencji zarządzają zasobami GPU przez dedykowanego menedżera, który balansuje żądania agentów między dostępnymi GPU.
Rekomendacja dla Ralpha: ⭐⭐ — Nie jest bezpośrednio użyteczny dla self-improving code agent, ale wzorce asynchroniczne i load-balancing GPU mogą być inspiracją.

3.5 MetaGPT
6
MetaGPT to multi-agent framework, który na podstawie jednolinijkowego wymagania zwraca PRD, Design, Tasks lub Repo. Przypisuje różne role GPT-om, tworząc kolaboratywną encję software'ową. Wewnętrznie zawiera product managerów, architektów, project managerów i inżynierów. Dostarcza cały proces firmy software'owej z starannie zorchestrowanymi SOP. Główna filozofia: Code = SOP(Team).
Rekomendacja dla Ralpha: ⭐⭐⭐ — Wzorzec wielorolowy jest wartościowy, ale MetaGPT jest bardziej do generowania nowych projektów niż do samodoskonalenia.

3.6 Podsumowanie porównawcze
Framework Orchestracja Pętle cykliczne State Mgmt Self-Improvement Rekomendacja
LangGraph Graf stanowy ✅ Natywne ✅ Najlepszy ✅ Gödel Agent pattern ⭐⭐⭐⭐⭐
CrewAI Role-based Przez Flows ✅ Dobry Replay/rollback ⭐⭐⭐⭐
AutoGen Konwersacyjny Event loop ✅ Async Przez debatę ⭐⭐⭐
MetaGPT SOP pipeline ❌ Liniowy Limitowany ❌ ⭐⭐⭐
OASIS Async GPU ✅ Równoległy Bazy danych ❌ (social sim) ⭐⭐ 4. WZORCE ARCHITEKTONICZNE (PATTERNS)
4.1 Ralph Loop (Podstawowy wzorzec)
24
Ralph Loop to aktualnie trendujące pojęcie w przestrzeni AI, szczególnie w AI-driven software development. Opisuje proste ale potężne podejście, gdzie agent AI jest wykonywany wielokrotnie w ciągłej pętli aż zdefiniowany cel jest w pełni osiągnięty. Zamiast polegać na jednej długiej konwersacji z rosnącym kontekstem, każda iteracja startuje ze świeżym kontekstem agenta, podczas gdy postęp jest zachowywany przez pliki, testy i kontrolę wersji.
4.2 Minimalny Agent Pattern (learn-claude-code)
27
Fundamentalny wzorzec agenta: User → messages[] → LLM → response | stop_reason == „tool_use"? → yes: execute tools, append results, loop back → no: return text. To jest minimalna pętla. Każdy agent AI potrzebuje tej pętli. MODEL decyduje kiedy wywoływać narzędzia i kiedy się zatrzymać. KOD po prostu wykonuje to, o co pyta model.
27
Claude Code = jedna pętla agenta + narzędzia (bash, read, write, edit, glob, grep, browser...) + on-demand skill loading + context compression + subagent spawning + task system z grafem zależności + team coordination z async mailboxami + worktree isolation dla równoległego wykonania + permission governance. To cała architektura. Każdy komponent to mechanizm harness'u. Harness daje modelowi ręce, oczy i przestrzeń roboczą.
4.3 Writer-Critic Loop (Gödel Agent Pattern)
text

┌─────────────────────────────────────────┐
│ 1. AGENT proponuje samomodyfikację │
│ 2. CRITIC analizuje (symuluje efekt) │
│ 3. TESTER uruchamia testy │
│ 4. Jeśli pass → commit. Jeśli fail → │
│ feedback do AGENTA, retry │
│ 5. Pętla aż konwergencja lub limit │
└─────────────────────────────────────────┘
4.4 GEPA (Genetic-Pareto) Pattern — Samoewolucja promptów
Wzorzec z OpenAI Cookbook:

Sample — zbierz trajektorie agenta
Reflect — refleksja w języku naturalnym nad sukcesami i porażkami
Propose — zaproponuj rewizję promptu
Evaluate — uruchom graders na nowej wersji
Select — Pareto-front (najlepsza wersja)
Loop — iteruj aż konwergencja
4.5 MAP-Elites + MCTS Pattern (z SEA-TS)
Wzorzec dla quality-diversity optymalizacji:

Generuj warianty kodu algorytmu (populacja)
MCTS prowadzi eksplorację przestrzeni rozwiązań
MAP-Elites zachowuje archiwum najlepszych rozwiązań w różnych niszach
Running Prompt Memory koduje wzorce korekcyjne
Cross-trajectory knowledge transfer 5. ISTNIEJĄCE IMPLEMENTACJE RALPH LOOP
5.1 michaelshimeles/ralphy (bash wrapper)
Uruchamia Claude Code, Cursor, Codex, Gemini w pętli
Tracking progresu przez checkbox'y w Markdown
Git Worktrees + Sandbox Mode (symlinkowanie node_modules)
--max-retries z fallback na sandbox mode
Agent-browser do auto-testów E2E
5.2 iannuttall/ralph (minimalny, file-based)
30
Ralph to minimalna, plikowa pętla agenta dla autonomicznego kodowania. Każda iteracja startuje od świeżego stanu, czyta ten sam stan z dysku i commituje pracę dla jednej story na raz. Ralph traktuje pliki i git jako pamięć, nie kontekst modelu.
30
Jeśli pętla crashnie i story zostaje w stanie in_progress, można ustawić STALE_SECONDS w config.sh, aby Ralph automatycznie reopenował zatrzymane stories. Można wskazać Ralpha na inny plik PRD JSON przez flagi CLI. Obsługuje wiele backendów agentów: codex, claude, droid, opencode.
5.3 daegwang/ralph-code (Claude Code + Codex)
25
Autonomiczna pętla agenta AI, która zamienia opis w działający kod. Napędzana przez Claude Code i Codex. Describe → Plan → Execute. Commit per task — każde ukończone zadanie to czysty git commit. Context between tasks — git diff + ustrukturyzowany log progresu przekazywany do każdej iteracji. Auto retry — zadania failujące ponawiają się do 3 razy przed zatrzymaniem. Mix agents — użyj Claude do planowania i Codex do egzekucji.
5.4 fstandhartinger/ralph-wiggum (spec-driven)
29
Agent outputuje `<promise>DONE</promise>` tylko gdy kryteria akceptacji są w 100% zweryfikowane. Bash loop to wymusza. Zasada: ufaj AI do self-identify, self-correct i self-improve. Obserwuj wzorce i dostosowuj prompty.
29
Kompatybilny z: Claude Code, Cursor, Codex, Windsurf, Amp, OpenCode i więcej. Łączy podejście bash loop ze specyfikacjami w stylu SpecKit i płynnym procesem instalacji AI-driven. 6. CHECKPOINT & RESUME — WZORCE Z ISTNIEJĄCYCH PROJEKTÓW
6.1 Wzorzec z iannuttall/ralph
Stan przechowywany w plikach JSON (PRD ze statusami: pending, in_progress, done)
STALE_SECONDS — automatyczny reopen zatrzymanych stories
Git jako pamięć długoterminowa
6.2 Wzorzec z ralphy
progress.txt jako checkpoint
Odznaczanie checkboxów w Markdown PRD
Stash/restore stanu lokalnego przy merge
6.3 Wzorzec z LangGraph
48
create_deep_agent zwraca skompilowany graf LangGraph. Można go używać ze streamingiem, Studio, checkpointerami lub dowolną funkcją LangGraph. Produkcyjny runtime z persystencją i checkpointowaniem.
6.4 Wzorzec z CrewAI
26
Komendy do inspekcji stanu zadań i rollbacku do bezpiecznych punktów. CrewAI CLI wspiera funkcję replay do restartu od konkretnego kroku zadania. W praktyce rollback: jeśli samomodyfikacja produkuje niepożądane zachowanie, `crewai replay -t <task_id>` cofa agenta.
6.5 Rekomendacja dla Ralpha v1.6
Implementuj trzy warstwy checkpoint:

Plikowy checkpoint (progress.json) — status każdej story/task
Git checkpoint — commit po każdym pomyślnym kroku
Pamięć iteracji — log decyzji, błędów i korekcji (learnings.md)
Przy wznowieniu:

Przeczytaj progress.json
Zidentyfikuj stories w stanie in_progress (z STALE_SECONDS)
Przeczytaj ostatni git diff dla kontekstu
Kontynuuj od następnej niezakończonej story 7. INTELLIGENT ERROR RECOVERY — WZORCE Z ISTNIEJĄCYCH PROJEKTÓW
7.1 Wzorzec z Addy Osmani (self-reflection)
22
Cursor multi-agent experiment miał agenta „judge", który mógł zdecydować czy projekt jest ukończony, sprawdzając np. czy CI jest zielone. W prostszej pętli single-agent, można to symulować uruchamiając build scripts lub CI checks lokalnie. Pętla powinna się zatrzymać na red flag — jeśli check failuje, agent powinien to zaadresować (lub oznaczyć zadanie jako failing i potencjalnie iść dalej po N próbach).
7.2 Wzorzec z SWE-agent
43
Agent w 100 liniach rozwiązujący issues GitHuba z CLI. Radykalnie prosty, bez wielkich konfigów, ale osiąga >74% na SWE-bench verified.
7.3 Wzorzec z OpenAI Self-Evolving Agents
28
Gdy uruchomisz to lokalnie, graders inne niż chemical_name_grader mogą początkowo failować. To normalne, ponieważ graders mogą początkowo failować, ale wyniki powinny się poprawić przez feedback loop. Wczesne porażki po prostu odzwierciedlają dostosowywanie się modelu zanim skonwerguje na dokładniejszych wynikach.
7.4 Wzorzec z ralph-wiggum
Fundamentalny mechanizm: Agent musi outputować <promise>DONE</promise>. Jeśli nie — retry. Jeśli tak — następna iteracja.

7.5 Rekomendacja dla Ralpha v1.7
Wielopoziomowy Error Recovery:

text

Poziom 1: FORMAT ERROR (JSON malformed)
→ Feedback: "Your previous JSON was malformed. Error: {error}.
Re-format and try again."
→ Max 3 retries na ten sam błąd

Poziom 2: LOGIC ERROR (testy failują)
→ Feedback: "Tests failed: {test_output}.
Analyze the error and fix. Focus on: {specific_test}"
→ Max 5 retries z incremental feedback

Poziom 3: STUCK (ten sam błąd powtarza się)
→ Self-reflection prompt: "You've been stuck for {n} iterations.
Output your reasoning on why and propose a new approach."
→ Jeśli nadal stuck → skip story, log do learnings.md

Poziom 4: CATASTROPHIC (crash systemu, rate limit)
→ Checkpoint state, wait, resume 8. SELF-IMPROVEMENT LOOP — ARCHITEKTURA REFERENCYJNA
Na podstawie wszystkich przeanalizowanych projektów, oto proponowana architektura:

text

┌─────────────────────────────────────────────────────────────────┐
│ RALPH SELF-IMPROVEMENT LOOP │
├─────────────────────────────────────────────────────────────────┤
│ │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ │
│ │ 1. ORIENT │───▶│ 2. PROPOSE │───▶│ 3. IMPLEMENT│ │
│ │ Read state: │ │ LLM generuje│ │ Apply changes│ │
│ │ - progress │ │ plan zmiany │ │ to codebase │ │
│ │ - learnings │ │ (diff) │ │ │ │
│ │ - git log │ │ │ │ │ │
│ └──────────────┘ └──────────────┘ └──────────────┘ │
│ │ │
│ ┌─────────────────────────────────────────┘ │
│ ▼ │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ │
│ │ 4. VERIFY │───▶│ 5. EVALUATE │───▶│ 6. COMMIT │ │
│ │ Run tests: │ │ LLM-as-Judge│ │ Git commit │ │
│ │ - unit tests│ │ Score zmian │ │ Update prog. │ │
│ │ - lint │ │ vs baseline │ │ Log learnings│ │
│ │ - typecheck │ │ │ │ │ │
│ └──────────────┘ └──────────────┘ └──────────────┘ │
│ │ FAIL │ SUCCESS │
│ ▼ ▼ │
│ ┌──────────────┐ ┌──────────────┐ │
│ │ ERROR │ │ NEXT STORY │ │
│ │ RECOVERY │ │ or DONE │ │
│ │ (Levels 1-4)│ │ │ │
│ └──────────────┘ └──────────────┘ │
│ │ │
│ └──────────── retry loop ──────────────┘ │
│ │
├─────────────────────────────────────────────────────────────────┤
│ PERSISTENCE LAYER: │
│ [progress.json] [learnings.md] [git history] [AGENTS.md] │
└─────────────────────────────────────────────────────────────────┘
8.1 Kluczowe komponenty
Agent Harness (Bash/Python):

Pętla sterująca (while not DONE)
Checkpointowanie
Wywołania API (NVIDIA NIM lub inny provider)
Narzędzia (read, write, exec, git)
LLM Brain:

System prompt z AGENTS.md
Kontekst z progress.json + git diff + learnings.md
Tool calling (function calling)
Verification Layer:

Automatyczne testy (pytest, npm test, itp.)
Lint/typecheck
Opcjonalnie: LLM-as-Judge (drugi LLM ocenia zmiany)
Memory Layer:

Git commits jako pamięć długoterminowa
progress.json jako stan bieżący
learnings.md jako pamięć korekcyjna
AGENTS.md jako tożsamość i reguły 9. BEZPIECZEŃSTWO SAMOMODYFIKACJI
9.1 Wnioski z artykułu naukowego (ICLR 2025)
49
Należy starannie rozważyć implikacje bezpieczeństwa. System modyfikujący tylko scaffolding (bez wag) ma ograniczony potencjał (ale i ograniczone ryzyko). Gdyby modyfikował wagi — ryzyko byłoby wyższe. Wskazane są dwa mechanizmy mitygacji ryzyka.
9.2 Wzorzec Sandbox z ralphy
Symlinkowanie ciężkich zależności (node_modules)
Kopiowanie plików źródłowych do izolowanego folderu
Testy w izolacji → merge dopiero po pass
9.3 Wzorzec Docker z SWE-agent
Agent działa w izolowanym kontenerze Docker
Ograniczone uprawnienia systemowe
Odtwarzalne środowisko
9.4 Rekomendowane guardrails dla Ralpha:
Git-based rollback — każda zmiana commitowana, łatwy revert
Sandbox testing — zmiany testowane w izolacji przed merge
Diff size limit — alarm jeśli diff > N linii
Protected files — lista plików których agent NIE MOŻE modyfikować
Max iterations — twardy limit iteracji (np. 50)
Human approval gate — opcjonalny krok zatwierdzenia 10. PLAN IMPLEMENTACJI
v1.6 — Checkpoint & Resume
Cel: Agent potrafi wznowić pracę po crash'u, rate limicie lub restarcie.

Zadania:

Dodaj progress.json ze statusami: pending | in_progress | done | failed
Implementuj STALE_SECONDS — auto-reopen stories stuck w in_progress
Przy starcie: odczytaj progress.json, pomijaj done, wznów od in_progress lub następnego pending
Git commit po każdej ukończonej story
learnings.md — append-only log z wnioskami z każdej iteracji
v1.7 — Intelligent Error Recovery
Cel: Agent potrafi self-correct po błędach formatowania, logiki i zacięciach.

Zadania:

Implementuj 4-poziomowy error recovery (patrz sekcja 7.5)
Consecutive error counter per story
Feedback injection do następnego promptu po błędzie
Self-reflection prompt po N consecutive failures
Skip + log mechanizm dla uporczywych błędów
v1.8 — LLM-as-Judge Verification
Cel: Drugi LLM (lub ten sam z innym promptem) ocenia jakość zmian.

Zadania:

Implementuj verify_with_llm(diff, test_results) → score 0-10
Threshold: commit tylko jeśli score >= 7
Log scores do learnings.md z uzasadnieniem
v1.9 — Prompt Self-Evolution (GEPA Pattern)
Cel: Agent potrafi iteracyjnie ulepszać swoje własne prompty.

Zadania:

Wersjonowanie system promptów (v1, v2, v3...)
Po N iteracjach: meta-prompt analizuje wyniki i proponuje ulepszenie promptu
A/B testing: uruchom stary vs nowy prompt na tym samym zadaniu
Rollback do poprzedniej wersji jeśli wyniki gorsze
v2.0 — Multi-Agent Self-Improvement
Cel: Rozdzielenie ról na osobne agenty z różnymi LLM.

Zadania:

Agent CODER — generuje/modyfikuje kod (szybki model)
Agent REVIEWER — ocenia zmiany (thinking model)
Agent TESTER — uruchamia testy i interpretuje wyniki
Orchestrator — koordynuje przepływ (LangGraph graf) 11. REPOZYTORIA REFERENCYJNE (Z URLami)
Implementacje Ralph Loop:
Repo URL Opis
michaelshimeles/ralphy github.com/michaelshimeles/ralphy Bash wrapper, Claude/Cursor/Codex w pętli
iannuttall/ralph github.com/iannuttall/ralph Minimalna plikowa pętla agenta
daegwang/ralph-code github.com/daegwang/ralph-code Claude Code + Codex, commit per task
fstandhartinger/ralph-wiggum github.com/fstandhartinger/ralph-wiggum Spec-driven z <promise>DONE</promise>
Agent Harness / Edukacja:
Repo URL Opis
shareAI-lab/learn-claude-code github.com/shareAI-lab/learn-claude-code 12 sesji budowy harness'u od zera
SWE-agent/mini-swe-agent github.com/SWE-agent/mini-swe-agent 100-liniowy agent, >74% SWE-bench
Frameworki orchestracji:
Framework URL Filozofia
LangGraph github.com/langchain-ai/langgraph Graf stanowy, cykliczny, checkpointy
Deepagents github.com/langchain-ai/deepagents Agent harness na LangGraph, planning + subagenty
CrewAI github.com/crewAIInc/crewAI Role-based, YAML config, replay/rollback
AutoGen github.com/microsoft/autogen Konwersacyjny, async, event-driven
MetaGPT github.com/geekan/MetaGPT Software company simulation, SOP
Platformy i coding agents:
Narzędzie URL Opis
OpenHands github.com/All-Hands-AI/OpenHands Skalowalna platforma cloud coding agents, 65K+ stars
Cline github.com/cline/cline Local-first coding agent w IDE
Goose (Block) github.com/block/goose Extensible AI agent, install/execute/edit/test
Plandex plandex.ai Open-source terminal AI agent dla dużych projektów
Refact.ai refact.ai Self-hosted AI agent, #1 na SWE-bench (OS)
Aider github.com/paul-gauthier/aider AI pair programming, git-native
Artykuły naukowe i cookbooks:
Tytuł URL Opis
Self-Improving Coding Agent openreview.net/pdf?id=rShJCyLsOr ICLR 2025 Workshop paper
OpenAI Self-Evolving Agents developers.openai.com/cookbook/examples/partners/self_evolving_agents GEPA pattern, prompt evolution
Gödel Agent Tutorial gist.github.com/ruvnet/15c6ef... LangGraph + CrewAI recursive self-improvement
Addy Osmani: Self-Improving Agents addyosmani.com/blog/self-improving-agents/ Praktyczny poradnik Ralph Loop
e2b-dev/awesome-ai-agents github.com/e2b-dev/awesome-ai-agents Curated list of AI autonomous agents
tmgthb/Autonomous-Agents github.com/tmgthb/Autonomous-Agents Daily-updated research papers on autonomous agents 12. KLUCZOWE WNIOSKI
Ralph Loop jest dojrzałym wzorcem — wiele niezależnych implementacji potwierdza jego skuteczność. Kluczowe elementy: fresh context per iteration, git jako pamięć, pliki jako stan.

LangGraph jest najlepszym frameworkiem do budowy self-improving agenta — natywne pętle cykliczne, zarządzanie stanem, checkpointowanie i persystencja. Deepagents (LangChain) to gotowy harness na LangGraph.

Error recovery jest krytyczny — wszystkie produkcyjne implementacje mają wielopoziomowe mechanizmy recovery. Bez nich agent się zacina lub zapętla.

Samomodyfikacja scaffoldingu jest bezpieczna — artykuł z ICLR 2025 potwierdza, że modyfikacja tylko scaffoldingu (nie wag modelu) ma ograniczone ryzyko i daje mierzalne wyniki.

GEPA pattern (z OpenAI) to najlepszy udokumentowany wzorzec auto-ewolucji promptów — próbkowanie, refleksja, propozycja, ewaluacja, selekcja Pareto.

4 kanały pamięci (git history, progress file, learnings log, AGENTS.md) to minimum dla stabilnego działania.

OASIS nie jest odpowiedni dla tego use case — to symulator społeczny, nie framework kodowania. Jednak jego wzorce async i GPU load-balancing mogą być przydatne w przyszłości.

Koniec dokumentu. Wszystkie informacje powyżej pochodzą z publicznie dostępnych źródeł (GitHub, arxiv, blogi) i zostały zweryfikowane w marcu 2026.
