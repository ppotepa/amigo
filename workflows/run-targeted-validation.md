# Run targeted validation

## Choose validation by change type

```text
docs only: git diff --check
single crate: cargo check -p <crate>
crate tests: cargo test -p <crate> <filter>
render-api: cargo check -p amigo-render-api; cargo check -p amigo-render-wgpu
runtime bundles: cargo check -p amigo-runtime-bundles; cargo check -p amigo-app
plugin: cargo check -p <plugin-package>; cargo test -p <plugin-package>
```

## End state

Always finish with:

```powershell
git status --short
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
