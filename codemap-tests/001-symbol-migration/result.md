# 001 Result

## Method A: codemap-first

- Used: ~940 tokens
- Output lines: 117
- Manual files opened: 0
- Reports:
  - `impact EditorSelectionRef --group feature --limit 80`
  - `open-set EditorSelectionRef --task migrate --limit 12`
  - `slice crates/apps/amigo-editor/src/app/selectionTypes.ts --symbol EditorSelectionRef --radius 25`
  - `verify-plan --changed`
- Manual fallback: none during the measurement pass
- Verify: `verify-plan --changed`
- Notes:
  - od razu wskazał warstwy `selection`, `store`, `main-window`
  - nie wymagał otwierania wielu plików ręcznie

## Method B: standard-first

- Used: ~3698 tokens
- Output lines: 295
- Manual files opened: 4
- Standard commands:
  - `git diff --stat`
  - `rg -n "EditorSelectionRef" crates/apps/amigo-editor/src`
  - `Get-Content selectionTypes.ts`
  - `Get-Content selectionSelectors.ts`
  - `Get-Content editorActions.ts`
  - `Get-Content editorState.ts`
- Verify: manual review only in the measurement pass
- Notes:
  - duzo szerszy i bardziej hałaśliwy output
  - szybciej wchodzi w pełne czytanie plików

## Comparison

- Saved: ~2758 tokens
- Fewer output lines with codemap: 117 vs 295
- Fewer manual file opens with codemap: 0 vs 4
- Best command: `impact`
