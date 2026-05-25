# Camera Profiles Contributions

Camera Profiles does not emit render contributions or consume source candidates.

## Provides
- `CameraProfile2d` records with id, label, optional lens profile, optional film
  profile, and optional focus distance in meters.
- `CameraQualityProfile2d` presets: preview, gameplay, cinematic, and debug.
- `CameraQualitySettings2d` values such as render scale, visual source buffer
  quality, motion source quality, and layer mask quality.

## Does Not Provide
- No `SceneColor`, `SceneDepth`, `SceneHighlight`, `SceneEmissive`, or
  `SceneVelocity` targets.
- No camera optics, focus depth, or shutter motion candidates.
- No WGPU resource policy.
