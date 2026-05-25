# Composite Contributions

Composite does not emit render-source contributions.

## Contract
- Capability: `postfx.composite@1`.
- Required slot: `render.backend`.
- Standard target plan reads `SceneColor` and `CameraArtifactLayer`, then writes
  `FinalComposite`.

## Effect Data
- Post-fx effects are declared in scoped stacks, not discovered from renderer
  resources.
- Camera-owned effects and scene-owned effects remain separate through host id,
  role, scope, and pipeline fields.
- Missing executors should surface as diagnostics or validation errors, not
  silent copy-through behavior.
