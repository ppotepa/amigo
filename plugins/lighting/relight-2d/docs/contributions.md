# Relight 2D Contributions

Relight 2D is a lighting target consumer. It consumes declared lighting intent
and turns it into relight candidates.

## Consumes
- `LightContribution2d` with `ExplicitOnly` policy.
- Scene color, normal, lighting, and lightmap inputs required by the active
  relight pass.

## Candidate Data
- `Relight2dContribution` carries source id, intensity, and RGBA color.
- `Relight2dCandidate` collects contributions and declares read and write
  targets for relight execution.

Relight does not execute another lighting plugin's source policy. Source
plugins must declare their lighting contributions before relight consumes them.
