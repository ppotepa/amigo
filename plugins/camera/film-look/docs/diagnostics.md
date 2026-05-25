# Film Look Diagnostics

Channels:
- `film-look.composite`

## What To Trace
- Active profile id and label.
- Normalized enabled state, grain, halation, sensor response, film response, and
  tone curve.
- Input target availability for `SceneColor` and `CameraArtifactLayer`.
- Output routing to `FinalComposite`.

Diagnostics should make disabled profiles explicit so a missing film pass can be
distinguished from an empty or unavailable input target.
