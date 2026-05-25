# Tilemap 2D Contributions

Tilemap 2D is a renderable source with camera participation.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- `FocusDepthContribution2d` with `DerivedAtHydration` policy.

## Data
- Renderable candidates carry entity name, layer data, status, reason, and target
  ids.
- The focus depth adapter maps tilemap candidates to explicit target ids.
- Highlight participation is role-driven; tile existence alone is not optical
  intent.
