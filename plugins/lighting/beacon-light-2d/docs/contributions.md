# Beacon Light 2D Contributions

Beacon Light 2D is a semantic light source that can also render its beacon
overlay.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- Light contributions derived from beacon intensity, color, halo, and role flags.
- Relight, bloom, and camera FX roles are explicit render contribution roles.

## Source Data
- Camera optics coverage is a beacon source with active status.
- Animated beacons can be adapted into camera motion data where shutter motion is
  wired by the runtime bundle.
- The plugin does not consume another plugin's lighting output directly.
