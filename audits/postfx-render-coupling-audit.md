# PostFX render coupling audit

Status: audit only  
Scope: `PostFx2d` model, render-wgpu execution, frame graph bridge, visual source/debug paths  
Behavior changes: none

## Goal

Identify every central location that currently needs to change when a new `PostFx2d` effect is added.

## Summary

Current PostFX architecture is usable but still centrally coupled.

The biggest remaining coupling points are:

1. `PostFx2d` enum and helper methods.
2. `render-wgpu` screen-space dispatch in `post_fx/registry.rs`.
3. WGPU renderer state fields in `renderer/service/model.rs`.
4. WGPU pipeline/bootstrap code in `renderer/service/init.rs`.
5. Frame graph post-fx bridge in `render/scoped_post_fx.rs`.
6. Camera debug ordering in `render/visual_debug.rs`.
7. Cached image post-fx handling in `texture_batches.rs`.
8. Flat metadata parsing in `post_fx_model/flat_metadata.rs`.

## Coupling inventory

| Area | File | Symbol / region | Coupling type | Severity | Notes |
|---|---|---|---|---|---|
| Effect model | `crates/engine/render-api/src/post_fx_model/effect.rs` | `PostFx2d`, helper methods | Domain enum tax | Medium | Every new effect requires enum/helper updates. |
| Composite duplicate model | `plugins/postfx/composite/src/model/effect.rs` | same model | Duplicate domain model | Medium | Confirm ownership before changing. |
| Renderer dispatch | `crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs` | `execute_screen_space_post_fx` | Central renderer dispatch | High | Main switch executing screen-space effects. |
| Renderer state | `crates/engine/render-wgpu/src/renderer/service/model.rs` | `WgpuSceneRenderer` fields | Effect-specific renderer fields | High | Pipelines/runtimes are concrete fields. |
| Pipeline bootstrap | `crates/engine/render-wgpu/src/renderer/service/init.rs` | shader/layout/pipeline creation | Central WGPU bootstrap | High | New GPU effects require central init edits. |
| Module registry | `crates/engine/render-wgpu/src/renderer/service/post_fx/mod.rs` | module declarations | Manual module list | Medium | New effects require manual module addition. |
| Cached image CPU path | `crates/engine/render-wgpu/src/renderer/service/post_fx/mod.rs` | `apply_cached_image_post_fx_rgba` | Central cached image dispatch | Medium | Cached-image effects are matched centrally. |
| Frame graph bridge | `crates/engine/render-wgpu/src/renderer/service/render/scoped_post_fx.rs` | `execute_post_fx_graph_node` | Effect-specific graph behavior | High | FocusBlur/debug/depth special cases. |
| Camera debug routing | `crates/engine/render-wgpu/src/renderer/service/render/visual_debug.rs` | debug view helpers | Hardcoded visual debug routing | Medium | Debug views are string-matched. |
| Camera debug ordering | `crates/engine/render-wgpu/src/renderer/service/render/visual_debug.rs` | `camera_debug_feature_rank` | Hardcoded order | High | New camera-chain effects need manual rank. |
| Visual source policy | `crates/engine/render-wgpu/src/renderer/service/render/visual_source_buffer_pass/policy.rs` | policy builder | Central visual source policy | Medium | Acceptable short-term but still central. |
| Optical coverage rendering | `crates/engine/render-wgpu/src/renderer/service/render/visual_source_buffer_pass/procedural_material.rs` | candidate appenders | Renderer knows coverage geometry | Medium | Better than heuristic; still backend branching. |
| Texture cache path | `crates/engine/render-wgpu/src/renderer/service/texture_batches.rs` | layered image texture path | Cached texture effect switch | Medium | Cached-image effects require cache key handling. |
| Layer size path | `crates/engine/render-wgpu/src/renderer/service/texture_batches.rs` | layer render size | Bounds special case | Medium | Currently effect-specific. |
| Flat metadata parser | `crates/engine/render-api/src/post_fx_model/flat_metadata.rs` | `post_fx_from_flat_metadata` | Central parser switch | Medium | Metadata-authored effects need parser branch. |

## Recommended next stage

Add metadata-only descriptors first:

```text
PostFxRenderDescriptor
PostFxRenderInput
PostFxRenderOutput
PostFxDebugPolicy
PostFxCachedImagePolicy
```

Do not migrate executors until descriptors can describe existing behavior.
