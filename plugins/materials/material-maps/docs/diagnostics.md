# Material Maps Diagnostics

Channels:
- `material-maps.targets`
- `material-maps.diagnostics`

## What To Trace
- Material id.
- Map kind and resolved target id.
- Asset path.
- Input `SceneColor` availability.
- Consumed optics and focus-depth contribution ids.
- Invalid map references with empty material id or asset path.

Diagnostics should keep material map failures tied to the map reference, not to
the backend pass that later consumes the target.
