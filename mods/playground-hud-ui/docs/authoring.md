# HUD UI Authoring Notes

`playground-hud-ui` is the reference mod for screen-space UI authoring.

## Theme Switching

Theme definitions live in scene YAML through `UiThemeSet`. Runtime switching should go through `world.ui.set_theme(...)` or a `ui_theme_switcher` behavior when the mapping is static.

```yaml
- type: Behavior
  kind: ui_theme_switcher
  bindings:
    ui.theme.space_dark: space_dark
    ui.theme.clean_dev: clean_dev
  cycle: ui.theme.cycle
```

## UI State

Prefer `UiModelBindings` for labels, progress values, selected options, visibility, enabled state, and colors. Rhai should update scene state and domain systems, not push every widget manually.

```yaml
- type: UiModelBindings
  bindings:
    - path: playground-hud-ui-showcase.root.debug.theme
      state: active_theme
      kind: text
    - path: playground-hud-ui-showcase.root.controls.volume
      state: volume
      kind: value
    - path: playground-hud-ui-showcase.root
      state: active_theme
      kind: theme
```

## Control Events

Controls still emit normal script events for custom UI behavior. Use EventPipeline only when the reaction is a simple built-in step such as `set_state`, `show_ui`, `hide_ui`, or `transition_scene`.

Keep complex preview logic in Rhai until it becomes reusable enough to promote to a behavior.
