# Tilemap 2D Diagnostics

Channels:
- `tilemap-2d.render`
- `tilemap-2d.contributions`

## What To Trace
- Hydrated tilemap dimensions, tile size, tileset id, render layer, and layer
  count.
- Ruleset resolution and validation failures before render extraction.
- Candidate status, reason, and target ids for contribution routing.

Renderer diagnostics should consume the extracted command and candidate data; the
source plugin owns invalid tilemap data reports.
