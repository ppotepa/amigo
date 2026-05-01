# Particle Playground Authoring Notes

The particle playground is split into three scenes:

- `menu`: declarative scene transitions through `EventPipeline` and `scene_transition_controller`.
- `showcase`: preset selection and particle intensity logic in Rhai, UI display through `UiModelBindings`.
- `editor`: runtime emitter mutation in Rhai, display state through `UiModelBindings`.

## Editor State Pattern

The editor script should mutate domain state, then refresh derived state keys through `sync_model_state(...)`. UI widgets consume those keys through bindings. `sync_model_state(...)` is intentionally a model refresh, not a manual UI push.

```rhai
world.state.set_float("spawn_rate", spawn_rate);
world.state.set_string("spawn_rate_label", "Spawn rate " + spawn_rate.to_int());
world.state.set_float("spawn_rate_normalized", spawn_rate / 200.0);
world.particles.set_spawn_rate("preview-emitter", spawn_rate);
```

```yaml
- type: UiModelBindings
  bindings:
    - path: playground-2d-particles-editor-ui.root.main.controls.tab-panels.emission-panel.spawn-rate
      state: spawn_rate_normalized
      kind: value
    - path: playground-2d-particles-editor-ui.root.main.controls.tab-panels.emission-panel.spawn-rate-label
      state: spawn_rate_label
      kind: text
```

## What Stays In Rhai

Keep these in Rhai because they are editor/domain logic:

- converting normalized slider values to emitter values;
- applying particle presets;
- calculating labels and derived state keys;
- exporting YAML;
- custom color ramp and curve editing behavior.

Do not manually push UI text/color/value when a `UiModelBindings` entry can consume a state key.

Dropdown option lists can be bound with `kind: options` using a pipe-separated state string. Active theme can be bound with `kind: theme`.
