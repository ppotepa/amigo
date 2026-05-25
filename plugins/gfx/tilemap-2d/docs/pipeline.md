# Tilemap 2D Pipeline

Tilemap 2D owns authored tilemap sources, ruleset resolution, and render
extraction.

## Flow
- Scene components hydrate tilemap size, tile size, layers, tileset references,
  ruleset data, render layer, and transform data.
- `TileMap2dSceneService` stores tilemap draw commands for the active scene.
- `TileMap2dRenderExtractor` emits visible tilemap commands into the render
  output.

## Targets
- Writes `SceneColor` and `SceneAlpha`.
- Contributes `SceneDepth` and `SceneHighlight` through declared roles.
- Ruleset and validation logic stay in the plugin, not in the renderer.
