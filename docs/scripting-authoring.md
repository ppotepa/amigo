# Scripting Authoring

Amigo scripting is now layered. Prefer the highest layer that fits the scene:

1. YAML components for reusable engine behavior.
2. Rhai packages for compact glue code.
3. Raw `world.*` APIs for custom logic.
4. Rust systems/bindings for reusable engine capabilities.

Raw `world.*` APIs remain available, but new scenes should not duplicate input, motion, UI, and particle plumbing when a declarative component already exists.

## Script Packages

Engine packages live in `assets/scripts/amigo/packages`. Mod-local packages live in `mods/<mod-id>/scripts/packages`.

```rhai
import "pkg:amigo.std/input" as input;
import "pkg:amigo.std/ui" as ui;
import "pkg:amigo.std/particles" as particles;

fn on_enter() {
    ui::theme(world, "space_dark");
}

fn update(dt) {
    if input::pressed(world, "ui.back") {
        world.scene.select("main-menu");
    }
}
```

Supported import forms:

```rhai
import "pkg:amigo.std/input" as input;
import "mod:editor_ext/custom_presets" as custom_presets;
import "./local_helpers.rhai" as local;
```

Package and relative imports are resolved inside the configured engine scripts root or the current mod root. Relative imports must not escape the mod root.

## InputActionMap

Use action IDs instead of raw key checks in gameplay scripts.

```yaml
- type: InputActionMap
  id: gameplay
  active: true
  actions:
    ship.thrust:
      kind: axis
      positive: [ArrowUp, KeyW]
      negative: [ArrowDown, KeyS]
    ship.turn:
      kind: axis
      positive: [ArrowLeft, KeyA]
      negative: [ArrowRight, KeyD]
    ship.fire:
      kind: button
      pressed: [Space]
```

Rhai usage:

```rhai
let thrust = input::axis(world, "ship.thrust");
let fire = input::pressed(world, "ship.fire");
```

Raw `world.input.*` is still available for low-level cases.

## Behavior

Use behaviors for standard controller glue that does not need scene-specific script logic.

```yaml
- type: Behavior
  enabled_when:
    state: game_mode
    equals: playing
  kind: freeflight_input_controller
  target: ship
  input:
    thrust: ship.thrust
    turn: ship.turn
  particles:
    thruster: ship-main-thruster
```

Projectile firing can also be YAML-driven:

```yaml
- type: Behavior
  enabled_when:
    state: game_mode
    equals: playing
  kind: projectile_fire_controller
  emitter: ship
  source: ship
  action: ship.fire
  cooldown: 0.16
  cooldown_id: ship.fire.cooldown
  audio: my-mod/audio/shot
```

Theme switching can be declarative:

```yaml
- type: Behavior
  kind: ui_theme_switcher
  bindings:
    ui.theme.space_dark: space_dark
    ui.theme.clean_dev: clean_dev
  cycle: ui.theme.cycle
```

State changes can be action-driven without Rhai:

```yaml
- type: Behavior
  kind: set_state_on_action_controller
  action: editor.open_color
  key: editor.panel
  value: color

- type: Behavior
  kind: toggle_state_controller
  action: debug.toggle
  key: debug_visible
  default: false
```

Camera follow presets can be switched by action:

```yaml
- type: Behavior
  kind: camera_follow_mode_controller
  camera: camera
  action: camera.fast
  target: ship
  lerp: 0.12
  lookahead_velocity_scale: 0.35
  lookahead_max_distance: 180.0
  sway_amount: 18.0
  sway_frequency: 1.4
```

Scene transitions can be declarative:

```yaml
- type: Behavior
  kind: scene_transition_controller
  action: menu.open_editor
  scene: editor

- type: Behavior
  kind: scene_back_controller
  action: ui.back
  scene: menu

- type: Behavior
  kind: scene_auto_transition_controller
  scene: main-menu
```

`enabled_when` accepts string, boolean, and numeric checks. String checks are useful for modes:

```yaml
enabled_when:
  state: game_mode
  not_equals: paused
```

Boolean checks avoid spelling `"true"`/`"false"` in YAML:

```yaml
enabled_when:
  state: debug_visible
  is_true: true
```

Numeric checks are useful for threshold-driven controller variants:

```yaml
enabled_when:
  state: boost_charge
  greater_or_equal: 0.5
  less_than: 1.0
```

Menu navigation can also be declarative. The controller owns index changes, move/select audio, selected item color state, and emits one topic per selected row on confirm:

