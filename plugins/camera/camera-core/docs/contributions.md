# Camera Core Contributions

Camera Core is a frame provider, not a render contribution source.

## Provides
- Capability: `camera.frame_context.2d@1`.
- Slot: `camera.frame_provider.2d`.
- Runtime state: camera transforms, projection, viewport, depth motion, quality
  profile selection, and focus target lookup.

## Does Not Emit
- No `SceneColor`, `SceneDepth`, `SceneHighlight`, `SceneEmissive`, or
  `SceneVelocity` writes.
- No camera optics, focus depth, or shutter motion candidates.
- No renderer-side domain policy.

Downstream plugins must declare their own candidates and targets when they use
camera data.
