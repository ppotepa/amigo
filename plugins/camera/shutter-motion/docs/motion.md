# Shutter Motion Runtime

The runtime side of this plugin owns reusable 2D motion services.

## Responsibilities
- Queue and update platformer-style motion controllers.
- Queue and update freeflight controllers.
- Step velocity and bounds behavior.
- Launch projectiles from authored emitters.
- Publish scene events for queued motion and lifecycle outcomes.

## Boundaries
- Collision resolution belongs to physics.
- Input mapping belongs to input actions.
- Motion blur target planning belongs to the shutter contribution pipeline.
