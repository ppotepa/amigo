# Amigo Project Workflow

Przewodnik opisuje zalecany sposób pracy z projektem **Amigo** przy współpracy z ChatGPT / agentem kodującym. Celem jest ograniczenie zużycia tokenów, unikanie zalewania kontekstu dużymi logami oraz utrzymanie czytelnej, modułowej pracy nad projektem.

Projekt zakłada pracę głównie na **Windowsie**, najlepiej w **PowerShellu**.

Domyślnym pierwszym źródłem kontekstu jest teraz **amigo-codemap**.

`amigo-codemap` jest utrzymywany jako osobne repo i podpięty tutaj jako **git submodule** pod:

```text
crates/tools/amigo-codemap
```

Po świeżym klonie repo:

```powershell
git submodule update --init --recursive
```

Jeżeli submodule ma zostać podciągnięty do nowszej wersji:

```powershell
git submodule update --remote -- crates/tools/amigo-codemap
```

Na start preferujemy najkrotszy widok:

```powershell
cargo run -p amigo-codemap -- brief
```

Gdy potrzebna jest pelna mapa diagnostyczna:

```powershell
cargo run -p amigo-codemap -- compact
```

W codziennej pracy preferuj jednak raporty operacyjne:

```powershell
cargo build -p amigo-codemap
Copy-Item target\debug\amigo-codemap.exe target\debug\amigo-codemap-stable.exe
$cm = "target\debug\amigo-codemap-stable.exe"
& $cm change-plan <query> --limit 20
& $cm trace <thing> --limit 20
& $cm open-set <thing> --why --limit 10
```

Ręczne komendy `git`, `rg`, `fd`, `cargo` i `npm` służą jako doprecyzowanie po odczytaniu mapy projektu albo jako fallback, gdy codemap jest niedostępny.

---

## 1. Główna zasada

Nie pokazujemy od razu całego repozytorium, pełnych diffów ani pełnych logów.

W każdej pracy wybieramy **najbardziej optymalną ścieżkę**, czyli taką, która daje poprawną odpowiedź przy najmniejszym:

- koszcie tokenów,
- koszcie ręcznego czytania,
- koszcie hałaśliwych logów,
- ryzyku przypadkowego rozszerzenia zakresu.

To oznacza:

1. najpierw gotowy raport `amigo-codemap`,
2. potem tylko minimalny ręczny doprecyzowujący krok,
3. dopiero na końcu build/test/fallout.

Nie zaczynamy od “ręcznej archeologii”, jeśli to samo można uzyskać krótszą ścieżką przez istniejący raport.

Najpierw pokazujemy **mapę problemu przez codemap**:

```powershell
cargo run -p amigo-codemap -- compact
```

Jeżeli agent potrzebuje aktualnego stanu zmian:

```powershell
cargo run -p amigo-codemap -- changed
```

Jeżeli potrzebne są symbole albo sygnatury:

