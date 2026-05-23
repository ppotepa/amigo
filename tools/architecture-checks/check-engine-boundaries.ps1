$ErrorActionPreference = "Stop"

function Assert-NoMatch {
    param(
        [string]$Pattern,
        [string]$Path,
        [string]$Message
    )

    $matches = rg -n --glob "src/**" $Pattern $Path
    if ($LASTEXITCODE -eq 0) {
        Write-Host $matches
        throw $Message
    }
}

Assert-NoMatch `
    "\bwgpu\b|wgpu::|RenderPipeline|RenderPass|BindGroup|CommandEncoder" `
    "plugins" `
    "Plugins must not import WGPU."

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

Write-Host "Engine boundary checks passed."
