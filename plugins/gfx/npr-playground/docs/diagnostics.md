# NPR playground diagnostics

The explicit layer-stack migration reports each converted look patch and entry
count. It rejects mixed formats, duplicate/mismatched IDs and malformed layer
entries. An external edit after preview blocks Apply through the existing SHA
check. A pre-existing backup blocks overwrite; successful backups retain the
original bytes. Runtime loading rejects the former `paint`/`stroke` collections
instead of converting or replacing them silently.

The plugin publishes `RenderFrameStats` through the existing render diagnostics
service. Relevant values include geometry, topology edges, feature segments,
silhouettes, creases, strokes, stroke vertices/indices, hatching, construction
marks and stroke-budget rejections.

During a hydrated scene load, the engine overlay reports missing GLB geometry
and then `Preparing dynamic NPR scene`. Presentation becomes ready only after a
covered loading frame has been emitted. Runtime diagnostics distinguish prepared
Drawing Studio packets (`npr`) from animated model-space contributions
(`npr_meshes`); camera and actor motion must change the latter's rendered image
without increasing `packet_builds`.

The NPR domain supports `Final`, `FeatureClasses` and `StrokeIds` debug views.
These are resolved in the NPR domain packet, not by renderer-side inspection.

The companion reports presented-frame FPS averaged over 500 ms, actual mode and
resolution; a stopped scene reports Idle. Frame headers expose sequence, surface
generation, scene revision and processed input ID. `stages` separates extraction,
CPU submit, readback wait/copy and JPEG encoding. Native GPU has zero readback and
encoding time. Consumer acknowledgments contain decode/presentation durations;
these describe submission to the presentation API, not physical monitor scanout.
`packet_builds` distinguishes Drawing Studio geometry rebuilds from fade-only frames.
Runtime mesh NPR follows host frames and is not rate-limited by `NprMotionPolicy`.
`Pause sketch` affects
gesture variation independently from object animation playback.

Action errors carry request IDs and control identifiers. Import/catalog failures
remain source errors; save conflicts leave edits dirty. Domain events include
`selection_changed`, `model_loaded`, `look_changed`, `save_completed` and
`save_failed`. Packet counts and transfer status have separate UI fields.

Entry edits stay inline in the main window. Preview actions update transient
render state and may advance the transport revision, but never dirty the authored
document or create undo entries. Apply creates one history operation; Cancel
and transport disconnection discard the preview. Saving a reusable appearance
and updating matching references is atomic, including lock validation.

`layer_previews` contains cached backend reference samples of each entry's
appearance; `layer_preview_errors` exposes failures. These SVG samples are not
pixel-identical GPU captures. Native GPU/JPEG/Local RGBA live viewport preview
is authoritative for paper, analytic edges, granulation and model coverage.

Mask inputs are explicit object-local surface samples (position, normal,
normalized Y and illumination), never ink luminance or camera depth. WGPU
evaluates up to 128 recursively multiplied terms per fragment with perspective
interpolation. Missing inputs produce `missing-coverage-inputs` diagnostics and
are rejected by the backend. `mask_coverage` is an approximate CPU quadrature
over triangle interiors, not a GPU pixel counter; conservative exclusion avoids
misreporting a narrow unsampled band as having no effect. SVG look previews
subdivide masked triangles at thumbnail resolution. Seeded local noise uses the
same hash and interpolation on CPU/GPU.

Queued actions retain their enqueue timestamp for transport diagnostics.
`AMIGO_NPR_BENCHMARK=1` enables a
one-second host frame cadence log for comparison with/without the companion.
Presentation timing comes from the current single Drawing Studio window and
its Native GPU/JPEG/Local RGBA transport events.
Use the optimized `playground` Cargo profile and measure the actual Tauri window.
The earlier offscreen measurements remain in `viewport-performance-analysis.md`;
they are a baseline, not a claim about current Tauri FPS.

`playground transport: sending on a full channel` identified a transport bug:
a temporarily full host input queue ended the WebSocket worker, and companion
cleanup killed Tauri while Winit remained open. The transport now retains one
pending message and pauses socket reads until space is available, while still
sending replies and observing shutdown. Queue capacity stays bounded and actions
retain their order. The maintained manual checklist exercises burst recovery
around a temporary pause of the test engine.
