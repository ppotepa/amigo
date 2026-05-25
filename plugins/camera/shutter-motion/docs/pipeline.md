# Shutter Motion Pipeline

Shutter Motion has two responsibilities: 2D motion runtime services and the
camera shutter motion contract.

## Motion Runtime
- Scene commands queue motion controllers, freeflight controllers, velocity,
  bounds, lifetimes, entity pools, and projectile emitters.
- Runtime systems update motion state and publish scene events.
- The motion service is reset per scene to avoid stale controller state.

## Shutter Contract
- Source plugins provide motion coverage through explicit candidates.
- The shutter plan can read `SceneVelocity`, produce `TemporalExposure`, and
  participate in `FinalComposite`.
- The plugin requires `camera.frame_context.2d@1` and does not infer motion blur
  from renderable existence alone.
