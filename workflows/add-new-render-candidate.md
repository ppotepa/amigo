# Add a new render candidate

## Waterfall

```text
domain authored data
  -> explicit contribution/response
  -> render-api candidate/coverage contract
  -> extraction bridge
  -> backend adapter
  -> diagnostics/tests
```

## Operation plan

```text
READ crates/engine/render-api candidate contracts
READ relevant plugin/domain contributor
READ runtime/bundles extraction bridge
READ render-wgpu adapter location
MODIFY render-api contract only if needed
MODIFY extraction bridge to emit candidate
MODIFY backend adapter to consume declared candidate
ADD diagnostics for candidate count/skip reason
ADD tests where available
```

## Forbidden

Do not make renderer infer candidates from object names or mere existence of assets.


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
