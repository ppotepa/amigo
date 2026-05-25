# Light Groups 2D Pipeline

Light Groups 2D owns semantic grouping for 2D lighting.

## Flow
- Scene data declares a `LightGroup2D` component.
- Runtime creates a `LightGroup2dSource` with an id and optional lightmap source
  and channel.
- The source becomes an active `LightGroup2dCandidate` for `SceneLighting`.
- The camera optics adapter converts complete lightmap channel references into
  explicit optical coverage.

## Targets
- Reads `LightMap` metadata when a group references a lightmap channel.
- Contributes highlight and emissive intent.
- Does not render pixels directly.
