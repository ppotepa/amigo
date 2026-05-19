# Plugin Migration Inventory

| current_path | target_plugin | target_path | operation | status |
|---|---|---|---|---|
| `crates/engine/camera` | `amigo.camera.camera-core` | `plugins/camera/camera-core` | MOVE | mapped |
| `crates/engine/camera/src/focus_targets.rs` | `amigo.camera.focus-depth` | `plugins/camera/focus-depth/src/runtime/focus_targets.rs` | MOVE | mapped |
| `crates/engine/camera/src/optics.rs` | `amigo.camera.camera-optics` | `plugins/camera/camera-optics/src/runtime/optics.rs` | MOVE | mapped |
| `crates/engine/camera/src/film_grain.rs` | `amigo.camera.film-look` | `plugins/camera/film-look/src/runtime/film_grain.rs` | MOVE | mapped |
| `crates/engine/camera/src/profiles.rs` | `amigo.camera.camera-profiles` | `plugins/camera/camera-profiles/src/runtime/profiles.rs` | MOVE | mapped |
| `crates/engine/camera/src/quality.rs` | `amigo.camera.camera-profiles` | `plugins/camera/camera-profiles/src/api/quality.rs` | MOVE | mapped |
| `crates/engine/scene` | scene contracts | `crates/engine/scene` | REDUCE | mapped |
| `crates/engine/scene/src/document/camera.rs` | camera plugins | `plugins/camera/*/src/scene/document.rs` | MOVE | mapped |
| `crates/engine/scene/src/document/material2d.rs` | `amigo.materials.material-2d` | `plugins/materials/material-2d/src/scene/document.rs` | MOVE | mapped |
| `crates/engine/scene/src/document/visual2d/lighting.rs` | lighting plugins | `plugins/lighting/*/src/scene/document.rs` | MOVE | mapped |
| `crates/engine/scene/src/document/visual2d/post_fx*.rs` | postfx plugins | `plugins/postfx/*/src/scene/document.rs` | MOVE | mapped |
| `crates/engine/scene/src/component_metadata.rs` | owning plugins | `plugins/*/*/src/scene/descriptors.rs` | MOVE | mapped |
| `crates/engine/scene/src/render_commands/render_2d.rs` | gfx/lighting/material/postfx plugins | `plugins/*/*/src/scene/commands.rs` | MOVE | mapped |
| `crates/runtime/bundles` | runtime composition | `crates/runtime/bundles` | REDUCE | mapped |
| `crates/runtime/bundles/src/render_extractor_bridges` | owning plugins | `plugins/*/*/src/runtime/extract.rs` | MOVE | mapped |
| `crates/runtime/bundles/src/focus_targets_2d.rs` | `amigo.camera.focus-depth` | `plugins/camera/focus-depth/src/runtime/focus_targets.rs` | MOVE | mapped |
| `crates/engine/render-wgpu` | render backend | `crates/engine/render-wgpu` | REDUCE | mapped |
| `crates/engine/render-wgpu/src/renderer/service/render/visual_source_buffer_pass` | camera/material plugins | `plugins/*/*/src/render_wgpu` | MOVE | mapped |
| `crates/engine/render-wgpu/src/renderer/service/post_fx` | postfx/camera plugins | `plugins/postfx/*/src/render_wgpu` and `plugins/camera/*/src/render_wgpu` | MOVE | mapped |
| `crates/scripting/rhai` | scripting host | `crates/scripting/rhai` | REDUCE | mapped |
| `crates/scripting/rhai/src` | owning plugins | `plugins/*/*/src/scripting` | MOVE | mapped |
| `crates/2d/sprite` | `amigo.gfx.sprite-2d` | `plugins/gfx/sprite-2d` | MOVE | mapped |
| `crates/2d/text` | `amigo.gfx.text-2d` | `plugins/gfx/text-2d` | MOVE | mapped |
| `crates/2d/vector` | `amigo.gfx.vector-2d` | `plugins/gfx/vector-2d` | MOVE | mapped |
| `crates/2d/tilemap` | `amigo.gfx.tilemap-2d` | `plugins/gfx/tilemap-2d` | MOVE | mapped |
| `crates/2d/layered-image` | `amigo.gfx.layered-image-2d` | `plugins/gfx/layered-image-2d` | MOVE | mapped |
| `crates/2d/lighting` | lighting plugins | `plugins/lighting/*` | MOVE | mapped |
| `crates/2d/lighting/beacon` | `amigo.lighting.beacon-light-2d` | `plugins/lighting/beacon-light-2d` | MOVE | mapped |
| `crates/2d/particles` | `amigo.vfx.particles-2d` | `plugins/vfx/particles-2d` | MOVE | mapped |
| trail-related code | `amigo.vfx.trails-2d` | `plugins/vfx/trails-2d` | MOVE | mapped |
| `crates/2d/post-fx` | postfx plugins | `plugins/postfx/*` | MOVE | mapped |
| `crates/2d/motion` | `amigo.camera.shutter-motion` | `plugins/camera/shutter-motion` and `plugins/vfx/*` | MOVE | mapped |
| `crates/2d/depth-map` | `amigo.camera.focus-depth` | `plugins/camera/focus-depth` | MOVE | mapped |
| `crates/engine/render-api/src/material2d.rs` | `amigo.materials.material-2d` | `plugins/materials/material-2d/src/api` | MOVE | mapped |
| `crates/engine/render-api/src/light_source_2d.rs` | lighting plugins | `plugins/lighting/*/src/api` | MOVE | mapped |
