# NprPlayground

Opt-in NPR drawing with a Tauri/Svelte companion and embedded WGPU viewport.
The initial companion is Basic Mode: choose one model, choose one complete Look
and save the resulting drawing. It intentionally does not expose layers,
geometry sources, masks, blend modes or individual brush editing.

Run `npm ci` in `plugins/gfx/npr-playground/playground-client`, then from
the repository root:
`rtk cargo run --profile playground -p amigo-app -- --hosted --mod npr-playground --scene gallery`.

The viewport uses Native GPU when available and fails over to JPEG if required.
Windows Native GPU uses a DX12 worker and a native child surface; Local RGBA
remains a transport implementation detail rather than a Basic Mode control.
Scenes without a companion use the ordinary Winit host and its own backend.

See [client and authoring](docs/npr-playground-ui.md), [pipeline](docs/pipeline.md),
[contributions](docs/contributions.md) and [diagnostics](docs/diagnostics.md).
Measured results and remaining limits: [viewport validation](docs/viewport-validation.md).

Selecting another model saves a changed drawing as a local, model-bound draft
and opens a clean drawing. Save writes the authored scene sidecar. Look cards
use asynchronous renderer-generated previews, never decorative HTML strokes.
The current curated choices are Comic Ink, Pencil Study and Watercolour Wash.

The NPR domain owns surface interpretation, typed ComicInk and layer contracts,
feature extraction and gesture policy. HardSurface and Organic are explicit
authored intents. Smooth uses a prepared welded proxy; crease, contour cleanup
and suggestive-contour thresholds remain domain policy. Paper is scene-owned.
One authored `layers` list preserves stable IDs and explicit order across paint
and strokes. Existing split documents require the explicit
[layer-stack migration](docs/npr-playground-ui.md#layer-stack-migration). WGPU executes the
declared commands without inferring style from model names.

Humanized strokes combine seeded gesture confidence, pressure, taper, correction
and overstroke. Paper grain and ink dryness affect coverage; surface-anchored
hatching preserves local identities as the camera moves. `NprMotionPolicy`
controls stable or redrawn gestures independently of frame rate and object
playback. Existing render statistics and humanization tests remain applicable.

Validation: `cargo test -p amigo-npr-playground-plugin`,
`cargo test -p amigo-playground-api`, `cargo test -p amigo-assets browser`,
and `npm run check`, `npm test`, `npm run build` in `playground-client`.
