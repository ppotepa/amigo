# NPR Comic Line Rendering Architecture

Scope for this phase: silhouettes, boundaries, and feature lines only.
Hatching, halftones, painterly fill, and paper simulation are deferred.

## What `npr.html` Proves

The prototype is not just a post effect. It builds lines from mesh topology and
camera visibility:

- `buildFaceIdVisibilityBuffer` rasterizes face id, depth, material, and tone.
- `classifyEdges` marks boundary, silhouette, and feature edges.
- `visibleFragmentsForEdge` clips edges by face-id/depth visibility.
- `buildPathsForFragments` joins fragments into stable drawable paths.
- `drawStyledPath` adds humanized stroke width, jitter, taper, dry gaps, and multi-pass ink.
- `renderDrawing` draws the final paper/comic image with progressive reveal.

The engine version should keep the same conceptual stages, but move the costly
visibility and stroke expansion work into explicit contracts and GPU-friendly
buffers.

## Target Pipeline

```text
Mesh3D asset
  -> NprMeshCache
  -> NprVisibleEdgePlan
  -> NprStrokePlan
  -> WGPU stroke renderer
  -> optional post stages later
```

## Data Contracts

ADD `crates/engine/render-api/src/npr_3d.rs`

Intent:

- describe backend-neutral NPR input and output packets;
- avoid renderer-side guessing from mesh names or material debug labels;
- keep hatching and halftones out of the first contract.

Initial model:

```rust
pub struct NprLine3dCommand {
    pub entity_id: u64,
    pub entity_name: String,
    pub mesh_asset: AssetKey,
    pub transform: Transform3,
    pub style: NprStrokeStyle3d,
    pub extraction: NprLineExtraction3d,
}

pub struct NprLineExtraction3d {
    pub boundary: bool,
    pub silhouette: bool,
    pub feature: bool,
    pub feature_angle_degrees: f32,
    pub min_screen_length_px: f32,
    pub visibility: NprVisibilityMode3d,
}

pub struct NprStrokeStyle3d {
    pub ink_color: ColorRgba,
    pub width_px: f32,
    pub width_jitter_px: f32,
    pub path_jitter_px: f32,
    pub taper: f32,
    pub overshoot_px: f32,
    pub dropout: f32,
    pub passes: u8,
    pub seed: u64,
}
```

ADD `plugins/rendering/npr-lines`

Intent:

- own authored NPR line intent;
- hydrate scene documents into `NprLine3dCommand`;
- expose diagnostics for cache hits, visible edge count, stroke count, and GPU budgets.

Do not add this behavior to `apps/app`.

## Mesh Cache

ADD `crates/engine/render-api` cache-facing models or a small `crates/3d/npr-cache`
crate after the importer boundary is chosen.

Cache immutable topology per mesh asset:

- vertex positions, normals, tangents, UVs;
- triangle list after triangulation;
- material id per face;
- edge table with adjacent face ids;
- boundary flags;
- crease candidate flags by rest-pose normal angle;
- stable edge ids and chain ids;
- mesh bounds and LOD bins.

Runtime input should never rebuild adjacency every frame.

For static meshes:

- bake cache once when the asset is prepared;
- reuse cache until the asset changes.

For skinned/animated meshes:

- cache topology once;
- update skinned vertex positions/normals separately;
- keep edge ids stable across animation frames.

## Visibility

The `npr.html` software face-id buffer should become a GPU prepass:

```text
mesh prepass
  -> depth texture
  -> face_id texture R32Uint
  -> optional normal/tone texture
```

Then edge visibility can be evaluated by:

- compute shader sampling edge points against `face_id` and depth;
- CPU fallback only for tests and small debug meshes.

Use CPU extraction only as a correctness oracle. It will not scale for real
comic-game scenes.

## Stroke Generation

Stroke generation should be deterministic and stable:

- stable seed = mesh asset id + edge id + chain id + style seed;
- human jitter should be screen-space but temporally stable under small camera motion;
- taper, pressure, dry gaps, and overshoot are style attributes, not random per frame;
- path simplification happens after visibility clipping.

The renderer should expand strokes from compact path buffers:

```text
NprStrokePath {
  path_id,
  stroke_type,
  depth,
  first_point,
  point_count,
  style_id,
}

NprStrokePoint {
  position_px,
  depth,
  tangent,
  pressure,
  noise_phase,
}
```

The GPU path should expand line segments into quads/joins in a stroke pipeline.
Avoid generating a large CPU vertex mesh every frame unless a debug mode asks
for it.

## Draw Order

Use explicit ordering:

1. optional flat/tone fill from normal material path;
2. boundary lines;
3. feature/crease lines;
4. silhouette lines;
5. optional progressive reveal mask.

Silhouettes should be thicker and allowed to overshoot more than internal
feature lines.

## Scene Authoring Target

Future scene component:

```yaml
- type: amigo.rendering.npr-lines.NprLineStyle3D
  enabled: true
  mesh: self
  extraction:
    boundary: true
    silhouette: true
    feature: true
    feature_angle_degrees: 32.0
    min_screen_length_px: 2.0
    visibility: face_id_depth
  style:
    ink_color: "#101010FF"
    width_px: 2.4
    width_jitter_px: 0.55
    path_jitter_px: 0.75
    taper: 0.65
    overshoot_px: 1.8
    dropout: 0.035
    passes: 2
    seed: 2002
```

Do not place this component in shipped scenes until the plugin exists.

## Implementation Plan

READ `npr.html`

- symbols: `buildFaceIdVisibilityBuffer`, `classifyEdges`,
  `visibleFragmentsForEdge`, `buildPathsForFragments`, `drawStyledPath`;
- intent: keep algorithmic behavior without copying browser architecture.

READ `crates/engine/render-api/src/commands_3d.rs`

- intent: extend renderer-facing 3D contracts.

READ `crates/engine/render-wgpu/src/renderer/world_3d.rs`

- intent: replace debug-cube-only mesh rendering with imported mesh geometry before
  relying on real silhouettes.

ADD `plugins/rendering/npr-lines`

- intent: domain-owned NPR scene document, hydration, diagnostics, and tests.

ADD `crates/engine/render-api/src/npr_3d.rs`

- intent: backend-neutral line extraction and stroke plan contracts.

MODIFY `crates/engine/render-wgpu`

- intent: add a WGPU stroke renderer and, later, a face-id/depth prepass.

MODIFY `crates/runtime/bundles`

- intent: bridge NPR line commands into the render packet, not into `apps/app`.

ADD tests:

- cache builds stable topology ids for a static mesh;
- silhouette toggles when camera crosses an edge normal;
- feature angle threshold changes feature edge count;
- style seed produces stable jitter;
- hatching and halftone fields are absent in the phase-one contract.

## Performance Rules

- cache topology once per mesh asset;
- do not rebuild adjacency per frame;
- use GPU face-id/depth prepass for visibility;
- use compact stroke path buffers and GPU line expansion;
- keep deterministic seeds stable to avoid temporal buzzing;
- expose budgets: max edges, max stroke points, max passes, max visible fragments;
- add diagnostics before adding visual tuning controls.

## Current Workbench Limitation

`mods/playground-npr/scenes/comic-lines` is intentionally a shell. It loads
`Mesh3D` commands and GLB descriptors, but the current WGPU path still renders
debug procedural meshes. Real comic lines require the importer/cache and NPR
stroke renderer described above.
