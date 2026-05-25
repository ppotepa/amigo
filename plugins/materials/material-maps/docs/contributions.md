# Material Maps Contributions

Material Maps is a target writer. It consumes declared material and camera
participation data and writes material-related targets.

## Consumes
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- `FocusDepthContribution2d` with `DerivedAtHydration` policy.
- Material map references with non-empty material id and asset path.

## Map Kinds
- `SceneHighlight`
- `SceneEmissive`
- `RelightMask`
- `RefractiveMask`

The map kind selects the target id. Material Maps does not emit new source
contributions; it writes target buffers from already declared material data.
