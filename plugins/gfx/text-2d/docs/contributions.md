# Text 2D Contributions

Text 2D is a renderable source with camera participation.

## Emits
- `CameraOpticsContribution2d` with `ExplicitOnly` policy.
- `FocusDepthContribution2d` with `DerivedAtHydration` policy.

## Data
- Camera optics coverage is `Glyphs` with entity and render layer metadata.
- Focus depth contributions are derived from the hydrated render layer.
- Material optical and lighting fields remain part of the text scene command.

Text does not emit shutter motion contributions in this plugin.
