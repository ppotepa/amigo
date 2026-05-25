# Light Groups 2D Contributions

Light Groups 2D is a semantic lighting source. It groups lights and optional
lightmap channels into explicit downstream roles.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- Semantic contributions for `SceneHighlight` and `SceneEmissive`.

## Coverage
- A group with both `lightmap_source` and `lightmap_channel` maps to
  `CameraOpticalCoverage2d::LightMapChannel`.
- A group without a complete lightmap channel reports unsupported coverage with
  reason `light_group_missing_lightmap_channel`.
- Active candidates target `SceneLighting`.

The group declaration is the source of optical intent; lightmap existence alone
is not enough to route camera effects.
