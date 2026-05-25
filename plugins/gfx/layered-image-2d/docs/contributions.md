# Layered Image 2D Contributions

Layered Image 2D is a renderable source.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- `FocusDepthContribution2d` with `DerivedAtHydration` policy.
- Layer data that can map to focus depth targets through the participation
  adapter.

## Render Roles
- Authored visual maps and role flags determine highlight, emissive, and depth
  participation.
- Layer order and blend mode remain part of the source command.
- No other plugin executes layered image effects directly; they consume declared
  targets and candidates.
