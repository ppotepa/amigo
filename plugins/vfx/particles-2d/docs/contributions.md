# Particles 2D Contributions

Particles 2D is a renderable source with camera participation.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- `MotionShutterContribution2d` with `DisabledByDefault` policy.

## Coverage
- Camera optics coverage is `ParticleCoverage` keyed by emitter entity name.
- Shutter motion coverage uses the emitter render layer.
- Particle light conversion emits lighting roles from the particle light mode and
  response.

The plugin does not consume post-fx or lighting results directly; it declares
source data for downstream consumers.
