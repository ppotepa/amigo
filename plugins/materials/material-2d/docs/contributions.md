# Material 2D Contributions

Material 2D is a semantic material source.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- `FocusDepthContribution2d` with `DerivedAtHydration` policy.
- Semantic roles for `SceneHighlight`, `SceneEmissive`, and `SceneDepth`.

## Optical Adapter
- `material_to_camera_optics_response` enables optical response when opacity is
  greater than zero.
- Optical intensity is copied from material opacity.
- Bloom, glare, ghosting, streaks, chromatic smear, dirt response, halation, and
  threshold remain zero unless another material path declares them.

Material semantics are declared by the material source and then consumed by
target writers such as material maps or procedural materials.
