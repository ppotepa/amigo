# Amigo NPR GPU/CPU Parity - Plan naprawczy v2, pełna implementacja

Status: plan implementacji po review aktualnego źródła z `concat.zip`.
Zakres: renderowanie kreski komiksowej/NPR dla 3D. Bez hatchingu, halftone, painterly, fill shading i watercolor.
Cel: `gpu_realtime` ma dawać prawie taki sam rysunek jak `cpu_reference`, ale wykonany ścieżką GPU bez `auto`, bez `hybrid`, bez cichego fallbacku GPU -> CPU.

Ten dokument zastępuje poprzedni plan. Najważniejsza korekta względem wersji 1: w aktualnym źródle część fundamentów już istnieje (`NprRenderStrategy3d`, `NprFillMode3d`, `NprGpuRealtimeTuning3d`, `GpuStableComic`, routing CPU/GPU, podstawowy GPU pipeline). Brakiem nie jest już samo przełączenie strategii, tylko pełny path model na GPU: endpoint bins, path walk, path-level stylizacja, debug mode i frame-level multi-mesh face-id.

---

## 0. Definicja sukcesu

Efekt końcowy:

1. Ten sam model, kamera i preset dają bardzo podobny rysunek w `cpu_reference` i `gpu_realtime`.
2. GPU nie pokazuje losowych długich kresek, prostokątów, siatek ani edge-chainów, które nie odpowiadają obrysowi modelu.
3. Presety NPR działają semantycznie tak samo w CPU i GPU:
   - `width_px`,
   - `silhouette_width_multiplier`,
   - `boundary_width_multiplier`,
   - `feature_width_multiplier`,
   - `width_pressure_curve`,
   - `alpha_pressure_curve`,
   - `taper`,
   - `dropout`,
   - `passes`,
   - `search_line_count`,
   - `humanization`,
   - `temporal_path_smoothing`.
4. `cpu_reference` pozostaje jawna ścieżka referencyjna.
5. `gpu_realtime` pozostaje domyślną ścieżką runtime.
6. Nie ma `auto`, `hybrid` ani cichego fallbacku GPU -> CPU.
7. GPU final render korzysta z `path_segments`, nie bezpośrednio z edge-local `visible_segments`.
8. Stare pola `next_a`, `next_b`, `alt_next_a`, `alt_next_b` nie są autorytatywnym modelem path ownership. Mogą zostać tylko jako opcjonalne hinty diagnostyczne do czasu cleanupu.
9. W normalnym realtime nie ma readbacku GPU -> CPU.
10. Debug GPU jest konfigurowalny w YAML i przełączalny z playgroundu.

---

## 1. Stan obecnego źródła, który dokument musi uwzględniać

### 1.1. Już istnieje w obecnym kodzie

Plik: `crates/engine/render-api/src/commands_3d.rs`

Aktualne symbole:

- `NprRenderStrategy3d` około linii 55-68:
  - `GpuRealtime`,
  - `CpuReference`.
- `NprFillMode3d` około linii 70-86:
  - `Shaded`,
  - `None`,
  - `DepthOnly`.
- `NprGpuRealtimeTuning3d` około linii 88-177:
  - `max_render_length_px`,
  - `max_segment_length_px`,
  - `max_terminal_walk_edges`,
  - `max_chained_walk_edges`,
  - `max_chain_angle_degrees`,
  - `search_enabled`,
  - `search_max_render_length_px`,
  - `search_alpha_multiplier`,
  - `feature_min_length_multiplier`,
  - `feature_alpha_multiplier`,
  - `silhouette_min_length_multiplier`.
- `NprLineSettings3d` około linii 179-241 już posiada:
  - `style_preset`,
  - `stroke_tool`,
  - `render_strategy`,
  - `gpu_realtime_tuning`,
  - `fill_mode`,
  - komplet głównych parametrów kreski.
- `NprStylePreset3d` około linii 243-289 już ma:
  - `GpuStableComic`,
  - `RoughComicInk`.
- `Default for NprStylePreset3d` już wskazuje `GpuStableComic`.
- `Default for NprLineSettings3d` już robi `from_preset(NprStylePreset3d::default())`.
- `from_preset(GpuStableComic)` już ustawia `render_strategy: GpuRealtime` i `fill_mode: None`.

Wniosek do dokumentu: nie planować od nowa strategii i defaultów. One już są. Nowe prace muszą rozszerzać istniejący kontrakt, nie wymieniać go.

Plik: `crates/engine/scene/src/document/components.rs`

Aktualne symbole:

- `NprLine3dDocument` około linii 41-46.
- `NprLine3dSettingsDocument` około linii 48-192.
- `NprGpuRealtimeTuningDocument` około linii 194-218.

Wniosek do dokumentu: trzeba dopisać `debug_mode` do istniejącego `NprGpuRealtimeTuningDocument`, nie tworzyć równoległego formatu YAML.

Plik: `crates/engine/scene/src/hydration/plan/components_domains.rs`

Aktualne symbole:

- `npr_line_settings_3d_from_document(...)` około linii 570-765.
- `apply_npr_gpu_realtime_tuning_3d(...)` około linii 935-974.
- `npr_render_strategy_3d_from_document(...)` około linii 999-1025.
- `npr_fill_mode_3d_from_document(...)` około linii 1027-1046.
- `npr_style_preset_3d_from_document(...)` około linii 1048-1072.

Istotne: `npr_render_strategy_3d_from_document(...)` już odrzuca `hybrid` i `auto`. Tego nie zmieniać.

Plik: `crates/engine/render-wgpu/src/renderer/service/render/world.rs`

Aktualne symbole:

- `route_npr_mesh(settings: &NprLineSettings3d) -> NprMeshRenderRoute` około linii 14-18.
- routing CPU/GPU w render loop około linii 270-300.
- testy routingowe około linii 760+.

Wniosek do dokumentu: routing strategii jest gotowy. Brak dotyczy jakości i pełności GPU path pipeline.

Plik: `crates/engine/render-wgpu/src/renderer/npr/gpu_realtime.rs`

Aktualny pipeline w `GpuRealtimeNprRenderer3d::execute(...)`:

```text
begin_frame / enqueue_mesh
ensure_topology_uploaded
ensure_face_id_target
ensure_frame_buffers
for each job:
  face_id render pass
  project_vertices compute pass
  classify_edges compute pass
  build_strokes compute pass
draw_indirect
```

Linie orientacyjne:

- `NprGpuMeshJob3d` około linii 13-20.
- `NprGpuFrameStats3d` około linii 22-32.
- `GpuRealtimeNprRenderer3d` około linii 34-40.
- `execute(...)` około linii 66-274.
- `face_id pass` około linii 187-214.
- `compute bind group` około linii 216-230.
- `project_vertices` około linii 233-241.
- `classify_edges` około linii 242-250.
- `build_strokes` około linii 251-259.
- `draw_indirect` około linii 285-296.
- `uniforms_for_job(...)` około linii 330+.

Wniosek do dokumentu: aktywna ścieżka nie robi jeszcze endpoint/path passów. Trzeba dodać je między `classify_edges` i `build_strokes`.

Plik: `crates/engine/render-wgpu/src/renderer/npr/gpu_pipelines.rs`

Aktualne symbole:

- `NprGpuPipelines3d` około linii 6-16.
- `compact_owners_pipeline` istnieje w strukturze.
- `compute_bind_group_layout` około linii 73-88 ma tylko bindy `0,1,2,3,4,5,6,8,9,10`.

Wniosek do dokumentu: trzeba rozszerzyć bind layout o endpoint/path/counter buffers. Samo dodanie shaderów bez bind layoutu nie wystarczy.

Plik: `crates/engine/render-wgpu/src/renderer/npr/gpu_types.rs`

Aktualne symbole:

- `GpuNprVertex3d`,
- `GpuNprTriangle3d`,
- `GpuNprEdge3d`,
- `GpuNprProjectedVertex3d`,
- `GpuNprVisibleSegment3d`,
- `GpuNprPathLink3d`,
- `GpuNprFrameUniforms3d`,
- `gpu_edges_from_geometry(...)`.

Aktualne `GpuNprEdge3d` nadal zawiera:

```rust
pub next_a: u32,
pub next_b: u32,
pub degree_a: u32,
pub degree_b: u32,
pub alt_next_a: u32,
pub alt_next_b: u32,
```

Wniosek do dokumentu: pełna implementacja musi dodać nowe path struktury i potem usunąć lub zdegradować stare next/alt-next pola.

Plik: `crates/engine/render-wgpu/src/renderer/shaders/mod.rs`

Aktualne includy:

```rust
NPR_FACE_ID_SHADER
NPR_PROJECT_VERTICES_SHADER
NPR_CLASSIFY_EDGES_SHADER
NPR_COMPACT_OWNERS_SHADER
NPR_BUILD_STROKES_SHADER
```

Wniosek do dokumentu: trzeba dopisać includy dla nowych WGSL.

Plik: `mods/playground-npr/scenes/comic-lines/scene.yml`

Aktualne fakty:

- `npr_presets:` zaczyna się od `default-gpu-comic.yml`.
- Presety CPU-reference istnieją obok GPU.
- Input map ma `G`, `7`, `8`, `9`, `0`, ale nie ma jeszcze dedykowanego `V` dla GPU debug cycle.
- HUD ma teksty `Strategy`, `Debug`, ale debug nie jest jeszcze jawnie spięty z `gpu_realtime_tuning.debug_mode`.

Plik: `mods/playground-npr/scenes/comic-lines/scene.rhai`

Aktualne symbole:

- `apply_active_npr_preset(world_api)` około linii 159+.
- `toggle_npr_strategy(world_api)` około linii 222+.
- `set_npr_debug_view(world_api, view_id, label)` około linii 230+.
- input obsługuje debug overlay `7/8/9/0`, ale nie ma `cycle_npr_gpu_debug_mode`.

Wniosek do dokumentu: trzeba dopisać API i script bridge dla `V`, inaczej debug mode będzie tylko YAML-owy i statyczny.

---

## 2. Korekta błędnej diagnozy z wersji 1 dokumentu

Poprzedni dokument mówił, że `npr_compact_owners.wgsl` aktywnie psuje finalny render. To trzeba zapisać precyzyjniej.

Stan aktualny:

