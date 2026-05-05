# Amigo Project Workflow

Przewodnik opisuje zalecany sposób pracy z projektem **Amigo** przy współpracy z ChatGPT / agentem kodującym. Celem jest ograniczenie zużycia tokenów, unikanie zalewania kontekstu dużymi logami oraz utrzymanie czytelnej, modułowej pracy nad projektem.

Projekt zakłada pracę głównie na **Windowsie**, najlepiej w **PowerShellu**.

Domyślnym pierwszym źródłem kontekstu jest teraz **amigo-codemap**.

Na start preferujemy najkrotszy widok:

```powershell
cargo run -p amigo-codemap -- brief
```

Gdy potrzebna jest pelna mapa:

```powershell
cargo run -p amigo-codemap -- compact
```

Ręczne komendy `git`, `rg`, `fd`, `cargo` i `npm` służą jako doprecyzowanie po odczytaniu mapy projektu albo jako fallback, gdy codemap jest niedostępny.

---

## 1. Główna zasada

Nie pokazujemy od razu całego repozytorium, pełnych diffów ani pełnych logów.

Najpierw pokazujemy **mapę problemu przez codemap**:

```powershell
cargo run -p amigo-codemap -- compact
```

Jeżeli agent potrzebuje aktualnego stanu zmian:

```powershell
cargo run -p amigo-codemap -- changed
```

Jeżeli potrzebne są symbole:

```powershell
cargo run -p amigo-codemap -- symbols --level 1
```

Jeżeli potrzebne są relacje techniczne między plikami, używamy compact albo level 2/3:

```powershell
cargo run -p amigo-codemap -- compact
cargo run -p amigo-codemap -- scan --level 2 --ai
cargo run -p amigo-codemap -- scan --level 3 --ai
```

W polu `d` codemap pokazuje wtedy neutralne relacje typu:

```text
imports
declares
test-candidate
test-candidate:in-file
```

Codemap core nie interpretuje projektu i nie zna pojęć takich jak editor, engine, Tauri command czy asset registry. Dostarcza skompresowane fakty techniczne; interpretację robi agent/LLM.

Dopiero gdy codemap wskaże obszar, zawężamy ręcznie:

```powershell
git status --short
git diff --stat
git diff --name-status
```

Dopiero potem pokazujemy konkretny plik, fragment diffu albo zawężony wynik wyszukiwania.

Dobre podejście:

1. Wygeneruj albo odczytaj `.amigo/codemap.json`.
2. Ustal z codemap, które obszary i pliki są istotne.
3. Pokaż ręcznie tylko zmienione pliki albo symbole z tego obszaru.
4. Pokaż tylko potrzebny fragment kodu lub błędu.
5. Dopiero po analizie rozszerz kontekst.

---

## 1a. Priorytet narzędzi

Kolejność zbierania kontekstu:

1. `amigo-codemap brief` - minimalny start.
2. `amigo-codemap changed --group ...` - szybkie zawężenie zmian.
3. `amigo-codemap find/scope/refs/docs` - mały kontekst pod zadanie.
4. `amigo-codemap compact` / `symbols` - pełniejsza mapa, gdy brief nie wystarcza.
5. `rg -l`, `fd`, `git diff --stat`, `git diff --name-status` - fallback/doprecyzowanie.
6. `rg -n -C`, `Get-Content`, zawężony `git diff -- <plik>` - konkretne fragmenty.
7. `cargo`, `npm`, `vitest` albo `amigo-codemap verify` - weryfikacja po zmianach.

Nie zaczynamy od pełnego `git diff`, pełnego `rg` po repo ani pełnych logów builda.

---

## 1b. Log operacji

Po kazdym zakonczonym zadaniu aktualizujemy root `operations.md`.

Format ma byc krotki:

```text
### Nazwa zadania
- Task: ...
- Ops: ...
- Files: ...
- Verify: ...
- Tokens: used ~N, saved ~N.
```

Tokeny sa szacunkiem. Celem jest widziec, ktore operacje sa drogie i co warto przeniesc do `amigo-codemap`.

---

## 1c. Male widoki codemap

Najczestsze komendy:

```powershell
cargo run -p amigo-codemap -- brief
cargo run -p amigo-codemap -- changed --group package --limit 20
cargo run -p amigo-codemap -- find "AssetTreePanel" --limit 20
cargo run -p amigo-codemap -- scope AssetTreePanel --limit 30
cargo run -p amigo-codemap -- refs asset-tree-section --limit 20
cargo run -p amigo-codemap -- docs
```

`brief`, `find`, `changed` i `docs` nie licza symboli, jesli nie trzeba. `scope` i `refs` wlaczaja glebszy kontekst lokalny.

