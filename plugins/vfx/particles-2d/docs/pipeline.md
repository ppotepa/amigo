# Particles 2D Pipeline

Particles 2D owns authored particle emitters, simulation state, and render
extraction.

## Flow
- Scene components hydrate emitter shape, rates, lifetime, size, color ramp,
  velocity, forces, lighting, spawn area, simulation space, and render layer.
- `Particle2dSceneService` stores emitters, bursts, jobs, and runtime state.
- Runtime systems spawn and update particles.
- The render extractor emits particle draw commands and light contributions.

## Targets
- Writes `SceneColor` and `SceneAlpha`.
- Contributes `SceneVelocity`, `SceneHighlight`, and `SceneEmissive`.
- Particle lighting can provide lighting emit, bloom source, and camera FX source
  roles when enabled.
