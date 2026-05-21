# Known file hotspots

These files should be opened only with a specific reason and narrow range.

## Renderer / PostFX

```text
crates/engine/render-wgpu/src/renderer/service/init.rs
crates/engine/render-wgpu/src/renderer/service/model.rs
crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs
crates/engine/render-wgpu/src/renderer/service/post_fx/mod.rs
crates/engine/render-wgpu/src/renderer/service/render/scoped_post_fx.rs
crates/engine/render-wgpu/src/renderer/service/render/visual_debug.rs
crates/engine/render-wgpu/src/renderer/service/texture_batches.rs
```

## Scene metadata

```text
crates/engine/scene/src/component_metadata.rs
```

## Runtime bridge

```text
crates/runtime/bundles/src/render_extractor_bridges/**
```

## Rule

Before modifying any hotspot, state whether the change is:

```text
contract addition
bridge adaptation
backend execution
diagnostic only
cleanup/removal
```
