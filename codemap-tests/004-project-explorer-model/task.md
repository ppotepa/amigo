# 004 Project Explorer Model

## Goal

Implement a real frontend refactor in `amigo-editor` by extracting tree/model logic from `ProjectExplorerPanel.tsx` into `projectTreeModel.ts`, then add focused unit tests.

This task is meant to compare:

1. `codemap-first`
2. `standard-first`

against the same implementation target.

## Why this task

This is a good comparison candidate because:

- `ProjectExplorerPanel.tsx` is large enough to create navigation overhead.
- `projectTreeModel.ts` already exists as an empty placeholder.
- the change is real code, not docs-only or a trivial rename.
- it should produce measurable test/build fallout if done badly.
- it exercises a different codemap surface than previous tasks:
  - `open-set`
  - `slice`
  - `impact`
  - `verify-plan`
  - optionally `fallout`

## Implementation target

Extract and test a small, coherent chunk of project explorer model logic, for example:

- search filtering for project tree nodes
- normalization helpers for tree nodes
- merge behavior between structure tree and fallback tree
- small derived summaries or child grouping behavior

The implementation should:

- move code into `src/features/project/projectTreeModel.ts`
- reduce logic inside `ProjectExplorerPanel.tsx`
- add unit tests in `projectTreeModel.test.ts`
- keep behavior stable or only introduce one small intentional UI improvement

## Small allowed behavior change

One narrow improvement is allowed if it stays local, for example:

- stable child sorting
- better empty-state summary
- normalized search matching
- keeping ghost/expected nodes visible during filtered views

Do not widen scope into:

- registry changes
- backend/Tauri changes
- store migrations
- CSS refactors outside what is strictly needed

## Method A: codemap-first

Suggested sequence:

1. `amigo-codemap open-set ProjectExplorerPanel --task implement --limit 12`
2. `amigo-codemap slice crates/apps/amigo-editor/src/features/project/ProjectExplorerPanel.tsx --symbol ProjectExplorer --radius 80`
3. `amigo-codemap impact ProjectExplorerPanel --group feature --limit 40`
4. optionally:
   - `amigo-codemap large-files --top 20 --with-split-hints`
   - `amigo-codemap move-plan crates/apps/amigo-editor/src/features/project/ProjectExplorerPanel.tsx --by symbol`
5. implement the extraction
6. `amigo-codemap verify-plan --changed`
7. run tests/build
8. if needed:
   - `npm run build 2>&1 | amigo-codemap fallout --limit 80`

## Method B: standard-first

Suggested sequence:

1. `rg -n "ProjectExplorer|mergeProjectTrees|normalizeProjectTreeNode|buildEngineProjectTree" crates/apps/amigo-editor/src/features/project`
2. manually open:
   - `ProjectExplorerPanel.tsx`
   - `EngineProjectTree.tsx`
   - `ProjectNodeActionStrip.tsx`
   - maybe `builtinComponents.tsx`
3. decide extraction boundary by reading code directly
4. implement extraction
5. run tests/build
6. inspect raw build output manually

## Success criteria

- `projectTreeModel.ts` contains real model logic
- `ProjectExplorerPanel.tsx` is smaller and simpler
- a new test file covers the extracted logic
- `npm test` passes
- `npm run build` passes

## Metrics to collect

For both methods record:

- commands used
- output lines inspected
- number of files manually opened
- estimated tokens used
- estimated tokens saved
- whether build fallout needed extra investigation

## Notes

This task is intentionally medium-sized:

- bigger than the asset browser model extraction
- smaller than a full panel split
- good for practical measurement of navigation cost
