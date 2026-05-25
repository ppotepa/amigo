# Particles 2D Diagnostics

Channels:
- `particles-2d.render`
- `particles-2d.contributions`

## What To Trace
- Hydrated emitter id, render layer, spawn rate, max particles, lifetime, shape,
  simulation space, and light mode.
- Runtime particle count, burst activity, force application, and source space.
- Contribution roles for scene velocity, highlight, emissive, bloom, and camera
  FX routing.

Diagnostics should identify the emitter and source role that produced a visual
or camera contribution, not a backend side effect.