1. `npr_compact_owners.wgsl` i `compact_owners_pipeline` istnieją.
2. `GpuNprPathLink3d` i `path_links` istnieją.
3. Normalna kolejność dispatchy w `gpu_realtime.rs` nie odpala obecnie `compact_owners_pipeline`.
4. Problem finalnego renderu wynika głównie z tego, że `npr_build_strokes.wgsl` nadal czyta `visible_segments` i robi edge-local/chain-local stylizację zamiast path-level `NprStrokePath`.
5. Stare `next_a/next_b/alt_next_a/alt_next_b` nadal są w typach i shaderach, więc mogą wracać jako heurystyka. Trzeba je usunąć z normalnej ścieżki albo oznaczyć jako nieautorytatywne hinty debug.

Zamiana treści w dokumencie:

```text
BYŁO:
  compact owners i stary build strokes łączą krawędzie przez heurystyki.

MA BYĆ:
  compact owners istnieje jako stary pipeline, ale normalna ścieżka go nie dispatchuje.
  Aktywny problem to brak GPU path graphu: build_strokes pracuje na visible_segments,
  czyli na edge-local fragmentach, nie na path_segments z path_id/path_t/path_length.
```

---

## 3. Docelowy model architektury

CPU reference pozostaje modelem semantycznym:

```text
mesh topology
  -> projected vertices
  -> face visibility
  -> visible line fragments
  -> fragments grouped by NprLineKind
  -> screen-space endpoint bins
  -> path walk
  -> NprStrokePath
  -> simplify/resample
  -> path-space stroke style
  -> stroke segments
```

GPU realtime ma odtworzyć ten sam model:

```text
uploaded topology buffers
  -> npr_reset_frame.wgsl
  -> npr_face_id.wgsl
  -> npr_project_vertices.wgsl
  -> npr_classify_edges.wgsl
  -> npr_emit_endpoint_refs.wgsl
  -> npr_endpoint_bins.wgsl
  -> npr_connect_paths.wgsl
  -> npr_emit_path_segments.wgsl
  -> npr_build_strokes.wgsl
  -> draw_indirect
```

Zasada:

```text
CPU and GPU can differ in execution.
CPU and GPU must not differ in authored meaning.
```

Nie robić:

```text
visible edge -> local t -> final stroke
```

Robić:

```text
visible edge -> endpoint refs -> endpoint bins -> path ownership -> path segment -> path t -> final stroke
```

---

## 4. Pakiet A - zamrożenie referencji CPU

### READ

Plik:

- `crates/engine/render-wgpu/src/renderer/world_3d.rs`

Symbole do traktowania jako reference model:

- `build_npr_stroke_paths_for_mesh`
- `collect_npr_edge_fragments_for_mesh`
- `build_npr_stroke_paths`
- `build_npr_stroke_paths_for_kind`
- `walk_npr_path`
- `npr_point_key`
- `npr_path_join_score`
- `append_npr_styled_path_vertices`

### ADD

Plik:

- `crates/engine/render-wgpu/src/renderer/npr/parity_notes.rs`

Dodać plik modułowy z dokumentacją invariants:

```rust
//! NPR CPU/GPU parity notes.
//!
//! CPU reference is the semantic source of truth. GPU realtime may use a
//! different execution model, but must preserve authored meaning:
//!
//! CPU NprLineFragment -> GPU GpuNprVisibleSegment3d
//! CPU npr_point_key -> GPU endpoint key
//! CPU walk_npr_path -> GPU path owner / path id
//! CPU NprStrokePath.arc_lengths_px -> GPU path t and path_length_px
//! CPU append_npr_styled_path_vertices -> GPU npr_build_strokes.wgsl
//!
//! Required invariants:
//! - path grouping is per NprLineKind;
//! - path continuation is screen-space, not object-space only;
//! - silhouette has priority over feature/detail lines;
//! - dropout/search never destroy silhouette readability;
//! - no GPU->CPU readback in normal realtime;
//! - no fallback from gpu_realtime to cpu_reference.
```

### MODIFY

Plik:

- `crates/engine/render-wgpu/src/renderer/npr/mod.rs`

Dodać:

```rust
mod parity_notes;
```

Jeżeli compiler ostrzega przez pusty moduł, dodać atrybut:

```rust
#[allow(dead_code)]
mod parity_notes;
```

### WALIDACJA

```powershell
cargo check -p amigo-render-wgpu
```

Czego nie zmieniać:

- nie zmieniać zachowania CPU reference;
- nie przepisywać CPU;
- nie usuwać `world_3d.rs` jako referencji.

---

## 5. Pakiet B - pełny kontrakt debug mode w Render API

### MODIFY

Plik:

- `crates/engine/render-api/src/commands_3d.rs`

Lokalizacja:

- po `NprFillMode3d` około linii 70-86;
- przed `NprGpuRealtimeTuning3d` około linii 88.

Dodać enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprGpuRealtimeDebugMode3d {
    #[default]
    Final,
    RawFragmentsByKind,
    EndpointBins,
    PathOwners,
    PathSegments,
}

impl NprGpuRealtimeDebugMode3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Final => "final",
            Self::RawFragmentsByKind => "raw_fragments_by_kind",
            Self::EndpointBins => "endpoint_bins",
            Self::PathOwners => "path_owners",
            Self::PathSegments => "path_segments",
        }
    }

    pub fn to_gpu_u32(self) -> u32 {
        match self {
            Self::Final => 0,
            Self::RawFragmentsByKind => 1,
            Self::EndpointBins => 2,
            Self::PathOwners => 3,
            Self::PathSegments => 4,
        }
    }
}
```

Następnie zmienić `NprGpuRealtimeTuning3d`.

Było około linii 88-101:

```rust
pub struct NprGpuRealtimeTuning3d {
    pub max_render_length_px: f32,
    ...
    pub silhouette_min_length_multiplier: f32,
}
```

Ma być:

```rust
pub struct NprGpuRealtimeTuning3d {
    pub debug_mode: NprGpuRealtimeDebugMode3d,
    pub max_render_length_px: f32,
    pub max_segment_length_px: f32,
    pub max_terminal_walk_edges: u32,
    pub max_chained_walk_edges: u32,
    pub max_chain_angle_degrees: f32,
    pub search_enabled: bool,
    pub search_max_render_length_px: f32,
    pub search_alpha_multiplier: f32,
    pub feature_min_length_multiplier: f32,
    pub feature_alpha_multiplier: f32,
    pub silhouette_min_length_multiplier: f32,
    pub endpoint_bin_capacity_multiplier: f32,
    pub path_segment_capacity_multiplier: f32,
    pub max_path_walk_segments: u32,
}
```

W `Default for NprGpuRealtimeTuning3d` dodać:

```rust
debug_mode: NprGpuRealtimeDebugMode3d::Final,
endpoint_bin_capacity_multiplier: 2.0,
path_segment_capacity_multiplier: 2.0,
max_path_walk_segments: 64,
```

W `rough_comic_experimental()` dodać:

```rust
debug_mode: NprGpuRealtimeDebugMode3d::Final,
endpoint_bin_capacity_multiplier: 3.0,
path_segment_capacity_multiplier: 3.0,
max_path_walk_segments: 96,
```

W `normalized(mut self)` dodać przed `self`:

```rust
self.endpoint_bin_capacity_multiplier = if self.endpoint_bin_capacity_multiplier.is_finite() {
    self.endpoint_bin_capacity_multiplier.clamp(1.0, 8.0)
} else {
    2.0
};
self.path_segment_capacity_multiplier = if self.path_segment_capacity_multiplier.is_finite() {
    self.path_segment_capacity_multiplier.clamp(1.0, 8.0)
} else {
    2.0
};
self.max_path_walk_segments = self.max_path_walk_segments.clamp(1, 512);
```

### ADD TESTS

Ten sam plik, w istniejącym module testów około końca pliku, dodać:

```rust
#[test]
fn default_npr_gpu_debug_mode_is_final() {
    assert_eq!(
        NprLineSettings3d::default()
            .gpu_realtime_tuning
            .debug_mode,
        NprGpuRealtimeDebugMode3d::Final
    );
}

#[test]
fn npr_gpu_debug_mode_labels_are_yaml_safe() {
    assert_eq!(NprGpuRealtimeDebugMode3d::Final.as_str(), "final");
    assert_eq!(
        NprGpuRealtimeDebugMode3d::RawFragmentsByKind.as_str(),
        "raw_fragments_by_kind"
    );
    assert_eq!(NprGpuRealtimeDebugMode3d::EndpointBins.as_str(), "endpoint_bins");
    assert_eq!(NprGpuRealtimeDebugMode3d::PathOwners.as_str(), "path_owners");
    assert_eq!(NprGpuRealtimeDebugMode3d::PathSegments.as_str(), "path_segments");
}
```

### WALIDACJA

```powershell
cargo check -p amigo-render-api
```

---

## 6. Pakiet C - YAML document + hydration dla debug mode

### MODIFY

Plik:

- `crates/engine/scene/src/document/components.rs`

Lokalizacja:

- `NprGpuRealtimeTuningDocument` około linii 194-218.

Dodać na początku structa:

```rust
#[serde(default)]
pub debug_mode: Option<String>,
```

Dodać też capacity tuning:

```rust
#[serde(default)]
pub endpoint_bin_capacity_multiplier: Option<f32>,
#[serde(default)]
pub path_segment_capacity_multiplier: Option<f32>,
#[serde(default)]
pub max_path_walk_segments: Option<u32>,
```

Po zmianie struct ma mieć co najmniej:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprGpuRealtimeTuningDocument {
    #[serde(default)]
    pub debug_mode: Option<String>,
    #[serde(default)]
    pub max_render_length_px: Option<f32>,
    #[serde(default)]
    pub max_segment_length_px: Option<f32>,
    #[serde(default)]
    pub max_terminal_walk_edges: Option<u32>,
    #[serde(default)]
    pub max_chained_walk_edges: Option<u32>,
    #[serde(default)]
    pub max_chain_angle_degrees: Option<f32>,
    #[serde(default)]
    pub search_enabled: Option<bool>,
    #[serde(default)]
    pub search_max_render_length_px: Option<f32>,
    #[serde(default)]
    pub search_alpha_multiplier: Option<f32>,
    #[serde(default)]
    pub feature_min_length_multiplier: Option<f32>,
    #[serde(default)]
    pub feature_alpha_multiplier: Option<f32>,
    #[serde(default)]
    pub silhouette_min_length_multiplier: Option<f32>,
    #[serde(default)]
    pub endpoint_bin_capacity_multiplier: Option<f32>,
    #[serde(default)]
    pub path_segment_capacity_multiplier: Option<f32>,
    #[serde(default)]
    pub max_path_walk_segments: Option<u32>,
}
```

