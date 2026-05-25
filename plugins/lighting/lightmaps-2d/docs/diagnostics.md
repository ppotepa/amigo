# Lightmaps 2D Diagnostics

Channels:
- `lightmaps-2d.contributions`

## What To Trace
- Lightmap source id.
- Channel ids and their layer lists.
- Resolved `LightMap` target id.
- Scene lighting, highlight, and emissive roles declared by the source.
- Empty channel lists or references that cannot be matched by a light group.

Diagnostics should point to the authored lightmap source that declared the
channel data.
