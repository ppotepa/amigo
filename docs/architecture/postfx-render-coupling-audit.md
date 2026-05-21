# PostFX Render Coupling Audit

Status: audit only  
Scope: `PostFx2d` model, `render-wgpu` execution, frame graph bridge, visual source/debug paths  
No behavior changes in this stage.

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

The audit confirms that the main remaining problem is no longer app-centric wiring. The bigger issue is that the central renderer still owns PostFX dispatch, WGPU pipeline bootstrap, and effect-specific exceptions.

## Coupling inventory

| Area | File | Symbol / lines | Coupling type | Severity | Notes |
|---|---|---:|---|---|---|
| Effect model | `crates/engine/render-api/src/post_fx_model/effect.rs` | `PostFx2d`, helpers at `4-134` | Domain enum tax | Medium | Every new effect requires enum/helper updates: `kind`, `default_role`, `photographic_family`, `is_cached_image_compatible`, `is_frame_graph_compatible`, `normalized`, `is_active`. |
| Composite flat metadata duplicate | `plugins/postfx/composite/src/model/flat_metadata.rs` | same parser at `50-560` | Duplicate parser switch | Medium | Composite still keeps its own flat-metadata parsing path even though the effect model now comes from `render-api`. |
| Renderer dispatch | `crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs` | `execute_screen_space_post_fx`, `11-228` | Central renderer dispatch | High | Main `match effect` for every screen-space effect. |
| CameraOptics special case | `crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs` | `PostFx2d::CameraOptics(...)` branch | Effect-specific renderer inputs | High | Manually fetches `scene_normal`, `scene_wetness`, `scene_highlight`, `scene_emissive`, then composites visual-source response. |
| FilmEmulsion special case | `crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs` | `PostFx2d::FilmEmulsion(...)` branch | Effect-specific renderer inputs | High | Manually fetches the same visual-source targets and runs a custom response/composite path. |
| Request/runtime special cases | `crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs` | `RainGlass`, `FocusBlur`, `ShutterBlur`, `WetReflections` branches | Effect-specific executor plumbing | High | `RainGlass` takes `request`, `host_id`, `effect_id`; `FocusBlur` and `WetReflections` take full request; `ShutterBlur` takes host/effect identity. |
| Cached-image dispatch | `crates/engine/render-wgpu/src/renderer/service/post_fx/mod.rs` | `apply_cached_image_post_fx_rgba`, `20-63` | Central cached image dispatch | Medium | Cached-image effects are manually matched; unsupported effects silently no-op. |
| PostFX module registry | `crates/engine/render-wgpu/src/renderer/service/post_fx/mod.rs` | module list at `1-18` | Manual module registry | Medium | New effect modules must be added by hand. |
| Renderer state | `crates/engine/render-wgpu/src/renderer/service/model.rs` | `WgpuSceneRenderer` fields, `8-58` | Effect-specific renderer fields | High | Pipelines, layouts, and runtime maps are stored as concrete effect fields. |
| Pipeline bootstrap | `crates/engine/render-wgpu/src/renderer/service/init.rs` | shader consts + layouts + pipelines + `Self { ... }`, especially `416-590`, `894-1170`, `1588-1675`, `2026-2109`, `2186-2764` | Central WGPU pipeline bootstrap | High | New GPU effects require central shader constants, layouts, pipeline creation, and struct initialization edits. |
| Frame graph bridge | `crates/engine/render-wgpu/src/renderer/service/render/mod.rs` | `execute_post_fx_graph_node`, `46-67` | Thin bridge | Low | Mostly acceptable adapter layer. |
| Graph node trampoline | `crates/engine/render-wgpu/src/renderer/service/render/graph_nodes.rs` | `execute_post_fx_graph_node`, `88-96` | Thin bridge | Low | Only forwards into scoped executor. |
| Frame graph executor | `crates/engine/render-wgpu/src/renderer/service/render/scoped_post_fx.rs` | `execute_post_fx_graph_node`, `89-397` | Effect-specific frame graph behavior | High | Contains hardcoded bypasses, scope mapping, debug handling, and post-pass behavior. |
| Debug bypasses | `crates/engine/render-wgpu/src/renderer/service/render/scoped_post_fx.rs` | `camera.raw_scene_color`, relight debug, unsupported pipeline copy, `106-128` | Hardcoded bypass behavior | Medium | Renderer decides when to bypass PostFX completely. |
| FocusBlur debug depth path | `crates/engine/render-wgpu/src/renderer/service/render/scoped_post_fx.rs` | `camera.scene_depth` / `camera.computed_z_depth`, `146-160` | Effect-specific debug path | High | Debug depth view currently works only through `focus_blur`. |
| FocusBlur post-pass plan | `crates/engine/render-wgpu/src/renderer/service/render/scoped_post_fx.rs` | replay-scoped-layer handling, `195-278` and `280-397` | Effect-specific post render phase | High | Explicit z-depth layers, overlay layers, and implicit affected layers are hardcoded after generic PostFX execution. |
| Scope mapping | `crates/engine/render-wgpu/src/renderer/service/render/scoped_post_fx.rs` | `ScopedPostFxTarget::from_scope` path | Central scope mapping | Medium | Central scope logic is acceptable short-term, but still centralized. |
| Camera debug routing | `crates/engine/render-wgpu/src/renderer/service/render/visual_debug.rs` | `try_execute_camera_debug_view`, `8-111` | Hardcoded visual debug routing | Medium | Debug views are string-matched and manually mapped to visual-source behavior. |
| Camera debug ordering | `crates/engine/render-wgpu/src/renderer/service/render/visual_debug.rs` | `camera_debug_feature_rank`, `321-349` | Hardcoded debug ordering | High | New camera-chain effects need manual rank insertion. |
| Visual source policy | `crates/engine/render-wgpu/src/renderer/service/render/visual_source_buffer_pass/policy.rs` | `from_request`, `source_policy`, `debug_view_wants_source`, `21-144` | Central visual source policy | Medium | Request policy is centralized; acceptable for now but still string/flag-driven. |
| Optical candidate rendering | `crates/engine/render-wgpu/src/renderer/service/render/visual_source_buffer_pass/procedural_material.rs` | `append_camera_optical_candidate_*`, `20-55`, `147-362`, `471-489` | Renderer knows optical coverage geometry | Medium | Better than old heuristics, but renderer still branches on `Hotspot`, `ParticleCoverage`, `Glyphs`, `TextureAlpha`, `VectorCoverage`, `LightMapChannel`, `Unsupported`. |
| Texture cache path | `crates/engine/render-wgpu/src/renderer/service/texture_batches.rs` | `ensure_layered_image_texture_from_path`, `295-340` | Cached texture effect switch | Medium | New cached-image effects require cache-key and path handling here. |
| Layer render size | `crates/engine/render-wgpu/src/renderer/service/texture_batches.rs` | `layered_image_layer_render_size`, `655-669` | Effect-specific bounds expansion | Medium | Currently knows only `PostFx2d::Blur`. |
| Flat metadata parser | `crates/engine/render-api/src/post_fx_model/flat_metadata.rs` | `post_fx_from_flat_metadata`, `50-560` | Central document parser switch | Medium | New metadata-authored effects require parser branch and aliases. |
| Composite flat metadata duplicate | `plugins/postfx/composite/src/model/flat_metadata.rs` | same parser at `50-560` | Duplicate parser switch | Medium | Duplicates the central parser exactly today. |