### MODIFY

Plik:

- `crates/engine/scene/src/hydration/plan/components_domains.rs`

Problem aktualny:

- `apply_npr_gpu_realtime_tuning_3d(...)` około linii 935-974 zwraca `()`, więc nie może zwrócić błędu dla nieznanego `debug_mode`.

Zmienić sygnaturę:

Było:

```rust
fn apply_npr_gpu_realtime_tuning_3d(
    document: &crate::document::NprGpuRealtimeTuningDocument,
    resolved: &mut amigo_render_api::NprLineSettings3d,
) {
```

Ma być:

```rust
fn apply_npr_gpu_realtime_tuning_3d(
    document: &crate::document::NprGpuRealtimeTuningDocument,
    resolved: &mut amigo_render_api::NprLineSettings3d,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<()> {
```

W miejscu wywołania około linii 750 zmienić:

Było:

```rust
if let Some(gpu_realtime_tuning) = settings.gpu_realtime_tuning.as_ref() {
    apply_npr_gpu_realtime_tuning_3d(gpu_realtime_tuning, &mut resolved);
}
```

Ma być:

```rust
if let Some(gpu_realtime_tuning) = settings.gpu_realtime_tuning.as_ref() {
    apply_npr_gpu_realtime_tuning_3d(
        gpu_realtime_tuning,
        &mut resolved,
        scene_id,
        entity_id,
        component_kind,
    )?;
}
```

Na początku funkcji `apply_npr_gpu_realtime_tuning_3d(...)` dodać:

```rust
if let Some(value) = document.debug_mode.as_deref() {
    tuning.debug_mode = npr_gpu_realtime_debug_mode_3d_from_document(
        value,
        scene_id,
        entity_id,
        component_kind,
    )?;
}
```

Na końcu przed normalizacją dodać:

```rust
if let Some(value) = document.endpoint_bin_capacity_multiplier {
    tuning.endpoint_bin_capacity_multiplier = value;
}
if let Some(value) = document.path_segment_capacity_multiplier {
    tuning.path_segment_capacity_multiplier = value;
}
if let Some(value) = document.max_path_walk_segments {
    tuning.max_path_walk_segments = value;
}
resolved.gpu_realtime_tuning = tuning.normalized();
Ok(())
```

Dodać nową funkcję obok `npr_render_strategy_3d_from_document(...)`:

```rust
fn npr_gpu_realtime_debug_mode_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprGpuRealtimeDebugMode3d> {
    match value.trim() {
        "final" => Ok(amigo_render_api::NprGpuRealtimeDebugMode3d::Final),
        "raw_fragments_by_kind" => Ok(amigo_render_api::NprGpuRealtimeDebugMode3d::RawFragmentsByKind),
        "endpoint_bins" => Ok(amigo_render_api::NprGpuRealtimeDebugMode3d::EndpointBins),
        "path_owners" => Ok(amigo_render_api::NprGpuRealtimeDebugMode3d::PathOwners),
        "path_segments" => Ok(amigo_render_api::NprGpuRealtimeDebugMode3d::PathSegments),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.gpu_realtime_tuning.debug_mode `{other}`; expected `final`, `raw_fragments_by_kind`, `endpoint_bins`, `path_owners`, or `path_segments`"
            ),
        }),
    }
}
```

### MODIFY PRESETS

Pliki:

- `mods/playground-npr/scenes/comic-lines/npr-presets/default-gpu-comic.yml`
- opcjonalnie wszystkie `*-cpu-reference.yml` i GPU presety.

Dodać do `gpu_realtime_tuning`:

```yaml
gpu_realtime_tuning:
  debug_mode: final
  endpoint_bin_capacity_multiplier: 2.0
  path_segment_capacity_multiplier: 2.0
  max_path_walk_segments: 64
```

Dla rough/pencil/brush presetów można zwiększyć:

```yaml
endpoint_bin_capacity_multiplier: 3.0
path_segment_capacity_multiplier: 3.0
max_path_walk_segments: 96
```

### WALIDACJA

```powershell
cargo check -p amigo-scene
cargo check -p amigo-render-api
```

---

## 7. Pakiet D - bridge Rhai/script dla GPU debug cycle

Samo YAML `debug_mode` nie wystarcza. Playground ma mieć klawisz `V`, który przełącza tryby bez reloadu sceny.

### MODIFY

Plik:

- `crates/3d/mesh/src/lib.rs`

Lokalizacja:

- `impl MeshSceneService`, po `set_npr_temporal_path_smoothing(...)` około linii 102-118.

Dodać:

```rust
pub fn set_npr_gpu_debug_mode(
    &self,
    entity_name: &str,
    debug_mode: amigo_render_api::NprGpuRealtimeDebugMode3d,
) -> bool {
    let mut commands = self
        .commands
        .lock()
        .expect("mesh scene service mutex should not be poisoned");
    let Some(command) = commands
        .iter_mut()
        .find(|command| command.entity_name == entity_name)
    else {
        return false;
    };
    let Some(npr) = command.mesh.npr.as_mut() else {
        return false;
    };
    npr.gpu_realtime_tuning.debug_mode = debug_mode;
    true
}
```

### MODIFY

Plik:

- `crates/3d/mesh/src/script_command.rs`

Lokalizacja:

- enum `Mesh3dScriptCommandOutcome` około linii 13-25.

Dodać wariant:

```rust
SetNprGpuDebugMode {
    entity_name: String,
    debug_mode: amigo_render_api::NprGpuRealtimeDebugMode3d,
},
```

Dodać helper funkcję nad `handle_mesh3d_script_command(...)`:

```rust
fn parse_npr_gpu_debug_mode(value: &str) -> Option<amigo_render_api::NprGpuRealtimeDebugMode3d> {
    match value {
        "final" => Some(amigo_render_api::NprGpuRealtimeDebugMode3d::Final),
        "raw_fragments_by_kind" => Some(amigo_render_api::NprGpuRealtimeDebugMode3d::RawFragmentsByKind),
        "endpoint_bins" => Some(amigo_render_api::NprGpuRealtimeDebugMode3d::EndpointBins),
        "path_owners" => Some(amigo_render_api::NprGpuRealtimeDebugMode3d::PathOwners),
        "path_segments" => Some(amigo_render_api::NprGpuRealtimeDebugMode3d::PathSegments),
        _ => None,
    }
}
```

W `match (command.name.as_str(), command.arguments.as_slice())` dodać przed `_`:

```rust
("set_npr_gpu_debug_mode", [entity_name, debug_mode]) => {
    let Some(mesh_scene_service) = ctx.mesh_scene_service else {
        return Mesh3dScriptCommandOutcome::Unhandled;
    };
    let Some(debug_mode) = parse_npr_gpu_debug_mode(debug_mode) else {
        return Mesh3dScriptCommandOutcome::Unhandled;
    };
    if mesh_scene_service.set_npr_gpu_debug_mode(entity_name, debug_mode) {
        Mesh3dScriptCommandOutcome::SetNprGpuDebugMode {
            entity_name: entity_name.clone(),
            debug_mode,
        }
    } else {
        Mesh3dScriptCommandOutcome::Unhandled
    }
}
```

W `can_handle(...)` około linii 88-94 dodać:

```rust
|| (command.name == "set_npr_gpu_debug_mode" && command.arguments.len() == 2)
```

W `handle(...)` około linii 106-111 dodać:

```rust
Mesh3dScriptCommandOutcome::SetNprGpuDebugMode { .. } => {}
```

### MODIFY

Plik:

- `mods/playground-npr/scenes/comic-lines/scene.yml`

Input map około linii 85-96 dodać:

```yaml
npr.gpu_debug_cycle:
  kind: key
  key: V
```

HUD help około linii 590 zmienić z:

```yaml
text: "1-6 model | Up/Down style | G strategy | F camera | R rotate | T temporal"
```

na:

```yaml
text: "1-6 model | Up/Down style | G strategy | V GPU debug | F camera | R rotate | T temporal"
```

### MODIFY

Plik:

- `mods/playground-npr/scenes/comic-lines/scene.rhai`

Dodać helpery obok `toggle_npr_strategy(world_api)` około linii 222:

```rhai
fn npr_gpu_debug_mode_count() { 5 }

fn npr_gpu_debug_mode_id(index) {
    if index == 1 { return "raw_fragments_by_kind"; }
    if index == 2 { return "endpoint_bins"; }
    if index == 3 { return "path_owners"; }
    if index == 4 { return "path_segments"; }
    "final"
}

fn npr_gpu_debug_mode_label(index) {
    if index == 1 { return "Raw Fragments"; }
    if index == 2 { return "Endpoint Bins"; }
    if index == 3 { return "Path Owners"; }
    if index == 4 { return "Path Segments"; }
    "Final"
}

fn apply_npr_gpu_debug_mode(world_api) {
    let index = world_api.state.get_int("npr_gpu_debug_mode_index");
    let mode = npr_gpu_debug_mode_id(index);
    let entity_name = model_entity_name(world_api.state.get_int("active_npr_model"));
    world_api.mesh3d.set_npr_gpu_debug_mode(entity_name, mode);
    world_api.state.set_string("npr_gpu_debug_mode", mode);
    world_api.dev.event("playground-npr.gpu-debug-mode", mode);
}

fn cycle_npr_gpu_debug_mode(world_api) {
    let next = (world_api.state.get_int("npr_gpu_debug_mode_index") + 1) % npr_gpu_debug_mode_count();
    world_api.state.set_int("npr_gpu_debug_mode_index", next);
    apply_npr_gpu_debug_mode(world_api);
}
```

W `on_scene_loaded(...)` około linii 253-259 dodać:

```rhai
world.state.set_int("npr_gpu_debug_mode_index", 0);
world.state.set_string("npr_gpu_debug_mode", "final");
```

