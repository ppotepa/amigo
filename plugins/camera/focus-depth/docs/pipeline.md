# Focus Depth Pipeline

Focus Depth owns the 2D focus field contract. It turns explicit depth sources
into focus candidates and plans the render target that downstream focus blur
uses.

## Flow
- Authored depth map components hydrate into depth map scene commands.
- The depth map extractor emits render contributions for visible depth sources.
- Runtime collection resolves `FocusDepthCandidate2d` values from source
  coverage and response data.
- The focus depth render plan reads `SceneDepth` and writes `FocusField`.

## Boundaries
- The plugin requires `camera.frame_context.2d@1`.
- It does not infer focus depth from object names or renderer debug state.
- Source plugins own their coverage; Focus Depth only resolves declared inputs.