```powershell
cargo run -p amigo-codemap -- symbols --level 1
cargo run -p amigo-codemap -- signature <symbol>
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

Codemap core pozostaje kompaktowym, language-agnostic indeksem. Dodatkowe raporty `amigo-codemap` moga jednak zawierac lekkie adaptery heurystyczne dla `amigo-editor`, np. Tauri commands, registry, service bags albo plan weryfikacji. Te raporty nie zastepuja kompilatora, LSP ani pelnego analizatora AST; sluza do szybkiego zawężenia pracy i wskazania nastepnego kroku.

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
3. `amigo-codemap change-plan <query>` - pierwszy plan pracy dla taska.
4. `amigo-codemap trace <thing>` - gdy wejście może być symbolem, stringiem, ID, command name, YAML key albo CSS class.
5. `amigo-codemap open-set <thing> --why` - ranking plików do czytania.
6. `amigo-codemap signature <symbol>` i `slice <file> --symbol <symbol>` - minimalny kod zamiast pełnego pliku.
7. `amigo-codemap impact <query>` i `verify-plan --changed` - ryzyko i weryfikacja.
8. `amigo-codemap taxonomy`, `anchors`, `anchor-check` - gdy praca dotyczy domen, anchorów albo nawigacji repo.
9. `rg -l`, `fd`, `git diff --stat`, `git diff --name-status` - fallback/doprecyzowanie.
10. `rg -n -C`, `Get-Content`, zawężony `git diff -- <plik>` - konkretne fragmenty.
11. `cargo`, `npm`, `vitest` albo `amigo-codemap verify` - weryfikacja po zmianach.

Nie zaczynamy od pełnego `git diff`, pełnego `rg` po repo ani pełnych logów builda.
Nie zaczynamy od `Get-Content` całych plików, repo-wide `rg` ani `concat-output.txt`.

Każda odpowiedź implementacyjna powinna mieć: cel zmiany, nawigację codemap (`change-plan`, `trace`, `open-set`, `symbols`, `signature`, `slice/range`), oczekiwany open-set, instrukcje per plik, konkretne zmiany kodu (`CREATE FILE`, `REPLACE SYMBOL`, `INSERT`, `MODIFY ENUM`, `MODIFY MATCH`) oraz testy/verify.

W czacie preferujemy raw ops zamiast YAML:

```text
ACTION:
FILE:
SYMBOL:
WITHIN_SYMBOL:
FIND:
REPLACE:
CONTENT:
END
```

Jeśli komenda wewnętrznie potrzebuje `OpsPlan`, używamy `ops-preview|ops-check|ops-apply --raw`, a adapter w `amigo-codemap` tłumaczy raw bloki na model ops.

### Codemap taxonomy workflow

Przed większą pracą nawigacyjną:

```powershell
target\debug\amigo-codemap.exe taxonomy
target\debug\amigo-codemap.exe anchors priority:P0 --limit 20
target\debug\amigo-codemap.exe anchor-check
```

Dla pracy domenowej:

```powershell
target\debug\amigo-codemap.exe anchors domain:ui-document --limit 20
target\debug\amigo-codemap.exe open-set ui-document --why --limit 10
target\debug\amigo-codemap.exe change-plan ui-document --limit 20
```

Dla nieznanego symbolu, stringa, ID albo anchora:

```powershell
target\debug\amigo-codemap.exe trace <thing>
```

Reguły anchorów:

1. P0/P1 anchory powinny być ręczne i znaczące.
2. P2 anchory mogą być file-level coverage.
3. Nie opieramy ważnych edycji wyłącznie na numerach linii.
4. Po zmianie anchorów uruchamiamy `anchors --write` i `anchor-check`.

### Codemap jako część feature work

`amigo-codemap` jest żywym indeksem repo, więc większe zmiany funkcjonalne powinny aktualizować go razem z kodem.

Gdy dodajesz albo przenosisz feature w engine, editorze, runtime, backendzie albo modach:

1. Dodaj ręczne P0/P1 anchory w nowych istotnych miejscach: entrypointy, dispatchery, registry, modele, DTO, command handlers, root editory, sceny YAML, skrypty.
2. Jeśli pojawia się nowy obszar, pojęcie albo warstwa, zaktualizuj `.amigo/codemap.taxonomy.yml`.
3. Po zmianach uruchom:

```powershell
target\debug\amigo-codemap.exe anchors --write
target\debug\amigo-codemap.exe anchor-check
```

4. Commituj razem kod feature’a, zmienione ręczne anchory/taksonomię oraz wygenerowane:

```text
.amigo/codemap.anchors.generated.json
.amigo/codemap.coverage.generated.md
```

Acceptance criterion dla większych zmian: `codemap` musi dalej prowadzić agenta do nowych miejsc przez `trace`, `open-set`, `change-plan` albo `anchors`.

### Jeśli narzędzia brakuje

Jeżeli przy pracy regularnie pojawia się potrzeba:

- ręcznego powtarzania tej samej sekwencji `rg`,
- ręcznego grupowania tych samych plików,
- ręcznego filtrowania dużych logów,
- ręcznego budowania tych samych checklist `co czytać / co odpalić / co może się zepsuć`,

to znaczy, że prawdopodobnie **brakuje raportu lub helpera w `amigo-codemap`**.

Wtedy preferowana ścieżka jest taka:

1. zanotować wzorzec w `operations.md`,
2. ocenić, czy to powtarzalny problem,
3. dodać nowe wejście do `crates/tools/amigo-codemap`,
4. dopiero potem wrócić do właściwego taska z krótszym workflow.

Innymi słowy:

```text
jeśli narzędzie nie istnieje, a problem jest powtarzalny,
to warto dołożyć je do amigo-codemap zamiast utrwalać ręczny workflow
```

Dotyczy to szczególnie:

- impact/refactor planning,
- file move/delete fallout,
- stale cleanup,
- registry/service bag checks,
- build-log condensation,
- task/workset planning.

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

## 1c. Małe widoki codemap

Najczęstsze komendy:

```powershell
cargo run -p amigo-codemap -- brief
cargo run -p amigo-codemap -- changed --group package --limit 20
cargo run -p amigo-codemap -- change-plan "AssetTreePanel" --limit 20
cargo run -p amigo-codemap -- trace "asset-tree-section" --limit 20
cargo run -p amigo-codemap -- open-set "AssetTreePanel" --why --limit 10
cargo run -p amigo-codemap -- signature AssetTreePanel
cargo run -p amigo-codemap -- where AssetTreePanel --limit 20
cargo run -p amigo-codemap -- docs
```

`brief`, `changed` i `docs` są najtańsze. `change-plan`, `trace`, `open-set`, `signature`, `where`, `impact` włączają głębszy kontekst tylko wtedy, gdy jest potrzebny.

Na Windowsie nie uruchamiamy wielu `cargo run -p amigo-codemap` rownolegle, bo `target/debug/amigo-codemap.exe` moze zablokowac sie przy przebudowie. Do wielu szybkich prob najpierw budujemy binarke:

```powershell
cargo build -p amigo-codemap
target\debug\amigo-codemap.exe brief
```

## 1d. Codemap fast mode

Przy dłuższych sesjach trzymaj `watch --write` w osobnym terminalu. Watcher utrzymuje zarówno kompaktowy output, jak i szybki snapshot:

```text
.amigo/codemap.snapshot.json
```

Terminal watcher:

```powershell
cargo build -p amigo-codemap
target\debug\amigo-codemap.exe watch --write
```

Terminal pracy:

```powershell
$cm = "target\debug\amigo-codemap.exe"