Po `apply_active_npr_preset(world);` dopisać:

```rhai
apply_npr_gpu_debug_mode(world);
```

W `update(...)` po obsłudze `npr.strategy_toggle` około linii 283 dodać:

```rhai
if input::pressed(world, "npr.gpu_debug_cycle") { cycle_npr_gpu_debug_mode(world); }
```

W `apply_active_npr_preset(world_api)` po `world_api.mesh3d.apply_npr_preset(...)` dodać:

```rhai
apply_npr_gpu_debug_mode(world_api);
```

### WALIDACJA

```powershell
cargo check -p amigo-3d-mesh
cargo test -p amigo-app playground_npr_comic_lines_bootstrap_applies_gpu_realtime_preset_to_soldier
```

Manual:

```powershell
cargo run
```

Sprawdzić:

- `V` zmienia state `npr_gpu_debug_mode`;
- aktywny model dostaje nowy debug mode;
- zmiana modelu i presetu zachowuje aktualny debug mode.

---

## 8. Pakiet E - GPU types: endpoint refs, endpoint bins, path states, path segments

### MODIFY

Plik:

- `crates/engine/render-wgpu/src/renderer/npr/gpu_types.rs`

Lokalizacja:

- po `GpuNprVisibleSegment3d` około linii 48-55;
- przed `GpuNprPathLink3d` około linii 56.

Dodać:

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprEndpointRef3d {
    pub key: [i32; 2],
    pub segment_index: u32,
    pub endpoint: u32, // 0=start, 1=end
    pub kind: u32,
    pub flags: u32,
    pub _pad0: [u32; 2],
}