## Highest-risk hotspots

### 1. `post_fx/registry.rs`

`execute_screen_space_post_fx` is the main renderer-side switch.

Problem:  
The renderer decides how every effect executes and what auxiliary targets it receives.

Observed special handling:

- `CameraOptics` manually wires `scene_normal`, `scene_wetness`, `scene_highlight`, `scene_emissive`.
- `FilmEmulsion` manually wires the same visual-source targets.
- `RainGlass` manually uses `request`, `host_id`, `effect_id`.
- `FocusBlur` manually gets the whole request.
- `ShutterBlur` manually gets `host_id` and `effect_id`.
- `Blur` and `EmbossEdges` use a central copy/no-op style path.

Target direction:  
Replace with a descriptor-backed executor registry.

Future shape:

```txt
PostFxRenderDescriptor {
  feature_id
  executor_id
  pipeline_kind
  required_inputs
  optional_inputs
  output_policy
  debug_rank
}
```

### 2. `renderer/service/init.rs` and `renderer/service/model.rs`

Problem:  
Effect-specific GPU pipelines are concrete fields on `WgpuSceneRenderer` and initialized centrally.

Today a new shader-backed effect typically requires:

- a `*_SHADER` constant,
- `create_shader_module`,
- `create_pipeline_layout`,
- `create_color_pipeline`,
- new fields on `WgpuSceneRenderer`,
- new initialization entries in `Self { ... }`.

