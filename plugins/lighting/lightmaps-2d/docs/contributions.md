# Lightmaps 2D Contributions

Lightmaps 2D is the semantic owner of authored lightmap sources and channels.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- Lightmap contributions for `SceneLighting`, `SceneHighlight`, and
  `SceneEmissive`.

## Source Data
- `Lightmap2dSource` carries the lightmap id.
- `Lightmap2dChannel` carries the channel id and layer list.
- `lightmap_target_id()` resolves the shared `LightMap` target.

Downstream consumers must bind to declared source and channel ids instead of
treating any available lightmap texture as optical intent.
