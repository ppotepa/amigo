# NPR playground pipeline

Authored look patches use one `layers` list for all generators. Entries retain
`layer_id`, `order` and typed `parameters`; inheritance replaces by stable ID,
then sorts by `(order, layer_id)`. Duplicate IDs are rejected before merging any
style or layer changes. The renderer consumes these layer declarations in global
order, applies target filtering and procedural mask coverage, and composes group
opacity with the layer result.

Build-up is editor-only state applied by `NprPlaygroundState::render_snapshot`:
it disables layers after the reveal fraction without changing authored visibility
or dirty state. Named variants and brush definitions are persisted by the scene
profile and restored before the render service receives a packet. Each layer
keeps a pinned brush reference; extraction resolves that definition first and
then applies instance overrides, so a library update cannot silently alter an
existing document. The definition seed is the fallback for procedural
humanization; an instance seed explicitly overrides it.

The scene's `NprSettings.profile` refers to the authored NPR sidecar. The shared
`NprPlaygroundService` resolves it into `NprPlaygroundState`. The plugin owns object
selection, camera intent, style inheritance and typed drawing policy. During
`RenderExtract`, it resolves scene layers plus sparse object overrides into an
`NprDrawCommand` and submits it through the registered extractor bridge.

`amigo-render-npr` owns projection, visible features, strokes and tessellation.
`amigo-render-wgpu` only executes the resulting packet in declared layer order:
depth, then colour batches. The backend does not infer style from mesh names or
topology.

The companion bridge uses the same explicit NPR extraction and packet contracts
with its own offscreen target. `WgpuNprRenderer` executes NPR for both full scenes
and companions, without initializing unrelated scene pipelines for the companion.
Its vertex/index buffers are reused; unchanged payloads skip upload.

Prepared source meshes, topology, smooth proxies, direction fields and corner
normal/component caches are shared across views. Camera-dependent packets, LOD,
variant clocks and temporal histories remain per view. Packet keys are per object
and include effective style/layers, camera, viewport, seed and effective variant
epoch. Sketch pause is applied before cache comparison. Fade advances from cached
source geometry independently of packet generation; `packet_builds` counts rebuilds.
Surface hatch traversal uses the canonical sorted topology for logarithmic edge
lookup; triangle/plane intersection uses stack storage. The synchronous companion
submission borrows the finished packets. Camera zoom stores an authored target;
host-frame interpolation changes only render state, preserving action revisions.

Native GPU copies the final offscreen texture into one of three shared DX12 slots.
The platform presenter copies it to its child-window swapchain; readiness and
reuse are GPU-fenced. Local RGBA and JPEG use three reusable asynchronous staging
buffers. RGBA goes into a mapped WebView2 SharedBuffer; JPEG is encoded in a
separate bounded worker. Neither RGBA nor Native GPU sends pixels over WebSocket.
Frame credits cover production through consumer acknowledgment, and surface
generations reject obsolete frames on resize or mode changes.

Paper is scene-owned because one `NprBackgroundCommand` covers the viewport.
Object-level Paper edits are rejected. Every active draw command must agree on
Paper enabled state and opacity.
