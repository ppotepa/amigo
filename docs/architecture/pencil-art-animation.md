# Pencil Art Animation

`Pencil Art Animation` is a runtime NPR composition, not a post-process filter.
It combines role-specific line selection, graphite stroke material, surface
hatching, and an authored artistic redraw cadence.

## Presets

The preset is resolved in this order:

`scene runtime object -> named preset -> scene style -> engine defaults`

The built-in roles are:

- `Pencil Character`: expressive contour, stronger pressure variation and
  hatching for animated subjects.
- `Pencil Architecture`: confident, restrained lines and sparse hatching for
  buildings and hard surfaces.
- `Pencil Background`: low-contrast, low-density marks for distant scenery.

Mods select roles explicitly in `npr.scene.yml`; the renderer never guesses a
role from an entity name.

## Stroke material

Every graphite ribbon carries gesture-local coordinates, pressure, grain,
hardness, dryness and a stable seed. The shader uses those attributes to vary
coverage and edge softness. A long gesture is tessellated from correlated
samples; the noise is continuous along the gesture rather than independent per
segment. The runtime stroke motor combines a stable hand bias with a smaller
artistic-frame realization, using separate broad, wrist and tooth bands. It
also applies a bounded endpoint aim error and a small tangential overshoot;
joined contour interiors remain connected while outer gesture ends do not snap
perfectly to mesh vertices. Pressure is similarly mostly stable but receives a
controlled frame component, so the 8 FPS drawing changes without flickering
independently at every vertex. A lighter under-stroke may be added with
deterministic dropout.

Hatching is anchored to reference/model-space geometry. Each lattice line uses
the entity seed, so instanced geometry shares a coherent field and animated
deformation does not randomly rotate the pattern. Hatching inherits the parent
stroke's pressure, hardness, dryness, wobble and sample count.
Adjacent clipped segments are joined when the group is small enough for a
bounded adjacency pass. Very large meshes retain the same lattice phase and
material realization but skip the quadratic join search, keeping frame time
bounded.

The mesh preparation cache also preserves the source glTF material index per
triangle. Role policies can enable `material_seams`; NPR then emits a restrained
boundary when adjacent faces belong to different source materials, while the
standard 3D material pass remains disabled.

Smooth/suggestive presets also declare `draw_form_lines` through the neutral
mesh style contract. The 3D path emits only the middle dihedral-angle range;
stronger angles remain authored creases, and flatter triangulation is omitted.
This keeps form description available on low-detail meshes without turning
every topology edge into an accidental stroke.

Pencil roles with sufficient tonal density may additionally declare
`draw_contact_lines`. These are selected from adjacent visible faces whose
lighting values differ materially, so a building receives readable plane and
contact boundaries even when its source mesh has few authored seams.

## Animation and temporal policy

The host continues to update at its normal refresh rate. `redraw_hz: 8` samples
poses and line variants every 125 ms for every runtime NPR object, including
stationary architecture. Camera movement changes projection, while the next
artistic epoch also creates a new hand realization. The bounded contour history
is refreshed at the same cadence and discarded on topology or camera
discontinuity.

## Visibility and loading

The WGPU backend renders background/2D, a depth-writing world-surface pass, and
the final UI pass separately. Graphite strokes read world depth but do not write
it, so translucent marks cannot hide geometry behind them. The offscreen target
owns a reusable `Depth32Float` attachment.

The engine loading service covers the frame until scene assets and domain NPR
preparation are ready. It is independent from a mod's own UI scene.

## Validation

- `cargo test -p amigo-render-wgpu world_depth_resolves_crossings`
- `cargo test -p amigo-render-wgpu graphite_coordinates_follow_stroke`
- `cargo test -p amigo-render-wgpu hatch_lattice_is_shared_at_seams_and_held_through_deformation`
- `cargo test -p amigo-render-wgpu suggestive_form_line_uses_the_explicit_middle_angle_band`
- `cargo test -p amigo-render-wgpu hatch_segments_share_one_gesture_phase`
- `cargo test -p amigo-app npr_city_publishes_a_complete_npr_packet_after_loading`

These tests cover GPU visibility, material coordinates, surface stability,
loading, and animated packet progression. Visual style still requires reviewing
the generated captures for each authored mod.

## Research basis

The surface-field direction follows Praun et al., *Real-Time Hatching*, which
uses coherent tonal art maps and curvature-aware parameterization. Temporal
silhouette handling follows Kalnins et al., *Coherent Stylized Silhouettes*.
Graphite coverage parameters follow Sousa and Buchanan, *Observational Models
of Graphite Pencil Materials*.
