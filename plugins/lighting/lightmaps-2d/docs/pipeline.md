# Lightmaps 2D Pipeline

Lightmaps 2D defines backend-neutral lightmap source contracts.

## Flow
- Scene data declares a `LightMap2DSource` component.
- Runtime builds `Lightmap2dSource` records with named channels and layers.
- The source resolves to the shared `LightMap` render target contract.
- Light groups, relight, and camera optics adapters consume the declared source
  and channel ids.

## Targets
- Writes `LightMap` as authored lighting data.
- Contributes `SceneLighting`, `SceneHighlight`, and `SceneEmissive` roles.
- Does not own relight execution.
