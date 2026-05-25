# Film Look Contributions

Film Look is a camera target consumer. It does not emit scene-source
contributions; it consumes camera frame context and declared render targets.

## Consumes
- `SceneColor` as the graded image source.
- `CameraArtifactLayer` when the frame includes upstream camera artifacts.
- `FilmLookProfile2d` values with stable `id`, `label`, and normalized response
  settings.

## Response Data
- `FilmLookResponse2d` controls enabled state, grain, halation, sensor response,
  film response, and tone curve.
- Runtime resolution clamps non-finite or out-of-range numeric values before the
  backend uses them.

Film look must not infer bloom, optics, or lighting intent from rendered object
presence. Those effects come from their own declared contributions.
