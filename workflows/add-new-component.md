# Add a new scene component

## Preferred plan

```text
READ crates/engine/scene component metadata/provider contracts
READ relevant plugin doc
ADD or MODIFY plugin-owned component descriptor provider if available
MODIFY scene metadata only if no provider path exists yet
ADD hydration/validation tests
ADD diagnostics for missing/invalid authored fields
```

## Avoid

Do not keep expanding a central `component_metadata.rs` table if provider registration is available.


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
