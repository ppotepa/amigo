# Pipeline

The plugin owns the rotating playground state and publishes a snapshot during
render extraction. `amigo-render-npr` owns projection, feature classification and tessellation.

Settings are validated before replacement through RuntimeControlProvider or
PresetProvider. Update advances rotation using simulation delta; pause preserves
it, and a requested step advances 1/60 second. RenderExtract reads the real viewport
and publishes only for scenes in the NPR mod. Scene changes clear old output.
Viewport orbit/zoom runs once in PostUpdate using host delta, not once per simulation
tick. Zoom eases logarithmic distance to a bounded target; external distance/target
changes and scene initialization discard stale motion.

At hydration, an optional typed `NprSettings` scene component produces a
plugin scene command. Its declared fields are applied over the canonical
single-object or gallery defaults, and then go through the same validation as
live metadata edits. This keeps authored scene intent separate from Rhai and
from backend packet generation.

Static glTF positions/indices are imported by amigo-3d-mesh, welded per primitive,
normalized and cached with topology. Source geometry, hatch anchors and their IDs
remain in model space; the domain transforms camera and directional light into
that space for each rigid/uniform object transform. It then clips fill triangles
and strokes against the near plane, computes perspective depth per vertex,
assembles feature chains and tessellates pixel-space rounded strokes. WGPU
executes global depth, fill and stroke passes; the regular MeshDrawCommand path
remains separate.

Smooth proxy preparation uses an explicit per-object relative seam-weld
tolerance. It is part of the immutable proxy cache key, so changing it cannot
reuse topology prepared with another import policy. A zero value retains only
exactly coincident positions; Polygonal never invokes this path.

The tessellator owns the drawing vocabulary: tool response curves (pencil,
fineliner, nib and brush), pressure, nib angle/aspect, RDP gesture cleanup,
endpoint taper, rounded joins/caps and deterministic correction strokes. Grain
is attached to stroke coverage rather than added as a renderer-side heuristic.
Optional tone paths are traced over selected surface faces from local plane seeds,
then projected and clipped before entering the same stroke path. This keeps a
camera move from constructing a new hatch grid and ensures triangulation diagonals
are never promoted to feature lines. Smooth silhouettes use a smoothed
corner-normal zero field. Topology-derived creases require explicit authoring in
Smooth; a large dihedral still partitions normal smoothing but is not silently
converted to ink. The assembled spans are then length-gated and simplified in
pixel space, including a deterministic closed-loop anchor. A stable source id
plus seed makes a gesture repeatable across frames and resize rebuilds.

The session owns two independent temporal mechanisms. `DrawingHistory` eases the
coverage of genuinely entering IDs once per logical view frame. `StrokeVariantClock`
measures projected surface-anchor motion and changes a gesture epoch only for an
explicit `RedrawOnMotion` policy; Stable returns epoch zero. Neither mechanism is
part of WGPU or the app host.

The workshop presentation remains engine-generic: authored choices resolve model
and status bindings, while the egui backend draws neutral thumbnail triangles.
`appearance.*` resolves to global or selected-object style in the domain provider.
Rhai buttons invoke metadata actions for camera fit, rotation, layout and history.
Before comparison substitutes only render-snapshot styles; live settings stay intact.
Selection markers are explicit domain fill annotations in the packet, not a WGPU
selection heuristic. They leave model ink, strokes and feature statistics unchanged.
