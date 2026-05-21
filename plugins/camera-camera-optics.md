# camera-camera-optics

Path: `plugins/camera/camera-optics`  
Cargo package: `amigo-camera-optics-plugin`  
Plugin id: `amigo.camera.camera-optics`  
Family: `camera`  
Kind: `target-consumer`  
Renderable: `False`  
Render participation: `target-consumer`

## Role

camera plugin `camera-optics`. Owns camera optical contribution consumption, candidates, target diagnostics, and camera artifact participation.

## Manifest capabilities

- provides: `camera.optics.2d@1`
- requires: `camera.frame_context.2d@1`

## Manifest slots

- implements: `camera.optics.consumer.2d`
- requires: `camera.frame_provider.2d`
- replaces: none declared

## Manifest targets

- reads: `SceneHighlight`
- reads: `SceneEmissive`
- reads: `SceneDepth`
- writes: `CameraArtifactLayer`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`

## Manifest diagnostics

- channels: `camera.optical.contributions`
- channels: `camera.optical.candidates`
- channels: `camera.optical.targets`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/camera/camera-optics/src/api/candidate.rs`
- `plugins/camera/camera-optics/src/api/coverage.rs`
- `plugins/camera/camera-optics/src/api/diagnostics_model.rs`
- `plugins/camera/camera-optics/src/api/mod.rs`
- `plugins/camera/camera-optics/src/api/response.rs`
- `plugins/camera/camera-optics/src/api/source.rs`
- `plugins/camera/camera-optics/src/api/targets.rs`
- `plugins/camera/camera-optics/src/diagnostics/format.rs`
- `plugins/camera/camera-optics/src/diagnostics/mod.rs`
- `plugins/camera/camera-optics/src/diagnostics/snapshot.rs`
- `plugins/camera/camera-optics/src/lib.rs`
- `plugins/camera/camera-optics/src/manifest.rs`
- `plugins/camera/camera-optics/src/participation/mod.rs`
- `plugins/camera/camera-optics/src/plugin.rs`
- `plugins/camera/camera-optics/src/render_wgpu/candidate_buffers.rs`
- `plugins/camera/camera-optics/src/render_wgpu/color_coverage.rs`
- `plugins/camera/camera-optics/README.md`
- `plugins/camera/camera-optics/plugin.toml`
- `plugins/camera/camera-optics/docs/pipeline.md`
- `plugins/camera/camera-optics/docs/contributions.md`
- `plugins/camera/camera-optics/docs/diagnostics.md`
- `plugins/camera/camera-optics/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-assets`
- `amigo-camera`
- `amigo-camera-profiles-plugin`
- `amigo-composite-plugin`
- `amigo-plugin-api`
- `serde`

## Allowed changes

```text
plugin-owned domain models
plugin manifest capabilities/slots/targets/contributions
diagnostics declared by the plugin
waterfall tests for plugin-owned behavior
local docs when the plugin is touched
```

## Forbidden changes

```text
direct renderer hacks outside declared backend adapter path
app-side wiring for plugin behavior
silent fallback if a contribution is missing
legacy/v2 duplicate plugin paths
```

## Validation commands

```powershell
cargo check -p amigo-camera-optics-plugin
cargo test -p amigo-camera-optics-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/camera/camera-optics
rg -n "pub struct|pub enum|pub trait|impl " plugins/camera/camera-optics/src
```
