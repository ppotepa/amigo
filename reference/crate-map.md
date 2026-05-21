# Crate map

| Doc | Path | Package | Layer | Role |
|---|---|---|---|---|
| [`2d-composition`](../crates/2d-composition.md) | `crates/2d/composition` | `amigo-2d-composition` | 2D domain support | 2D composition/material/spatial/physics support used by plugins/runtime. |
| [`2d-material-api`](../crates/2d-material-api.md) | `crates/2d/material-api` | `amigo-material-api` | 2D domain support | 2D composition/material/spatial/physics support used by plugins/runtime. |
| [`2d-physics`](../crates/2d-physics.md) | `crates/2d/physics` | `amigo-2d-physics` | 2D domain support | 2D composition/material/spatial/physics support used by plugins/runtime. |
| [`2d-spatial`](../crates/2d-spatial.md) | `crates/2d/spatial` | `amigo-2d-spatial` | 2D domain support | 2D composition/material/spatial/physics support used by plugins/runtime. |
| [`3d-material`](../crates/3d-material.md) | `crates/3d/material` | `amigo-3d-material` | 3D domain support | 3D mesh/text/material support. Keep independent from 2D renderer hacks. |
| [`3d-mesh`](../crates/3d-mesh.md) | `crates/3d/mesh` | `amigo-3d-mesh` | 3D domain support | 3D mesh/text/material support. Keep independent from 2D renderer hacks. |
| [`3d-text`](../crates/3d-text.md) | `crates/3d/text` | `amigo-3d-text` | 3D domain support | 3D mesh/text/material support. Keep independent from 2D renderer hacks. |
| [`apps-app`](../crates/apps-app.md) | `crates/apps/app` | `amigo-app` | application host | Thin runtime host, bootstrap seam, platform/window orchestration, and presentation entrypoint. |
| [`apps-launcher`](../crates/apps-launcher.md) | `crates/apps/launcher` | `amigo-launcher` | launcher | Launcher/profile selection surface. Should choose runtime presets, not own domain behavior. |
| [`audio-api`](../crates/audio-api.md) | `crates/audio/api` | `amigo-audio-api` | audio subsystem | Audio API/generated assets/mixer/output layering. |
| [`audio-generated`](../crates/audio-generated.md) | `crates/audio/generated` | `amigo-audio-generated` | audio subsystem | Audio API/generated assets/mixer/output layering. |
| [`audio-mixer`](../crates/audio-mixer.md) | `crates/audio/mixer` | `amigo-audio-mixer` | audio subsystem | Audio API/generated assets/mixer/output layering. |
| [`audio-output`](../crates/audio-output.md) | `crates/audio/output` | `amigo-audio-output` | audio subsystem | Audio API/generated assets/mixer/output layering. |
| [`core-codemap-api`](../crates/core-codemap-api.md) | `crates/core/codemap-api` | `amigo-codemap-api` | codemap API | Codemap/navigation contracts used by tools and agents. |
| [`core-plugin-api`](../crates/core-plugin-api.md) | `crates/core/plugin-api` | `amigo-plugin-api` | plugin infrastructure | Plugin API, manifests, loader/index mechanics, and plugin metadata contracts. |
| [`core-plugin-index`](../crates/core-plugin-index.md) | `crates/core/plugin-index` | `amigo-plugin-index` | plugin infrastructure | Plugin API, manifests, loader/index mechanics, and plugin metadata contracts. |
| [`core-plugin-loader`](../crates/core-plugin-loader.md) | `crates/core/plugin-loader` | `amigo-plugin-loader` | plugin infrastructure | Plugin API, manifests, loader/index mechanics, and plugin metadata contracts. |
| [`core-plugin-manifest`](../crates/core-plugin-manifest.md) | `crates/core/plugin-manifest` | `amigo-plugin-manifest` | plugin infrastructure | Plugin API, manifests, loader/index mechanics, and plugin metadata contracts. |
| [`engine-assets`](../crates/engine-assets.md) | `crates/engine/assets` | `amigo-assets` | crate | Project crate. Confirm exact ownership using Cargo.toml, README, and codemap before modifying. |
| [`engine-camera`](../crates/engine-camera.md) | `crates/engine/camera` | `amigo-camera` | camera contracts | Shared camera data/contracts used by domain plugins and render extraction. |
| [`engine-capabilities`](../crates/engine-capabilities.md) | `crates/engine/capabilities` | `amigo-capabilities` | crate | Project crate. Confirm exact ownership using Cargo.toml, README, and codemap before modifying. |
| [`engine-devtools`](../crates/engine-devtools.md) | `crates/engine/devtools` | `amigo-devtools` | developer tooling runtime | Console, diagnostics, overlay, debug commands, and in-runtime developer surfaces. |
| [`engine-editor-api`](../crates/engine-editor-api.md) | `crates/engine/editor-api` | `amigo-editor-api` | editor subsystem | Editor contracts/sessions/authoring/runtime ingame overlay depending on suffix. |
| [`engine-editor-authoring`](../crates/engine-editor-authoring.md) | `crates/engine/editor-authoring` | `amigo-editor-authoring` | editor subsystem | Editor contracts/sessions/authoring/runtime ingame overlay depending on suffix. |
| [`engine-editor-ingame`](../crates/engine-editor-ingame.md) | `crates/engine/editor-ingame` | `amigo-editor-ingame` | editor subsystem | Editor contracts/sessions/authoring/runtime ingame overlay depending on suffix. |
| [`engine-editor-session`](../crates/engine-editor-session.md) | `crates/engine/editor-session` | `amigo-editor-session` | editor subsystem | Editor contracts/sessions/authoring/runtime ingame overlay depending on suffix. |
| [`engine-event-pipeline`](../crates/engine-event-pipeline.md) | `crates/engine/event-pipeline` | `amigo-event-pipeline` | crate | Project crate. Confirm exact ownership using Cargo.toml, README, and codemap before modifying. |
| [`engine-font`](../crates/engine-font.md) | `crates/engine/font` | `amigo-font` | crate | Project crate. Confirm exact ownership using Cargo.toml, README, and codemap before modifying. |
| [`engine-hot-reload`](../crates/engine-hot-reload.md) | `crates/engine/hot-reload` | `amigo-hot-reload` | crate | Project crate. Confirm exact ownership using Cargo.toml, README, and codemap before modifying. |
| [`engine-input-actions`](../crates/engine-input-actions.md) | `crates/engine/input-actions` | `amigo-input-actions` | crate | Project crate. Confirm exact ownership using Cargo.toml, README, and codemap before modifying. |
| [`engine-modding`](../crates/engine-modding.md) | `crates/engine/modding` | `amigo-modding` | crate | Project crate. Confirm exact ownership using Cargo.toml, README, and codemap before modifying. |
| [`engine-overlay-api`](../crates/engine-overlay-api.md) | `crates/engine/overlay-api` | `amigo-overlay-api` | crate | Project crate. Confirm exact ownership using Cargo.toml, README, and codemap before modifying. |
| [`engine-render-api`](../crates/engine-render-api.md) | `crates/engine/render-api` | `amigo-render-api` | render contract layer | Renderer-facing contracts: frame packets, graph models, targets, camera capture inputs, PostFX models. |
| [`engine-render-wgpu`](../crates/engine-render-wgpu.md) | `crates/engine/render-wgpu` | `amigo-render-wgpu` | WGPU backend | Concrete backend implementation for render-api contracts, pipelines, frame graph, resources, and passes. |
| [`engine-runtime`](../crates/engine-runtime.md) | `crates/engine/runtime` | `amigo-runtime` | runtime core | Plugin/system runtime contracts, service registration, phases, and scheduling interfaces. |
| [`engine-runtime-control`](../crates/engine-runtime-control.md) | `crates/engine/runtime-control` | `amigo-runtime-control` | runtime core | Plugin/system runtime contracts, service registration, phases, and scheduling interfaces. |
| [`engine-scene`](../crates/engine-scene.md) | `crates/engine/scene` | `amigo-scene` | scene core | Scene document model, hydration, commands, metadata, and validation. Should not own WGPU behavior. |
| [`engine-session`](../crates/engine-session.md) | `crates/engine/session` | `amigo-session` | session/frame services | Frame/session lifecycle, timing, runtime session services, and frame-scoped state. |
| [`engine-state`](../crates/engine-state.md) | `crates/engine/state` | `amigo-state` | crate | Project crate. Confirm exact ownership using Cargo.toml, README, and codemap before modifying. |
| [`foundation-core`](../crates/foundation-core.md) | `crates/foundation/core` | `amigo-core` | foundation utility | Low-level reusable utility/math/fx primitives. Keep dependency direction downward only. |
| [`foundation-fx`](../crates/foundation-fx.md) | `crates/foundation/fx` | `amigo-fx` | foundation utility | Low-level reusable utility/math/fx primitives. Keep dependency direction downward only. |
| [`foundation-math`](../crates/foundation-math.md) | `crates/foundation/math` | `amigo-math` | foundation utility | Low-level reusable utility/math/fx primitives. Keep dependency direction downward only. |
| [`platform-app-host-api`](../crates/platform-app-host-api.md) | `crates/platform/app-host-api` | `amigo-app-host-api` | platform adapter | Platform abstractions and concrete winit/notify/input implementations. |
| [`platform-app-host-winit`](../crates/platform-app-host-winit.md) | `crates/platform/app-host-winit` | `amigo-app-host-winit` | platform adapter | Platform abstractions and concrete winit/notify/input implementations. |
| [`platform-file-watch-api`](../crates/platform-file-watch-api.md) | `crates/platform/file-watch-api` | `amigo-file-watch-api` | platform adapter | Platform abstractions and concrete winit/notify/input implementations. |
| [`platform-file-watch-notify`](../crates/platform-file-watch-notify.md) | `crates/platform/file-watch-notify` | `amigo-file-watch-notify` | platform adapter | Platform abstractions and concrete winit/notify/input implementations. |
| [`platform-input-api`](../crates/platform-input-api.md) | `crates/platform/input-api` | `amigo-input-api` | platform adapter | Platform abstractions and concrete winit/notify/input implementations. |
| [`platform-input-winit`](../crates/platform-input-winit.md) | `crates/platform/input-winit` | `amigo-input-winit` | platform adapter | Platform abstractions and concrete winit/notify/input implementations. |
| [`platform-window-api`](../crates/platform-window-api.md) | `crates/platform/window-api` | `amigo-window-api` | platform adapter | Platform abstractions and concrete winit/notify/input implementations. |
| [`platform-window-winit`](../crates/platform-window-winit.md) | `crates/platform/window-winit` | `amigo-window-winit` | platform adapter | Platform abstractions and concrete winit/notify/input implementations. |
| [`runtime-bundles`](../crates/runtime-bundles.md) | `crates/runtime/bundles` | `amigo-runtime-bundles` | runtime composition | Composes runtime bundles, plugin registration, and extraction/backend bridges. |
| [`scripting-api`](../crates/scripting-api.md) | `crates/scripting/api` | `amigo-scripting-api` | scripting layer | Scripting contracts and Rhai backend integration. |
| [`scripting-rhai`](../crates/scripting-rhai.md) | `crates/scripting/rhai` | `amigo-scripting-rhai` | scripting layer | Scripting contracts and Rhai backend integration. |
| [`ui-core`](../crates/ui-core.md) | `crates/ui/core` | `amigo-ui` | UI subsystem | UI core/layout primitives and runtime-facing UI contracts. |
| [`ui-layout`](../crates/ui-layout.md) | `crates/ui/layout` | `amigo-ui-layout` | UI subsystem | UI core/layout primitives and runtime-facing UI contracts. |
