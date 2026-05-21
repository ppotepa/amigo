# Useful rg queries

## PostFX

```powershell
rg -n "PostFx2d|execute_screen_space_post_fx|post_fx" crates plugins
rg -n "CameraOptics|FocusBlur|RainGlass|FilmEmulsion|ShutterBlur" crates/engine/render-wgpu crates/engine/render-api plugins
rg -n "is_cached_image_compatible|apply_cached_image_post_fx_rgba|layered_image_layer_render_size" crates/engine/render-wgpu crates/engine/render-api plugins
```

## Camera optics

```powershell
rg -n "CameraOpticalCandidate2d|CameraOpticalCoverage2d|CameraOpticalResponse2d" crates plugins
rg -n "SceneHighlight|SceneEmissive|LightMapChannel|CameraOpticalRenderTargetPlan" crates plugins mods
rg -n "camera_response|render_contributions|camera.fx_source|bloom.source" crates plugins mods
```

## Scene metadata

```powershell
rg -n "component_metadata|ComponentDescriptor|ComponentDescriptorProvider" crates/engine/scene crates plugins
rg -n "SceneCommand|Command|hydrate|hydration" crates/engine/scene crates/runtime plugins
```

## App-centric regressions

```powershell
rg -n "camera_optics|lighting|material|postfx|render_contributions" crates/apps/app
rg -n "apps/app" crates/runtime crates/engine plugins
```
