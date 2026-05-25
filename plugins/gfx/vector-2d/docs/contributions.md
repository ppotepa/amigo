# Vector 2D Contributions

Vector 2D is a renderable source with camera participation.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- `FocusDepthContribution2d` with `DerivedAtHydration` policy.

## Defaults
- `world.color` defaults to enabled.
- `material.mask`, `optics.refract`, `transmission.source`, `bloom.source`, and
  `camera.fx_source` default to disabled.
- Camera optics coverage is `VectorCoverage` with entity and render layer data.
