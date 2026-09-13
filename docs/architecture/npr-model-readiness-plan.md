# NPR Model Readiness Plan

This plan describes how to make NPR line rendering robust for arbitrary GLB models, including dense or noisy assets such as `mods/playground-npr/source-models/khronos/riders.glb`.

The goal is not to special-case one model. The goal is a model-independent readiness layer:

```text
loaded mesh asset
  -> mesh analysis
  -> NPR candidate cleanup
  -> auto-scaled thresholds and budgets
  -> stroke path grouping
  -> renderer diagnostics
```

## Current Problem

Complex GLB models fail visually because the NPR pipeline often sees raw geometry instead of drawing intent.

Typical symptoms:

- too many technical edges from triangulation
- flat surfaces split into triangles become visible line features
- tiny accessories or micro-polygons produce noisy strokes
- model scale changes the apparent line threshold behavior
- feature lines are selected before there is a line density budget
- debug output does not explain why a line candidate was accepted or rejected

The right fix is a reusable model-readiness layer, not per-file YAML hacks.

## Target Contracts

### Render API

Path:

```text
crates/engine/render-api/src/commands_3d.rs
crates/engine/render-api/src/stats.rs
```

Add renderer-facing contracts only. Do not add WGPU details here.

Proposed symbols:

```rust
pub struct NprMeshAnalysis3d {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub edge_count: usize,
    pub degenerate_triangle_count: usize,
    pub boundary_edge_count: usize,
    pub non_manifold_edge_count: usize,
    pub planar_internal_edge_count: usize,
    pub tiny_edge_count: usize,
    pub median_edge_length_model: f32,
    pub p10_edge_length_model: f32,
    pub p90_edge_length_model: f32,
    pub bounds_extent: [f32; 3],
    pub recommended_feature_angle_degrees: f32,
    pub recommended_min_screen_length_px: f32,
    pub recommended_line_budget: NprLineBudget3d,
}

pub struct NprLineBudget3d {
    pub max_total_candidates: usize,
    pub max_feature_candidates: usize,
    pub max_candidates_per_screen_tile: usize,
    pub screen_tile_px: f32,
}

pub enum NprEdgeCandidateReason3d {
    Silhouette,
    Boundary,
    FeatureAngle,
    MaterialSeam,
    Contact,
    RejectedPlanarTriangulation,
    RejectedTinyEdge,
    RejectedDensityBudget,
    RejectedLowImportance,
}
```

## Pipeline Phases

### Phase 1 - Mesh Analysis

Intent:

Compute model complexity and edge statistics once per prepared mesh. This gives the NPR renderer stable facts instead of guessing every frame.

Operations:

- READ `crates/engine/render-wgpu/src/renderer/mesh_geometry.rs`
  - Locate `CachedMeshGeometry3d`, triangle/edge construction, and seam detection.
  - Do not change draw output yet.

- ADD `crates/engine/render-wgpu/src/renderer/npr/mesh_analysis.rs`
  - Add `analyze_npr_mesh_geometry_3d`.
  - Inputs: `CachedMeshGeometry3d`.
  - Output: `NprMeshAnalysis3d`.

- MODIFY `crates/engine/render-wgpu/src/renderer/npr/mod.rs`
  - Export the analysis helper internally.

- MODIFY `crates/engine/render-wgpu/src/renderer/npr/types.rs`
  - Store analysis beside cached NPR geometry if this crate owns the backend-only cached representation.

Validation:

```powershell
cargo test -p amigo-render-wgpu npr_mesh_analysis
cargo check -p amigo-render-wgpu
```

Do not:

- Do not infer character semantics from entity names.
- Do not change YAML defaults in this phase.

### Phase 2 - Planar Triangulation Rejection

Intent:

Reject internal edges caused by flat surface triangulation. This directly addresses "flat side split into triangles draws feature lines".

Definition:

An edge is planar internal triangulation when:

