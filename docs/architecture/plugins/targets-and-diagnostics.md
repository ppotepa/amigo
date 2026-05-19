# Targets and Diagnostics

Targets are named domain outputs or buffers.

Diagnostics must show the full route from source to output.

## Target examples

```txt
SceneColor
SceneAlpha
SceneDepth
SceneVelocity
SceneHighlight
SceneEmissive
SceneLighting
LightMap
CameraArtifactLayer
FinalComposite
DiagnosticsSnapshot
```

## Diagnostic trace

```txt
source plugin
-> source component
-> contribution
-> candidate
-> target
-> consumer
-> final output
-> status/reason
```

## Rules

* Each target has an owner.
* Each write must be declared in `plugin.toml`.
* Each read must be declared in `plugin.toml`.
* Devtools aggregate diagnostics but do not own domain logic.
* Codemap must be able to jump from diagnostic channel to owning plugin.
