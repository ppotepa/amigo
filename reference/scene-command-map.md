# Scene command map

Use this as a navigation file, not as the final source of truth.

## Command waterfall

```text
authored document / script command
  -> scene command enum/model
  -> parser/hydration path
  -> domain service handling
  -> state mutation
  -> diagnostics/test
```

## Before adding a command

Read:

```text
workflows/add-new-scene-command.md
crates/engine-scene.md
relevant plugin doc
```

Search:

```powershell
rg -n "SceneCommand|Command|hydrate|apply_.*command" crates/engine/scene crates/runtime plugins
```
