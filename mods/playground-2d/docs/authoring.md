# Playground 2D Authoring Notes

`playground-2d` is the small reference mod for mixed authoring:

- Use `InputActionMap` for all keyboard bindings.
- Use `pkg:amigo.std/input` in Rhai instead of raw key strings.
- Use `Behavior` for generic scene flow.
- Use `ScriptComponent` when reusable entity-local script logic is clearer than a global scene script.

## Package Imports

Scene scripts import the standard input package at top level. The alias is available inside lifecycle callbacks:

```rhai
import "pkg:amigo.std/input" as input;

fn update(dt) {
    if input::down(world, "square.move_left") {
        // custom scene logic
    }
}
```

## ScriptComponent Example

`basic-scripting-demo` attaches `scripts/components/lifecycle_probe.rhai` to the demo square:

```yaml
- type: ScriptComponent
  script: scripts/components/lifecycle_probe.rhai
  params:
    label: square-probe
    speed: 1.0
```

The component owns attach/update/detach logic for that entity. Runtime diagnostics include the entity name, script path, source name, and lifecycle phase when a component fails.

## Scene Flow

Simple input-driven transitions should be declarative:

```yaml
- type: Behavior
  kind: scene_transition_controller
  action: demo.goto_square
  scene: hello-world-square
```

Keep Rhai for custom per-frame examples and engine API demos.
