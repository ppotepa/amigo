# Vector 2D Pipeline

Vector 2D owns authored vector shape sources and their render extraction.

## Flow
- Scene components hydrate shape kind, points, style, material, contribution
  roles, z index, render layer, and transform data.
- `VectorSceneService` stores vector draw commands for the active scene.
- `Vector2dRenderExtractor` emits visible vector commands into the render output.

## Targets
- Writes `SceneColor` and `SceneAlpha`.
- Contributes `SceneDepth` and `SceneHighlight` when roles declare those uses.
- Shape types include polyline, polygon, and circle.
