# Add a new plugin

## Operation plan

```text
ADD plugins/<family>/<name>/Cargo.toml
ADD plugins/<family>/<name>/plugin.toml
ADD plugins/<family>/<name>/src/lib.rs
ADD plugins/<family>/<name>/README.md
ADD plugins/<family>/<name>/docs/pipeline.md
ADD plugins/<family>/<name>/docs/contributions.md
ADD plugins/<family>/<name>/docs/diagnostics.md
ADD plugins/<family>/<name>/tests/waterfall_tests.rs
MODIFY workspace registration only where required
```

## Required manifest sections

```text
id
family
kind
renderable
render_participation
capabilities
slots
targets
contributions
diagnostics
docs
tests
```

## Rule

A plugin declares contributions and capabilities. It should not directly patch renderer behavior unless it is a backend/plugin specifically owning that backend path.


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
