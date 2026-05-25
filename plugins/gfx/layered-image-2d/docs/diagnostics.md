# Layered Image 2D Diagnostics

Channels:
- `layered-image-2d.render`
- `layered-image-2d.contributions`

## What To Trace
- Hydrated entity name, render layer, layer count, blend modes, and viewport fit.
- Extracted draw commands after scene visibility filtering.
- Contribution roles for highlight, emissive, depth, and focus depth targets.

Missing assets or empty layer lists should be reported against the source entity
so the renderer does not need to guess why no image appeared.
