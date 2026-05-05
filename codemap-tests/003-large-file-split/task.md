# 003 Large File Split

Cel:
- sprawdzić planowanie podziału większego pliku,
- ocenić `large-files`, `move-plan`, `tauri-commands`, `dup`, `patch-preview`.

Proponowany kandydat:
- większy plik z `large-files --with-split-hints`

Kroki:

```powershell
cargo build -p amigo-codemap
target\debug\amigo-codemap.exe large-files --top 20 --with-split-hints
target\debug\amigo-codemap.exe move-plan <FILE> --by tauri-command
target\debug\amigo-codemap.exe tauri-commands --limit 100
target\debug\amigo-codemap.exe dup <HELPER> --limit 80
git diff --stat
rg -n "fn |export |command" <FILE>
target\debug\amigo-codemap.exe verify-plan --changed
```

Po zmianach:

```powershell
git diff > patch.diff
target\debug\amigo-codemap.exe patch-preview --from patch.diff --limit 80
target\debug\amigo-codemap.exe commit-summary --changed --limit 80
```

Co mierzyć:
- czy `large-files` dał sensowny kandydat,
- czy `move-plan` dał sensowną propozycję podziału,
- czy `tauri-commands` / `dup` złapały regresje po splicie.