Na Windowsie nie uruchamiamy wielu `cargo run -p amigo-codemap` rownolegle, bo `target/debug/amigo-codemap.exe` moze zablokowac sie przy przebudowie. Do wielu szybkich prob najpierw:

```powershell
cargo build -p amigo-codemap
target\debug\amigo-codemap.exe brief
```

---

## 2. Narzędzia zalecane na Windowsie

### Instalacja

W PowerShellu:

```powershell
winget install BurntSushi.ripgrep.MSVC
winget install sharkdp.fd
winget install difftastic
cargo install tokei
cargo install ast-grep --locked
```

Opcjonalnie:

```powershell
winget install jqlang.jq
```

`jq` nie jest wymagany, bo PowerShell potrafi filtrować JSON przez `ConvertFrom-Json`, ale bywa wygodny.

---

## 3. Czego unikać

### Nie używać bez potrzeby

```powershell
git diff
rg "Scene" .
tree /F
cargo check
cargo clippy
cargo tree
cat Cargo.lock
cat package-lock.json
cat pnpm-lock.yaml
```

Te komendy mogą wygenerować bardzo dużo tekstu, który szybko zużywa tokeny i utrudnia analizę.

---

## 4. Katalogi i pliki ignorowane przy analizie

Zwykle nie należy wrzucać do kontekstu:

```text
target/
node_modules/
dist/
build/
out/
.cache/
.git/
Cargo.lock
package-lock.json
pnpm-lock.yaml
yarn.lock
*.min.js
*.map
*.wasm
*.png
*.jpg
*.jpeg
*.webp
*.svg
```

Wyjątki:

- lockfile analizujemy tylko przy problemach z dependency resolution,
- SVG analizujemy tylko wtedy, gdy problem dotyczy konkretnej ikony,
- assety analizujemy tylko wtedy, gdy problem dotyczy ładowania assetów.

---

## 5. Bezpieczny workflow przed rozmową z agentem

### Mapa repozytorium

Zawsze zaczynamy od:

```powershell
cargo run -p amigo-codemap -- compact
```

Jeżeli pracujemy dłużej nad tym samym tematem, warto uruchomić watcher:

```powershell
cargo run -p amigo-codemap -- watch --level 1 --ai
```

Wtedy agentowi przekazujemy aktualną zawartość:

```text
.amigo/codemap.json
```

### Stan repozytorium

Po codemap, gdy trzeba zobaczyć tylko zmienione pliki:

```powershell
cargo run -p amigo-codemap -- changed
```

Fallback bez codemap:

```powershell
git status --short
git diff --stat
git diff --name-status
```

To daje szybki obraz zmian bez pełnego diffu.

### Lista istotnych plików Rust

Najpierw sprawdzamy `files`, `symbols` i `areas` w codemap. Ręcznie używamy:

```powershell
fd -e rs -e toml -e yaml . crates
```

### Lista istotnych plików frontendu / edytora

Najpierw sprawdzamy `areas` typu `editor-*` w codemap. Ręcznie używamy:

```powershell
fd -e ts -e tsx -e html -e css . apps/amigo-editor
```

### Skala projektu

```powershell
tokei crates apps
```

`tokei` pokazuje skalę projektu bez wypisywania treści plików.

---

## 6. Wyszukiwanie w kodzie

### Najpierw szukamy plików, nie wszystkich linii

Zamiast:

```powershell
rg "Scene" .
```

Używamy:

```powershell
rg -l "Scene" crates --type rust
```

Dopiero potem zawężamy:

```powershell
rg "Scene" crates/amigo-engine/src --type rust -n -C 2
```

### Przydatne opcje `rg`

```text
-l              pokaż tylko pliki z trafieniami
-n              pokaż numery linii
-C 2            pokaż 2 linie kontekstu
--type rust     tylko pliki Rust
--glob '*.tsx'  tylko pliki TSX
```

### Przykłady dla Amigo

```powershell
rg -l "load_scene" crates --type rust
rg -l "ModManifest" crates --type rust
rg -l "Scene" crates --type rust
rg "trait .*Loader" crates --type rust -n -C 2
rg "StartupDialog" apps/amigo-editor -n -C 2
```

---

## 7. Strukturalne szukanie kodu przez `ast-grep`

`ast-grep` jest lepsze od `rg`, gdy szukamy struktury kodu, a nie zwykłego tekstu.

W PowerShellu przy wzorcach z `$` używamy pojedynczych apostrofów.

### Rust

```powershell
ast-grep --lang rust -p 'struct $NAME { $$$ }' crates
ast-grep --lang rust -p 'impl $TYPE { $$$ }' crates
ast-grep --lang rust -p 'fn load_scene($$$) { $$$ }' crates
ast-grep --lang rust -p 'trait $NAME { $$$ }' crates
```

### TypeScript / React

