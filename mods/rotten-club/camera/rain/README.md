# Rotten Club Rain Glass Profiles

These files define camera-owned rain-on-lens presets.
They are mod assets, not hardcoded runtime tweaks.

## Where to put them

Use:
- `mods/rotten-club/camera/rain/*.yml`

Recommended presets in this mod:
- `rotten-club/camera/rain/realistic-lens-rain`
- `rotten-club/camera/rain/heavy-condensation`
- `rotten-club/camera/rain/thin-neon-drizzle`

## How to attach to Camera2D

Add this to a `Camera2D` component:

```yaml
lens_surface:
  rain_profile: rotten-club/camera/rain/realistic-lens-rain
```

Runtime ownership stays on `Camera2D`.
Authoring storage stays in the mod asset.

## Runtime control

Lens rain is owned by `Camera2D`, not by scene/global post-fx.

Use this from Rhai:

```rhai
world.camera.set_main_lens_rain("spawn_rate=6 opacity=0.6 mist_opacity=0.04");
world.camera.set_lens_rain("main", "preset=thin");
world.camera.clear_main_lens_rain_override();
```

Do not use this for camera lens rain:

```rhai
world.postfx.set_rain_glass(...)
```

That API is old/debug-only and targets global frame post-fx.

## YAML shape

```yaml
kind: camera-rain-glass-profile-2d
id: realistic_lens_rain
label: Realistic Lens Rain

spawn:
  enabled: true
  spawn_rate: 8.5
  spawn_limit: 720
  seed: 121713

droplets:
  min_radius_px: 32.0
  max_radius_px: 96.0
  gravity_px_per_sec2: 2200.0
  slip_rate: 0.30
  initial_spread: 0.46
  shrink_rate: 0.012
  evaporate: 10.5
  velocity_spread: 0.28
  motion_interval_min: 0.22
  motion_interval_max: 0.85
  x_shift_min: -10.0
  x_shift_max: 10.0
  collider_scale: 0.92

trails:
  enabled: true
  trail_drop_density: 0.34
  trail_drop_size_min: 5.0
  trail_drop_size_max: 16.0
  trail_distance_min_px: 10.0
  trail_distance_max_px: 54.0
  trail_spread: 0.28
  trail_shrink_rate: 0.015
  trail_evaporate: 7.5
  trail_taper: 0.70
  trail_refract_scale: 0.72
  trail_opacity: 0.42
  streak_boost: 0.26
  streak_length: 0.56

micro_droplets:
  enabled: true
  micro_droplets_per_second: 62.0
  micro_droplet_min_px: 2.4
  micro_droplet_max_px: 6.4

mist:
  enabled: true
  mist_opacity: 0.042
  mist_blur_px: 2.2
  mist_accumulation: 0.018
  mist_time: 0.65
  mist_color_strength: 0.08
  mist_blur_step: 1

optics:
  refract_base: 0.20
  refract_scale: 0.88
  distortion_px: 14.0
  normal_strength: 4.4
  chromatic_aberration: 0.08
  focus_blur_strength: 0.14
  background_blur_px: 2.0
  background_blur_steps: 2
  smooth_edge_min: 0.18
  smooth_edge_max: 0.92

compose:
  opacity: 0.88
  body_opacity: 0.42
  scene_blend: 0.84
  scene_darken: 0.03
  drop_plane_blur_px: 0.9
  reference_mode: true
  raindrop_compose: smoother
  raindrop_eraser_size: [0.78, 0.78]

lighting:
  receives_scene_light: true
  scene_light_response: 0.82
  scene_light_tint_strength: 0.22
  scene_shadow_floor: 0.05
  rim_strength: 0.34
  light_pos: [0.32, 0.22, 0.0, 0.0]
  diffuse_light: [0.18, 0.18, 0.18]
  shadow_offset: 0.0
  specular_light: [0.72, 0.72, 0.72]
  specular_shininess: 42.0
  light_bump: 0.24

contamination:
  blood_tint: [0.0, 0.0, 0.0]
  blood_amount: 0.0

debug:
  view: final
```

## Property guide

### `spawn`
- `enabled`: `true|false`. Master switch.
- `spawn_rate`: `0.0..20.0`. Higher = more new droplets per second.
- `spawn_limit`: `0..2000`. Hard cap for active droplets.
- `seed`: integer. Changes pattern repeat.

### `droplets`
- `min_radius_px`, `max_radius_px`: usually `18..120`.
  Lower than `2.0` for micro-sized droplets is a bad idea. Tiny values collapse into ugly pixel specks.
- `gravity_px_per_sec2`: usually `1200..3200`.
  Higher = faster downward travel.
- `slip_rate`: usually `0.1..0.6`.
  Higher = droplets break loose more easily.
- `initial_spread`: usually `0.2..0.7`.
  Higher = fatter wet bodies.
- `shrink_rate`: usually `0.005..0.03`.
  Higher = droplets thin out faster.
- `evaporate`: usually `4.0..16.0`.
  Higher = shorter life.
- `velocity_spread`: usually `0.1..0.5`.
  Higher = more irregular motion.
- `motion_interval_min`, `motion_interval_max`: usually `0.1..1.5`.
  Controls cadence of droplet updates.
