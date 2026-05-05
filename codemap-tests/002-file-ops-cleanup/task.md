# 002 File Ops Cleanup

Cel:
- sprawdzić workflow usuwania albo przenoszenia pliku,
- ocenić `orphan-files`, `delete-plan`, `file-move-plan`, `import-fix-plan`, `stale`.

Proponowany kandydat:
- jeden z shimów / kandydatów z `features/*`

Kroki:

```powershell
cargo build -p amigo-codemap
target\debug\amigo-codemap.exe orphan-files crates/apps/amigo-editor/src/features --limit 50
target\debug\amigo-codemap.exe delete-plan <FILE>
target\debug\amigo-codemap.exe file-move-plan <FROM> --to <TO>
target\debug\amigo-codemap.exe import-fix-plan --changed
target\debug\amigo-codemap.exe stale --patterns <OLD_NAME> --limit 80
git diff --name-status
rg -l "<OLD_NAME>" crates/apps/amigo-editor/src
target\debug\amigo-codemap.exe verify-plan --changed
```

Po zmianach:

```powershell
npm run build 2>&1 | target\debug\amigo-codemap.exe fallout --limit 80
target\debug\amigo-codemap.exe commit-files --changed
```

Co mierzyć:
- czy `orphan-files` trafnie rozróżnił shim od używanego pliku,
- czy `delete-plan` / `file-move-plan` ograniczyły ręczne szukanie importów,
- czy `import-fix-plan` wykrył fallout przed buildem.