```powershell
ast-grep --lang tsx -p 'function $NAME($$$) { $$$ }' apps/amigo-editor
ast-grep --lang tsx -p 'const $NAME = ($$$) => $$$' apps/amigo-editor
ast-grep --lang tsx -p '<$COMP $$$ />' apps/amigo-editor
```

### Kiedy używać `ast-grep`

Używamy, gdy pytanie brzmi np.:

- gdzie są definicje struktur,
- gdzie są implementacje,
- gdzie tworzymy komponent,
- gdzie emitujemy event,
- gdzie wywołujemy konkretną funkcję,
- gdzie mamy podobny wzorzec kodu.

---

## 8. Praca z diffami

### Nie zaczynamy od pełnego diffu

Nie używać jako pierwszej komendy:

```powershell
git diff
```

Najpierw, jeżeli potrzebujemy tylko zmian z Git:

```powershell
cargo run -p amigo-codemap -- changed
```

Fallback:

```powershell
git status --short
git diff --stat
git diff --name-status
```

Potem tylko konkretny plik:

```powershell
git diff -- crates/amigo-engine/src/scene.rs
git diff -- apps/amigo-editor/src/StartupDialog.tsx
```

### Dla większych zmian

```powershell
git diff --stat
git diff --name-only
```

Następnie wybieramy 1-3 najważniejsze pliki i dopiero je pokazujemy agentowi.

### `difftastic`

Dla bardziej czytelnych diffów kodu można używać:

```powershell
difft old.rs new.rs
```

Albo jako narzędzie Git, jeżeli jest skonfigurowane lokalnie.

`difftastic` bywa lepszy przy zmianach strukturalnych, ale nie zawsze będzie krótszy. Używać wtedy, gdy zwykły diff jest nieczytelny.

---

## 9. Rust: kompilacja i błędy

### Standardowy szybki check

```powershell
cargo check -q 2>&1 | Select-Object -First 120
```

Albo końcówka logu:

```powershell
cargo check -q 2>&1 | Select-Object -Last 120
```

### Tylko błędy z JSON przez PowerShell

```powershell
cargo check --message-format=json 2>$null |
  ForEach-Object {
    try { $_ | ConvertFrom-Json } catch {}
  } |
  Where-Object { $_.reason -eq "compiler-message" -and $_.message.level -eq "error" } |
  Select-Object -First 3 |
  ForEach-Object { $_.message.rendered }
```

To jest szczególnie dobre przy dużych błędach Rustowych, gdzie zwykłe `cargo check` zwraca ścianę tekstu.

### Clippy

Nie wrzucamy pełnego outputu z całego workspace, jeśli nie trzeba.

Lepiej:

```powershell
cargo clippy -q -p amigo-engine 2>&1 | Select-Object -First 120
```

Albo dla konkretnego crate’a:

```powershell
cargo clippy -q -p amigo-foundation 2>&1 | Select-Object -First 120
```

---

## 10. Cargo workspace i zależności

### Struktura workspace bez pełnych zależności

Zamiast:

```powershell
cargo tree
```

Używamy:

```powershell
cargo metadata --no-deps --format-version 1
```

Czytelniej przez PowerShell:

```powershell
cargo metadata --no-deps --format-version 1 |
  ConvertFrom-Json |
  Select-Object -ExpandProperty packages |
  Select-Object name, manifest_path
```

### Gdy naprawdę trzeba sprawdzić zależność

```powershell
cargo tree -p amigo-engine
cargo tree -i rhai
```

Nie używać pełnego `cargo tree` bez zawężenia, jeśli nie analizujemy zależności całego workspace.

---

## 11. Frontend / amigo-editor

Dla `amigo-editor` aktualne założenie: najpierw proste mockupy HTML/CSS, później funkcjonalny frontend Tauri + React + TypeScript.

### Lista plików edytora

Najpierw sprawdzamy codemap:

```powershell
cargo run -p amigo-codemap -- compact
```

Fallback:

```powershell
fd -e ts -e tsx -e html -e css . apps/amigo-editor
```

### Szukanie komponentów

Najpierw:

```powershell
cargo run -p amigo-codemap -- symbols --level 1
```

Potem zawężamy ręcznie:

```powershell
rg -l "StartupDialog" apps/amigo-editor
rg -l "useState" apps/amigo-editor --glob '*.tsx'
ast-grep --lang tsx -p 'function $NAME($$$) { $$$ }' apps/amigo-editor
```

### Build / TypeScript

Nie wrzucać pełnego logu builda.

```powershell
npm run build 2>&1 | Select-Object -First 120
```

Albo:

```powershell
npx tsc --noEmit --pretty false 2>&1 | Select-Object -First 120
```

Jeżeli projekt używa `pnpm`:

```powershell
pnpm build 2>&1 | Select-Object -First 120
pnpm tsc --noEmit --pretty false 2>&1 | Select-Object -First 120
```

