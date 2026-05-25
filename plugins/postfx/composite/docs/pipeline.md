# Composite Pipeline

Composite owns the target-consumer side of the post-fx stack and final frame
composition.

## Flow
- Scene documents and runtime commands build scoped `PostFx2dStack` values.
- `PostFx2dService` stores frame, draw-layer, object, group, source-image, and
  image-part stacks.
- `PostFx2dRenderExtractor` sends scoped stacks into render extraction.
- The backend executes declared post-fx descriptors and writes the final output.

## Targets
- Reads `SceneColor` and `CameraArtifactLayer`.
- Writes `FinalComposite`.
- It does not consume source plugin candidates directly.
