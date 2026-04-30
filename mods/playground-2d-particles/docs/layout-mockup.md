# Playground 2D Particles Layout Mockup

Target viewport: `1440x900`.

SVG version: [`layout-mockup.svg`](layout-mockup.svg).

Reason: `1280x720` is too tight for a readable particle preview plus controls. The playground is a dev tool, so the layout should prioritize clarity over compactness.

## Shared Layout Rules

- Header is placed at `left=24 top=18 width=1392 height=72`.
- Footer is placed at `left=24 top=834 width=1392 height=48`.
- Body starts at `top=108` and uses explicit left spacer + right panel layout.
- World preview is never covered by opaque UI.
- UI panels use transparent root and only draw contained panels.
- UI documents use a fixed design viewport with `scaling: fit`, so maximize/resizing does not create a larger UI workspace.
- Primary preview area is a `520x520` world-space grid.
- Grid center is always the active emitter origin.
- Controls are grouped into clear panels, not scattered.

## Showcase Screen

Purpose: quickly compare particle presets.

```text
--------------------------------------------------------------------------------+
| HEADER 72                                                                      |
| PARTICLE SHOWCASE                  Selected: fire                  [Menu]      |
+-----------------------------+--------------------------------------------------+
| BODY 780                    |                                                  |
|                             | Right Side Panel 360x640                         |
|      World Preview          | +----------------------------------------------+ |
|      520x520 grid           | | Presets                                      | |
|      centered left          | | [fire smoke sparks magic]                    | |
|                             | | [snow dust thruster]                         | |
|                             | |                                              | |
|                             | | Intensity                                    | |
|                             | | [slider] 100%                                | |
|                             | |                                              | |
|                             | | Current preset details                       | |
|                             | | Active / shape / lifetime notes              | |
|                             | |                                              | |
|                             | | [Start] [Stop] [Reset]                       | |
|                             | +----------------------------------------------+ |
+--------------------------------------------------------------------------------+
| FOOTER 48                                                                      |
| Keys: 1-7 preset | Space toggle | R reset | Esc menu                           |
+--------------------------------------------------------------------------------+
```

World positions:

```text
preview center: x=-260, y=0
preview bounds: x=-520..0, y=-260..260
emitter origin: x=-260, y=0
```

UI positions:

```text
root:     1440x900 transparent
viewport: width=1440 height=900 scaling=fit
header:   left=24 top=18 width=1392 height=72
side:     left=930 top=108 width=450 height=640
footer:   left=24 top=834 width=1392 height=48
```

## Editor Screen

Purpose: edit one emitter with enough room for readable controls.

```text
+--------------------------------------------------------------------------------+
| HEADER 72                                                                      |
| PARTICLE EDITOR V0                 Editing preview-emitter          [Menu]      |
+-----------------------------+--------------------------------------------------+
| BODY 780                    |                                                  |
|                             | Right Inspector 520x720                         |
|      World Preview          | +----------------------------------------------+ |
|      520x520 grid           | | State                                        | |
|      centered left          | | [x] Emitter active                           | |
|                             | |                                              | |
|                             | | Emission                                     | |
|                             | | Spawn Rate   [slider] value                  | |
|                             | | Lifetime     [slider] value                  | |
|                             | | Speed        [slider] value                  | |
|                             | | Spread       [slider] value                  | |
|                             | |                                              | |
|                             | | Size                                         | |
|                             | | Initial      [slider]                        | |
|                             | | Final        [slider]                        | |
|                             | |                                              | |
|                             | | Color                                        | |
|                             | | [dropdown] [swatch]                          | |
|                             | |                                              | |
|                             | | [Reset] [Print YAML]                         | |
|                             | +----------------------------------------------+ |
+--------------------------------------------------------------------------------+
| FOOTER 48                                                                      |
| Sliders mutate ParticleEmitter2D runtime params | Esc menu                      |
+--------------------------------------------------------------------------------+
```

World positions:

```text
preview center: x=-330, y=0
preview bounds: x=-590..-70, y=-260..260
emitter origin: x=-330, y=0
```

UI positions:

```text
root:      1440x900 transparent
viewport:  width=1440 height=900 scaling=fit
header:    left=24 top=18 width=1392 height=72
inspector: left=790 top=108 width=590 height=700
footer:    left=24 top=834 width=1392 height=48
```

## Follow-up UI Features

- `GridPanel` or reusable debug grid component for world/preview panels.
- `LabelSlider` composite control to avoid repeating slider + label rows.
- Scroll container for inspectors once controls exceed viewport.
