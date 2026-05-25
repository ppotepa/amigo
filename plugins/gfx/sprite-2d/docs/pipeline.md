# Sprite 2D Pipeline

Sprite 2D owns authored sprite sources and their 2D render extraction.

## Flow
- Scene components hydrate sprite texture, sprite sheet, animation, material,
  visual maps, render contributions, z index, and transform data.
- `SpriteSceneService` stores sprite draw commands for the active scene.
- `Sprite2dRenderExtractor` emits visible sprite commands into the render output.

## Targets
- Writes `SceneColor` and `SceneAlpha`.
- Contributes `SceneDepth`, `SceneHighlight`, and `SceneVelocity` through
  declared render roles.
- Sprite coverage is texture alpha; unsupported coverage is retained as candidate
  state instead of becoming a renderer fallback.
