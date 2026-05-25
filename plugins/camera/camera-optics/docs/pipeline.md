# Camera Optics Pipeline

Camera Optics owns the 2D optical response contract. It consumes explicit optical
sources, resolves them into camera optical candidates, and plans which camera
artifact buffers are required.

## Flow
- Source plugins emit `CameraOpticalSource2d` or equivalent contribution data.
- Runtime collection resolves coverage, response, render roles, status, reason,
  and target ids into `CameraOpticalCandidate2d`.
- Render target helpers route active candidates to `SceneHighlight` and
  `SceneEmissive` only when declared roles require those targets.

## Boundaries
- The plugin requires `camera.frame_context.2d@1`.
- It reads authored visual sources such as `SceneHighlight`, `SceneEmissive`, and
  `SceneDepth`, but it does not guess optical intent from their existence.
- The WGPU backend consumes the resolved contract; it does not invent domain
  policy.
