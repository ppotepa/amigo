# Procedural Materials Contributions

Procedural Materials is a target writer for generated material buffers.

## Consumes
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- `FocusDepthContribution2d` with `DerivedAtHydration` policy.
- Valid procedural material declarations with non-empty id and generator name.

## Target Kinds
- `SceneHighlight`
- `SceneEmissive`
- `RelightMask`
- `RefractiveMask`

The generator and seed define the produced material data; the target kind
defines where that data is written.
