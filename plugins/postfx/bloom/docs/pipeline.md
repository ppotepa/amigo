# Bloom Pipeline

Bloom consumes declared bright-source targets and produces a camera artifact.

## Flow
- Source plugins declare highlight, emissive, or optics contributions.
- Runtime extraction routes those declarations into scene highlight and emissive
  targets.
- Bloom reads those targets through its target plan.
- The backend bloom pass writes the camera artifact output for later composite
  stages.

## Targets
- Reads `SceneHighlight` and `SceneEmissive` in the standard target plan.
- Writes `CameraArtifactLayer`.
- Does not own source brightness policy.
