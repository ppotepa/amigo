# Rotten Club

Clean-room Rotten Club main menu built on the current plugin-owned scene architecture.

This mod keeps the old main-menu spaghetti out of the runtime path:

- one small Rhai lifecycle director
- authored timeline tracks in `scenes/main-menu/timeline/intro.yml`
- scene fragments grouped by `camera`, `world`, `lighting`, `weather`, and `state`
- a minimal event hook for reload/debug messages
- explicit `visual2d.render_layers` for depth, optical roles, and title fade
- explicit `visual2d.light_groups` for neon, bar, skyline, and lightning optical response
- two lightweight particle rain emitters driven by timeline intensity
- intro work stops after `intro.complete` so the menu idles cheaply
- no cached game-frame presentation

The scene is built from plugin-owned scene components:

- `amigo.camera.camera-core.Camera2D`
- `amigo.camera.focus-depth.DepthMap2D`
- `amigo.gfx.layered-image-2d.LayeredImage2D`
- `amigo.gfx.text-2d.Text2D`
- `amigo.lighting.light-2d.GlobalLight2D`
- `amigo.lighting.beacon-light-2d.BeaconLight2D`
- `amigo.vfx.particles-2d.ParticleEmitter2D`

The timeline applies supported `RuntimeControlService` paths for camera focus,
layer opacity, rain intensity, beacon intensity, light-group intensity, and
state values. Rhai no longer performs per-frame animation math.

Use `rotten-club-dev` for hosted dev checks and `rotten-club-release` for release performance checks.

The intro beat sheet is documented in `docs/timeline.md`.
