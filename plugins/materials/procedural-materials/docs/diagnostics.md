# Procedural Materials Diagnostics

Channels:
- `procedural-materials.targets`
- `procedural-materials.diagnostics`

## What To Trace
- Procedural material id.
- Generator name and seed.
- Target kind and resolved target id.
- Input `SceneColor` availability.
- Consumed optics and focus-depth contributions.
- Invalid declarations with empty ids or generator names.

Diagnostics should make generated target ownership explicit so relight and
refraction passes can identify the material producer.