- it has exactly two adjacent faces
- both adjacent face normals are nearly equal
- it is not a material seam
- it is not a boundary edge
- it is not part of a silhouette transition

Operations:

- READ `crates/engine/render-wgpu/src/renderer/npr/cpu_edges.rs`
  - Locate edge classification and candidate importance.

- MODIFY `crates/engine/render-wgpu/src/renderer/npr/cpu_edges.rs`
  - Add `npr_edge_is_planar_internal_triangulation`.
  - Run it before feature acceptance.
  - Count rejected edges in CPU debug stats.

- MODIFY `crates/engine/render-wgpu/src/renderer/npr/tests_cpu_paths.rs` or add `tests_cpu_edges.rs`
  - Add a two-triangle quad test.
  - Assert the diagonal internal edge is rejected.
  - Assert a real crease edge is still accepted.

Validation:

```powershell
cargo test -p amigo-render-wgpu npr_cpu
```

Do not:

- Do not reject material seams here.
- Do not reject silhouette edges here.

### Phase 3 - Auto Threshold Scaling

Intent:

Make presets portable across models with different scale and triangle density.

Operations:

- MODIFY `crates/engine/render-api/src/commands_3d.rs`
  - Add optional settings:

```rust
pub struct NprAutoTune3d {
    pub enabled: bool,
    pub scale_min_screen_length: bool,
    pub scale_feature_angle: bool,
    pub scale_line_budget: bool,
    pub aggressiveness: f32,
}
```

- MODIFY `crates/engine/scene/src/document/components.rs`
  - Add YAML document fields under `npr.auto_tune`.

- MODIFY `crates/engine/scene/src/hydration/plan/components_domains.rs`
  - Hydrate `auto_tune`.

- MODIFY `crates/engine/render-wgpu/src/renderer/npr/style.rs`
  - Add helpers:

```rust
pub(crate) fn npr_effective_min_screen_length_px(...)
pub(crate) fn npr_effective_feature_angle_degrees(...)
pub(crate) fn npr_effective_line_budget(...)
```

Validation:

```powershell
cargo test -p amigo-render-api npr
cargo test -p amigo-scene npr
cargo test -p amigo-render-wgpu npr_cpu
```

Do not:

- Do not make auto-tune a hidden fallback.
- Preset must explicitly opt in, or scene/global NPR defaults must declare it clearly.

### Phase 4 - Screen-Space Density Budget

Intent:

Reject low-importance feature lines when too many lines occupy the same screen region.

Operations:

- MODIFY `crates/engine/render-wgpu/src/renderer/npr/cpu_edges.rs`
  - Accumulate candidates by screen tile before final accept.
  - Preserve silhouettes first.
  - Preserve material seams second.
  - Rank feature/crease candidates by importance and length.

- MODIFY `crates/engine/render-wgpu/src/renderer/npr/cpu_paths.rs`
  - Apply path-level density cleanup after stitching.

- ADD tests in `crates/engine/render-wgpu/src/renderer/npr/tests_cpu_paths.rs`
  - Dense cluster keeps the highest-importance path.
  - Isolated important feature remains.

Validation:

```powershell
cargo test -p amigo-render-wgpu npr_cpu
```

Do not:

- Do not make density budget affect outer silhouette.
- Do not use nondeterministic ordering.

### Phase 5 - Stroke Path Grouping

Intent:

Convert many short edge fragments into longer, manga-like strokes where geometry supports it.

Operations:

- READ `crates/engine/render-wgpu/src/renderer/npr/cpu_paths.rs`
  - Locate path stitching and join angle logic.

- MODIFY `crates/engine/render-wgpu/src/renderer/npr/cpu_paths.rs`
  - Add role-aware path grouping:
    - contour: longer joins, stronger endpoint stability
    - feature/detail: medium joins, seeded bow
    - hatch/fold: short-to-medium joins

