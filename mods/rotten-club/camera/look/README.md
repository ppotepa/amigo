# Rotten Club Camera Look Profiles

These files define camera-owned final look presets.
They are reusable mod assets, not scene-local post-fx blobs.

## Where to put them

Use:
- `mods/rotten-club/camera/look/*.yml`

Recommended preset in this mod:
- `rotten-club/camera/look/rotten-noir-print`

## How to attach to Camera2D

Add this to a `Camera2D` component:

```yaml
look:
  profile: rotten-club/camera/look/rotten-noir-print
  intensity: 0.85
```

Runtime ownership stays on `Camera2D`.
Authoring storage stays in the mod asset.

## YAML shape

```yaml
kind: camera-look-profile-2d
label: Rotten Noir Print
id: rotten_noir_print
palette_size: 28
dither_strength: 0.10
dither_scale: 1.0
layered_dither: 0.18
opacity: 0.82
luma_preserve: 0.60
highlight_bias: 0.08
shadow_bias: -0.10
contrast: 1.12
saturation: 0.84
gamma: 0.96
seed: 1986
```

## Property guide

- `palette_size`: usually `12..64`. Lower values push a harsher print-style palette.
- `dither_strength`: usually `0.0..0.3`. Higher values add more visible grainy breakup.
- `dither_scale`: usually `0.5..2.0`. Changes dither cell scale.
- `layered_dither`: usually `0.0..0.5`. Adds a second layer of texture.
- `opacity`: usually `0.0..1.0`. Base effect opacity before `Camera2D.look.intensity`.
- `luma_preserve`: usually `0.0..1.0`. Higher values preserve original luminance more closely.
- `highlight_bias`: usually `-0.3..0.3`. Positive values brighten highlights.
- `shadow_bias`: usually `-0.3..0.3`. Negative values deepen shadows.
- `contrast`: usually `0.6..1.4`.
- `saturation`: usually `0.0..1.4`.
- `gamma`: usually `0.7..1.3`.
- `seed`: integer. Changes dither pattern.
