# Bloom Diagnostics

Channels:
- `postfx.bloom`

## What To Trace
- Active target plan.
- Availability of `SceneHighlight` and `SceneEmissive`.
- Output routing to `CameraArtifactLayer` or the manifest-declared scene color
  target.
- Empty input buffers that produce no bloom contribution for the frame.

Diagnostics should identify which declared source target fed bloom, so missing
glow can be fixed at the source plugin.