- MODIFY `crates/engine/render-wgpu/src/renderer/npr/cpu_stroke_tessellation.rs`
  - Keep seeded bow stable per path id.
  - Avoid frame-dependent noise.

- MODIFY `mods/playground-npr/scenes/comic-lines/npr-presets/toriyama-contour-ink-cpu-reference.yml`
  - Tune family settings only after engine grouping is stable.

Validation:

```powershell
cargo test -p amigo-render-wgpu npr_cpu
powershell -ExecutionPolicy Bypass -File scripts\npr-capture.ps1 -Preset toriyama_contour_ink_cpu_reference -Model soldier -DebugMode final -Output images\runtime-check-stroke-grouping.png -Warmup 1 -Settle 0
```

Do not:

- Do not solve grouping by increasing random wobble.
- Do not add model-name-specific logic.

### Phase 6 - Diagnostics

Intent:

Make bad NPR results explainable.

Operations:

- MODIFY `crates/engine/render-api/src/stats.rs`
  - Add counters:

```rust
pub rejected_planar_triangulation_edges: usize,
pub rejected_tiny_edges: usize,
pub rejected_density_budget_edges: usize,
pub accepted_silhouette_edges: usize,
pub accepted_feature_edges: usize,
pub accepted_material_seam_edges: usize,
```

- MODIFY `crates/engine/render-wgpu/src/renderer/npr/cpu_debug.rs`
  - Add debug views:
    - accepted features
    - rejected planar triangulation
    - rejected density budget
    - line importance

- MODIFY `mods/playground-npr/scenes/comic-lines/scene.rhai`
  - Expose HUD lines for candidate counts.

Validation:

```powershell
cargo test -p amigo-render-api npr
cargo test -p amigo-render-wgpu npr
cargo test -p amigo-scene playground_npr
```

Do not:

- Do not add GPU readbacks for normal frame diagnostics.
- CPU diagnostics can be richer because CPU reference owns the full candidate path.

### Phase 7 - GLB Readiness Report

Intent:

Before rendering a complex model, report why it may render poorly.

Operations:

- MODIFY `crates/engine/assets/src/model_discovery.rs`
  - Include source model path and asset key in discovered model metadata.

- ADD a console/debug command if a suitable devtools seam exists:

```text
npr.model.report
```

Output example:

```text
NPR model report: playground-npr/meshes/riders
triangles: 184230
edges: 291002
planar internal edges: 138551
tiny edges p10: 0.0018
recommended min_screen_length_px: 3.8
recommended feature_angle_degrees: 48.0
recommended budget: high-complexity
```

Validation:

```powershell
cargo check -p amigo-assets
cargo check -p amigo-app
```

Do not:

- Do not put model-specific policy in `apps/app`.
- Do not make renderer infer intent from file names.

## First Implementation Slice

The best first slice is:

1. ADD mesh analysis.
2. MODIFY CPU edge classification to reject planar internal triangulation.
3. ADD CPU tests for a triangulated quad.
4. ADD stats for rejected planar edges.
5. ADD one debug/capture pass for `riders.glb`.

This slice improves all complex models and directly targets the visible failure mode.

## Acceptance Criteria

Minimum:

- `riders.glb` no longer looks like a wireframe from flat triangulated surfaces.
- Soldier/Toriyama preset does not regress in silhouette quality.
- CPU reference tests cover planar triangulation rejection.
- Debug stats show accepted/rejected edge classes.

Full:

- Presets behave consistently across Soldier, Khronos Male, and Riders.
- Dense models automatically raise feature thresholds or lower feature budgets.
- Long strokes remain stable under camera motion.
- Debug overlay explains why a candidate was rejected.

## Backlog

After the first slice:

- material role inference from authored asset metadata
- semantic ink guides for faces, hands, cloth folds, and hair
- GPU parity after CPU behavior is correct
- per-model cached `NprMeshAnalysis3d`
- optional author override YAML for model-specific NPR readiness