```yaml
- type: Behavior
  kind: menu_navigation_controller
  index_state: menu_index
  item_count: 4
  # Optional alternative for dynamic menus:
  # item_count_state: menu_item_count
  up_action: menu.up
  down_action: menu.down
  confirm_action: menu.confirm
  move_audio: my-mod/audio/menu-move
  confirm_audio: my-mod/audio/menu-select
  selected_color_prefix: menu_color
  confirm_events:
    - menu.start
    - menu.options
    - menu.highscores
    - menu.quit
```

Bind `menu_color.0`, `menu_color.1`, etc. to text color through `UiModelBindings`:

```yaml
- type: UiModelBindings
  bindings:
    - path: my-menu.root.start
      state: menu_color.0
      kind: color
    - path: my-menu.root.options
      state: menu_color.1
      kind: color
```

## UiModelBindings

Use UI model bindings to remove repetitive `sync_ui()` code. The binding system copies values from `world.state` to UI state each frame.

```yaml
- type: UiModelBindings
  bindings:
    - path: particle-editor.root.emission.spawn-rate-value
      state: editor.spawn_rate_label
      kind: text
    - path: particle-editor.root.emission.spawn-rate-slider
      state: editor.spawn_rate_normalized
      kind: value
    - path: particle-editor.root.color.active-swatch
      state: editor.color_hex
      kind: background
    - path: particle-editor.root.debug-panel
      state: editor.debug_visible
      kind: visible
```

Rhai should update domain state and engine systems, not push every label manually:

```rhai
fn on_spawn_rate_changed(value) {
    let spawn_rate = value * 240.0;
    world.state.set_float("editor.spawn_rate", spawn_rate);
    world.state.set_string("editor.spawn_rate_label", spawn_rate.to_string());
    world.state.set_float("editor.spawn_rate_normalized", value);
    world.particles.set_spawn_rate("preview-emitter", spawn_rate);
}
```

## EventPipeline

Use event pipelines for simple reactions to UI or script events.

```yaml
- type: EventPipeline
  id: back-to-menu
  topic: playground-2d-particles.menu
  steps:
    - kind: transition_scene
      scene: menu
```

Other supported steps:

```yaml
- kind: set_state
  key: ui.panel
  value: settings
- kind: increment_state
  key: score
  by: 100
- kind: show_ui
  path: hud.root.toast
- kind: hide_ui
  path: hud.root.toast
- kind: burst_particles
  emitter: explosion-emitter
  count: 48
- kind: emit_event
  topic: custom.event
  payload: [value]
- kind: script
  function: on_custom_pipeline_step
```

`script` calls the named function on active scripts with `(topic, payload)`. Missing functions are ignored, so it can be used as a narrow custom fallback without routing everything through `on_event`.

Keep `on_event` for broad custom gameplay branches that cannot be expressed clearly as data.

## ScriptComponent

Use `ScriptComponent` for reusable scripts attached to entities.

```yaml
- type: ScriptComponent
  script: scripts/components/lifecycle_probe.rhai
  params:
    label: square-probe
    speed: 1.0
```

Component script lifecycle:

```rhai
fn on_attach(entity, params) {
    world.state.set_string(entity + ".component.label", params.label);
}

fn update(entity, params, dt) {
    let key = entity + ".component.elapsed";
    world.state.set_float(key, world.state.get_float(key) + dt * params.speed);
}

fn on_detach(entity, params) {
    world.dev.event("component.detached", entity);
}
```

Errors include the entity name and component script path to make scene diagnostics actionable.

## Manual vs Package vs Behavior

Manual:

```rhai
fn update(dt) {
    let thrust = if world.input.down("ArrowUp") { 1.0 } else { 0.0 };
    world.motion.drive_freeflight("ship", #{ thrust: thrust, strafe: 0.0, turn: 0.0 });
    world.particles.set_intensity("ship-main-thruster", thrust);
}
```

Package-based:

```rhai
import "pkg:amigo.arcade_2d/freeflight" as freeflight;

fn update(dt) {
    freeflight::drive_ship_with_thruster(
        world,
        "ship",
        "ship-main-thruster",
        "ship.thrust",
        "ship.turn"
    );
}
```

Behavior-based:

```yaml
- type: Behavior
  kind: freeflight_input_controller
  target: ship
  input:
    thrust: ship.thrust
    turn: ship.turn
  particles:
    thruster: ship-main-thruster
```

Prefer behavior-based authoring when the logic is generic and reusable.
