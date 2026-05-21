# Extend camera optics

## Correct flow

```text
authored domain object
  -> explicit camera_response / render_contribution
  -> CameraOpticalCandidate2d
  -> CameraOpticalCoverage2d
  -> CameraOpticalRenderTargetPlan
  -> SceneHighlight / SceneEmissive
  -> CameraOptics effect
  -> diagnostics
```

## Operation plan

```text
READ docs/09-camera-optics-pipeline.md
READ plugins/camera-camera-optics.md
READ relevant contributor plugin doc
READ crates/engine-render-api.md
READ render-wgpu visual source adapter only if backend coverage changes
MODIFY contributor to emit explicit response/candidate
MODIFY extraction bridge if required
ADD diagnostics and tests
```

## Forbidden

Do not add heuristic behavior such as "if a lightmap exists, treat it as a lens source". Authoring must be explicit.


## Common requirements

```text
Start with git status.
Use amigo-codemap first.
Read only relevant symbols/ranges.
Make minimal ADD/MODIFY/DELETE/MOVE changes.
Validate with targeted commands.
Report risks and next action.
```

## Hard prohibitions

```text
No legacy/v2 parallel paths.
No silent fallbacks.
No renderer-side domain guessing.
No large formatting-only diffs.
No workspace-wide check/test by default.
```
