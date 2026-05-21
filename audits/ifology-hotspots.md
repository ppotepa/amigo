# Ifology hotspots

This file lists places that should be treated carefully because they centralize special-case behavior.

## Highest-risk files

```text
crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs
crates/engine/render-wgpu/src/renderer/service/init.rs
crates/engine/render-wgpu/src/renderer/service/model.rs
crates/engine/render-wgpu/src/renderer/service/render/scoped_post_fx.rs
crates/engine/render-wgpu/src/renderer/service/render/visual_debug.rs
crates/engine/render-wgpu/src/renderer/service/texture_batches.rs
crates/engine/scene/src/component_metadata.rs
crates/engine/render-wgpu/src/renderer/service/render/visual_source_buffer_pass/procedural_material.rs
```

## Rule for agents

Do not add another branch in these files without first asking whether the branch should be a descriptor, contribution, candidate, provider, or registry entry.

## Acceptable short-term edits

Small fixes are acceptable if they do not expand central ownership. For example:

```text
bug fix inside an existing branch
additional diagnostic when a declared contribution is missing
test proving current behavior before migration
```

## Bad edits

```text
new hardcoded feature id rank
new renderer-side guess based on object existence
new effect-specific field on WgpuSceneRenderer without a registry plan
new parser branch without documenting descriptor migration impact
```
