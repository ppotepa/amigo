#!/usr/bin/env bash
set -euo pipefail

assert_no_match() {
  local pattern="$1"
  local path="$2"
  local message="$3"
  shift 3

  if rg -n "$@" "$pattern" "$path"; then
    echo "$message" >&2
    exit 1
  fi
}

assert_no_match \
  "\\bwgpu\\b|wgpu::|RenderPipeline|RenderPass|BindGroup|CommandEncoder" \
  "plugins" \
  "Plugins must not import WGPU." \
  --glob "src/**"

assert_no_match \
  "amigo-.*-plugin" \
  "crates/engine/render-wgpu/Cargo.toml" \
  "render-wgpu must not depend on plugin crates."

assert_no_match \
  "amigo_.*_plugin" \
  "crates/engine/render-wgpu/src" \
  "render-wgpu source must not import plugin crates."

assert_no_match \
  "Renderable2dPayload|Renderable2dPayloadKind" \
  "crates/engine/render-wgpu/src" \
  "Renderable2dPayload must not return."

assert_no_match \
  "RenderPrimitive2d::" \
  "crates/engine/render-wgpu/src/renderer/service/render/world.rs" \
  "world.rs must not branch on RenderPrimitive2d."

assert_no_match \
  "match descriptor.executor_id" \
  "crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs" \
  "PostFX registry must not central-match executor_id."

assert_no_match \
  "enum PluginSceneCommandPayload" \
  "crates/engine/scene/src" \
  "PluginSceneCommandPayload must not be a central enum."

assert_no_match \
  "CAMERA_EXPOSURE_SHADER|CAMERA_OPTICS_SHADER|FOCUS_BLUR_SHADER|FILM_EMULSION_SHADER|FILM_NOISE_SHADER|SCAN_OUTPUT_SHADER" \
  "crates/engine/render-wgpu/src/renderer/service/init.rs" \
  "init.rs must not keep pilot post-fx shader sources."

assert_no_match \
  "execute_layered_image_parts_to_offscreen|layered_textured_quads\(" \
  "crates/engine/render-wgpu/src/renderer/service/render/world.rs" \
  "world.rs must not contain layered image part pass logic."

assert_no_match \
  "const COLOR_SHADER|const TEXTURE_SHADER" \
  "crates/engine/render-wgpu/src/renderer.rs" \
  "renderer.rs must not keep core shader source constants."

assert_no_match \
  "color_alpha_pipeline|color_additive_pipeline|color_multiply_pipeline|color_screen_pipeline|texture_alpha_pipeline|texture_opaque_pipeline|texture_additive_pipeline|texture_multiply_pipeline|texture_screen_pipeline|texture_lighten_pipeline" \
  "crates/engine/render-wgpu/src/renderer/service/model.rs" \
  "WgpuSceneRenderer must not own dedicated core pipeline fields."

assert_no_match \
  "create_color_pipeline|COLOR_SHADER|TEXTURE_SHADER" \
  "crates/engine/render-wgpu/src/renderer/service/init.rs" \
  "init.rs must not manually create core color or texture pipelines."

echo "Engine boundary checks passed."
