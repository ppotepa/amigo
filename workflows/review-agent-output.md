# Review agent output

## Review checklist

```text
[ ] Did the agent obey scope?
[ ] Did it avoid legacy/v2/fallback paths?
[ ] Did it avoid broad repo scans?
[ ] Are changed files expected?
[ ] Is validation targeted and relevant?
[ ] Did it report real failures instead of "should work"?
[ ] Are docs/tests updated only where needed?
```

## Red flags

```text
new compatibility shim
new renderer guess
new app-side domain wiring
mass formatting
workspace-wide validation used as a substitute for targeted reasoning
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
