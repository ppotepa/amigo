# Render targets

Common render target concepts used in architecture discussions:

```text
SceneColor       main rendered scene color
SceneDepth       depth target for focus/depth-aware effects
SceneNormal      normal target for relight/reflection/optical effects
SceneWetness     wetness/rain/reflection influence target
SceneHighlight   highlight source for bloom/camera optics
SceneEmissive    emissive source for glow/camera optics
CameraArtifactLayer optical/lens artifact output layer
DebugViews       developer/debug output variants
```

Target rule:

A plugin or domain should declare what it contributes to a target. The backend should not infer target contributions implicitly.
