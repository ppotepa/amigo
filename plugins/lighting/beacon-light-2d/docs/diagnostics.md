# Beacon Light 2D Diagnostics

Primary channel: `beacon-light-2d.contributions`.

## What To Trace
- Hydrated beacon id, render layer, color, intensity envelope, halo/core radii,
  beam data, and role flags.
- Runtime intensity after animation and jitter.
- Light contribution kinds: relight plate, bloom source, and camera FX source.

Diagnostics should report disabled or missing contribution roles at the beacon
source instead of letting the renderer infer light intent.
