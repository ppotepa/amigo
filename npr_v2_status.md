# Amigo NPR v2 - status wdrozenia

Status: aktualny zapis tego, co zostalo wdrozone i co zostaje do naprawy po pierwszej implementacji `gpu_realtime`.
Powiazany plan: `npr_v2.md`.

## 1. Co zostalo zrobione

### 1.1. Kontrakt render-api

- Dodano jawna strategie NPR 3D:
  - `gpu_realtime`,
  - `cpu_reference`.
- Domyslna strategia jest GPU realtime.
- CPU zostalo zachowane jako jawny tryb referencyjny, bez `auto`, bez `hybrid` i bez cichego fallbacku.
- Dodano tryb wypelnienia NPR:
  - `shaded`,
  - `none`,
  - `depth_only`.
- Dodano tuning GPU realtime:
  - limity dlugosci segmentow,
  - limity chain walk,
  - parametry search lines,
  - mnozniki silhouette/feature.

Glowne pliki:

- `crates/engine/render-api/src/commands_3d.rs`
- `crates/engine/render-api/src/stats.rs`

### 1.2. Scene document i hydracja YAML

- `Mesh3D.npr.strategy` jest parsowane z YAML.
- Brak `strategy` oznacza domyslne `gpu_realtime`.
- `strategy: cpu_reference` uruchamia stara sciezke CPU.
- `strategy: auto`, `strategy: hybrid` i nieznane wartosci sa bledami.
- Presety NPR moga przenosic strategie i tuning GPU.

Glowne pliki:

- `crates/engine/scene/src/document/components.rs`
- `crates/engine/scene/src/hydration/plan/components_domains.rs`
- `crates/engine/scene/src/render_commands/render_3d.rs`

### 1.3. Mesh service, scripting i playground

- Mesh commands przenosza ustawienia NPR razem ze strategia.
- Skrypty playground potrafia przelaczac strategie.
- Domyslnie scena NPR startuje na GPU realtime.
- `G` dziala jako toggle GPU/CPU.
- `T` przelacza temporal smoothing w scenie.
- `R` przelacza automatyczny obrot modelu.
- Presety sa plikami YAML i maja odpowiedniki GPU oraz CPU reference.
- Zamiast latajacego tekstu 3D dodano stale UI/debug overlay z modelem, presetem, strategia i wybranymi statystykami.

Glowne pliki:

- `crates/3d/mesh/src/lib.rs`
- `crates/3d/mesh/src/scene_command.rs`
- `crates/scripting/rhai/src/bindings/mesh3d.rs`
- `mods/playground-npr/scenes/comic-lines/scene.yml`
- `mods/playground-npr/scenes/comic-lines/scene.rhai`
- `mods/playground-npr/scenes/comic-lines/npr-presets/*.yml`

### 1.4. Routing renderera

- Render loop rozdziela meshe wedlug `NprRenderStrategy3d`.
- `cpu_reference` idzie przez dotychczasowa sciezke CPU.
- `gpu_realtime` idzie przez osobny executor GPU.
- Nie ma runtime fallbacku GPU -> CPU.
- Statystyki rozrozniaja meshe CPU i GPU.

Glowne pliki:

- `crates/engine/render-wgpu/src/renderer/service/render/world.rs`
- `crates/engine/render-wgpu/src/renderer.rs`
- `crates/apps/app/src/render_runtime.rs`

### 1.5. Moduly NPR w render-wgpu

- Dodano rozdzial plikow NPR:
  - `cpu_reference.rs`,
  - `gpu_realtime.rs`,
  - `gpu_buffers.rs`,
  - `gpu_pipelines.rs`,
  - `gpu_types.rs`,
  - `style.rs`.
- CPU reference zostalo wydzielone jako referencja stylu i zachowania.
- GPU realtime ma osobne zasoby, topologie, bufory i pipeline.

Glowne pliki:

- `crates/engine/render-wgpu/src/renderer/npr/mod.rs`
- `crates/engine/render-wgpu/src/renderer/npr/cpu_reference.rs`
- `crates/engine/render-wgpu/src/renderer/npr/gpu_realtime.rs`
- `crates/engine/render-wgpu/src/renderer/npr/gpu_buffers.rs`
- `crates/engine/render-wgpu/src/renderer/npr/gpu_pipelines.rs`
- `crates/engine/render-wgpu/src/renderer/npr/gpu_types.rs`
- `crates/engine/render-wgpu/src/renderer/npr/style.rs`

### 1.6. Shadery GPU NPR

- Dodano shadery WGSL dla pierwszej sciezki GPU:
  - face-id,
  - projection,
  - edge classification,
  - owner compaction,
  - stroke build.
- GPU renderuje kreski bez budowania CPU `NprFaceVisibilityBuffer`.
- GPU nie powinno generowac CPU `Vec<NprStrokeSegmentVertex>` dla `gpu_realtime`.

