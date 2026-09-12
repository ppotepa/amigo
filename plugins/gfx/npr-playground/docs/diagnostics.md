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

The NPR domain supports `Final`, `FeatureClasses` and `StrokeIds` debug views.
These are resolved in the NPR domain packet, not by renderer-side inspection.

The companion reports presented-frame FPS averaged over 500 ms, actual mode and
resolution; a stopped scene reports Idle. Frame headers expose sequence, surface
generation, scene revision and processed input ID. `stages` separates extraction,
CPU submit, readback wait/copy and JPEG encoding. Native GPU has zero readback and
encoding time. Consumer acknowledgments contain decode/presentation durations;
these describe submission to the presentation API, not physical monitor scanout.
`packet_builds` distinguishes geometry rebuilds from fade-only frames. Stroke redraw cadence
is controlled by `NprMotionPolicy`, not by display FPS. `Pause sketch` affects
gesture variation independently from object animation playback.

Action errors carry request IDs and control identifiers. Import/catalog failures
remain source errors; save conflicts leave edits dirty. Domain events include
`selection_changed`, `model_loaded`, `look_changed`, `save_completed` and
`save_failed`. Packet counts and transfer status have separate UI fields.

Brush Editor messages carry a monotonic session ID and request ID. Apply and
Cancel are accepted only for the active session; stale drafts are rejected by
the host and never reach the document mutation queue.

For reproducible browser-side measurements the client dispatches
`playground-input-sent` and `playground-frame-presented` DOM events. They contain
only request IDs and neutral frame metadata, never native handles or credentials.
Input samples include the oldest queued movement timestamp, so measurements also
cover time waiting for a preceding mutation. `AMIGO_NPR_BENCHMARK=1` enables a
one-second host frame cadence log for comparison with/without the companion.
That comparison applies to the former two-window setup; Gallery now runs without
the host renderer, so its presentation timing comes from the companion events.
Use the optimized `playground` Cargo profile and measure the actual Tauri window.
The earlier offscreen measurements remain in `viewport-performance-analysis.md`;
they are a baseline, not a claim about current Tauri FPS.

`playground transport: sending on a full channel` identified a transport bug:
a temporarily full host input queue ended the WebSocket worker, and companion
cleanup killed Tauri while Winit remained open. The transport now retains one
pending message and pauses socket reads until space is available, while still
sending replies and observing shutdown. Queue capacity stays bounded and actions
retain their order. `viewport_qa.py` exercises a burst of 256 viewport requests
around a temporary pause of its own test engine and verifies recovery.
