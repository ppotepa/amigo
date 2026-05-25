# Debug Views Contributions

Debug Views is a tooling target consumer.

## Consumes
- Render targets selected for inspection, including scene color, depth,
  highlight, emissive, and camera artifact targets when available.
- The active debug view target plan.

## Emits
- Diagnostics snapshot output for devtools inspection.

Debug Views does not create render-source contributions. It exposes declared
targets so source and target ownership can be inspected.
