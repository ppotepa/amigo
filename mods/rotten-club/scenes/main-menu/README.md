# Main Menu Scene Layout

- `camera/` - the scene camera entity plus scene-local motion and DOF control state.
- `render/` - render layers and frame ordering.
- `world/` - background and static scene objects.
- `lighting/` - global lights, beacons, and light routing.
- `weather/` - world-space rain particles.
- `interaction/` - local control entity, action map, and menu event pipelines.
- `ui/` - mounted UI fragments for the scene.
- `input/` - input actions used by the scene.
- `events/` - scene event pipelines.
- `state/` - initial scene state.

Camera profiles remain mod-level reusable assets in `mods/rotten-club/camera/*`.
Scene-local camera entities only reference those assets.
Camera motion lives beside the scene camera in `camera/motion.yml`; Rhai only executes the authored values.
Camera DOF keyboard controls live in `camera/dof-controls.yml`: `[` / `]` focal length, `;` / `'` f-stop, `.` / `/` focus depth.
