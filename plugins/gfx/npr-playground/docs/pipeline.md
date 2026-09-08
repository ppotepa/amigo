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

Static glTF positions/indices are imported by amigo-3d-mesh, welded per primitive,
normalized and cached with topology. The domain clips fill triangles and strokes
against the near plane, computes perspective depth per vertex, assembles feature
chains and tessellates pixel-space rounded strokes. WGPU executes global depth,
fill and stroke passes; the regular MeshDrawCommand path remains separate.

The tessellator owns the drawing vocabulary: tool response curves (pencil,
fineliner, nib and brush), pressure, nib angle/aspect, RDP gesture cleanup,
endpoint taper, rounded joins/caps and deterministic correction strokes. Grain
is attached to stroke coverage rather than added as a renderer-side heuristic.
Optional tone lines are clipped against the projected face triangle before they
enter the same stroke path, so triangulation diagonals are never promoted to
feature lines. A stable feature-chain id plus seed makes the gesture repeatable
across frames and resize rebuilds.

The workshop presentation remains engine-generic: authored choices resolve model
and status bindings, while the egui backend draws neutral thumbnail triangles.
`appearance.*` resolves to global or selected-object style in the domain provider.
Rhai buttons invoke metadata actions for camera fit, rotation, layout and history.
Before comparison substitutes only render-snapshot styles; live settings stay intact.
Selection markers are explicit domain fill annotations in the packet, not a WGPU
selection heuristic. They leave model ink, strokes and feature statistics unchanged.
