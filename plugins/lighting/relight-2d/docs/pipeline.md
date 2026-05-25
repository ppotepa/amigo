# Relight 2D Pipeline

Relight 2D consumes declared lighting data and produces a relit scene target.

## Flow
- Lighting sources emit explicit light contributions.
- Runtime resolves those records into a `Relight2dCandidate`.
- The candidate declares the scene buffers it reads and the lighting or color
  target it writes.
- The backend relight pass combines surface data, lighting data, and lightmaps.
- Optional plate relight debug views expose intermediate buffers.

## Targets
- Reads scene color, normals, lighting, and lightmap inputs.
- Writes the relit lighting or color output declared by the active target plan.
- Does not infer light sources from rendered object names.
