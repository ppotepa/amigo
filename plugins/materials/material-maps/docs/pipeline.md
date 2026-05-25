# Material Maps Pipeline

Material Maps writes authored material maps into renderer-facing targets.

## Flow
- Scene or tool data declares `MaterialMapRef2d` records.
- Each record names a material id, map kind, and asset.
- Runtime validates the reference and resolves the map kind to a target id.
- The backend target writer reads scene color as context and writes material map
  targets for downstream relight, optics, or refraction passes.

## Targets
- Reads `SceneColor`.
- Writes `SceneHighlight`, `SceneEmissive`, and scene-depth style material
  targets.
- Does not own material source semantics.
