# Trails 2D Contributions

Trails 2D is a renderable VFX source with explicit camera participation.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- `MotionShutterContribution2d` with `DisabledByDefault` policy.
- Semantic roles for `SceneHighlight`, `SceneEmissive`, and `SceneVelocity`.

## Shutter Adapter
- `trail_to_shutter_motion` covers the trail render layer.
- A trail is declared for shutter motion only when `length_px` is greater than
  zero.
- Motion blur is `length_px / 100.0` clamped to the supported response range.

Trail rendering does not imply optics or shutter participation unless the trail
declares those contributions.
