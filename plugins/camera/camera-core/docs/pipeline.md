# Camera Core Pipeline

Camera Core owns the shared 2D camera frame contract. It registers `CameraService`,
focus target storage, follow/parallax scene services, scene hydrators, and runtime
scene command handlers.

## Flow
- Authored `Camera2D`, camera follow, parallax, and focus target data is hydrated
  into camera scene commands.
- Runtime systems update follow, parallax, and focus transitions before the frame
  context is read by downstream camera plugins.
- The plugin provides `camera.frame_context.2d@1` and implements
  `camera.frame_provider.2d`.

## Boundaries
- It does not render images or write render targets.
- It does not infer optics, focus blur, or shutter behavior from scene objects.
- Downstream plugins consume explicit camera frame state instead of reaching into
  app or renderer setup.
