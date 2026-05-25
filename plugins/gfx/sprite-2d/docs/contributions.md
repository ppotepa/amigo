# Sprite 2D Contributions

Sprite 2D is a renderable source and a participant in camera-side contracts.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- `FocusDepthContribution2d` with `DerivedAtHydration` policy.
- `MotionShutterContribution2d` with `DisabledByDefault` policy.

## Defaults
- `world.color` defaults to enabled.
- `material.mask`, `optics.refract`, `transmission.source`, `bloom.source`, and
  `camera.fx_source` default to disabled.
- Camera optics and focus depth adapters use sprite texture-alpha coverage and
  render layer data from the hydrated command.
