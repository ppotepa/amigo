# Focus Depth Contributions

Focus Depth declares `FocusDepthContribution2d` with `DerivedAtHydration`
policy.

## Inputs
- Sprite, layered image, vector, tilemap, and other source plugins can derive
  focus depth contributions during scene hydration.
- Coverage kinds include `SceneDepth`, render layer, scene object, distance, and
  unsupported coverage with a reason.
- Responses carry blur radius, focus weight, near/far range, and related focus
  field data.

## Targets
- `SceneDepth` is the source buffer for depth information.
- `FocusField` is the resolved output target.
- Inactive or unsupported candidates remain diagnostic data and must not create
  fallback focus targets.
