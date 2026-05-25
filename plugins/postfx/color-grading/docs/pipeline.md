# Color Grading Pipeline

Color Grading owns the color transform stage for the post-fx stack.

## Flow
- Scene data declares a `ColorGrading` post-fx component.
- Runtime resolves the active grading target plan.
- The backend reads the scene color target.
- The grading pass writes the final composite or the manifest-declared color
  output for the current stack position.

## Targets
- Reads `SceneColor`.
- Writes `FinalComposite` in the public target plan.
- Does not emit lighting, material, or camera source contributions.
