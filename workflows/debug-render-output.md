# Debug render output

## Diagnostic chain

```text
scene authoring
  -> component hydration
  -> domain contribution
  -> extraction bridge
  -> render-api frame packet
  -> render target allocation
  -> visual source buffer / candidate buffer
  -> post-fx execution
  -> final composite/debug view
```

## Checklist

```text
[ ] Is the authored data present?
[ ] Did hydration keep it?
[ ] Did the plugin emit a contribution/candidate?
[ ] Did diagnostics count it?
[ ] Did the extraction bridge pass it into render-api?
[ ] Did render-wgpu allocate the expected target?
[ ] Did the effect read the target?
[ ] Is the final debug/present path showing the right output?
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
