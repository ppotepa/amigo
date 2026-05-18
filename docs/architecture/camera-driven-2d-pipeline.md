# 2D Camera-Driven Pipeline

## Overview
1. Scene, lights and visual content produce renderable visual sources.
2. `crates/2d/spatial` owns the metric authoring space used by 2D scenes.
3. `CameraCaptureInput2d` is the renderer-facing contract for scene capture inputs.
4. `ResolvedCameraRig2d` is the camera-facing contract for style, focus and quality.
5. Camera-owned post-fx implement the camera look; they are not the main authoring language.
6. Scene-scoped post-fx are local exceptions.
7. Presentation post-fx are final display effects.
8. UI and debug overlays render after game capture.

## Authoring Rules
- Author normal scene depth in meters through `visual2d.spatial.depth_space` and `depth.distance_m`.
- Author camera focus in meters through `focus_distance_m`.
- Use `z_depth` only as a low-level debug or override value.
- Do not add film grain, DOF, lens aberration or rain glass as default scene frame post-fx when they should come from the camera rig.

## Runtime Flow
`2d/spatial -> visual sources -> CameraCaptureInput2d -> ResolvedCameraRig2d -> camera-owned post-fx stack -> scoped local post-fx exceptions -> presentation/debug output`

## Smoke Tests
1. Start `rotten-club` main menu.
2. Switch camera slots `1-0`.
3. Change `focus_distance_m` live.
4. Set `camera.debug_view computed_z_depth`.
5. Set `camera.debug_view camera_after_dof`.
6. Run `postfx.diagnostics`.
7. Run `layers.list` and verify `distance_m`, `computed_z_depth` and `optical_role`.
8. Run `camera.capture` and verify the listed capture sources.
