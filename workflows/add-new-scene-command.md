# Add a new scene command

## Waterfall

```text
command model
  -> parser/hydration
  -> domain service handler
  -> diagnostics
  -> tests
```

## Operation plan

```text
READ crates/engine/scene command model and hydration code
READ target plugin/domain service
MODIFY command enum/model
MODIFY parser/hydration path
MODIFY domain handler
ADD tests for command parse/apply behavior
ADD diagnostics if command can be skipped
```

## Forbidden

Do not handle a scene command in `apps/app`.
Do not add WGPU fields to scene command data.


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
