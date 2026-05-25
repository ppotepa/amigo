# Relight 2D Diagnostics

Channels:
- `relight-2d.targets`
- `relight-2d.diagnostics`

## What To Trace
- Contribution source id, intensity, and color.
- Candidate read targets and write targets.
- Availability of scene color, normal, lighting, and lightmap inputs.
- Active plate relight debug view id.

## Debug Views
Relight registers `relight.plate.*` debug views for auxiliary depth, height,
occluder, surface, normal, occlusion, contribution, shadow, light mask, NDL,
specular, material gate, and raw lit outputs.
