# Ralph — Architektura i Pętla Samo-Modyfikacji (Self-Modification Flow)

Ten dokument przedstawia wizualną oraz techniczną koncepcję działania długotrwale pętli (Long-Running Loop) autonomicznego agenta Ralpha ze szczególnym uwzględnieniem mechanizmów samozachowawczych, autokorekty błędów (Intelligent Error Recovery) oraz zapisu stanu (Checkpoints).

---

## Graf Operacyjny Ralpha

Poniższy graf [Mermaid] ilustruje pełen cykl życia pętli po uruchomieniu komendy `./ralph.sh` dla LLM z integracją NVIDIA API.

```text
       [ Start: Tworzysz PRD ]
                 │
                 ▼
          ( ./ralph.sh )
                 │
                 ▼
         /¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\    TAK
        / .checkpoint_iter\ ───────► [ Wczytaj START_ITERATION (v1.6) ]
        \   istnieje?     /                        │
         \_______________/                         │
                 │ NIE                             │
                 ▼                                 │
     [ Checkout 'ralph/feature' ] ◄────────────────┘
                 │
                 ▼
 ┌──────────────►│
 │               ▼
 │      /¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\    NIE
 │     /  Masz nieukończone \ ───────► [ Zakończ + Cleanup ]
 │     \   stories w PRD?   /                  │
 │      \__________________/                   ▼
 │               │ TAK                🫵 TWOJA WERYFIKACJA
 │               ▼                      I GIT MERGE
 │    [ Pobierz Story (pass: false) ]
 │               │
 │ ┌────────────►│
 │ │             ▼
 │ │       ( NVIDIA API )
 │ │             │
 │ │             ▼
 │ │     /¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\   NIE
 │ │    /   Valid JSON?   \ ───────┐
 │ │    \                 /        │
 │ │     \_______________/         ▼
 │ │             │ TAK     /¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\  TAK
 │ │             │        / consecutive_fails\ ──────► [ Abort: Zabezpieczenie przed infinity-loop ]
 │ │             │        \      >= 3 ?      /
 │ │             │         \________________/
 │ │             │                 │ NIE
 │ │             ▼                 ▼
 │ │   [ Ewaluacja Akcji ] ◄── [ Wstrzyknij Feedback v1.7 ]
 │ │             │
 │ │             ▼
 │ │     /¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\   TAK
 │ │    / Edycja ralph.sh \ ─────► [ .staging/ralph.sh ]
 │ │    \                 /                │
 │ │     \_______________/                 ▼
 │ │             │ NIE             /¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\   NIE
 │ │             │                /  bash -n OK?    \ ────┐
 │ │             │                \                 /     │
 │ │             │                 \_______________/      │
 │ │             │                         │ TAK          │
 │ │             ▼                         ▼              │
 │ │   [ Zmiany w projekcie ] ◄── [ Nadpisz ralph.sh ]    │
 │ │             │                                        │
 │ │             ▼                                        │
 │ └──────── /¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\                            │
 │          /  Akcja finish?  \ ◄─────────────────────────┘
 │          \                 /
 │           \_______________/
 │                   │ TAK
 │                   ▼
 │           /¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\    FAIL
 │          / Weryfikacja TDD \ ─────────► ( Zwiększ odrzucenia )
 │          \       OK?       /                     │
 │           \_______________/                      │
 │                   │ TAK                          │
 │                   ▼                              │
 │         [ passes: true w PRD ]                   │
 │                   │                              │
 │                   ▼                              │
 │           /¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\                      │
 │          / Inne stories w  \                     │
 │          \     PRD?        /                     │
 │           \_______________/                      │
 │               │ TAK       │ NIE                  │
 │               ▼           └──────────────────────┘
 │   [ <promise>STORY_COMPLETED ]
 │   [  Wyczyść historię API  ]
 │               │
 └───────────────┘
```

## Kluczowe Paradygmaty Arichitektury

Cała stabilność autonomicznego agenta Ralpha bazuje i ewoluuje dzięki silnym "bezpiecznikom" ograniczającym nieskończone generowanie tekstu i chroniącym projekt przed zepsuciem operacyjnym.

### 1. Izolacja Zmian Systemowych (Staging Sandbox)
Jeżeli LLM w czasie swojej iteracji wygeneruje akcję modyfikacji logiki Ralpha (`ralph.sh`), edycje kierowane są do odciętego "Sandboxu" (`.staging/ralph.sh`). Kod przechodzi systemową kompilację i testy poprzez standard `bash -n`. Jakikolwiek błąd syntax (np. utracony tag `fi`) zapobiega zepsuciu głównego skryptu — zmiana zostaje anulowana, a LLM otrzymuje reprymendę ("Your changes to ralph.sh had syntax errors").

### 2. Statefulness & Wznawianie Pętli (v1.6 Checkpoints)
Autonomiczne odpytywanie może zająć nawet pół godziny z uwagi na skomplikowane `userStories`. System monitoruje swój krok pętli poprzez zapis aktualnej `START_ITERATION` w pliku runtime (`.checkpoint_iter`). Wymuszone wyjście z terminalu (crush lokalnego PC `CTRL+C`, błąd łącza, czy brak kredytów API) nie niszczy progresu pętli. Ralph wznawia iterację w bezpiecznym miejscu od kroku w jakim ją przerwał.

### 3. Inteligentne Odzyskiwanie Struktury API (v1.7: Error Recovery)
Gdy narzędzie takie jak NVIDIA nie zachowa formatowania pluosującego (pożądany i czysty plik wyjściowy JSON), Ralph unika wyłączenia z powodu unparsable object. Zamiast tego zapisuje niepowodzenie (`consecutive_fails++`) i wymusza ponowną odpowiedź LLM wrzucając do promptu twardą poprawkę (`ERROR: Your previous response was NOT valid JSON...`). Bezpiecznik limitujący zapętlenia anuluje proces permanentnie dopiero po 3 skrajnie uporczywych błędach z rzędu.

### 4. Zero-Trust Multitasking (Multi-story Handling)
Zamiast wrzucać agenta do studni bez dna z całością problemów, Ralph przyjmuje zadania pojedyńczo pobierając je z `prd.json`.
Mechanizm rzutuje komendę `<promise>STORY_COMPLETED</promise>` zaraz gdy uzna pojedyncze Story za wykonane do zadowalającego stopnia i zaliczy procedurą TDD. Agent po potwierdzeniu **automatycznie wyrzuca dotychczasową wiedzę** oczyszczając `Conversation History` do bezpiecznego minimum (zapobiega to dziedziczeniu zepsutych poleceń z poprzeniego tematu) i przechodzi do kolejnego Story. Operacja powtarza się dopóki plik nie zostanie skondensowany wynikiem zerowym, finalizując pracę agenta do archiwizacji (`<promise>COMPLETE</promise>`). 

### 5. Finalna Cenzura Ludzkiego Oka
Proces modyfikacji zawsze kończy cię w oddzielonym od produkcji branchu (`git branch 'ralph/X'`). Całościowy "Live" skryptów i posklejanych przez AI poprawek musi zostać przeprocedowany ręcznym zleceniem `git merge` co zapewnia ostateczny środek chroniący przez katastrofalnymi lukami logicznymi, jakich TDD Ralpha nie mógł przewidzieć.
