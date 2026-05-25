# Material 2D Pipeline

Material 2D owns authored 2D material semantics.

## Flow
- Scene data declares a `Material2D` component.
- Runtime builds a `Material2dSource` with id, base color, and opacity.
- The source becomes an active `Material2dCandidate` for scene color routing.
- Participation adapters derive camera optics and focus-depth contributions.
- Target writer plugins consume those contributions when filling material
  buffers.

## Targets
- Contributes `SceneHighlight`, `SceneEmissive`, and `SceneDepth`.
- The candidate model can target `SceneColor`.
- Does not render material maps directly.
