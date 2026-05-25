# Procedural Materials Pipeline

Procedural Materials writes generated 2D material data into declared targets.

## Flow
- Scene or tooling data declares a `ProceduralMaterial2d`.
- Runtime validates the id and generator name.
- The material target kind resolves to a renderer-facing target id.
- The backend target writer reads scene color as context and writes generated
  highlight, emissive, relight mask, or refractive mask data.

## Targets
- Reads `SceneColor`.
- Writes material targets used by optics, relight, and refraction consumers.
- Does not replace the authored `Material2D` source model.