Glowne pliki:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_face_id.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_project_vertices.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_classify_edges.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_compact_owners.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_build_strokes.wgsl`

### 1.7. Codemap / tooling

- `amigo-codemap` dostal szybsze stale handling, incremental refresh, persistent anchor cache, coverage dla `concat.zip`, daemon auto-start i lepsze `open-set` / `change-plan`.
- `amigo-symbol-explorer` indeksuje teraz `wgsl`.

Glowne pliki:

- `crates/tools/amigo-codemap/**`
- `crates/tools/amigo-symbol-explorer/src/scan/files.rs`
- `crates/tools/amigo-symbol-explorer/src/scan/text_occurrences.rs`

## 2. Co dziala

- Scena `playground-npr/comic-lines` laduje sie z presetami NPR.
- GPU realtime daje znacznie wyzszy FPS niz CPU reference.
- Toggle strategii jest dostepny z poziomu sceny.
- Presety YAML istnieja w wariantach GPU i CPU reference.
- Czesc presetow GPU zaczyna przypominac CPU reference, szczegolnie tam, gdzie ustawienia sa proste i mniej zalezne od path-level stylizacji.
- Debug overlay zastapil tekst 3D i pokazuje bardziej praktyczne informacje runtime.

## 3. Znane problemy

### 3.1. GPU realtime nie jest jeszcze wiernym odpowiednikiem CPU reference

Najwazniejszy problem: GPU nadal renderuje visible edge segments bardziej niz prawdziwe stroke paths.
Efekt: niektore presety pokazuja dlugie, zwykle kreski albo linie, ktore tylko przyblizaja ksztalt modelu, ale nie maja tej samej charakterystyki obrysu i detalu co CPU.

### 3.2. Brak pelnego GPU path model

Brakuje GPU odpowiednika:

- endpoint bins,
- stabilnego path walk,
- path ids,
- path-level simplification,
- path-level pressure,
- path-level humanization,
- dropout mask na poziomie stroke path,
- search/correction passes zgodnych z CPU.

To jest glowna roznica wizualna miedzy CPU i GPU.

### 3.3. Face-id / owner sampling wymaga dopracowania

Objawy:

- czasem widac pojedyncze glitchowe kreski,
- przy niektorych katach kamera potrafi pokazac dziwna, powtarzalna klatke,
- czesc linii wyglada jak wlasciciel edge'a albo face-id zostal dobrany zle.

Do sprawdzenia:

- wspolrzedne viewportu w face-id pass,
- normalizacja clip/NDC/screen,
- depth compare,
- owner compaction,
- zakres face id dla wielu meshy,
- czyszczenie targetow per frame.

### 3.4. Presety nie sa jeszcze semantycznie identyczne w obu strategiach

CPU interpretuje parametry jako stroke/path model.
GPU interpretuje czesc parametrow jako segment-local model.

Najbardziej widoczne roznice:

- `width_pressure_curve`,
- `alpha_pressure_curve`,
- `taper`,
- `dropout`,
- `passes`,
- `search_line_count`,
- `humanization`,
- `temporal_path_smoothing`.

### 3.5. Temporal smoothing powinien byc feature flag

Efekt "stroke zostaje i wraca na miejsce" jest pozadany jako opcjonalny efekt, ale nie powinien byc wymuszony.
W scenie ma byc przelaczany przez `T`.
Docelowo powinien byc tez kontrolowany przez YAML.

## 4. Co zostalo do zrobienia

1. Doprowadzic GPU do path-first pipeline.
   - Wejscie: visible edge fragments.
   - Wyjscie: stable stroke paths.
   - Dopiero potem stroke segments.

2. Dodac GPU endpoint binning.
   - Endpoint quantization w screen space.
   - Biny per line kind.
   - Stabilny limit wpisow.

3. Dodac GPU path walk.
   - Walk po wlascicielach edge/fragment.
   - Limity `max_terminal_walk_edges` i `max_chained_walk_edges`.
   - Rejection po angle/depth/kind.

4. Ujednolic stylizacje CPU/GPU.
   - Wydzielic wspolny model preset -> resolved style.
   - GPU ma uzywac tych samych semantyk co CPU.

5. Zrobic path-level humanization w GPU.
   - Noise po `path_id + t`.
   - Endpoint lock.
   - Pressure/alpha curves po arc length.

6. Zrobic path-level dropout.
   - Nie punktowy random.
   - Przedzialy przerw.
   - Ochrona silhouette.

7. Uporzadkowac search lines.
   - Search pass ma byc osobnym pass planem.
   - Silhouette domyslnie nie powinna dostawac agresywnych search lines.

8. Naprawic face-id / owner sampling.
   - Debug pass dla face id.
   - Debug pass dla owner edge.
   - Porownanie CPU/GPU na jednym modelu i jednej kamerze.

9. Dodac debug mode do YAML.
   - `gpu_realtime_tuning.debug_mode: final|face_id|owners|fragments|paths|segments`.
   - Bez readbacku w normalnym renderze.

10. Dodac parity preset audit.
    - Dla kazdego presetu porownac CPU/GPU.
    - Oznaczyc, ktore pola jeszcze nie sa wspolne.

11. Zrobic cleanup po doprowadzeniu parity.
    - Usunac tymczasowe hinty segmentowe, ktore dubluja path ownership.
    - Nie zostawiac `v2`, `legacy`, `hybrid`, `auto`.

## 5. Minimalny nastepny etap

Najkrotsza droga do widocznej poprawy:

1. Naprawic face-id / owner sampling.
2. Zrobic endpoint binning.
3. Zrobic pierwszy GPU path walk bez zaawansowanej humanizacji.
4. Renderowac path segments zamiast raw visible segments.
5. Dopiero potem przenosic pressure/dropout/search z CPU na GPU.

Do tego momentu GPU moze miec wyzszy FPS, ale nie bedzie wygladac jak CPU reference.
