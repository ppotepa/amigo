# Rotten Club Camera Profiles

Normalny workflow:
- wybierz camera preset
- koryguj `focus_distance_m`, `f_stop` i profil obiektywu
- używaj `z_depth` tylko jako debug override
- local scene post-fx zostawiaj dla świadomych wyjątków, nie jako domyślny język kamery

Camera2D uses built-in engine profile IDs for now.
Camera2D can also reference YAML asset keys for custom profiles.

Recommended lens profiles:
- vintage_soviet_35mm_dirty
- gritty_club_lens
- night_bus_window
- wet_window_macro
- cheap_cctv_1996

Recommended film profiles:
- polish_1994_push_800
- rotten_neon_push_1600
- expired_orwo_400
- newsprint_bleach_bypass
- surveillance_tape_color

Custom asset examples:
- `rotten-club/camera/lens/gritty-club-custom`
- `rotten-club/camera/film/polish-night-custom`
- `rotten-club/camera/look/rotten-noir-print`
- `rotten-club/camera/rain/realistic-lens-rain`

Rain glass / lens rain profiles live in `camera/rain/*.yml`.
Attach them from `Camera2D` with:

```yaml
lens_surface:
  rain_profile: rotten-club/camera/rain/realistic-lens-rain
```

Camera lens-surface effects are camera-owned. Runtime animation should go through
`world.camera.*`, not `world.postfx.*`.

Look profiles live in `camera/look/*.yml`.
Attach them from `Camera2D` with:

```yaml
look:
  profile: rotten-club/camera/look/rotten-noir-print
  intensity: 0.85
```

Film profiles live in `camera/film/*.yml`. Their `grain` section controls the
camera-owned scan grain model: density response, chroma/luma grain balance,
grain scale, clumpiness, highlight cleanup and per-frame sampling.
