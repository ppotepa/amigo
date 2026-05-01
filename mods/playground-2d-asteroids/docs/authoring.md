# Asteroids Authoring Notes

Asteroids now uses the layered authoring model:

- `InputActionMap` owns raw key bindings.
- `Behavior` owns reusable controller glue.
- `EventPipeline` owns simple scene transitions and state writes.
- Rhai remains for game-specific rules: waves, scoring, splitting asteroids, lives, highscores.

## Scene Flow

Main menu and options navigation use `menu_navigation_controller`.

```yaml
- type: Behavior
  kind: menu_navigation_controller
  index_state: menu_index
  item_count: 4
  up_action: menu.up
  down_action: menu.down
  confirm_action: menu.confirm
  selected_color_prefix: menu_color
  confirm_events:
    - asteroids.menu.start
    - asteroids.menu.options
    - asteroids.menu.highscores
    - asteroids.menu.quit
```

The emitted topics are handled by `EventPipeline` where no custom logic is required.

```yaml
- type: EventPipeline
  id: start-game
  topic: asteroids.menu.start
  steps:
    - kind: transition_scene
      scene: game
```

Options keeps a small `on_event` only for custom session logic:

```rhai
fn on_event(topic, payload) {
    if topic == "asteroids.options.toggle-low" {
        world.session.set_bool("asteroids.low_mode", !world.session.get_bool("asteroids.low_mode"));
    }
}
```

## Gameplay Controllers

The ship uses behavior-driven freeflight and firing:

```yaml
- type: Behavior
  enabled_when:
    state: game_mode
    equals: playing
  kind: freeflight_input_controller
  target: playground-2d-asteroids-ship
  input:
    thrust: ship.thrust
    turn: ship.turn

- type: Behavior
  enabled_when:
    state: game_mode
    equals: playing
  kind: projectile_fire_controller
  emitter: playground-2d-asteroids-ship
  source: playground-2d-asteroids-ship
  action: ship.fire
  cooldown: 0.16
```

Rhai should not reimplement these controllers. Keep Rhai focused on Asteroids-specific game rules.
