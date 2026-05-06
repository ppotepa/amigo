# 002 Result

## Method A: codemap-first

- Used: ~1007 tokens
- Output lines: 125
- Manual files opened: 0
- Reports:
  - `orphan-files crates/apps/amigo-editor/src/features --limit 20`
  - `delete-plan crates/apps/amigo-editor/src/features/assets/AssetRegistryTree.tsx`
  - `stale --patterns AssetRegistryTree --limit 80`
  - `import-fix-plan --changed --limit 20`
- Manual fallback: none during the measurement pass
- Verify: `import-fix-plan --changed`
- Notes:
  - szybko wykrył, że `AssetRegistryTree.tsx` wygląda jak shim/empty
  - ograniczył potrzebę ręcznego szukania referencji

## Method B: standard-first

- Used: ~7331 tokens
- Output lines: 707
- Manual files opened: 2
- Standard commands:
  - `git diff --name-status`
  - `rg -l "AssetRegistryTree" crates/apps/amigo-editor/src`
  - `rg -n "AssetRegistryTree" crates/apps/amigo-editor/src -C 2`
  - `Get-Content AssetRegistryTree.tsx`
  - `Get-Content AssetBrowserPanel.tsx`
- Verify: manual review only in the measurement pass
- Notes:
  - największy koszt dało otwarcie dużego `AssetBrowserPanel.tsx`
  - standardowy przebieg szybko rozlewa się na sąsiednie pliki

## Comparison

- Saved: ~6324 tokens
- Fewer output lines with codemap: 125 vs 707
- Fewer manual file opens with codemap: 0 vs 2
- Best command: `orphan-files`