unsafe impl bytemuck::Zeroable for GpuNprEndpointRef3d {}
unsafe impl bytemuck::Pod for GpuNprEndpointRef3d {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprEndpointBin3d {
    pub key: [i32; 2],
    pub first_ref: u32,
    pub ref_count: u32,
    pub kind: u32,
    pub flags: u32,
    pub _pad0: [u32; 2],
}

unsafe impl bytemuck::Zeroable for GpuNprEndpointBin3d {}
unsafe impl bytemuck::Pod for GpuNprEndpointBin3d {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprPathState3d {
    pub owner_segment: u32,
    pub path_id: u32,
    pub kind: u32,
    pub flags: u32,
    pub segment_count: u32,
    pub _pad0: [u32; 3],
}

unsafe impl bytemuck::Zeroable for GpuNprPathState3d {}
unsafe impl bytemuck::Pod for GpuNprPathState3d {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprPathSegment3d {
    pub start: [f32; 4],      // xy screen, z depth, w valid
    pub end: [f32; 4],        // xy screen, z depth, w valid
    pub path: [u32; 4],       // path_id, kind, source_segment, flags
    pub metrics: [f32; 4],    // t0, t1, path_length_px, importance
}

unsafe impl bytemuck::Zeroable for GpuNprPathSegment3d {}
unsafe impl bytemuck::Pod for GpuNprPathSegment3d {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprFrameCounters3d {
    pub endpoint_ref_count: u32,
    pub endpoint_bin_count: u32,
    pub path_state_count: u32,
    pub path_segment_count: u32,
    pub stroke_segment_count: u32,
    pub _pad0: [u32; 3],
}

unsafe impl bytemuck::Zeroable for GpuNprFrameCounters3d {}
unsafe impl bytemuck::Pod for GpuNprFrameCounters3d {}
```

### MODIFY `GpuNprFrameUniforms3d`

W aktualnym `GpuNprFrameUniforms3d` około linii 65+ dodać albo wykorzystać nowe `params17`.

Najbezpieczniej dodać na końcu przed `ink_color`:

```rust
pub params17: [f32; 4], // x debug_mode, y endpoint_bin_capacity, z path_segment_capacity, w max_path_walk_segments
```

Po stronie WGSL każdy shader ma mieć identyczny layout:

```wgsl
params17: vec4<f32>,
```

Nie wciskać debug mode w istniejący `overlay_mode`, bo overlay debug i GPU debug to różne pojęcia.

### MODIFY `uniforms_for_job(...)`

Plik:

- `crates/engine/render-wgpu/src/renderer/npr/gpu_realtime.rs`

Lokalizacja:

- `uniforms_for_job(...)` około linii 330+.

Po `gpu_tuning = settings.gpu_realtime_tuning.normalized();` wartości do uniformów:

```rust
params17: [
    gpu_tuning.debug_mode.to_gpu_u32() as f32,
    gpu_tuning.endpoint_bin_capacity_multiplier,
    gpu_tuning.path_segment_capacity_multiplier,
    gpu_tuning.max_path_walk_segments as f32,
],
```

### WALIDACJA

```powershell
cargo check -p amigo-render-wgpu
```

---

## 9. Pakiet F - GPU buffers i bind layout

### MODIFY

Plik:

- `crates/engine/render-wgpu/src/renderer/npr/gpu_buffers.rs`

Aktualny `NprGpuFrameBuffers3d` ma mniej więcej:

```rust
projected_vertices
visible_segments
path_links
stroke_segments
indirect_args
uniforms
projected_vertices_capacity
visible_segments_capacity
path_links_capacity
stroke_segments_capacity
```

Dodać pola:

```rust
pub endpoint_refs: wgpu::Buffer,
pub endpoint_bins: wgpu::Buffer,
pub path_states: wgpu::Buffer,
pub path_segments: wgpu::Buffer,
pub counters: wgpu::Buffer,
pub endpoint_refs_capacity: u64,
pub endpoint_bins_capacity: u64,
pub path_states_capacity: u64,
pub path_segments_capacity: u64,
pub counters_capacity: u64,
```

### MODIFY `ensure_frame_buffers(...)`

Zmień sygnaturę:

Było:

```rust
ensure_frame_buffers(
    device,
    projected_capacity,
    visible_segments_capacity,
    path_links_capacity,
    stroke_segments_capacity,
)
```

Ma być:

```rust
ensure_frame_buffers(
    device,
    projected_capacity,
    visible_segments_capacity,
    endpoint_refs_capacity,
    endpoint_bins_capacity,
    path_states_capacity,
    path_segments_capacity,
    stroke_segments_capacity,
)
```

Warunek realokacji rozszerzyć o wszystkie nowe capacity.

### MODIFY `create_frame_buffers(...)`

Dodać create buffers:

```rust
endpoint_refs: create_empty_buffer(device, "amigo-npr-endpoint-refs", endpoint_refs_capacity),
endpoint_bins: create_empty_buffer(device, "amigo-npr-endpoint-bins", endpoint_bins_capacity),
path_states: create_empty_buffer(device, "amigo-npr-path-states", path_states_capacity),
path_segments: create_empty_buffer(device, "amigo-npr-path-segments", path_segments_capacity),
counters: create_empty_buffer(
    device,
    "amigo-npr-frame-counters",
    std::mem::size_of::<GpuNprFrameCounters3d>() as u64,
),
```

### MODIFY `frame_buffer_capacity_bytes(...)`

Doliczyć nowe capacities:

```rust
+ buffers.endpoint_refs_capacity
+ buffers.endpoint_bins_capacity
+ buffers.path_states_capacity
+ buffers.path_segments_capacity
+ buffers.counters_capacity
```

### MODIFY

Plik:

- `crates/engine/render-wgpu/src/renderer/npr/gpu_pipelines.rs`

Aktualny `compute_bind_group_layout` ma bindy 0,1,2,3,4,5,6,8,9,10.

Rozszerzyć entries:

```rust
entries: &[
    storage_entry(0, true),  // vertices
    storage_entry(1, true),  // triangles
    storage_entry(2, true),  // edges
    storage_entry(3, false), // projected_vertices
    texture_entry(4),        // face_id
    storage_entry(5, false), // visible_segments
    storage_entry(6, false), // stroke_segments
    uniform_entry(8),        // uniforms
    storage_entry(9, false), // indirect_args
    storage_entry(10, false),// old path_links, temporary
    storage_entry(11, false),// endpoint_refs
    storage_entry(12, false),// endpoint_bins
    storage_entry(13, false),// path_states
    storage_entry(14, false),// path_segments
    storage_entry(15, false),// counters
]
```

Wszystkie nowe WGSL muszą mieć ten sam layout bind group.

### WALIDACJA

```powershell
cargo check -p amigo-render-wgpu
```

---

## 10. Pakiet G - shader include i pipeline expansion

### ADD FILES

Dodać pliki:

```text
crates/engine/render-wgpu/src/renderer/shaders/npr_reset_frame.wgsl
crates/engine/render-wgpu/src/renderer/shaders/npr_emit_endpoint_refs.wgsl
crates/engine/render-wgpu/src/renderer/shaders/npr_endpoint_bins.wgsl
crates/engine/render-wgpu/src/renderer/shaders/npr_connect_paths.wgsl
crates/engine/render-wgpu/src/renderer/shaders/npr_emit_path_segments.wgsl
```

### MODIFY

Plik:

- `crates/engine/render-wgpu/src/renderer/shaders/mod.rs`

Dodać includy:

```rust
pub(crate) const NPR_RESET_FRAME_SHADER: &str = include_str!("npr_reset_frame.wgsl");
pub(crate) const NPR_EMIT_ENDPOINT_REFS_SHADER: &str = include_str!("npr_emit_endpoint_refs.wgsl");
pub(crate) const NPR_ENDPOINT_BINS_SHADER: &str = include_str!("npr_endpoint_bins.wgsl");
pub(crate) const NPR_CONNECT_PATHS_SHADER: &str = include_str!("npr_connect_paths.wgsl");
pub(crate) const NPR_EMIT_PATH_SEGMENTS_SHADER: &str = include_str!("npr_emit_path_segments.wgsl");
```

`NPR_COMPACT_OWNERS_SHADER` zostaje tymczasowo, ale ma komentarz:

```rust
// Legacy chain experiment. Do not dispatch in normal gpu_realtime path.
```

### MODIFY

Plik:

- `crates/engine/render-wgpu/src/renderer/npr/gpu_pipelines.rs`

Import:

```rust
use crate::renderer::shaders::{
    NPR_BUILD_STROKES_SHADER,
    NPR_CLASSIFY_EDGES_SHADER,
    NPR_COMPACT_OWNERS_SHADER,
    NPR_CONNECT_PATHS_SHADER,
    NPR_EMIT_ENDPOINT_REFS_SHADER,
    NPR_EMIT_PATH_SEGMENTS_SHADER,
    NPR_ENDPOINT_BINS_SHADER,
    NPR_FACE_ID_SHADER,
    NPR_PROJECT_VERTICES_SHADER,
    NPR_RESET_FRAME_SHADER,
};
```

Struktura:

```rust
pub(crate) struct NprGpuPipelines3d {
    pub face_id_pipeline: wgpu::RenderPipeline,
    pub face_id_bind_group_layout: wgpu::BindGroupLayout,
    pub compute_bind_group_layout: wgpu::BindGroupLayout,
    pub reset_frame_pipeline: wgpu::ComputePipeline,
    pub project_vertices_pipeline: wgpu::ComputePipeline,
    pub classify_edges_pipeline: wgpu::ComputePipeline,
    pub emit_endpoint_refs_pipeline: wgpu::ComputePipeline,
    pub endpoint_bins_pipeline: wgpu::ComputePipeline,
    pub connect_paths_pipeline: wgpu::ComputePipeline,
    pub emit_path_segments_pipeline: wgpu::ComputePipeline,
    pub build_strokes_pipeline: wgpu::ComputePipeline,
    pub compact_owners_pipeline: wgpu::ComputePipeline, // legacy, not normal dispatch
}
```

W `create(...)` dodać pipeline creation:

```rust
let reset_frame_pipeline = create_compute_pipeline(
    device,
    "amigo-npr-reset-frame-pipeline",
    NPR_RESET_FRAME_SHADER,
    &compute_layout,
);
let emit_endpoint_refs_pipeline = create_compute_pipeline(
    device,
    "amigo-npr-emit-endpoint-refs-pipeline",
    NPR_EMIT_ENDPOINT_REFS_SHADER,
    &compute_layout,
);
let endpoint_bins_pipeline = create_compute_pipeline(
    device,
    "amigo-npr-endpoint-bins-pipeline",
    NPR_ENDPOINT_BINS_SHADER,
    &compute_layout,
);
let connect_paths_pipeline = create_compute_pipeline(
    device,
    "amigo-npr-connect-paths-pipeline",
    NPR_CONNECT_PATHS_SHADER,
    &compute_layout,
);
let emit_path_segments_pipeline = create_compute_pipeline(
    device,
    "amigo-npr-emit-path-segments-pipeline",
    NPR_EMIT_PATH_SEGMENTS_SHADER,
    &compute_layout,
);
```

### WALIDACJA

```powershell
cargo check -p amigo-render-wgpu
```

---

## 11. Pakiet H - reset pass, counters i no stale data

### ADD

Plik:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_reset_frame.wgsl`

Cel:

- wyzerować counters;
- wyczyścić indirect args;
- wyczyścić endpoint bins;
- nie zostawiać danych z poprzedniej klatki.

Szkielet WGSL:

```wgsl
struct GpuNprFrameCounters3d {
    endpoint_ref_count: atomic<u32>,
    endpoint_bin_count: atomic<u32>,
    path_state_count: atomic<u32>,
    path_segment_count: atomic<u32>,
    stroke_segment_count: atomic<u32>,
    _pad0: vec3<u32>,
}

struct GpuNprEndpointBin3d {
    key: vec2<i32>,
    first_ref: u32,
    ref_count: u32,
    kind: u32,
    flags: u32,
    _pad0: vec2<u32>,
}

@group(0) @binding(9) var<storage, read_write> indirect_args: array<atomic<u32>>;
@group(0) @binding(12) var<storage, read_write> endpoint_bins: array<GpuNprEndpointBin3d>;
@group(0) @binding(15) var<storage, read_write> counters: GpuNprFrameCounters3d;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;

    if (i == 0u) {
        atomicStore(&counters.endpoint_ref_count, 0u);
        atomicStore(&counters.endpoint_bin_count, 0u);
        atomicStore(&counters.path_state_count, 0u);
        atomicStore(&counters.path_segment_count, 0u);
        atomicStore(&counters.stroke_segment_count, 0u);
        atomicStore(&indirect_args[0], 6u);
        atomicStore(&indirect_args[1], 0u);
        atomicStore(&indirect_args[2], 0u);
        atomicStore(&indirect_args[3], 0u);
    }

    if (i < arrayLength(&endpoint_bins)) {
        endpoint_bins[i] = GpuNprEndpointBin3d(
            vec2<i32>(2147483647, 2147483647),
            0xffffffffu,
            0u,
            0u,
            0u,
            vec2<u32>(0u, 0u),
        );
    }
}
```

### MODIFY

Plik:

- `crates/engine/render-wgpu/src/renderer/npr/gpu_realtime.rs`

W `execute(...)` po stworzeniu bind group, przed `project_vertices`, dodać pass:

```rust
{
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("amigo-npr-reset-frame-pass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipelines.reset_frame_pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    let bins = frame_buffers.endpoint_bins_capacity
        / std::mem::size_of::<super::GpuNprEndpointBin3d>() as u64;
    pass.dispatch_workgroups(workgroup_count(bins as usize), 1, 1);
}
```

Uwaga: reset per mesh teraz działa tylko dla per-job pipeline. Przy frame-level multi-mesh w pakiecie O reset ma być jeden raz na frame.

### WALIDACJA

```powershell
cargo check -p amigo-render-wgpu
```

---

## 12. Pakiet I - visible fragments jako mierzona baza

### MODIFY

Plik:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_classify_edges.wgsl`

Docelowa odpowiedzialność:

1. Projected edge endpoints są gotowe.
2. Shader wybiera `kind` zgodnie z CPU:
   - boundary,
   - silhouette,
   - seam,
   - crease,
   - feature/suggestive,
   - contact.
3. Shader pobiera visible run z face-id texture.
4. Shader zapisuje fragment do `visible_segments`.
5. Nie emituje segmentów dłuższych niż dopuszcza `max_render_length_px`; zbyt długie fragmenty są skracane albo odrzucane jako debug warning path.

### REQUIREMENT

`GpuNprVisibleSegment3d.kind_edge` ma mieć stabilne znaczenie:

```wgsl
kind_edge.x = kind
kind_edge.y = edge_index albo edge_id
kind_edge.z = vertex_a
kind_edge.w = flags
```

Jeśli obecny kod używa `kind_edge.w` jako tymczasowego debug/owner flag, trzeba przenieść to do `path_states.flags` w pakiecie K.

### DEBUG MODE RAW FRAGMENTS

W `npr_build_strokes.wgsl` lub osobnym `npr_debug_fragments.wgsl`, dla `debug_mode == RawFragmentsByKind` renderować bez path linking:

```text
visible_segments -> stroke_segments
```

Kolorowanie:

```text
silhouette: black
boundary: dark blue
crease: dark red
seam: dark green
feature: gray
contact: brown
```

W tym trybie acceptance:

- brak ogromnych losowych linii;
- kształt modelu czytelny;
- każdy segment odpowiada realnemu edge/visible run.

### WALIDACJA

```powershell
cargo check -p amigo-render-wgpu
cargo test -p amigo-app playground_npr_preview_renders_gpu_and_cpu_reference_default_gpu_comic -- --ignored
```

---

## 13. Pakiet J - endpoint refs i endpoint bins na GPU

### ADD

Plik:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_emit_endpoint_refs.wgsl`

Cel:

- każdy valid `visible_segment` emituje 2 endpoint refs;
- refy nie są jeszcze sortowane;
- counter `endpoint_ref_count` rośnie atomicznie.

Szkielet:

```wgsl
fn endpoint_key(point: vec2<f32>) -> vec2<i32> {
    let quant = max(uniforms.params12.w, 0.5);
    return vec2<i32>(i32(round(point.x / quant)), i32(round(point.y / quant)));
}

fn emit_endpoint_ref(segment_index: u32, endpoint: u32, point: vec2<f32>, kind: u32) {
    let out_index = atomicAdd(&counters.endpoint_ref_count, 1u);
    if (out_index >= arrayLength(&endpoint_refs)) {
        return;
    }
    endpoint_refs[out_index] = GpuNprEndpointRef3d(
        endpoint_key(point),
        segment_index,
        endpoint,
        kind,
        0u,
        vec2<u32>(0u, 0u),
    );
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&visible_segments)) { return; }
    let seg = visible_segments[i];
    if (seg.kind_edge.x == KIND_NONE || seg.start.w <= 0.5 || seg.end.w <= 0.5) { return; }
    emit_endpoint_ref(i, 0u, seg.start.xy, seg.kind_edge.x);
    emit_endpoint_ref(i, 1u, seg.end.xy, seg.kind_edge.x);
}
```

### ADD

Plik:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_endpoint_bins.wgsl`

Cel:

- fixed-size hash table z linear probing;
- kluczem jest `(key_x, key_y, kind)`;
- `kind` jest częścią binu, żeby silhouette nie łączyło się z feature.

Szkielet:

```wgsl
fn hash_endpoint_key(key: vec2<i32>, kind: u32) -> u32 {
    var h = u32(key.x) * 73856093u;
    h = h ^ (u32(key.y) * 19349663u);
    h = h ^ (kind * 83492791u);
    return h;
}

fn same_bin(bin: GpuNprEndpointBin3d, key: vec2<i32>, kind: u32) -> bool {
    return bin.key.x == key.x && bin.key.y == key.y && bin.kind == kind;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let ref_index = id.x;
    let count = min(atomicLoad(&counters.endpoint_ref_count), u32(arrayLength(&endpoint_refs)));
    if (ref_index >= count) { return; }

    let endpoint_ref = endpoint_refs[ref_index];
    let capacity = u32(arrayLength(&endpoint_bins));
    if (capacity == 0u) { return; }

    var slot = hash_endpoint_key(endpoint_ref.key, endpoint_ref.kind) % capacity;
    for (var probe: u32 = 0u; probe < 16u; probe = probe + 1u) {
        let bin = endpoint_bins[slot];
        if (bin.first_ref == 0xffffffffu || same_bin(bin, endpoint_ref.key, endpoint_ref.kind)) {
            // v1: first_ref points into endpoint_refs; ref_count is approximate atomic-less unless using atomic struct.
            // Better: endpoint_bins must store atomic ref_count in a separate counters table if exact count is needed.
            endpoint_bins[slot].key = endpoint_ref.key;
            endpoint_bins[slot].kind = endpoint_ref.kind;
            endpoint_bins[slot].first_ref = min(endpoint_bins[slot].first_ref, ref_index);
            endpoint_bins[slot].ref_count = endpoint_bins[slot].ref_count + 1u;
            return;
        }
        slot = (slot + 1u) % capacity;
    }
}
```

Uwaga implementacyjna: WGSL assignment `ref_count = ref_count + 1u` nie jest atomiczny. Dla pełnej poprawności w realnej wersji dodać osobny `endpoint_bin_counts: array<atomic<u32>>` albo trzymać `ref_count` jako przybliżenie debugowe i w `connect_paths` skanować refy po kluczu. Pełny wariant zalecany:

```text
endpoint_bins: metadata
endpoint_bin_counts: array<atomic<u32>>
endpoint_bin_ref_lists: fixed array of u32, np. capacity * 8
```

Jeżeli implementujemy szybko, można na v1 użyć bounded scan po `endpoint_refs` w `connect_paths`, a bin traktować jako debug view. Jeżeli implementujemy finalnie, dodać listę refów per bin.

### REKOMENDOWANY FINALNY LAYOUT BINÓW

Dodać do `gpu_types.rs`:

```rust
#[repr(C)]
pub(crate) struct GpuNprEndpointBinItem3d {
    pub endpoint_ref_index: u32,
    pub _pad0: [u32; 3],
}
```

Dodać buffer:

```rust
endpoint_bin_items: wgpu::Buffer
```

Konfiguracja:

```text
MAX_REFS_PER_BIN = 8
endpoint_bin_items_capacity = endpoint_bins_count * MAX_REFS_PER_BIN
```

Wtedy każdy bin ma:

```rust
first_ref = bin_index * MAX_REFS_PER_BIN
ref_count = atomic count 0..MAX_REFS_PER_BIN
```

### MODIFY `gpu_realtime.rs`

Po `classify_edges` dodać:

```rust
{
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("amigo-npr-emit-endpoint-refs-pass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipelines.emit_endpoint_refs_pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(workgroup_count(topology.edge_count as usize), 1, 1);
}
{
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("amigo-npr-endpoint-bins-pass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipelines.endpoint_bins_pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(workgroup_count((topology.edge_count as usize) * 2), 1, 1);
}
```

### ACCEPTANCE

- endpoint refs = 2 * valid visible segments, z uwzględnieniem capacity;
- `kind` jest częścią klucza;
- debug `endpoint_bins` pokazuje punkty połączeń na modelu, nie losową chmurę;
- brak readbacku w normalnym trybie.

---

## 14. Pakiet K - GPU path ownership i path walk

### ADD

Plik:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_connect_paths.wgsl`

Cel:

- zastąpić edge-object-space `next_a/next_b` jako semantyczne źródło path ownership;
- wyznaczyć path state per visible segment;
- score bazuje na screen-space endpoint gap, line kind, tangent mismatch i depth gap.

Minimalny algorytm v1:

```text
for each visible segment:
  start_key = endpoint_key(start)
  end_key = endpoint_key(end)
  find best continuation at start among same-key same-kind refs
  find best continuation at end among same-key same-kind refs
  choose owner path id as minimum representative segment in connected component approximation
  write GpuNprPathState3d
```

WGSL helpers:

```wgsl
fn segment_dir_from_endpoint(segment_index: u32, endpoint: u32) -> vec2<f32> {
    let seg = visible_segments[segment_index];
    let delta = select(seg.end.xy - seg.start.xy, seg.start.xy - seg.end.xy, endpoint == 1u);
    let len = max(length(delta), 0.0001);
    return delta / len;
}

fn continuation_score(current_segment: u32, current_endpoint: u32, candidate_segment: u32, candidate_endpoint: u32) -> f32 {
    if (current_segment == candidate_segment) { return -1e9; }
    let a = visible_segments[current_segment];
    let b = visible_segments[candidate_segment];
    if (a.kind_edge.x != b.kind_edge.x) { return -1e9; }

    let ap = select(a.start.xy, a.end.xy, current_endpoint == 1u);
    let bp = select(b.start.xy, b.end.xy, candidate_endpoint == 1u);
    let gap = distance(ap, bp);
    let endpoint_snap = max(uniforms.params12.w, 0.5);
    if (gap > endpoint_snap * 1.65) { return -1e9; }

    let ad = segment_dir_from_endpoint(current_segment, current_endpoint);
    let bd = -segment_dir_from_endpoint(candidate_segment, candidate_endpoint);
    let tangent_mismatch = 1.0 - clamp(dot(ad, bd), -1.0, 1.0);
    let depth_gap = abs(select(a.start.z, a.end.z, current_endpoint == 1u) - select(b.start.z, b.end.z, candidate_endpoint == 1u));

    let cost = gap * 0.75 + tangent_mismatch * 12.0 + depth_gap * 0.025;
    return 1.0 / (1.0 + cost);
}
```

### Uproszczenie v1 akceptowalne

Na pierwszą implementację nie robić globalnego union-find, jeżeli to wydłuży pracę. Dopuszczalne:

```text
segment owner = min(current segment, best start neighbor, best end neighbor)
path_id = owner
```

Ale tylko jako etap przejściowy. Pełna implementacja ma mieć co najmniej 2-4 iteracje relax owner:

```text
npr_connect_paths.wgsl      -> local owners
npr_relax_path_owners.wgsl  -> owner propagation, 2-4 dispatches
npr_emit_path_segments.wgsl -> final path_id/path metrics
```

Jeżeli robimy pełniej, dodać plik:

```text
crates/engine/render-wgpu/src/renderer/shaders/npr_relax_path_owners.wgsl
```

### MODIFY `gpu_realtime.rs`

Po `endpoint_bins` dodać:

```rust
{
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("amigo-npr-connect-paths-pass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipelines.connect_paths_pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(workgroup_count(topology.edge_count as usize), 1, 1);
}
```

Jeśli dodajesz relax:

```rust
for _ in 0..4 {
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("amigo-npr-relax-path-owners-pass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipelines.relax_path_owners_pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(workgroup_count(topology.edge_count as usize), 1, 1);
}
```

### ACCEPTANCE

- path ownership jest per `NprLineKind`;
- silhouette nie miesza się z feature;
- junctiony są konserwatywne: brak długich przeskoków;
- debug `path_owners` pokazuje spójne kolory pathów, nie losowe edge ids.

---

## 15. Pakiet L - emit path segments i path-level metrics

### ADD

Plik:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_emit_path_segments.wgsl`

Cel:

- utworzyć `GpuNprPathSegment3d` jako finalne wejście do `npr_build_strokes.wgsl`;
- nadać `path_id`, `t0`, `t1`, `path_length_px`, `importance`.

Minimalny v1:

```text
path_length_px = visible segment length, jeżeli brak pełnej agregacji
path_id = path_states[segment].path_id
t0 = 0.0
t1 = 1.0
```

Pełniejszy v2:

```text
path_length_px = suma długości segmentów o tym samym path_id w bounded group
t0/t1 = approximated cumulative range
```

Szkielet:

```wgsl
@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&visible_segments)) { return; }
    let seg = visible_segments[i];
    if (seg.kind_edge.x == KIND_NONE || seg.start.w <= 0.5 || seg.end.w <= 0.5) { return; }

