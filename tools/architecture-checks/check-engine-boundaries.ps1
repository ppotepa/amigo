$ErrorActionPreference = "Stop"

function Assert-NoMatch {
    param(
        [string]$Pattern,
        [string]$Path,
        [string]$Message,
        [string[]]$ExtraArgs = @()
    )

    $matches = rg -n @ExtraArgs $Pattern $Path
    if ($LASTEXITCODE -eq 0) {
        Write-Host $matches
        throw $Message
    }
}

Assert-NoMatch `
    "\bwgpu\b|wgpu::|RenderPipeline|RenderPass|BindGroup|CommandEncoder" `
    "plugins" `
    "Plugins must not import WGPU." `
    @("--glob", "src/**")

Assert-NoMatch `
    "amigo-.*-plugin" `
    "crates/engine/render-wgpu/Cargo.toml" `
    "render-wgpu must not depend on plugin crates."

Assert-NoMatch `
    "amigo_.*_plugin" `
    "crates/engine/render-wgpu/src" `
    "render-wgpu source must not import plugin crates."

Assert-NoMatch `
    "Renderable2dPayload|Renderable2dPayloadKind" `
    "crates/engine/render-wgpu/src" `
    "Renderable2dPayload must not return."

Assert-NoMatch `
    "RenderPrimitive2d::" `
    "crates/engine/render-wgpu/src/renderer/service/render/world.rs" `
    "world.rs must not branch on RenderPrimitive2d."

Assert-NoMatch `
    "match descriptor.executor_id" `
    "crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs" `
    "PostFX registry must not central-match executor_id."

Assert-NoMatch `
    "enum PluginSceneCommandPayload" `
    "crates/engine/scene/src" `
    "PluginSceneCommandPayload must not be a central enum."

Assert-NoMatch `
    "CAMERA_EXPOSURE_SHADER|CAMERA_OPTICS_SHADER|FOCUS_BLUR_SHADER|FILM_EMULSION_SHADER|FILM_NOISE_SHADER|SCAN_OUTPUT_SHADER" `
    "crates/engine/render-wgpu/src/renderer/service/init.rs" `
    "init.rs must not keep pilot post-fx shader sources."

Assert-NoMatch `
    "execute_layered_image_parts_to_offscreen|layered_textured_quads\(" `
    "crates/engine/render-wgpu/src/renderer/service/render/world.rs" `
    "world.rs must not contain layered image part pass logic."

Assert-NoMatch `
    "const COLOR_SHADER|const TEXTURE_SHADER" `
    "crates/engine/render-wgpu/src/renderer.rs" `
    "renderer.rs must not keep core shader source constants."

Assert-NoMatch `
    "color_alpha_pipeline|color_additive_pipeline|color_multiply_pipeline|color_screen_pipeline|texture_alpha_pipeline|texture_opaque_pipeline|texture_additive_pipeline|texture_multiply_pipeline|texture_screen_pipeline|texture_lighten_pipeline" `
    "crates/engine/render-wgpu/src/renderer/service/model.rs" `
    "WgpuSceneRenderer must not own dedicated core pipeline fields."

Assert-NoMatch `
    "create_color_pipeline|COLOR_SHADER|TEXTURE_SHADER" `
    "crates/engine/render-wgpu/src/renderer/service/init.rs" `
    "init.rs must not manually create core color or texture pipelines."

Write-Host "Engine boundary checks passed."
