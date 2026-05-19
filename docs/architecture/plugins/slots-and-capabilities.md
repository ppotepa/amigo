# Slots and Capabilities

Capabilities describe what a plugin provides or requires.

Slots describe replaceable system positions.

## Examples

```txt
camera.frame_provider.2d
camera.focus_model.2d
camera.optics.consumer.2d
camera.shutter_model.2d
camera.film_model.2d

render.backend
scene.component_hydrator
scripting.binding_provider
diagnostics.provider
editor.panel
codemap.index_provider
```

## Rules

* Plugins depend on slots/capabilities, not concrete implementations.
* A custom plugin may replace a default plugin only by implementing the same slot contract.
* Required slots must fail clearly when missing.
* Optional slots must declare fallback behavior.
* No plugin may import another plugin's internal implementation to replace it.
