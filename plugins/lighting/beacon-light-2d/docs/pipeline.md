# Beacon Light 2D Pipeline

Beacon Light 2D owns authored beacon light sources and their render extraction.

## Flow
- Scene components hydrate beacon color, intensity envelope, halo/core radii,
  beam data, render depth, contribution roles, and transform data.
- `BeaconLight2dSceneService` stores beacon commands and runtime state.
- The beacon system updates animated intensity and the render extractor emits
  visible beacon primitives plus light contributions.

## Targets
- Writes `SceneLighting` and `SceneColor`.
- Contributes `SceneHighlight` and `SceneEmissive`.
- Default roles enable overlay visibility, relight plate, bloom source, and
  camera FX source.
