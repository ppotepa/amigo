# Tilemap 2D Runtime

Tilemap 2D runtime services own tilemap state for gameplay and rendering.

## Responsibilities
- Store hydrated tilemap commands for the active scene.
- Resolve rulesets before extraction.
- Validate authored tilemap dimensions, layer data, and tile references.
- Provide tilemap draw commands to the render extractor.

## Boundaries
- Tilemap editor UI is outside this plugin.
- GPU pipeline implementation is outside this plugin.
- Generic asset catalog behavior is outside this plugin.