---

## 12. Jak przekazywać kontekst agentowi

Najlepszy format wiadomości:

```text
Cel: chcę naprawić / dodać / przeprojektować X.

Kontekst architektury:
- crate / app: ...
- dotyczy: engine / editor / scripting / platform / assets / scene loading

Codemap:
[zawartość .amigo/codemap.json albo wynik cargo run -p amigo-codemap -- compact]

Stan repo:
[opcjonalnie wynik cargo run -p amigo-codemap -- changed]
[fallback: git status --short / git diff --stat]

Istotne pliki:
[file IDs i ścieżki z codemap]
[opcjonalnie wynik fd albo rg -l]

Błąd albo diff:
[tylko zawężony fragment]
```

Nie trzeba wrzucać całego repo. Lepiej dać agentowi codemap i pozwolić mu poprosić o konkretny fragment, jeżeli jest potrzebny.

---

## 13. Minimalny pakiet diagnostyczny

Gdy nie wiadomo, od czego zacząć, użyj:

```powershell
cargo run -p amigo-codemap -- compact
cargo run -p amigo-codemap -- changed
```

Jeżeli codemap nie wystarcza:

```powershell
git diff --stat
git diff --name-status
```

To zwykle wystarcza do rozpoczęcia pracy bez spalania tysięcy tokenów.

---

## 14. Minimalny pakiet dla błędu Rust

```powershell
cargo run -p amigo-codemap -- compact
cargo run -p amigo-codemap -- symbols --level 1
cargo check -q 2>&1 | Select-Object -First 120
```

Potem pokazujemy konkretny plik wskazany przez codemap albo zawężamy ręcznie:

```powershell
rg -l "NAZWA_SYMBOLU_Z_BŁĘDU" crates --type rust
rg "NAZWA_SYMBOLU_Z_BŁĘDU" crates/path/to/file.rs -n -C 5
```

---

## 15. Minimalny pakiet dla zmiany architektury

```powershell
cargo run -p amigo-codemap -- compact
cargo run -p amigo-codemap -- symbols --level 1
cargo metadata --no-deps --format-version 1
```

Następnie opisujemy oczekiwany kierunek:

```text
Chcę utrzymać SOLID/SRP.
Nie chcę mieszać platformy z engine.
Nie chcę zależności od edytora w runtime engine.
Scripting ma zostać odizolowany.
```

---

## 16. Minimalny pakiet dla amigo-editor

```powershell
cargo run -p amigo-codemap -- compact
cargo run -p amigo-codemap -- symbols --level 1
cargo run -p amigo-codemap -- changed
```

Potem zawężamy tylko potrzebny obszar:

```powershell
rg -l "StartupDialog" crates/apps/amigo-editor
npm run build 2>&1 | Select-Object -First 120
```

---

## 17. Zasady projektowe Amigo

Przy zmianach w projekcie należy pilnować:

- modułowości,
- pojedynczej odpowiedzialności crate’ów i modułów,
- izolacji platformy od logiki engine,
- izolacji edytora od runtime engine,
- czytelnych granic między foundation, platform, engine, scripting, 2D, 3D i apps,
- mod-first development,
- scen YAML jako podstawowego wejścia dla demo,
- Rhai jako warstwy skryptowej,
- prostych, testowalnych kontraktów między modułami.

---

## 18. Zasady dla amigo-editor

Aktualny kierunek:

- desktop viewer + lekki asset editor,
- nie pełny game editor,
- Tauri v2 + React + TypeScript + Vite jako główny kandydat,
- ciemny lub ciemnoniebieski styl UI,
- gotowe komponenty dla paneli, tree view, form, tabs, dialogs, search,
- custom Canvas/WebGL dla podglądów scen, tilesetów, atlasów i sprite’ów,
- CodeMirror dla YAML / tekstu,
- wavesurfer.js dla audio preview,
- mockupy HTML/CSS jako pierwszy etap iteracji UI.

Przy projektowaniu Startup Dialog:

- większe okno,
- fixed size,
- bez maksymalizacji,
- prawdziwa lista modów,
- prawdziwe metadane modów,
- generowany preview moda/sceny,
- brak placeholderów, jeśli da się podpiąć realne dane,
- UI emituje eventy,
- dispatcher/task registry obsługuje pracę,
- backend skanuje/validuje mody przez kontrakty engine,
- UI wyświetla state, diagnostics, busy indication i preview.

---

## 19. Reguła końcowa

Jeżeli output ma więcej niż około 120 linii, prawie zawsze należy go ograniczyć.

PowerShell:

```powershell
... | Select-Object -First 120
... | Select-Object -Last 120
```

Najpierw mapa, potem szczegół.

Najpierw pliki, potem linie.

Najpierw statystyka, potem diff.

Najpierw pierwszy błąd, potem reszta.
