# NprPlayground

Opt-in NPR drawing with a Tauri/Svelte Drawing Studio companion and embedded
WGPU viewport. One selected model owns one drawing document: paper, palette,
an ordered stack presented as Lines, Paints and Paper. Tools are properties of
these entries; pinned appearance revisions remain internal document references.

Run `npm ci` in `plugins/gfx/npr-playground/playground-client`, then from
the repository root:
`rtk cargo run --profile playground -p amigo-app -- --hosted --mod npr-playground --scene drawing-studio`.

The viewport uses Native GPU when available and fails over to JPEG if required.
Windows Native GPU uses a DX12 worker and a native child surface; Local RGBA
is also selectable in the Render diagnostics panel.
Scenes without a companion use the ordinary Winit host and its own backend.

See [client and authoring](docs/npr-playground-ui.md), [pipeline](docs/pipeline.md),
[contributions](docs/contributions.md) and [diagnostics](docs/diagnostics.md).
Measured results and remaining limits: [viewport validation](docs/viewport-validation.md).

Selecting another model saves a changed drawing as a local, model-bound draft
and preserves the current preset on the newly selected model. Save Drawing
writes the authored scene sidecar. Save Preset preserves includes, explicit
entry removals and the referenced appearance definitions. Save As creates a
standalone reusable preset. Model/camera changes do not mark a preset modified.
Tool cards use backend-generated reference strokes/surface samples, not CSS
swatches. Full material evaluation is available through live viewport preview.
The curated choices include Comic Ink, Hand Ink, Rough Pencil Keys, Clean TV Ink,
Pencil Study, Pencil Art Animation and Watercolour Wash. `Pencil Art Animation`
uses a dedicated GPU pencil batch with procedural grain/tooth coverage, pressure
variation, taper and a light graphite under-stroke. Dynamic stroke vertices carry
gesture-local longitudinal/lateral coordinates and graphite parameters; coverage
follows the gesture instead of sampling a screen-space noise overlay. Subpixel
grain fades toward average deposition to reduce aliasing. Runtime mesh scenes combine a
surface-attached hatch lattice with the sampled geometry: reference positions are
shared across animated frames, and hatch intersections use perspective-correct
edge interpolation. Current mapping uses a reference-pose box projection (256
drawing units over the longest source axis); it is not yet a curvature-aligned field.
Runtime mesh scenes combine a
look with the authored motion cadence; `temporal: true` is required for line
boiling. Runtime extraction compares transform and deformed vertex positions
against the last drawn pose at the authored cadence. A stopped character holds
its last graphite variant; camera-only motion does not advance that variant.
Invisible/removed subjects are pruned from the pose cache and scene reload clears
it. Shared immutable geometry takes a pointer-equality fast path; reallocated
identical geometry does not trigger redraw.

The NPR domain owns surface interpretation, typed ComicInk and layer contracts,
feature extraction and gesture policy. HardSurface and Organic are explicit
authored intents. Smooth uses a prepared welded proxy; crease, contour cleanup
and suggestive-contour thresholds remain domain policy. Paper is scene-owned.
One authored `layers` list preserves stable IDs and explicit order across paint
and strokes. Existing split documents require the explicit
[migration workflow](docs/npr-playground-ui.md#document-and-migration). WGPU executes the
declared commands without inferring style from model names.

Humanized strokes combine seeded, correlated gesture samples with local pressure,
confidence, taper, correction and overstroke. Longer edges receive more gesture
samples while small architectural marks stay inexpensive. Paper grain and ink dryness affect coverage; surface-anchored
hatching preserves local identities as the camera moves. `NprMotionPolicy`
controls stable or redrawn gestures independently of frame rate and object
playback. Existing render statistics and humanization tests remain applicable.

Validation: `cargo test -p amigo-npr-playground-plugin`,
`cargo test -p amigo-playground-api`, `cargo test -p amigo-assets browser`,
and `npm run check`, `npm test`, `npm run build` in `playground-client`.
Manual acceptance: [Drawing Studio QA](docs/manual-qa.md).