    let state = path_states[i];
    let out_index = atomicAdd(&counters.path_segment_count, 1u);
    if (out_index >= arrayLength(&path_segments)) { return; }

    let len = max(distance(seg.start.xy, seg.end.xy), 0.001);
    let importance = path_importance(seg.kind_edge.x, (seg.start.z + seg.end.z) * 0.5);
    path_segments[out_index] = GpuNprPathSegment3d(
        seg.start,
        seg.end,
        vec4<u32>(state.path_id, seg.kind_edge.x, i, state.flags),
        vec4<f32>(0.0, 1.0, len, importance),
    );
}
```

### DEBUG MODE PATH SEGMENTS

Jeśli `debug_mode == PathSegments`, `npr_build_strokes.wgsl` powinien rysować `path_segments` bez pressure/dropout i kolorować po `path_id`.

### ACCEPTANCE

- `path_segments` mają valid start/end;
- `path.kind` odpowiada `visible_segments.kind_edge.x`;
- `path.metrics.z` jest długością w px i nie jest zerowe;
- finalny build strokes nie musi już czytać `visible_segments` jako głównego wejścia.

---

## 16. Pakiet M - `npr_build_strokes.wgsl` przejście z visible edge na path segment

### MODIFY

Plik:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_build_strokes.wgsl`

Aktualny problem:

- shader definiuje `GpuNprPathLink3d`, `ChainOwnerPick` i korzysta z `visible_segments` + sąsiedztwa edge;
- `taper_multiplier(t)` działa lokalnie po segmencie, nie po path;
- search/primary logic nadal częściowo bazuje na chain heuristics.

Docelowy input:

```wgsl
@group(0) @binding(14) var<storage, read_write> path_segments: array<GpuNprPathSegment3d>;
```

Główna pętla:

```wgsl
@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let path_segment_index = id.x;
    if (path_segment_index >= u32(arrayLength(&path_segments))) { return; }
    let segment = path_segments[path_segment_index];
    if (segment.start.w <= 0.5 || segment.end.w <= 0.5) { return; }

    let kind = segment.path.y;
    let path_id = segment.path.x;
    let path_t0 = segment.metrics.x;
    let path_t1 = segment.metrics.y;
    let path_length_px = max(segment.metrics.z, 0.001);
    let importance = segment.metrics.w;

    let primary_passes = u32(uniforms.params5.x);
    let search_passes = u32(uniforms.params5.y);
    let total_passes = primary_passes + effective_search_passes(kind, search_passes);

    for (var pass_index: u32 = 0u; pass_index < total_passes; pass_index = pass_index + 1u) {
        emit_stroke_for_path_segment(segment, path_id, kind, path_t0, path_t1, path_length_px, importance, pass_index);
    }
}
```

Zmienić funkcje stylizacji:

```wgsl
fn pressure_multiplier(path_t: f32) -> f32 {
    return sample_curve4(uniforms.params8, clamp(path_t, 0.0, 1.0));
}

fn alpha_pressure_multiplier(path_t: f32) -> f32 {
    return clamp(sample_curve4(uniforms.params9, clamp(path_t, 0.0, 1.0)), 0.0, 1.5);
}

fn taper_multiplier(path_t: f32) -> f32 {
    let endpoint_weight = clamp(min(path_t, 1.0 - path_t) * 2.0, 0.0, 1.0);
    return 1.0 - clamp(uniforms.params5.w, 0.0, 1.0) * (1.0 - max(endpoint_weight, 0.35));
}
```

Search pass:

```wgsl
fn effective_search_passes(kind: u32, configured: u32) -> u32 {
    if (!npr_gpu_search_enabled()) { return 0u; }
    if (kind == KIND_SILHOUETTE || kind == KIND_CONTACT) { return 0u; }
    return configured;
}
```

Dropout:

```wgsl
fn dropout_alpha(kind: u32, path_id: u32, pass_index: u32, path_t: f32) -> f32 {
    if (kind == KIND_SILHOUETTE && pass_index == 0u) {
        return 1.0;
    }
    let amount = clamp(uniforms.params5.z, 0.0, 0.98);
    if (amount <= 0.001) {
        return 1.0;
    }
    let cell = floor(path_t * max(uniforms.params4.w, 1.0));
    let n = hash01(path_id ^ (pass_index * 1664525u), u32(cell));
    return select(1.0, 0.0, n < amount);
}
```

### REMOVE/DEGRADE

W normalnej ścieżce usunąć użycie:

```wgsl
ChainOwnerPick
GpuNprPathLink3d
best_endpoint_candidate
terminal_endpoint_walk
chained_endpoint_walk
continuation_follow_edge
edges[next_a/next_b]
```

Można zostawić tylko w `#`-brak WGSL preprocessor nie pomaga, więc najlepiej realnie usunąć albo przenieść do legacy shader.

### ACCEPTANCE

- taper działa na path-level;
- pressure curves działają na path-level;
- dropout nie rozwala silhouette;
- search nie działa dla silhouette;
- technical/default GPU nie generuje search lines;
- rough/pencil może generować krótkie search lines.

---

## 17. Pakiet N - stats, HUD i devtools

### MODIFY

Plik:

- `crates/engine/render-api/src/stats.rs`

Dodać pola do `RenderFrameStats` albo do istniejącej struktury NPR stats:

```rust
pub world_3d_npr_gpu_visible_segments_capacity: usize,
pub world_3d_npr_gpu_endpoint_refs_capacity: usize,
pub world_3d_npr_gpu_endpoint_bins_capacity: usize,
pub world_3d_npr_gpu_path_states_capacity: usize,
pub world_3d_npr_gpu_path_segments_capacity: usize,
pub world_3d_npr_gpu_stroke_segments_capacity: usize,
pub world_3d_npr_gpu_debug_mode: &'static str,
pub world_3d_npr_gpu_frame_jobs: usize,
pub world_3d_npr_gpu_topology_uploads: usize,
```

Jeśli `RenderFrameStats` nie może trzymać `&'static str`, użyć:

```rust
pub world_3d_npr_gpu_debug_mode: String,
```

albo enum/number:

```rust
pub world_3d_npr_gpu_debug_mode: u32,
```

### MODIFY

Plik:

- `crates/engine/render-wgpu/src/renderer/npr/gpu_realtime.rs`

Rozszerzyć `NprGpuFrameStats3d`:

```rust
pub endpoint_refs_capacity: usize,
pub endpoint_bins_capacity: usize,
pub path_states_capacity: usize,
pub path_segments_capacity: usize,
pub stroke_segments_capacity: usize,
pub debug_mode: amigo_render_api::NprGpuRealtimeDebugMode3d,
```

W `execute(...)` liczyć capacity z buforów, nie z readbacku.

### MODIFY

Pliki:

- `crates/apps/app/src/render_runtime.rs`
- `crates/engine/devtools/src/builder.rs`
- `crates/engine/devtools/src/commands/render.rs`

Dodać output:

```text
NPR strategy: gpu_realtime
GPU debug: final
GPU meshes/jobs: N
GPU topology uploads: N
visible segments capacity: N
endpoint refs capacity: N
endpoint bins capacity: N
path states capacity: N
path segments capacity: N
stroke segments capacity: N
```

Nie dodawać readback counters do normalnego HUD. Dokładne liczniki atomic tylko w osobnej komendzie debug, np. `render.npr.gpu.readback`, odpalanej ręcznie.

### WALIDACJA

```powershell
cargo check -p amigo-render-api
cargo check -p amigo-app
cargo check -p amigo-devtools
```

---

## 18. Pakiet O - frame-level multi-mesh face-id i inter-object occlusion

To jest krytyczne dla wielu modeli. Obecny `gpu_realtime.rs` iteruje po `frame_jobs` i robi face-id pass per job. To oznacza, że face-id/depth może być czyszczony per mesh i nie musi reprezentować całej sceny. Przy wielu modelach linie mogą nie znać wzajemnego zasłaniania.

### MODIFY ARCHITECTURE

Plik:

- `crates/engine/render-wgpu/src/renderer/npr/gpu_realtime.rs`

Aktualny flow:

```text
for job in frame_jobs:
  clear face-id/depth
  draw job faces
  classify job edges against its face-id
```

Docelowy flow:

```text
begin frame
collect all jobs
ensure all topology
allocate global frame buffers with offsets
clear face-id/depth once
for each job:
  render job faces into same face-id/depth target without clearing
for each job:
  project/classify/paths/strokes using global face-id/depth
end frame
```

### ADD OFFSETS

Dodać strukturę:

```rust
#[derive(Debug, Clone, Copy)]
struct NprGpuMeshFrameOffset3d {
    vertex_base: u32,
    triangle_base: u32,
    edge_base: u32,
    visible_segment_base: u32,
    endpoint_ref_base: u32,
    endpoint_bin_base: u32,
    path_state_base: u32,
    path_segment_base: u32,
    stroke_segment_base: u32,
}
```

W pierwszym etapie można dalej używać per-topology buffers, ale face-id musi być globalne:

```text
render pass clear once
for each job:
  set face_id bind group for job
  draw triangles, load existing depth/color
```

W WGPU render pass z wieloma drawami:

```rust
let mut pass = encoder.begin_render_pass(... load: Clear once ...);
for job in &self.frame_jobs {
    pass.set_bind_group(0, &face_id_bind_group_for_job, &[]);
    pass.draw(0..topology.triangle_count * 3, 0..1);
}
```

Potem compute per job może pozostać na razie per-job, ale sampling `face_id_texture` będzie globalny.

### FACE-ID musi kodować mesh/object

Aktualny face-id prawdopodobnie zapisuje `face_index + 1`. Przy globalnym face-id między meshami to się konfliktuje.

Dodać w uniform:

```rust
params18.x = mesh_face_id_base as f32
```

Lub w `GpuNprFrameUniforms3d` dodać:

```rust
pub object_face_id_base: u32
```

WGSL face-id:

```wgsl
out_face_id = uniforms.object_face_id_base + face_index + 1u;
```

Classify `face_id_matches(...)` musi sprawdzać:

```wgsl
let face0 = uniforms.object_face_id_base + edge.face0 + 1u;
let face1 = uniforms.object_face_id_base + edge.face1 + 1u;
```

### ACCEPTANCE

- dwa modele obok siebie renderują się bez wzajemnych artefaktów;
- model z przodu zasłania linie modelu z tyłu;
- face-id target jest czyszczony raz na frame, nie raz na mesh;
- nie ma readbacku.

---

## 19. Pakiet P - cleanup starego GPU chain path

### DELETE / DEGRADE

Pliki:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_compact_owners.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/mod.rs`
- `crates/engine/render-wgpu/src/renderer/npr/gpu_pipelines.rs`
- `crates/engine/render-wgpu/src/renderer/npr/gpu_types.rs`
- `crates/engine/render-wgpu/src/renderer/npr/gpu_buffers.rs`

Usunąć dopiero gdy:

- `npr_emit_endpoint_refs.wgsl` działa;
- `npr_endpoint_bins.wgsl` działa;
- `npr_connect_paths.wgsl` działa;
- `npr_emit_path_segments.wgsl` działa;
- `npr_build_strokes.wgsl` czyta `path_segments`;
- preview smoke przechodzi;
- manualnie nie wracają długie losowe kreski.

Usunąć:

```text
NPR_COMPACT_OWNERS_SHADER
compact_owners_pipeline
GpuNprPathLink3d
path_links
path_links_capacity
next_a
next_b
alt_next_a
alt_next_b
```

Jeżeli `next_*` zostają jako hint:

1. zmienić komentarz w `GpuNprEdge3d`:

```rust
// Optional topology hint only. Not authoritative for path ownership.
```

2. nie wolno ich używać w `npr_build_strokes.wgsl`.

### WALIDACJA CLEANUP

```powershell
rg -n "compact_owners_pipeline.*dispatch|NPR_COMPACT_OWNERS_SHADER" crates/engine/render-wgpu/src
rg -n "next_a|next_b|alt_next_a|alt_next_b" crates/engine/render-wgpu/src/renderer/shaders
rg -n "path_links" crates/engine/render-wgpu/src/renderer
cargo check -p amigo-render-wgpu
```

Acceptance:

```text
normalny gpu_realtime nie ma starej sciezki compact owners;
build_strokes nie korzysta z next/alt-next;
path_segments sa jedynym finalnym zrodlem kreski.
```

---

## 20. Pakiet Q - presety: semantyka CPU/GPU i różnice stylów

Aktualny kod ma tylko `GpuStableComic` i `RoughComicInk` jako enumy stylu. Presety YAML są bogatsze i to one powinny nieść styl. Dokument musi doprecyzować: albo rozbudowujemy enum, albo uznajemy YAML za źródło stylu.

### Decyzja zalecana

Nie rozbudowywać teraz `NprStylePreset3d` do wielu enumów. Zachować:

```rust
GpuStableComic
RoughComicInk
```

A wszystkie różnice stylów trzymać w YAML:

```text
clean-comic-ink.yml
technical-comic-line.yml
manga-fine-line.yml
european-clear-line.yml
animation-pencil.yml
loose-pencil.yml
dry-brush-ink.yml
heavy-noir-ink.yml
storyboard-marker.yml
cinematic-12fps.yml
low-120fps.yml
rough-comic-ink.yml
```

### MODIFY DOCUMENT WORDING

Zamiast pisać:

```text
Dodać style enum CleanComicInk, TechnicalComicLine, MangaFineLine...
```

Pisać:

```text
Style produkcyjne są presetami YAML. `NprStylePreset3d` jest tylko bazą domyślną / fallback preset factory.
```

### YAML rule

Każdy preset GPU powinien mieć jawnie:

```yaml
strategy: gpu_realtime
fill_mode: none
style_preset: gpu_stable_comic albo rough_comic_ink
stroke_tool: ...
gpu_realtime_tuning:
  debug_mode: final
```

Każdy preset CPU powinien mieć:

```yaml
strategy: cpu_reference
fill_mode: none
```

### Acceptance

- presety nie wyglądają identycznie;
- różnice wynikają z YAML parametrów;
- GPU/CPU wersje jednego presetu są semantycznie porównywalne.

---

## 21. Pakiet R - testy minimalne

Zasada: kod > testy, ale testy mają łapać regresje architektoniczne.

### Test 1 - strategy parser odrzuca auto/hybrid

Plik:

- `crates/engine/scene/src/hydration/plan/components_domains.rs` albo istniejący moduł testów sceny.

Wymóg:

```text
strategy: auto -> error
strategy: hybrid -> error
strategy: gpu_realtime -> OK
strategy: cpu_reference -> OK
```

### Test 2 - debug mode YAML parser

Wymóg:

```text
gpu_realtime_tuning.debug_mode: final -> OK
gpu_realtime_tuning.debug_mode: raw_fragments_by_kind -> OK
gpu_realtime_tuning.debug_mode: endpoint_bins -> OK
gpu_realtime_tuning.debug_mode: path_owners -> OK
gpu_realtime_tuning.debug_mode: path_segments -> OK
gpu_realtime_tuning.debug_mode: wrong -> error
```

### Test 3 - MeshSceneService debug mode mutator

Plik:

- `crates/3d/mesh/src/lib.rs`

Dodać test obok istniejących testów NPR:

```rust
#[test]
fn sets_npr_gpu_debug_mode_on_mesh_command() {
    let service = MeshSceneService::default();
    service.queue(MeshDrawCommand {
        entity_id: 11,
        entity_name: "playground-npr-model".to_owned(),
        mesh: Mesh3d {
            mesh_asset: AssetKey::new("playground-npr/meshes/soldier"),
            transform: Transform3::default(),
            npr: Some(NprLineSettings3d::default()),
        },
    });

    assert!(service.set_npr_gpu_debug_mode(
        "playground-npr-model",
        amigo_render_api::NprGpuRealtimeDebugMode3d::EndpointBins,
    ));

    assert_eq!(
        service.commands()[0]
            .mesh
            .npr
            .as_ref()
            .unwrap()
            .gpu_realtime_tuning
            .debug_mode,
        amigo_render_api::NprGpuRealtimeDebugMode3d::EndpointBins
    );
}
```

### Test 4 - routing tests utrzymać

Plik:

- `crates/engine/render-wgpu/src/renderer/service/render/world.rs`

Utrzymać:

```text
routes_npr_gpu_realtime_meshes
routes_npr_cpu_reference_meshes
```

### Test 5 - scene bootstrap

Plik:

- `crates/apps/app/src/tests/scene_loading_tests/threed.rs`

Utrzymać:

```text
playground_npr_comic_lines_bootstrap_applies_gpu_realtime_preset_to_soldier
```

Dodać, że default debug mode jest `final`, jeśli test ma dostęp do NPR settings.

### Test 6 - preview smoke

Plik:

- `crates/apps/app/src/tests/render_runtime_tests.rs`

Utrzymać ignored:

```powershell
cargo test -p amigo-app playground_npr_preview_renders_gpu_and_cpu_reference_default_gpu_comic -- --ignored
```

To nadal smoke, nie pełny certyfikat jakości.

---

## 22. Kolejność wdrażania, obowiązkowa

1. `READ` aktualny source i CPU reference invariants.
2. `ADD` `parity_notes.rs`.
3. `MODIFY` Render API: `NprGpuRealtimeDebugMode3d`, tuning fields.
4. `MODIFY` Scene document/hydration: YAML `debug_mode` i capacity tuning.
5. `MODIFY` MeshSceneService + script command: `set_npr_gpu_debug_mode`.
6. `MODIFY` playground: input `V`, Rhai cycle, HUD text.
7. `MODIFY` GPU types: endpoint/path/counter structs.
8. `MODIFY` GPU buffers: endpoint/path/counter buffers.
9. `MODIFY` GPU pipelines/bind layout: nowe bindings i pipelines.
10. `ADD` `npr_reset_frame.wgsl`.
11. `ADD` `npr_emit_endpoint_refs.wgsl`.
12. `ADD` `npr_endpoint_bins.wgsl`.
13. `ADD` `npr_connect_paths.wgsl`.
14. `ADD` `npr_emit_path_segments.wgsl`.
15. `MODIFY` `gpu_realtime.rs` dispatch order.
16. `MODIFY` `npr_build_strokes.wgsl`: input `path_segments`, path-level t.
17. `MODIFY` stats/HUD/devtools.
18. `MODIFY` frame-level face-id clear once for multi-mesh.
19. `CLEANUP` compact owners, path_links, next/alt-next normal usage.
20. `VERIFY` targeted checks and manual scene.

Nie robić:

- full workspace test;
- full workspace format;
- rewrite CPU reference;
- nowy renderer v2;
- fallback GPU -> CPU;
- `auto`/`hybrid`;
- readback per frame.

---

## 23. Komendy walidacyjne

Po Render API:

```powershell
cargo check -p amigo-render-api
```

Po Scene hydration:

```powershell
cargo check -p amigo-scene
```

Po Mesh script bridge:

```powershell
cargo check -p amigo-3d-mesh
```

Po WGPU:

```powershell
cargo check -p amigo-render-wgpu
```

Po playground/app:

```powershell
cargo test -p amigo-app playground_npr_comic_lines_bootstrap_applies_gpu_realtime_preset_to_soldier
```

Po runtime pipeline:

```powershell
cargo test -p amigo-app playground_npr_preview_renders_gpu_and_cpu_reference_default_gpu_comic -- --ignored
```

Cleanup grep:

```powershell
rg -n "compact_owners_pipeline.*dispatch|NPR_COMPACT_OWNERS_SHADER" crates/engine/render-wgpu/src
rg -n "search_pass_count = 0u|search_enabled: false" crates/engine/render-wgpu/src/renderer/shaders mods/playground-npr
rg -n "debug_mode" crates/engine/render-api crates/engine/scene crates/engine/render-wgpu crates/3d/mesh mods/playground-npr
rg -n "next_a|next_b|alt_next_a|alt_next_b" crates/engine/render-wgpu/src/renderer/shaders
```

Manual:

```powershell
cargo run
```

Sprawdzić:

1. Default: `gpu_realtime`.
2. Default preset: `default_gpu_comic`.
3. `G`: toggle CPU/GPU.
4. `V`: debug mode cycle:
   - final,
   - raw fragments,
   - endpoint bins,
   - path owners,
   - path segments.
5. Model Soldier:
   - stoi,
   - cały w kadrze,
   - orbit/freelook działa.
6. GPU final:
   - brak losowych długich kresek,
   - obrys jest czytelny,
   - szczegóły nie są siatką,
   - presety różnią się stylem, nie geometrią błędu.
7. CPU reference:
   - nadal porównywalny,
   - nie zmienił się przez refactor GPU.
8. Dwa modele testowo:
   - obiekt z przodu zasłania linie obiektu z tyłu.

---

## 24. Acceptance checklist

Przed uznaniem NPR GPU za naprawiony:

- [ ] `gpu_realtime` nie używa starego edge-chain owner modelu jako finalnej ścieżki.
- [ ] `npr_build_strokes.wgsl` czyta `path_segments`, nie `visible_segments` jako główne wejście finalnej kreski.
- [ ] GPU ma `endpoint_refs`.
- [ ] GPU ma `endpoint_bins`.
- [ ] GPU ma `path_states`.
- [ ] GPU ma `path_segments`.
- [ ] GPU path ownership jest per `NprLineKind`.
- [ ] GPU ma path-level `t0/t1/path_length_px`.
- [ ] GPU taper działa na path, nie na każdym edge osobno.
- [ ] GPU pressure curves odpowiadają CPU.
- [ ] GPU dropout nie niszczy silhouette.
- [ ] Search lines nie są włączone dla silhouette.
- [ ] `gpu_realtime_tuning.debug_mode` działa z YAML.
- [ ] `V` przełącza debug mode runtime.
- [ ] HUD pokazuje strategy, preset, model, GPU debug mode.
- [ ] Multi-mesh face-id jest globalny w obrębie frame.
- [ ] Face-id nie konfliktuje między meshami.
- [ ] Preset `default_gpu_comic` ma podobny region tuszu w CPU i GPU.
- [ ] Presety 0..N działają w obu strategiach.
- [ ] `G` toggle działa w obie strony.
- [ ] Brak `auto`.
- [ ] Brak `hybrid`.
- [ ] Brak cichego fallbacku.
- [ ] Brak readbacku w normalnym realtime.
- [ ] Stary `compact_owners` usunięty albo nieosiągalny w normalnej ścieżce.

---

## 25. Decyzje architektoniczne

1. CPU reference to golden model.
2. GPU ma parytet semantyczny, nie dowolną alternatywną estetykę.
3. Debug modes są jawne i autorskie, nie ukryte fallbacki.
4. `gpu_realtime_tuning` może sterować jakością/perf, ale nie może zmieniać sensu presetów.
5. Najpierw poprawny rysunek, potem optymalizacja.
6. Presety produkcyjne są YAML-first; enum stylu jest tylko bazą/factory.
7. Path graph GPU ma być screen-space i visibility-aware.
8. Multi-mesh wymaga globalnego face-id/depth dla całej ramki.

Koniec planu.
