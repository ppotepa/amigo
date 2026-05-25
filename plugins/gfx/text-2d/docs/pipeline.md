# Text 2D Pipeline

Text 2D owns authored text sources and their render extraction.

## Flow
- Scene components hydrate content, font, bounds, style, shadow, outline, glow,
  material, render contributions, z index, and transform data.
- `Text2dSceneService` stores text draw commands for the active scene.
- `Text2dRenderExtractor` emits visible text commands into the render output.

## Targets
- Writes `SceneColor` and `SceneAlpha`.
- Contributes `SceneHighlight` and `SceneDepth` when roles declare those uses.
- Glyph coverage is the camera optics source for text.
