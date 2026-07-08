# NPR Comic Line Rendering Architecture

Scope for V1: realtime comic/NPR stroke rendering for 3D meshes in the
`playground-npr/comic-lines` workbench.

V1 includes silhouettes, boundaries, creases/seams/features, material ink roles,
stable GPU path stitching, CPU reference rendering, and optional sparse
character hatch strokes. It does not include global hatching, halftone,
painterly fill, watercolor, or runtime GLB skinning/clip evaluation.

## Runtime Contract

NPR intent is authored in scene YAML and preset YAML:

```yaml
npr:
  strategy: gpu_realtime # or cpu_reference
  pipeline:
    candidate_strategy: character_semantic
    path_strategy: stable_stroked_paths
    stroke_strategy: akira_ink
    fill_strategy: material_black_mass
    hatching_strategy: sparse_character_hatching
    budget_strategy: face_and_silhouette_priority
    temporal_strategy: stable_arc_length
```

Rules:

- `gpu_realtime` and `cpu_reference` are explicit strategies.
- Missing `strategy` defaults to `gpu_realtime`.
- There is no `auto`, no `hybrid`, and no silent GPU -> CPU fallback.
- CPU reference is a correctness/reference path, not an automatic rescue path.
- Preset pairs should match except for `render_strategy`.

## Current Pipeline

```text
Mesh3D asset
  -> cached static topology
  -> GPU face-id/depth pass
  -> GPU edge classification
  -> endpoint bins / owner compaction
  -> path links / path states / relaxed owners
  -> path segments
  -> stroke segments
  -> WGPU stroke draw
```

The CPU reference path keeps the older projected-edge -> visible-fragment ->
stitched-path -> stylized-stroke model. It remains useful for parity checks and
debugging because it is easier to inspect than the GPU path.

## Character Ink V1

Character-oriented presets such as `akira` use explicit pipeline strategies
rather than renderer-side guesses from entity or model names.

Implemented V1 roles:

- `black_mass_material_ids` for solid black material fills.
- `ink_detail_material_ids` for preserving shorter face/eye/brow detail lines.
- `character_semantic` candidate filtering.
- `akira_ink` stroke shaping.
- `sparse_character_hatching` as short stroke-level hatches, not halftone.

Still out of V1:

- authored `ink_guides`;
- apparent ridges / full curvature line families;
- global halftone/screentone;
- runtime skinned GLB animation evaluation.

## Workbench

`mods/playground-npr/scenes/comic-lines` is now a real NPR workbench, not a
procedural placeholder. It stages two static GLB targets:

- Soldier;
- Khronos Male.

The HUD reports model, preset, backend strategy, debug view, frame stats, and
NPR GPU/CPU counters. The animation row is explicit: Khronos Male is a static
mesh in V1 because the current 3D renderer does not evaluate GLB skinning clips.

## Performance Rules

- Cache immutable mesh topology per asset.
- Keep GPU path execution bounded; avoid unbounded endpoint bucket traversal.
- Do not read back GPU visibility or stroke buffers during normal frames.
- Clamp indirect draw counts to actual stroke buffer capacity.
- Use diagnostics and smoke tests before increasing path walk budgets.
- Keep CPU reference and GPU realtime visually comparable through shared
  presets and explicit strategy fields.
