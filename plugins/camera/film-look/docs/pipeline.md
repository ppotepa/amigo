# Film Look Pipeline

Film Look owns the final camera film response pass for 2D frames.

## Flow
- Scene data declares a `FilmLook` camera component or profile.
- Runtime resolves a `FilmLookResponse2d` and normalizes numeric response
  values.
- The render target plan reads the scene color image, combines any camera
  artifact layer required by the active profile, and writes the final composite.
- The backend pass applies grain, halation, sensor response, film response, and
  tone curve as camera-space post processing.

## Targets
- Reads `SceneColor` and camera artifact input.
- Writes `FinalComposite`.
- Does not create lighting, optics, or material source contributions.