& $cm status
& $cm changes --compact --hide-generated --limit 20
& $cm trace <thing> --limit 20
& $cm open-set <thing> --why --limit 10
& $cm impact <thing> --limit 30
& $cm verify-plan --changed
```

Jeśli snapshot może być nieaktualny:

```powershell
& $cm refresh
```

Jeśli trzeba wymusić pełny skan:

```powershell
& $cm trace <thing> --no-cache
```

Do sprawdzania aktualnego dirty state używamy `changes`, bo czyta live git i nie polega na snapshot cache:

```powershell
& $cm changes --compact --hide-generated
& $cm changes --group domain
& $cm changes --warnings
& $cm commit-plan --compact
```

`changed` zostaje raportem snapshotowym i może być stale, jeśli watcher nie działał. `git status --short` i `git diff --stat` są fallbackiem, nie domyślną ścieżką.

`codemap.snapshot.json` jest cache’em runtime i nie powinien być commitowany.

---

## 1e. Raporty operacyjne codemap

Po zbudowaniu `amigo-codemap` preferujemy szybkie raporty z binarki:

```powershell
cargo build -p amigo-codemap
target\debug\amigo-codemap.exe changes --compact --hide-generated --limit 20
target\debug\amigo-codemap.exe verify-plan --changed
```

Przy dłuższej pracy nie używamy bez potrzeby w kółko `cargo run -p amigo-codemap`. Optymalna ścieżka na Windowsie to:

```powershell
cargo build -p amigo-codemap
target\debug\amigo-codemap.exe ...
```

To ogranicza przebudowy i blokowanie binarki przez kolejne wywołania.

Raporty operacyjne maja konczyc sie sekcja `next:`. Traktujemy ja jako domyslna kolejke pracy: co przeczytac, co poprawic i co odpalic po zmianach.

Przy dluzszych taskach zamiast odtwarzac recznie liste plikow z `impact` albo `open-set`, zapisujemy workset:

```powershell
target\debug\amigo-codemap.exe impact EditorSelectionRef --group feature --limit 80
target\debug\amigo-codemap.exe workset selection-migration --from-impact EditorSelectionRef --save
target\debug\amigo-codemap.exe workset selection-migration --status
```

Workset zapisuje manifest w `.amigo/worksets/*.json` i pokazuje tylko zapisane pliki/checki, bez fallbacku do calego dirty tree.

Szybki dobor komendy:

```text
co sie zmienilo live               -> changes, commit-plan
co sie zmienilo w snapshotcie      -> changed, diff-scope
jak dziala komenda codemap         -> command-map
co czytac najpierw                 -> open-set, slice
jaki jest najlepszy append anchor  -> append-plan
jaki donor skopiowac i co przemianowac -> copy-plan
czy unified diff wejdzie czysto       -> patch-check
juz sprawdzony unified diff zastosuj  -> patch-apply --write
jaki jest zasieg zmiany symbolu    -> impact
czy mozna usunac plik              -> delete-plan
co zepsuje move pliku              -> file-move-plan, import-fix-plan
czy zostaly stare aliasy/shimy     -> stale, orphan-files, shim-check
co sprawdzic po zmianie            -> verify-plan, fallout
jak rozbic zmiany na commity       -> commit-plan, commit-files, commit-summary
```

### Dobor raportu do zadania

```powershell
# Co zweryfikowac po obecnych zmianach
target\debug\amigo-codemap.exe verify-plan --changed

# Jaki jest zasieg zmiany symbolu/typu
target\debug\amigo-codemap.exe impact EditorSelectionRef --group feature --limit 80

# Czy stare nazwy/helpery zostaly po refaktorze
target\debug\amigo-codemap.exe stale --patterns workspacePanels,createEditorSelection,RegisteredComponentPlaceholder --limit 80

# Jak rozbic duzy plik commandow Tauri
target\debug\amigo-codemap.exe move-plan crates/apps/amigo-editor/src-tauri/src/commands/mod.rs --by tauri-command --limit 100

# Czy commandy Tauri sa zdefiniowane i zarejestrowane
target\debug\amigo-codemap.exe tauri-commands --limit 100

# Czy service bag jest za szeroki
target\debug\amigo-codemap.exe service-shape WorkspaceRuntimeServices --limit 100

# Czy registry ma duplikaty/placeholders/braki
target\debug\amigo-codemap.exe registry-check properties --limit 100
target\debug\amigo-codemap.exe registry-check components --limit 100

# Czy helper ma duplikaty
target\debug\amigo-codemap.exe dup reveal_path --limit 80

# Co bylo kosztowne w poprzednich pracach
target\debug\amigo-codemap.exe operations-summary --limit 20

# Jak jest podpięta konkretna komenda codemap
target\debug\amigo-codemap.exe command-map append-plan
target\debug\amigo-codemap.exe command-map copy-plan

# Co wpisac w final response/commit summary
target\debug\amigo-codemap.exe commit-summary --changed --limit 80
```

### File-ops workflow (operacje na plikach bez edycji)

Wersja `file-ops` ma być read-only i nastawiona na planowanie:

```powershell
# 1) zobacz, co się zmieniło
target\debug\amigo-codemap.exe diff-scope --changed --limit 80

# 2) ograniczanie kontekstu i zakresu czytania
target\debug\amigo-codemap.exe impact EditorSelectionRef --group feature --limit 80
target\debug\amigo-codemap.exe open-set EditorSelectionRef --task migrate --limit 12
target\debug\amigo-codemap.exe slice crates/apps/amigo-editor/src/app/editorStore.tsx --symbol EditorStoreProvider --radius 40
target\debug\amigo-codemap.exe append-plan crates/apps/amigo-editor/src/editor-components/builtinComponents.tsx --task component-definition --limit 12
target\debug\amigo-codemap.exe copy-plan crates/apps/amigo-editor/src/startup/NewPanel.tsx --from crates/apps/amigo-editor/src/startup/ModsPanel.tsx --task panel --limit 12

# 3) porzadkuj refaktory plikowe
target\debug\amigo-codemap.exe stale --patterns workspacePanels,createEditorSelection --limit 80
target\debug\amigo-codemap.exe move-plan crates/apps/amigo-editor/src-tauri/src/commands/mod.rs --by tauri-command --limit 100
target\debug\amigo-codemap.exe dup reveal_path --limit 80

# 4) przed usunięciem/przeniesieniem
target\debug\amigo-codemap.exe delete-plan crates/apps/amigo-editor/src/main-window/workspacePanels.tsx --changed
target\debug\amigo-codemap.exe file-move-plan crates/apps/amigo-editor/src/assets/AssetTreePanel.tsx --to crates/apps/amigo-editor/src/features/assets/AssetTreePanel.tsx
target\debug\amigo-codemap.exe rename-plan selectedAsset --to selectedAssetKey --group feature
target\debug\amigo-codemap.exe import-fix-plan --changed
target\debug\amigo-codemap.exe patch-preview --from patch.diff --limit 80
target\debug\amigo-codemap.exe patch-check --from patch.diff --limit 80
target\debug\amigo-codemap.exe patch-apply --from patch.diff --write

# 5) sprzatanie i walidacje
target\debug\amigo-codemap.exe orphan-files crates/apps/amigo-editor/src/features --limit 50
target\debug\amigo-codemap.exe shim-check --changed
target\debug\amigo-codemap.exe barrel-check crates/apps/amigo-editor/src/app/store
target\debug\amigo-codemap.exe large-files --top 20 --with-split-hints
target\debug\amigo-codemap.exe workset selection-migration --from-impact EditorSelectionRef --save
target\debug\amigo-codemap.exe workset selection-migration --status
target\debug\amigo-codemap.exe commit-files --changed
```

Każdy raport ma ten sam format:

```text
task:
scope:
findings:
risk:
verify:
next:
```

### Jak uzywac `command-map`, `append-plan`, `copy-plan`

To sa trzy raporty, ktore maja chronic przed recznym `rg` i przed czytaniem zbyt wielu plikow.

#### `command-map`

Uzywamy tylko wtedy, gdy rozwijamy samo `amigo-codemap`.

```powershell
target\debug\amigo-codemap.exe command-map copy-plan
```

Kolejnosc czytania:

1. `cli`
2. `dispatch`
3. `implementation`
4. `docs`
5. `tests`

To ma zastapic reczne szukanie typu:

```powershell
rg -n "copy-plan|AppendPlan|CopyPlan" crates/tools/amigo-codemap
```

#### `append-plan`

Uzywamy, gdy plik juz istnieje i chcemy cos **dopisać**:
- nowy wpis do registry
- nowy case w switchu
- nowa route
- nowy blok CSS
- nowy test

```powershell
target\debug\amigo-codemap.exe append-plan crates/apps/amigo-editor/src/editor-components/builtinComponents.tsx --task component-definition --limit 12
```

Domyslna interpretacja:

1. `append anchors` - wybierz pierwszy sensowny anchor strukturalny
2. `donor candidates` - czytaj tylko wtedy, gdy zmiana jest mechaniczna
3. `companion files` - sprawdz, czy trzeba dopisac import/rejestracje/style
4. `verify` - odpal tylko najmniejsze potrzebne checki

Nie dopisujemy w ciemno na EOF, jesli raport pokazuje lepszy anchor.

#### `copy-plan`

Uzywamy, gdy chcemy cos **skopiowac**:
- nowy panel na bazie podobnego panelu
- nowe okno na bazie istniejacego
- nowy test/scaffold na bazie innego pliku
- wiekszy blok przeniesiony z donor file

```powershell
target\debug\amigo-codemap.exe copy-plan crates/apps/amigo-editor/src/startup/NewPanel.tsx --from crates/apps/amigo-editor/src/startup/ModsPanel.tsx --task panel --limit 12
```

Domyslna interpretacja:

1. `selected donor` - to jest plik startowy
2. `alternate donors` - zwykle nie czytamy wiecej niz 1-2 alternatyw
3. `rename hotspots` - poprawiamy to przed importami i propsami
4. `mirrored companion files` - kopiujemy tylko jesli donor naprawde ich potrzebuje
5. `target anchors` - jesli target juz istnieje, przed wklejeniem odpal `append-plan <target>`

Praktyczna regula:

```text
target istnieje i dopisujesz -> append-plan
target nie istnieje albo kopiujesz wzorzec -> copy-plan
rozwijasz samo amigo-codemap -> command-map
```

### Build fallout

Nie wrzucamy pelnego logu builda do rozmowy. Najpierw przepuszczamy go przez `fallout`:

```powershell
npm run build 2>&1 | target\debug\amigo-codemap.exe fallout --limit 80
cargo test -p amigo-editor --lib 2>&1 | target\debug\amigo-codemap.exe fallout --limit 80
```

Z pliku:

```powershell
target\debug\amigo-codemap.exe fallout --from npm-build.log --limit 80
target\debug\amigo-codemap.exe fallout --from cargo-test.log --limit 80
```

Kolejnosc napraw po `fallout`:

1. missing imports / missing exports,
2. visibility / re-export fallout,
3. type shape mismatch,
4. property/argument mismatch,
5. ponowienie oryginalnej komendy.

### Workflow refaktoru

Przed czytaniem plikow:

```powershell
target\debug\amigo-codemap.exe changed --group package --limit 20
target\debug\amigo-codemap.exe impact NAZWA_SYMBOLU --group feature --limit 80
target\debug\amigo-codemap.exe verify-plan --changed
```

Przy splitach:

```powershell
target\debug\amigo-codemap.exe move-plan PATH_DO_PLIKU --by tauri-command --limit 100
target\debug\amigo-codemap.exe dup NAZWA_HELPERA --limit 80
target\debug\amigo-codemap.exe tauri-commands --limit 100
```

Przy cleanupie:

```powershell
target\debug\amigo-codemap.exe stale --patterns oldName,LegacyThing,PlaceholderName --limit 80
target\debug\amigo-codemap.exe registry-check components --limit 100
target\debug\amigo-codemap.exe registry-check properties --limit 100
```

Przed zakonczeniem:

```powershell
target\debug\amigo-codemap.exe verify-plan --changed
target\debug\amigo-codemap.exe commit-summary --changed --limit 80
```

Wynik `verify-plan` jest domyslna lista checkow. Pelny workspace test odpalamy tylko wtedy, gdy raport albo zmiana publicznego API wskazuje realne ryzyko.

### Reguła optymalnej ścieżki

Praktyczna reguła wyboru ścieżki:

```text
1. codemap report
2. minimalny manual read
3. implementacja
4. verify-plan
5. build/test
6. fallout tylko jeśli log jest głośny
```

Jeżeli w danym zadaniu ktoś zaczyna od:

- pełnego `git diff`,
- pełnego `Get-Content` dużego pliku,
- szerokiego `rg` po całym repo,
- pełnego logu `cargo` / `npm`,

to zwykle nie jest to ścieżka optymalna.

Najpierw pytamy:

```text
czy istnieje raport codemap, który zawęzi to do kilku plików, symboli albo ryzyk?
```

Jeżeli tak, raport ma pierwszeństwo.

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

### Patch-preview

Przed podaniem patcha lub dużego `git diff` dajemy krótką mapę zmian:

```powershell
git diff > patch.diff
target\debug\amigo-codemap.exe patch-preview --from patch.diff --limit 80
```

### Patch-check / patch-apply

Gdy użytkownik podaje gotowy unified diff i celem jest oszczędność tokenów, najpierw sprawdzamy, czy hunk context pasuje do workspace:

```powershell
target\debug\amigo-codemap.exe patch-check --from patch.diff --limit 80
```

Jeżeli raport pokazuje `applies: yes`, dopiero wtedy wolno zastosować patch:

```powershell
target\debug\amigo-codemap.exe patch-apply --from patch.diff --write
```

Bez `--write` `patch-apply` działa jak dry-run. Komendy przyjmują też patch ze stdin, ale preferujemy `--from patch.diff`, bo łatwiej powtórzyć weryfikację i uniknąć utraty kontekstu.

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

---

## 20. Codemap-first workflow 0.1

Ta sekcja jest domyślną procedurą pracy agenta w repo Amigo. `amigo-codemap` ma być pierwszym krokiem przy każdym nietrywialnym zadaniu, bo redukuje zgadywanie, liczbę otwieranych plików i koszt tokenów.

Najpierw zbuduj binarkę i używaj jej bez `cargo run`, szczególnie na Windowsie:

```powershell
cargo build -p amigo-codemap
$cm = "target\debug\amigo-codemap.exe"
```

Domyślna pętla:

```powershell
& $cm changed --group package --limit 20
& $cm change-plan <query> --limit 20
& $cm trace <thing> --limit 20
& $cm open-set <thing> --why --limit 10
& $cm symbols --file <file> --metadata --limit 40
& $cm signature <symbol>
& $cm slice <file> --symbol <symbol>
& $cm impact <symbol-or-query> --limit 30
& $cm verify-plan --changed
```

Reguła praktyczna:

```text
Nie otwieraj pełnych plików jako pierwszego kroku.
Najpierw ustal scope przez codemap.
Potem czytaj signature.
Potem slice konkretnego symbolu.
Dopiero potem pełny plik, jeśli nadal jest potrzebny.
```

### Problem -> komenda

| Problem | Pierwsza komenda | Następny krok |
| --- | --- | --- |
| Nie wiem, co obejmuje task | `change-plan <query>` | `open-set <query> --why` |
| Mam symbol i szukam definicji | `where <symbol>` | `signature <symbol>` |
| Potrzebuję parametrów i return type | `signature <symbol>` | `slice <file> --symbol <symbol>` |
| Chcę listę metod z jednego pliku | `symbols --file <path> --metadata` | `slice <file> --symbol <symbol>` |
| Mam string, ID, CSS class albo command name | `trace <thing>` | `open-set <thing> --why` |
| Nie wiem, które pliki czytać | `open-set <query> --why` | `slice` top symboli |
| Chcę zrozumieć jeden plik | `explain-file <path>` | `neighbors <path>` |
| Chcę powiązania pliku | `neighbors <path>` | `impact <symbol-or-query>` |
| Chcę public/export API | `api-surface` | `signature <symbol>` |
| Chcę graf komponentów TSX | `component-graph` | `trace <component>` |
| Chcę frontend/backend Tauri flow | `tauri-graph` | `trace <command>` |
| Chcę możliwe callsite'y | `callsite-candidates <symbol>` | `impact <symbol>` |
| Chcę TODO/risk scope | `todo-index` / `risk-index` | `workset <name> --save` |
| Mam gotowy diff | `patch-check --from patch.diff` | `patch-apply --from patch.diff --write` |
| Mam deklaratywny plan zmian | `ops-check --from plan.yml` | `ops-apply --from plan.yml --write` |
| Mam YAML jako string | `ops-check --yaml $yaml` | `ops-apply --yaml $yaml --write` |
| Chcę format YAML ops | `ops-schema --example replace_symbol` | `ops-skeleton <query>` |
| Mam kod w osobnych plikach | `content_from` w ops-planie | `content_root: updates` |
| Chcę szkielet planu zmian | `ops-skeleton <query> --out plan.yml --write` | `ops-check --from plan.yml` |
| Chcę stabilny zakres symbolu | `range-for-symbol <symbol>` | `ops-skeleton` albo ręczny `plan.yml` |
| Chcę YAML dla konkretnych linii | `range-for-lines <path> <start> <end> --yaml-op replace_range` | `ops-check --from plan.yml --strict` |
| Chcę stabilny zakres anchora | `anchor-range <anchor>` | `replace_between_anchors` albo `insert_after_anchor` |

### Jak czytać wyniki

Raporty codemap powinny mieć przewidywalne sekcje:

```text
scope
definitions
references
text/config
related files
risks
next
verify
```

Sekcja `next:` jest domyślną kolejką pracy. Jeśli raport mówi, żeby odpalić `signature`, `slice`, `impact` albo `verify-plan`, agent powinien wykonać to przed ręcznym `rg` lub pełnym `Get-Content`.

### Signature i slice

Przed czytaniem implementacji:

```powershell
& $cm signature <symbol>
```

Jeżeli najpierw potrzebujesz listy metod/symboli z jednego pliku:

```powershell
& $cm symbols --file <file> --metadata --limit 40
```

Ten krok jest szczególnie przydatny po `open-set`, gdy plik jest duży i nie wiadomo jeszcze, który symbol czytać. Reporter jest wspólny dla języków, ale jakość danych zależy od skanera:

```text
Rust/TS/TSX: dobre params/returns/generics/ranges.
Rhai: podstawowe funkcje i lekkie sygnatury.
YAML: key-like symbols bez klasycznych parametrów.
CSS: selektory jako symbole.
JSON/TOML/Markdown: zwykle używaj trace/explain-file zamiast symbols.
```

Jeżeli signature wskazuje właściwy plik i zakres:

```powershell
& $cm slice <file> --symbol <symbol>
```

Pełny plik wolno otworzyć dopiero wtedy, gdy:

1. symbol range jest błędny lub niepełny,
2. potrzebny jest kontekst kilku sąsiadujących symboli,
3. zmiana dotyczy importów, re-exportów lub układu całego pliku,
4. `slice` nie obejmuje potrzebnego kodu pomocniczego.

### Trace

Używaj `trace`, gdy wejście może być czymkolwiek innym niż zwykła nazwa symbolu:

```powershell
& $cm trace entity.inspector --limit 20
& $cm trace send_editor_pointer_event --limit 20
& $cm trace .workspace-panel --limit 20
```

`trace` jest właściwą komendą dla:

```text
string literal
dock id
scene id
asset id
CSS class
Tauri command
Rhai function
YAML key/value
@codemap anchor
```

### Open-set

Przed wyborem plików do czytania:

```powershell
& $cm open-set <thing> --why --limit 10
```

`--why` jest istotne, bo pokazuje powody rankingu: definition, signature match, text occurrence, anchor, changed state, domain tag albo risk tag.

Zasada:

```text
Czytaj top 1-3 pliki z open-set.
Nie otwieraj całej listy.
Jeśli top wyniki są złe, doprecyzuj query zamiast rozszerzać kontekst ręcznie.
```

### Impact i verify

Przed zmianą publicznego API, shared type, Tauri command, registry albo config ID:

```powershell
& $cm impact <symbol-or-query> --limit 30
```

Po zmianach:

```powershell
& $cm verify-plan --changed
```

Następnie uruchom realne komendy z raportu. Codemap nie zastępuje kompilatora, TypeScript, testów ani review.

### Patch i ops

Unified diff:

```powershell
& $cm patch-preview --from patch.diff
& $cm patch-check --from patch.diff
& $cm patch-apply --from patch.diff --write
```

Deklaratywny ops-plan:

```powershell
& $cm ops-skeleton <query> --out plan.yml --write
& $cm ops-schema --example replace_symbol
& $cm range-for-symbol <symbol>
& $cm anchor-range <anchor>
& $cm ops-preview --from plan.yml
& $cm ops-check --from plan.yml --strict
& $cm ops-apply --from plan.yml --write --backup --stop-on-error
& $cm ops-verify --from plan.yml
& $cm ops-summary --from plan.yml --changed
```

Ops-plan domyślnie jest traktowany jako v1. `version: 1` jest opcjonalne i zostaje tylko jako jawny znacznik kompatybilności dla długich albo archiwalnych planów.

Dla większych zmian preferuj paczkę:

```text
.amigo/ops/task-name/
  plan.yml
  updates/
    NewPanel.tsx
    replacement.ts
```

`plan.yml`:

```yaml
task: task-name
content_root: updates
ops:
  - id: create-panel
    kind: create_file
    path: crates/apps/amigo-editor/src/features/example/NewPanel.tsx
    content_from: NewPanel.tsx

  - id: replace-range
    kind: replace_range
    path: crates/apps/amigo-editor/src/features/example/Existing.tsx
    start_line: 20
    end_line: 40
    expected_hash: "abc12345"
    content_from: replacement.ts
```

Zasady `content_from`:

```text
content_from jest relative do katalogu planu albo do content_root pod katalogiem planu.
Używaj dokładnie jednego z content/replace albo content_from.
Ścieżki ops muszą być repo-relative.
Absolute path i .. są odrzucane.
```

Obsługiwane operacje plikowe w ops-planie:

```text
create_file
replace_file
delete_file
copy_file
move_file
rename_file
create_dir
delete_dir
append_to_file
replace_range
delete_range
insert_before_text
insert_after_text
replace_text
insert_before_anchor
insert_after_anchor
replace_between_anchors
```

`rename_file` jest aliasem semantycznym `move_file`.

`ops-check` rozumie podstawową sekwencję FS ops, np. `create_file -> copy_file -> move_file`.

`ops-apply --strict` respektuje strict validation, aplikuje wszystkie operacje niezależnie od `--limit` i kończy non-zero, jeśli dowolna operacja failuje.

Od teraz średnie i duże implementacje opisujemy w formacie ops-first:

```text
Task:
  short-name

Intent:
  co zmiana naprawia

Codemap:
  komendy użyte do zawężenia

Files:
  lista plików

Ops:
  plan.yml

Verify:
  komendy

Acceptance:
  warunki końcowe
```

Preferowana kolejność bezpieczeństwa dla operacji:

```text
symbol-aware op + expected_hash, jeśli signature/slice potwierdza zakres
@codemap anchor + context
replace_between_anchors
replace_range + expected_hash + context_before/context_after
replace_file
unified diff fallback
```

Operacje symbolowe (`replace_symbol`, `delete_symbol`, `insert_before_symbol`, `insert_after_symbol`, `replace_method_body`) traktuj jako eksperymentalne do czasu sprawdzenia `signature` i `slice` dla konkretnego pliku.

Ops-plan może wejść trzema drogami:

```powershell
& $cm ops-check --from plan.yml
Get-Content .\plan.yml | & $cm ops-check --from -
$yaml = "task: inline`nops: []`n"
& $cm ops-check --yaml $yaml
```

Zasady YAML-first:

1. Większy patch zapisuj jako `plan.yml`, nie jako prose z blokami kodu.
2. Dla większego kodu używaj `content_from`, nie inline `content`.
3. Każdy op powinien mieć `id`, `kind`, stabilny locator i możliwie `expected_hash`.
4. Dla `replace_range` dodawaj `context_before` i `context_after`.
5. Dla registry/CSS/sekcji preferuj `replace_between_anchors`.
6. Przed `ops-apply --write` uruchom `ops-check --strict`.
7. Przy planach wielooperacyjnych używaj `--backup --stop-on-error --strict`.
8. Po aplikacji użyj `ops-verify` i `ops-summary`, a potem realnych build/test.

### @codemap anchors

Dodawaj anchory do miejsc, które agent powinien stabilnie odnajdywać:

```ts
// @codemap anchor:workspace-dock domain:workspace role:registry
```

```rust
// @codemap anchor:editor-mode-pointer domain:editor-mode role:command
```

```yaml
# @codemap anchor:main-menu-scene domain:menu role:scene
```

Dobre miejsca:

```text
centralne registry
duże dispatchery
mapy komend
Tauri command registration
dock/component registries
scene transition points
ważne YAML scenes
granice generated/hand-maintained
pliki wysokiego ryzyka
```

Nie dodawaj anchorów do każdej funkcji ani oczywistych lokalnych helperów. Anchor ma zmniejszać koszt nawigacji, nie zaśmiecać kod.

### Workset

Jeżeli task trwa więcej niż jedną turę, zapisz scope:

```powershell
& $cm impact <symbol-or-query> --limit 50
& $cm workset <task-name> --from-impact <symbol-or-query> --save
& $cm workset <task-name> --status
```

Workset jest preferowany zamiast ponownego odtwarzania tej samej listy plików przez `rg`, `impact` i `open-set`.

### Rozwój samego amigo-codemap

Przy pracy w `crates/tools/amigo-codemap` zaczynaj od:

```powershell
& $cm command-map <command-name>
& $cm explain-file crates/tools/amigo-codemap/src/cli.rs
& $cm neighbors crates/tools/amigo-codemap/src/main.rs
```

Po dodaniu lub zmianie komendy zaktualizuj:

```text
crates/tools/amigo-codemap/src/cli.rs
crates/tools/amigo-codemap/src/main.rs
crates/tools/amigo-codemap/src/report/mod.rs
crates/tools/amigo-codemap/src/report/command_map.rs
crates/tools/amigo-codemap/README.md
AMIGO_WORKFLOW.md, jeśli zmienia się workflow agenta
operations.md
```

Minimalny verify dla codemap:

```powershell
cargo fmt -p amigo-codemap --check
cargo test -p amigo-codemap --no-run
cargo test -p amigo-codemap
cargo build -p amigo-codemap

& $cm trace patch-apply --limit 20
& $cm open-set patch-apply --why --limit 10
& $cm signature scan_symbols
& $cm slice crates/tools/amigo-codemap/src/scan/symbols.rs --symbol scan_symbols
& $cm change-plan codemap --limit 20
```

## Amigo Codemap Daemon

`amigo-codemap` może działać z lokalnym daemonem:

```powershell
cargo build -p amigo-codemap --bins

$daemon = Start-Process `
  -FilePath ".\target\debug\amigo-codemapd.exe" `
  -ArgumentList @("run", "--root", (Get-Location).Path, "--level", "2") `
  -PassThru

.\target\debug\amigo-codemapd.exe status
.\target\debug\amigo-codemap.exe brief
.\target\debug\amigo-codemap.exe trace EditorTarget --limit 20
.\target\debug\amigo-codemapd.exe shutdown
```

