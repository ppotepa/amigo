# Camera Film Profiles

Film profiles describe both tone response and scan/grain behavior for `Camera2D`.

Attach a profile from a scene camera:

```yaml
film:
  profile: rotten-club/camera/film/polish-night-custom
  intensity: 0.65
  seed: 1986
```

## Grain Model

`grain` is not a flat noise overlay. It is evaluated per frame in `ScanOutput`
and responds to image density:

- `shadow_amount` - grain contribution in dark density regions, usually `0.0..1.0`.
- `midtone_amount` - grain contribution around the usable straight-line region, usually `0.0..1.0`.
- `highlight_amount` - remaining grain in bright regions, usually lower than midtones.
- `highlight_suppression` - how strongly highlights clean up, `0.0..1.0`.
- `fine_grain_px`, `medium_grain_px`, `coarse_grain_px` - spatial grain scales in output pixels.
- `clumpiness` - blends medium grain toward coarse clustered grain, `0.0..1.0`.
- `softness` - blends sharp pixel grain toward softer dye-cloud grain, `0.0..1.0`.
- `luma_amount` - density/luma grain strength.
- `chroma_amount` - color dye-cloud grain strength.
- `regenerate_per_frame` - when `true`, the grain pattern is resampled every frame.
- `underexposure_boost` - extra grain in underexposed shadows.
- `push_process_boost` - extra roughness for pushed high-speed looks.
- `channel_r`, `channel_g`, `channel_b` - color layer balance.
- `temporal_jitter` - when `regenerate_per_frame` is `true`, `1.0` fully resamples each frame; lower values retain more static scan texture.

Keep grain shape and color noise inside the `grain:` block. Film YAML should not
define top-level `grain_size` or `chroma_noise`.

Recommended models:

- `clean_digital` - nearly neutral scan.
- `modern_color_negative` - Vision/Portra-like controlled color negative grain.
- `fast_color_negative` - ISO 800/1600 color negative with visible shadow grain.
- `bw_silver_pushed` - pushed black-and-white silver grain, high clumpiness.
- `fine_reversal` - fine slide/reversal grain with clean highlights.
- `dirty_scan` - expired/cheap lab scan with stronger chroma and coarse structure.
