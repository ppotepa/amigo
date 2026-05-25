# Camera Optics Contributions

Camera Optics declares the shared `CameraOpticsContribution2d` contract with
`ExplicitOnly` policy.

## Inputs
- Optical sources from lights, beacons, sprites, text, vectors, layered images,
  tilemaps, and particles when those plugins explicitly opt in.
- Coverage kinds include lightmap channel, hotspot, glyphs, texture alpha, vector
  coverage, particle coverage, and unsupported coverage with a reason.
- Responses include intensity, bloom, glare, ghosting, streaks, chromatic smear,
  dirt response, halation, and threshold data.

## Outputs
- Active candidates target `SceneHighlight` and/or `SceneEmissive` through
  declared target ids.
- Candidate roles are carried through the contract instead of renderer heuristics.
- Unsupported or inactive sources remain visible to diagnostics and do not create
  hidden fallbacks.
