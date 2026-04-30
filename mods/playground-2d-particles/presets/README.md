# Particle Presets

Preset files are the source of truth for showcase particle variants.

At app bootstrap the runtime scans loaded mods for `presets/*.yml`, registers every `kind: particle-preset-2d` document, and exposes the catalog through `world.particles.preset_ids()` / `world.particles.apply_preset(...)`. The showcase scene keeps one `preview-emitter` in `scenes/showcase/scene.yml`, fills the preset dropdown with `world.ui.set_options(...)`, and applies selected preset data from this catalog at runtime.

The catalog currently mirrors the showcase presets:

- `fire`
- `smoke`
- `sparks`
- `magic`
- `snow`
- `dust`
- `thruster`
- `plasma`
- `portal`
- `rain`
- `explosion`

`editor-export.example.yml` shows the shape emitted by the editor `Export` action and accepted by the preset loader.

## Format

```yaml
kind: particle-preset-2d
id: plasma
label: Plasma
category: energy
tags: [continuous, energy]
emitter:
  type: ParticleEmitter2D
  spawn_rate: 150.0
  max_particles: 140
```

The editor `Export` tab prints this same wrapper shape:

```yaml
kind: particle-preset-2d
id: editor-export
label: Editor Export
category: custom
tags: [editor, custom]
emitter:
  type: ParticleEmitter2D
  active: true
  spawn_rate: 90.0
  max_particles: 160
  particle_lifetime: 0.65
  initial_speed: 100.0
  spread_degrees: 28.0
  color_ramp:
    interpolation: linear_rgb
    stops:
      - { t: 0.0, color: "#FFFFFFFF" }
      - { t: 0.25, color: "#39D7FFFF" }
      - { t: 0.70, color: "#236DFFFF" }
      - { t: 1.0, color: "#0A1A6600" }
```
