# 001 Symbol Migration

Cel:
- sprawdzić zmianę typu/symbolu przez kilka warstw aplikacji,
- ocenić `impact`, `open-set`, `slice`, `verify-plan`, `fallout`.

Proponowany kandydat:
- `EditorSelectionRef`

Kroki:

```powershell
cargo build -p amigo-codemap
target\debug\amigo-codemap.exe changed --group package --limit 20
target\debug\amigo-codemap.exe impact EditorSelectionRef --group feature --limit 80
target\debug\amigo-codemap.exe open-set EditorSelectionRef --task migrate --limit 12
target\debug\amigo-codemap.exe slice crates/apps/amigo-editor/src/app/selectionTypes.ts --symbol EditorSelectionRef --radius 25
git diff --stat
rg -n "EditorSelectionRef" crates/apps/amigo-editor/src
target\debug\amigo-codemap.exe verify-plan --changed
```

Po zmianach:

```powershell
npm run build 2>&1 | target\debug\amigo-codemap.exe fallout --limit 80
target\debug\amigo-codemap.exe commit-summary --changed --limit 80
```

Co mierzyć:
- czy `impact` wskazał poprawne feature groups,
- czy `open-set` zmniejszył liczbę otwieranych plików,
- czy `slice` wystarczył zamiast czytania całego pliku,
- ile ręcznych `rg` było jeszcze potrzebne.
