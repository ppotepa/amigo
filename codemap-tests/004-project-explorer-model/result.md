# 004 Project Explorer Model Result

## Status

Method A completed. Implementation was validated and then reverted. Report kept.

## Candidate files

- `crates/apps/amigo-editor/src/features/project/ProjectExplorerPanel.tsx`
- `crates/apps/amigo-editor/src/features/project/projectTreeModel.ts`
- `crates/apps/amigo-editor/src/features/project/projectTreeModel.test.ts`

## Method A: codemap-first

### Commands

1. `amigo-codemap open-set ProjectExplorerPanel --task implement --limit 12`
2. `amigo-codemap slice crates/apps/amigo-editor/src/features/project/ProjectExplorerPanel.tsx --symbol ProjectExplorerPanel --radius 80`
3. `amigo-codemap impact ProjectExplorerPanel --group feature --limit 40`
4. `amigo-codemap verify-plan --changed`
5. manual open only after narrowing scope:
   - `ProjectExplorerPanel.tsx`
   - `projectTreeModel.ts`
6. implementation:
   - extracted project tree types and pure helpers into `projectTreeModel.ts`
   - updated `ProjectExplorerPanel.tsx` imports/usages
   - added `projectTreeModel.test.ts`
7. validation:
   - `npm test -- projectTreeModel`
   - `npm test`
   - `npm run build`

### Metrics

- codemap output lines: `195`
- manual file-open output lines: `738`
- total output lines inspected: `933`
- files manually opened: `2`
- implementation diff:
  - `ProjectExplorerPanel.tsx`: extraction only
  - `projectTreeModel.ts`: `100` lines of real model code
  - `projectTreeModel.test.ts`: new focused test file
- estimated tokens used: `~2800`
- estimated tokens saved vs standard-first baseline: `~5000-7000`

### Validation

- `npm test -- projectTreeModel` ✅
- `npm test` ✅
- `npm run build` ✅

### Notes

This was a clean codemap-guided implementation pass:

- discovery used only codemap reports
- only 2 source files were opened manually before editing
- no repo-wide `rg` sweep was needed
- extraction target was obvious from `open-set` + `slice`

The implemented extraction moved these pure helpers into `projectTreeModel.ts`:

- project tree node types
- `mergeProjectTrees`
- `normalizeProjectTreeNode`
- `projectNodeMatchesSearch`
- `projectNodeKindLabel`
- `relativeProjectPath`
- `statusForEditorStatus`
- `assetDisplayLabel`

The actual code change was then reverted after the experiment.

## Method B: standard-first

### Commands

1. `rg -n "ProjectExplorer|mergeProjectTrees|normalizeProjectTreeNode|projectNodeMatchesSearch|statusForEditorStatus|assetDisplayLabel|relativeProjectPath" crates/apps/amigo-editor/src/features/project`
2. manual open:
   - `ProjectExplorerPanel.tsx`
   - `EngineProjectTree.tsx`
   - `ProjectNodeActionStrip.tsx`
   - `builtinComponents.tsx`
   - `projectTreeModel.ts`
3. implementation:
   - same extraction as Method A
   - same test file
4. validation:
   - `npm test -- projectTreeModel`
   - `npm test`
   - `npm run build`

### Metrics

- grep output lines: `46`
- manual file-open output lines: `1506`
- total output lines inspected: `1552`
- files manually opened: `5`
- implementation diff:
  - effectively the same as Method A
- estimated tokens used: `~7600`
- estimated tokens saved: baseline only, none for this method

### Notes

This flow reached the same implementation result, but with much more manual reading.

The extra cost came from:

- opening helper re-export shims that added no useful model context
- opening `builtinComponents.tsx`, which confirmed usage but did not materially help the extraction
- reading the full `ProjectExplorerPanel.tsx` before narrowing the exact pure helper set

The actual code change was then reverted after the experiment.

## Comparison

For this task, `codemap-first` was materially smaller and cleaner:

- manual files opened:
  - codemap-first: `2`
  - standard-first: `5`
- total inspected output lines:
  - codemap-first: `933`
  - standard-first: `1552`
- estimated token usage:
  - codemap-first: `~2800`
  - standard-first: `~7600`
- estimated savings:
  - `~4800`

Main reason:

`open-set` and `slice` narrowed the change to the right implementation surface before any manual read. Standard flow found the same answer, but only after opening several files that turned out to be low-value for the extraction.