- `x_shift_min`, `x_shift_max`: usually `-20..20`.
  Small lateral drift. Too large looks fake.
- `collider_scale`: usually `0.7..1.2`.
  Affects collision / merge feel.

### `trails`
- `enabled`: enables streak children behind main drops.
- `trail_drop_density`: usually `0.1..0.6`.
- `trail_drop_size_min`, `trail_drop_size_max`: usually `2..20`.
- `trail_distance_min_px`, `trail_distance_max_px`: usually `4..80`.
- `trail_spread`: usually `0.1..0.5`.
- `trail_shrink_rate`: usually `0.005..0.03`.
- `trail_evaporate`: usually `3.0..12.0`.
- `trail_taper`: usually `0.4..0.9`.
- `trail_refract_scale`: usually `0.2..1.0`.
- `trail_opacity`: usually `0.1..0.7`.
- `streak_boost`: usually `0.0..0.6`.
- `streak_length`: usually `0.0..1.0`.

### `micro_droplets`
- `enabled`: enables fine peppering.
- `micro_droplets_per_second`: usually `0..140`.
- `micro_droplet_min_px`, `micro_droplet_max_px`: usually `2.0..8.0`.
  Keep the minimum above roughly `2.0` for this mod if you want to avoid single-pixel junk.

### `mist`
- `enabled`: condensation veil.
- `mist_opacity`: usually `0.0..0.18`.
  Above `0.20` the image gets milky fast.
- `mist_blur_px`: usually `0.0..6.0`.
- `mist_accumulation`: usually `0.0..0.08`.
- `mist_time`: usually `0.0..1.5`.
- `mist_color_strength`: usually `0.0..0.2`.
- `mist_blur_step`: small integer, usually `1..3`.

### `optics`
- `refract_base`: usually `0.0..0.4`.
- `refract_scale`: usually `0.3..1.2` for realism.
  Above `1.5` gets stylized quickly.
- `distortion_px`: usually `4..24`.
- `normal_strength`: usually `1..6`.
- `chromatic_aberration`: usually `0.0..0.2`.
- `focus_blur_strength`: usually `0.0..0.4`.
- `background_blur_px`: usually `0.0..4.0`.
  Too high makes the whole frame mushy.
- `background_blur_steps`: usually `1..4`.
- `smooth_edge_min`, `smooth_edge_max`: keep inside `0.0..1.0` and keep min below max.

### `compose`
- `opacity`: overall effect visibility. Usually `0.6..1.0`.
- `body_opacity`: droplet body density. Usually `0.2..0.6`.
- `scene_blend`: usually `0.6..1.0`.
- `scene_darken`: usually `0.0..0.08`.
  If blacks go gray, this is not the first knob. Reduce mist first, then check this.
- `drop_plane_blur_px`: usually `0.0..2.0`.
- `reference_mode`: keep `true` for the more grounded look in this mod.
- `raindrop_compose`: `smoother` or `harder`.
- `raindrop_eraser_size`: usually around `0.6..1.0` on each axis.

### `lighting`
- `receives_scene_light`: should usually stay `true`.
- `scene_light_response`: usually `0.4..1.0`.
- `scene_light_tint_strength`: usually `0.0..0.5`.
- `scene_shadow_floor`: usually `0.0..0.12`.
  Higher values flatten blacks.
- `rim_strength`: usually `0.0..0.6`.
- `light_pos`: `[x, y, z, w]`. Keep near the top-half for menu rain.
- `diffuse_light`, `specular_light`: RGB triplets, usually `0.0..1.0`.
- `specular_shininess`: usually `8..64`.
- `light_bump`: usually `0.0..0.5`.

### `contamination`
- `blood_tint`: RGB triplet, normally `[0,0,0]` here.
- `blood_amount`: usually `0.0` unless intentionally stylized.

### `debug`
- `view`: `final`, `scene_input`, `blurred_scene`, `raindrop_map`, `droplet_map`, `trail_map`, `drop_normals`, `drop_mask`, `mist`, `refraction`.

## Quick tuning recipes

### Bigger droplets
- Increase `droplets.min_radius_px`
- Increase `droplets.max_radius_px`
- Lower `spawn.spawn_rate` slightly

### Stronger refraction
- Increase `optics.refract_scale`
- Increase `optics.distortion_px`
- Increase `optics.normal_strength`
- Avoid compensating with large `background_blur_px`

### Less gray milk / cleaner blacks
- Lower `mist.mist_opacity`
- Lower `mist.mist_accumulation`
- Lower `mist.mist_color_strength`
- Lower `lighting.scene_shadow_floor`
- Only then touch `compose.scene_darken`

### Longer vertical streaks
- Increase `trails.streak_length`
- Increase `trails.streak_boost`
- Increase `trails.trail_distance_max_px`

### Cleaner frame with only a few heavy drops
- Lower `spawn.spawn_rate`
- Lower `micro_droplets.micro_droplets_per_second`
- Disable `mist.enabled`
- Raise `droplets.min_radius_px`

## Preset intent

- `realistic-lens-rain.yml`
  Base preset. Balanced refractive rain for Rotten Club.
- `heavy-condensation.yml`
  Wetter, foggier, more steamed-up glass.
- `thin-neon-drizzle.yml`
  Faster, leaner streak look with less mist.
