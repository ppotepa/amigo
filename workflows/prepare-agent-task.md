# Prepare an agent task

## Task format

```text
Goal:
  one sentence

Scope:
  allowed files/directories

Operations:
  READ / ADD / MODIFY / DELETE / MOVE

Do not touch:
  explicit exclusions

Validation:
  exact commands

Expected report:
  changed files, validation, risks, next step
```


## Common requirements

```text
Start with git status.
Use amigo-codemap first.
Read only relevant symbols/ranges.
Make minimal ADD/MODIFY/DELETE/MOVE changes.
Validate with targeted commands.
Report risks and next action.
```

## Hard prohibitions

```text
No legacy/v2 parallel paths.
No silent fallbacks.
No renderer-side domain guessing.
No large formatting-only diffs.
No workspace-wide check/test by default.
```