Target direction:  
Move effect pipeline creation into registered WGPU PostFX pipeline descriptors.

### 3. `render/scoped_post_fx.rs`

Problem:  
The frame graph executor contains effect-specific behavior after generic PostFX execution.

Hotspots:

- `camera.raw_scene_color` bypass,
- plate-relight debug bypass,
- unsupported pipeline copy fallback,
- `camera.scene_depth` / `camera.computed_z_depth` hardcoded to `focus_blur`,
- replay-scoped-layer planning for `FocusBlur`,
- explicit z-depth and overlay layer replay.

Target direction:  
Move this to effect-owned post-pass hooks or descriptor-declared post render phases.

### 4. `render/visual_debug.rs`

Problem:  
Camera debug ordering is hardcoded by string feature id.

Target direction:  
Use descriptor-provided `debug_rank` or `camera_capture_order`.

### 5. `texture_batches.rs`

Problem:  
Cached-image PostFX is a separate central path from screen-space PostFX.

Today a cached-image-compatible effect requires:

- `PostFx2d::is_cached_image_compatible()`,
- a cache-key path in `ensure_layered_image_texture_from_path`,
- CPU/image execution policy,
- optional bounds expansion handling in `layered_image_layer_render_size`.

Target direction:  
Use cached-image descriptors for CPU/image processing effects.

## Current positive signs

Camera optics is no longer only a renderer-side brightness heuristic. It now has candidate and target concepts:

- `CameraOpticalCandidate2d`
- `CameraOpticalCoverage2d`
- `CameraOpticalResponse2d`
- `CameraOpticalRenderTargetPlan`
- `SceneHighlight`
- `SceneEmissive`

This is the correct architectural direction.

The visual-source side is partially policy/descriptor-driven already:

- `CameraOpticalRenderTargetPlan::for_visual_kind_name`
- `coverage_uses_texture_path`
- `lightmap_channel_parts`
- `optical_candidate_color_rgba_for_target`

The remaining problem is that renderer-owned branching still consumes those concepts centrally.

## Central coupling count

This audit identifies **23** central coupling points that currently need review or edits when a new `PostFx2d` effect is introduced.

The highest-cost set is:

1. effect enum/helpers,
2. renderer dispatch,
3. renderer state fields,
4. WGPU pipeline bootstrap,
5. frame graph special cases,
6. debug ordering,
7. cached-image path,
8. flat metadata parsing.

## Do not fix in this stage

Do not perform any of these in the audit stage:

- no descriptor implementation,
- no enum migration,
- no WGPU registry migration,
- no plugin manifest schema change,
- no cached-image behavior change,
- no debug behavior change,
- no FocusBlur behavior change,
- no CameraOptics behavior change.

## Recommended next stage

Etap 2 should introduce a minimal read-only descriptor model before moving execution:

```txt
PostFxRenderDescriptor
PostFxRenderInput
PostFxRenderOutput
PostFxDebugPolicy
PostFxCachedImagePolicy
```

The first implementation target should be descriptor metadata only, not executor migration.

Suggested pilot effects:

1. `CameraOptics`
2. `FocusBlur`
3. `RainGlass`

Reason:  
They currently exercise the most problematic paths: visual sources, request access, runtime state, debug behavior, and post-pass handling.
