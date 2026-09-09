# NPR playground pipeline

Authored scene YAML hydrates `NprPlaygroundState`. The plugin owns object
selection, camera intent, style inheritance and typed drawing policy. During
`RenderExtract`, it resolves scene layers plus sparse object overrides into an
`NprDrawCommand` and submits it through the registered extractor bridge.

`amigo-render-npr` owns projection, visible features, strokes and tessellation.
`amigo-render-wgpu` only executes the resulting packet in declared layer order:
depth, then colour batches. The backend does not infer style from mesh names or
topology.

Paper is scene-owned because one `NprBackgroundCommand` covers the viewport.
Object-level Paper edits are rejected. Every active draw command must agree on
Paper enabled state and opacity.
