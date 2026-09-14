# NPR playground pipeline

Authored look patches use one `layers` list for all generators. Entries retain
`layer_id`, `order` and typed `parameters`; inheritance replaces by stable ID,
then sorts by `(order, layer_id)`. `removed_layers` explicitly suppresses inherited
entries and survives save/reload. Re-adding an entry clears its removal. Invalid
or contradictory patches are rejected before merging. The renderer consumes
these declarations in global order and applies per-entry targets and coverage.
UI categories never reorder the document or introduce compositing groups.

`NprStyleLayer::extraction_style` resolves each entry's `line` / `hatch` generator
parameters and its tool, pressure and gesture parameters before tessellation.
`render/layers.rs::build_drawing` reuses prepared surface analysis and a bounded
source-packet cache. Hatching selects its own tonal generator independently of
paint entries; two hatch entries can use different angles, spacing and density.
Expanded stroke storage is capped before backend submission, with structural
lines taking priority over tonal detail. All outputs retain their authored ID.

Build-up is editor-only state applied by `NprPlaygroundState::render_snapshot`:
it disables layers after the reveal fraction without changing authored visibility
or dirty state. Named variants and brush definitions are persisted by the scene
profile; reusable presets also embed their referenced appearance definitions.
These definitions are restored before the render service receives a packet. Each layer
keeps a pinned brush reference; extraction resolves that definition first and
then applies instance overrides, so a library update cannot silently alter an
existing document. The definition seed is the fallback for procedural
humanization; an instance seed explicitly overrides it.

The scene's `NprSettings.profile` refers to the authored NPR sidecar. The shared
`NprPlaygroundService` resolves it into `NprPlaygroundState`. The plugin owns object
selection, camera intent, style inheritance and typed drawing policy. Drawing
Studio resolves that state into `NprDrawCommand`; animated runtime scenes instead
emit camera-independent `NprMeshDrawCommand` contributions with explicit layer
visibility, silhouette/boundary/crease weights, tool pressure, hardness, dryness,
taper, overstroke and wobble. A time-derived artistic frame advances at authored
`motion.policy.redraw_hz`, so deterministic stroke variants remain fixed between
redraw ticks even when the host presents at a higher refresh rate.
Runtime mesh styles also declare correlated gesture samples and pressure
variation. WGPU turns those samples into a curved ribbon, rather than drawing
each projected edge as one perfectly straight quad.
High-value authored entities may opt into `join_strokes`; this joins their
silhouette edges into one continuous gesture while keeping large background
meshes on the bounded per-edge path.
Silhouette strokes also use a bounded renderer history: a briefly occluded edge
is held for two artistic frames and fades, preventing one-frame contour pops
without permanently ghosting topology changes.

For prepared Drawing Studio packets, `amigo-render-npr` owns projection, visible
features, strokes and tessellation. `amigo-render-wgpu` executes the packet's
declared layers. Runtime mesh contributions use the dynamic path below; neither
path infers a style from mesh names.

Hydrated NPR scenes do not build camera-dependent drawing packets. Mesh import
prepares immutable indexed topology once per unique asset and stores geometry
behind `Arc`; extraction therefore clones only references. The runtime bridge
copies current entity transforms into model-space NPR mesh contributions every
frame. WGPU projects the current camera, fills front faces and derives visible
boundary/silhouette edges from the prepared adjacency. Loading remains covered
by the engine overlay until every referenced GLB has geometry and the dynamic
NPR contribution is ready. Runtime GLB playback may additionally declare a sample
rate; this quantizes skeletal sampling without slowing the host loop. `npr-city`
uses eight pose samples and eight stroke redraws per second for held, hand-ink
motion. The host loop remains uncapped, so input and presentation stay responsive
between artistic frames.

Dynamic world rendering separates background/2D, depth-tested world, and final
UI passes. World surfaces establish per-pixel reversed depth before graphite
is drawn. Stroke pipelines read that depth without writing it, so transparent
graphite coverage cannot hide a later stroke. The offscreen target owns and
reuses the depth attachment across frames and resizes; no per-frame depth
texture is allocated. Surface and stroke vertices share one upload buffer.

`world_depth_resolves_crossings_preserves_graphite_overlap_and_ui` verifies GPU
pixels for crossing surfaces, overlapping translucent strokes, UI above the
world, and a subsequent UI-only frame. The city integration capture separately
checks loading, runtime mesh contributions and animation progression. These
checks do not prove art quality or stable temporal contour correspondence;
the screen-space stroke history still requires reprojection/invalidation work.

The companion bridge uses the same explicit NPR extraction and packet contracts
with its own offscreen target. `WgpuNprRenderer` executes NPR for both full scenes
and companions, without initializing unrelated scene pipelines for the companion.
Its vertex/index buffers are reused; unchanged payloads skip upload.

Prepared source meshes, topology, smooth proxies, direction fields and corner
normal/component caches are shared across views. Camera-dependent packets, LOD,
variant clocks and temporal histories remain per view. Packet keys are per object
and include effective style/layers, camera, viewport, seed and effective variant
epoch. Geometry keys include enabled state, line/hatch settings and appearance
references/overrides. Compositor-only edits refresh diagnostics without rebuilding
paths. Sketch pause is applied before cache comparison. Fade advances from cached
source geometry independently of packet generation; `packet_builds` counts rebuilds.
Surface hatch traversal uses the canonical sorted topology for logarithmic edge
lookup; triangle/plane intersection uses stack storage. The synchronous companion
submission borrows the finished packets. Camera zoom stores an authored target;
host-frame interpolation changes only render state, preserving action revisions.

Imported glTF/GLB poses come from `amigo-3d-mesh`: hierarchical TRS curves,
morph targets and joint skinning run before NPR feature extraction. Bind-pose
normalization and triangle identities stay fixed. `render/model.rs` retains one
replaceable sampled pose per asset; packet keys include surface content identity,
so a changed pose invalidates extraction while a paused pose is reused. Picking
uses that same sampled source surface.

`playback.rs` owns the single-model clock, clip/turntable source, pause, seek,
speed and looping. Transport is excluded from serialized `Settings` and history.
Camera actions and layer previews preserve it; changing models drops it. The
snapshot publishes `playback` and `animation_clips` separately from `npr` document
values. Neutral host deltas can carry changed telemetry at the same edit revision:
a frame advances the playhead, not the document's optimistic concurrency token.

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
