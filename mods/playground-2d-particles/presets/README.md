# Particle Presets

Preset files are the future source of truth for the particle editor.

Current scenes still hydrate concrete `ParticleEmitter2D` entities directly from `scenes/showcase/scene.yml`; these files document reusable authoring data and should become loadable assets in a later task.

## Format

```yaml
kind: particle-preset-2d
id: plasma
label: Plasma
category: energy
tags: [continuous, energy]
emitter:
  spawn_rate: 150.0
  max_particles: 140
```
