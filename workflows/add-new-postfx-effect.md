# Add a new PostFX effect

## Current warning

PostFX is not yet cleanly plugin-owned. Adding an effect currently touches central locations. Prefer completing descriptor work first unless the task explicitly requires a new effect now.

## Current-state locations

```text
crates/engine/render-api/src/post_fx_model/effect.rs
crates/engine/render-api/src/post_fx_model/flat_metadata.rs
plugins/postfx/composite/src/model/effect.rs
plugins/postfx/composite/src/model/flat_metadata.rs
crates/engine/render-wgpu/src/renderer/service/post_fx/mod.rs
crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs
crates/engine/render-wgpu/src/renderer/service/model.rs
crates/engine/render-wgpu/src/renderer/service/init.rs
crates/engine/render-wgpu/src/renderer/service/render/scoped_post_fx.rs
crates/engine/render-wgpu/src/renderer/service/render/visual_debug.rs
crates/engine/render-wgpu/src/renderer/service/texture_batches.rs
```

## Preferred next architecture step

Add descriptor metadata first:

```text
PostFxRenderDescriptor
PostFxRenderInput
PostFxRenderOutput
PostFxDebugPolicy
PostFxCachedImagePolicy
```

## Do not

```text
Do not add PostFx2dV2.
Do not add another hidden fallback.
Do not add a new WgpuSceneRenderer field without noting registry migration debt.
Do not add camera debug order by string without a descriptor plan.
```


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
