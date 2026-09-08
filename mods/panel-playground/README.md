# Engine panel example

Run `cargo run -p amigo-app -- --hosted --mod panel-playground --scene layer`.
The external panel controls a 2D render layer through runtime-control metadata.
No NPR plugin is activated. Slider writes are typed engine commands; the reset
button emits a scene event handled by `scene.rhai`.

Close the panel without closing the scene. Reopen from the scene's Rhai context
with `world.panels.open("layer")`. Editing `ui/layer.panel.yml` reloads the layout
without resetting the layer.
