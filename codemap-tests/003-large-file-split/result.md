# 003 Result

## Method A: codemap-first

- Used: ~1542 tokens
- Output lines: 158
- Manual files opened: 0
- Reports:
  - `large-files --top 10 --with-split-hints`
  - `move-plan crates/apps/amigo-editor/src/features/project/ProjectExplorerPanel.tsx --by symbol --limit 100`
  - `slice crates/apps/amigo-editor/src/features/project/ProjectExplorerPanel.tsx --symbol ProjectExplorerPanel --radius 30`
  - `verify-plan --changed`
- Manual fallback: none during the measurement pass
- Verify: `verify-plan --changed`
- Notes:
  - `large-files` i `move-plan` dobrze zawęziły kandydat i plan czytania
  - `slice` pozwolił obejrzeć tylko fragment zamiast całego pliku

## Method B: standard-first

- Used: ~9647 tokens
- Output lines: 925
- Manual files opened: 1
- Standard commands:
  - `git diff --stat`
  - `Get-Content ProjectExplorerPanel.tsx`
  - `rg -n "^(export function|export const|function |const )" ProjectExplorerPanel.tsx`
  - `rg -n "^import " ProjectExplorerPanel.tsx`
- Verify: manual review only in the measurement pass
- Notes:
  - koszt zdominowany przez pełne otwarcie pliku 762 linii
  - standardowe grepowanie nie daje tak szybko planu splitu jak `move-plan`

## Comparison

- Saved: ~8105 tokens
- Fewer output lines with codemap: 158 vs 925
- Fewer manual file opens with codemap: 0 vs 1
- Best command: `large-files` + `move-plan`
