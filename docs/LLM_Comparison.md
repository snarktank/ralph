Oto kompleksowe zestawienie najlepszych modeli z Twojej listy, w kontekście samodoskonalącego się kodu za pomocą LLM, podzielone na kategorie:

🏆 1. KODOWANIE (Coding)
Rank Model Dlaczego?
🥇 qwen/qwen3-coder-480b-a35b-instruct Dedykowany model koderski od Qwen, MoE z ogromną liczbą parametrów. Zaprojektowany specjalnie pod generowanie i edycję kodu.
🥈 moonshotai/kimi-k2-instruct
44
Kimi K2 osiąga 65.8% pass@1 na SWE-bench Verified i 47.3% na SWE-bench Multilingual.
44
Model 1T parametrów (MoE) pretrenowany na 15.5T tokenach.
44
Specjalnie zaprojektowany pod tool use, reasoning i autonomiczne rozwiązywanie problemów.
🥉 deepseek-ai/deepseek-v3.2
3
DeepSeek V3.2 ma konsekwentnie silne wyniki w niemal każdej kategorii benchmarkowej, w tym SWE-bench Verified 67.8% i LiveCodeBench 74.1%.
5
Jest jednym z najlepszych open-source LLM do reasoning i agentic workloads.
Wyróżnienia: mistralai/devstral-2-123b-instruct-2512 (dedykowany coding), qwen/qwen2.5-coder-32b-instruct, mistralai/codestral-22b-instruct-v0.1

🧠 2. GŁĘBOKIE ROZUMOWANIE (Deep Reasoning)
Rank Model Dlaczego?
🥇 moonshotai/kimi-k2-thinking
43
Kimi K2 Thinking to najnowszy, najbardziej zdolny open-source thinking model. Rozumuje krok po kroku, dynamicznie wywołując narzędzia, ustanawiając nowy SOTA na Humanity's Last Exam (HLE), BrowseComp i innych benchmarkach.
43
Natywna kwantyzacja INT4 z oknem kontekstu 256k.
🥈 moonshotai/kimi-k2.5
17
Kimi K2.5 (Reasoning) zajmuje 2. miejsce wśród open weights modeli wg Intelligence Index.
48
Obsługuje kontekst do 256,000 tokenów.
🥉 deepseek-ai/deepseek-r1-distill-qwen-32b
21
DeepSeek-R1-Distill-Qwen-32B osiąga 72.6% na AIME 2024, 94.3% na MATH-500 i 57.2% na LiveCodeBench.
23
Oferuje głębsze rozumowanie, widoczne kroki i lepszą samokorektę na trudnych problemach — wolniejszy, ale lepiej się łapie na błędach.
Wyróżnienia: qwen/qwq-32b (reasoning-first), qwen/qwen3.5-397b-a17b, z-ai/glm5

📏 3. NAJDŁUŻSZY KONTEKST
Rank Model Context Window
🥇 moonshotai/kimi-k2.5
47
256K tokenów
🥇 moonshotai/kimi-k2-thinking
43
256K tokenów
🥉 mistralai/mistral-large-3-675b-instruct-2512
3
Mistral Large oferuje 256K kontekstu.
Wyróżnienia: microsoft/phi-3-medium-128k-instruct (128K), microsoft/phi-3-mini-128k-instruct (128K), meta/llama-4-scout-17b-16e-instruct (~128K+), deepseek-ai/deepseek-v3.1 (
24
wspiera kontekst do 128K)

⚡ 4. NAJSZYBSZY (Speed / Throughput)
Rank Model Dlaczego?
🥇 nvidia/nemotron-3-super-120b-a12b
17
NVIDIA Nemotron 3 Super 120B A12B (Reasoning) osiąga 445.1 tokenów/s — najszybszy duży model na liście.
🥈 nvidia/nemotron-3-nano-30b-a3b
18
Ma najniższy czas do pierwszego tokena (0.38s). MoE z zaledwie 3B aktywowanych parametrów = ultra szybki.
🥉 google/gemma-3n-e4b-it
17
Gemma 3n E4B Instruct to najtańszy model ($0.03/1M tokenów) — bardzo mały, ultra szybki inference, idealny do szybkich iteracji.
Wyróżnienia: meta/llama-3.2-1b-instruct, meta/llama-3.2-3b-instruct, google/gemma-3n-e2b-it (ekstremalnie małe = ekstremalnie szybkie)

🎯 5. NAJDOKŁADNIEJSZY (Overall Accuracy / Intelligence)
Rank Model Dlaczego?
🥇 z-ai/glm5
3
GLM-5 ma najwyższy Chatbot Arena rating na leaderboardzie (1451) — najwyżej oceniany model wg ludzkich preferencji.
17
GLM-5 (Reasoning) zajmuje 1. miejsce wśród open weights modeli wg Intelligence Index (50).
🥈 z-ai/glm4.7
3
GLM-4.7 ma HumanEval 94.2 (najwyższy ze wszystkich), AIME 2025 95.7, GPQA Diamond 85.7, LiveCodeBench 84.9 i okno kontekstu 200K.
🥉 qwen/qwen3.5-397b-a17b
17
Qwen3.5 397B A17B (Reasoning) zajmuje 3. miejsce wśród open weights wg Intelligence Index (45).
5
Łączy dużą architekturę MoE z multimodalnym rozumowaniem i ultra-długim kontekstem.
🔄 6. NAJWIĘCEJ MOŻLIWOŚCI ZAPYTAŃ (RPM/RPD — Rate Limits na NVIDIA API)
Limity RPM/RPD na NVIDIA NIM API Catalog zależą od planu, nie od modelu.
31
Domyślny limit to ~40 requestów na minutę. Jednak z perspektywy efektywności (mniejszy model = szybsza odpowiedź = więcej zapytań w tym samym czasie):

Rank Model Dlaczego?
🥇 google/gemma-3n-e2b-it Najlżejszy (~2B eff.), najszybszy turnaround → max throughput
🥈 meta/llama-3.2-1b-instruct 1B parametrów, ultra lekki, minimalny latency
🥉 nvidia/nemotron-3-nano-30b-a3b MoE z 3B aktywnych parametrów — świetny stosunek jakości do szybkości
⚠️ Jeśli hostujesz sam (self-hosted), te modele pozwolą Ci zmaksymalizować RPM na danym GPU.

🏅 REKOMENDACJA DLA SAMODOSKONALĄCEGO SIĘ KODU
Dla systemu, który iteracyjnie ulepsza swój własny kod za pomocą LLM, potrzebujesz kombinacji:

Rola Rekomendowany model Zadanie
Generator kodu qwen/qwen3-coder-480b-a35b-instruct Pisanie i refaktoryzacja kodu
Weryfikator / Reviewer moonshotai/kimi-k2-thinking Głęboka analiza błędów, reasoning o poprawności
Szybki iterator nvidia/nemotron-3-nano-30b-a3b lub deepseek-ai/deepseek-r1-distill-qwen-32b Szybkie pętle feedback (lint, testy, drobne poprawki)
Fallback / orchestrator deepseek-ai/deepseek-v3.2 Wszechstronny model do orchestracji agentów
Taki pipeline pozwala na: szybkie generowanie → głęboką weryfikację → iteracyjne poprawki w zamkniętej pętli.
