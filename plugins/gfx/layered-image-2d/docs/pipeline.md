# Layered Image 2D Pipeline

Layered Image 2D owns authored layered image sources and their render extraction.

## Flow
- Scene components hydrate into `LayeredImage2dSceneCommand` values with layer
  overrides, blend modes, viewport fit, visual maps, and transforms.
- `LayeredImageSceneService` stores commands for the active scene.
- The render extractor emits visible layered image draw commands into the 2D
  render extraction output.

## Targets
- Writes `SceneColor` and `SceneAlpha`.
- Contributes `SceneHighlight`, `SceneEmissive`, and `SceneDepth` when declared
  roles require them.
- Backend execution is driven by extracted commands, not by renderer-side scans
  of authored data.
