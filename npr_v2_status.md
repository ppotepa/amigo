# Amigo NPR v2 - status wdrozenia

Status: aktualny zapis tego, co zostalo wdrozone i co zostaje do naprawy po pierwszej implementacji `gpu_realtime`.
Powiazany plan: `npr_v2.md`.

## 0. Postep globalny

- Szacowany postep calego `npr_v2`: okolo 98%.
- Zrobione: kontrakt, YAML, routing CPU/GPU, debug mode, endpoint bins, owner compaction, `path_segments`, path-level lock/dropout foundation.
- Zostalo: domkniecie parytetu wizualnego CPU/GPU, lepszy graph walk i stabilniejsze `path_t/path_id`, dalsze dopasowanie stylizacji do `cpu_reference`.

## 0.2. Stan wprost

### Domkniete

- Kontrakt `gpu_realtime` / `cpu_reference`.
- Domyslny GPU runtime bez `auto` i bez `hybrid`.
- Routing YAML -> scene hydration -> mesh runtime -> render loop.
- Playground controls dla presetow, modeli, strategii i debug mode.
- Podstawowy GPU pipeline:
  - `face-id`,
  - `project_vertices`,
  - `classify_edges`,
  - `endpoint bins`,
  - `owner compaction`,
  - `emit_path_segments`,
  - `build_strokes`.

### Dziala, ale nie ma jeszcze pelnego parytetu

- GPU path walk i identyfikacja sciezki.
- GPU `path_t` i `path_id`.
- GPU dropout, endpoint lock, taper i search shaping.
- Zgodnosc presetow CPU/GPU.
- Temporal/path smoothing w ujeciach granicznych.

### Nadal do zrobienia

- Dalsze ograniczenie falszywych dlugich chainow dla niektorych presetow i katow kamery.
- Lepsza zgodnosc GPU z CPU dla pressure/alpha/humanization/search.
- Cleanup tymczasowych hintow `next_*` / `alt_next_*` po pelnym przejsciu na mocniejszy path graph.
- Ostateczne strojenie budzetow, limitow i debug stats pod GPU realtime.

## 0.1. Co wnosi ta paczka

- GPU final przestal byc tylko raw-edge lokalnym `visible_segments -> strokes`.
- Doszedl osobny etap `emit_path_segments`.
- `build_strokes` czyta teraz kompaktowane `path_segments`.
- `path_id` jest stabilniejszy i nie zalezy wprost od owner edge.
- Lock, overshoot, drift i dropout zaczely korzystac z semantyki calej path, a nie tylko pojedynczego segmentu.
- `emit_path_segments` odrzuca teraz kontynuacje walk, ktore lamia limit kata, robia zbyt duzy skok glebokosci albo za mocno zmieniaja skale kolejnego segmentu.
- To powinno ograniczyc falszywe dlugie chainy, ktore dawaly losowe kreski niepodobne do obrysu modelu.
- `emit_path_segments` emituje teraz do 6 segmentow na chain zamiast 4:
  - start outer,
  - start middle,
  - start inner,
  - owner first half,
  - owner second half,
  - end inner,
  - end middle,
  - end outer.
- GPU path walk zachowuje juz pierwszy punkt posredni extension walk,
  wiec path nie jest redukowana od razu do samego `final_start/final_end + owner_mid`.
- GPU path walk zachowuje juz tez punkt `penultimate` dla dluzszych extension chainow.
- To daje bogatsze `path_t` i lepiej odwzorowuje wieloodcinkowa strukture sciezki,
  szczegolnie tam, gdzie extension ma wiecej niz 2 hopy.
- `gpu_realtime` buduje teraz `face-id/depth` globalnie dla calej ramki:
  - target nie jest juz czyszczony per mesh,
  - wszystkie meshe GPU sa najpierw rasteryzowane do wspolnego `face-id/depth`,
  - dopiero potem kazdy job przechodzi swoje compute passy.
- `face-id` dostalo per-job `face_id_base`, wiec identyfikatory trojkatow nie koliduja juz miedzy meshami GPU w tej samej ramce.
- `compact_owners` wybiera teraz bardziej kanonicznego ownera dla stabilnego lokalnego chainu:
  - przy `connected_both + chain_compactable` owner moze zostac zkanonizowany do najmniejszego stabilnego kandydata,
  - zmniejsza to skakanie ownera miedzy sasiednimi edge przy malych zmianach kamery.
- `compact_owners` wymaga teraz tez wzajemnosci polaczenia:
  - kandydat start/end musi lokalnie wybierac z powrotem biezacy edge,
  - redukuje to falszywe polaczenia jednostronne i poprawia spoistosc grafu path.
- Doszedl osobny GPU seam `path_states`:
  - `connect_paths` zapisuje teraz jawny stan path per edge,
  - `path_states` niosa `owner_segment`, `path_id`, `kind`, `flags`, `segment_count`,
  - `emit_path_segments` przestalo polegac wylacznie na `link.owner_edge == edge_index` jako jedynym zrodle autorytetu.
- Doszedl tez pierwszy etap `relax_path_owners`:
  - po `connect_paths` wykonywane sa teraz wielokrotne compute passy propagujace minimalnego ownera po lokalnych linkach,
  - `path_id` po relax jest stabilizowane juz nie tylko lokalnie, ale tez po sasiednich segmentach tej samej klasy linii,
  - to dalej nie jest finalny solver grafu sciezek, ale porzadkuje ownership lepiej niz pojedynczy pass.
- Naprawiono tez porzadek geometrii owner chainu w `emit_path_segments`:
  - srodkowe segmenty ownera sa teraz emitowane w logicznej kolejnosci `visible.start -> owner_mid -> visible.end`,
  - wczesniejszy uklad owner-centric potrafil pomijac pierwszy pol-odcinek ownera i dawac nielogiczne skoki geometrii,
  - to jest zmiana wysokiego wplywu wizualnego, bo dotyczy samego ksztaltu finalnej kreski, nie tylko jej stylizacji.
- `importance` path jest teraz delikatnie wzmacniane przez rozmiar chainu:
  - liczbe hopow,
  - liczbe segmentow w `path_states`.
  To nie zastępuje jeszcze pelnego CPU parity, ale lepiej odroznia duze, spojne sciezki od malych lokalnych fragmentow.
- `build_strokes` dostal teraz tez jawna modulacje `path_coherence`:
  - spójniejsze i dluzsze sciezki dostaja mniej agresywne humanization noise,
  - dropout jest slabszy dla mocnych chainow,
  - search lines sa bardziej ograniczane dla sciezek juz wystarczajaco spojnych,
  - width/alpha sa lekko modulowane przez coherence, zamiast traktowac wszystkie `path_segments` identycznie.
- `should_enable_search_passes()` nie jest juz martwe:
  - search passes sa teraz wlaczane przez jawny warunek path-level, a nie tylko prosty toggle po kind,
  - to dodatkowo ogranicza zbedny szkicowy szum na juz stabilnych pathach.
- Overshoot i endpoint tangent drift dostaly tez modulacje przez `path_coherence`,
  wiec mocne contour path mniej "plywaja" na koncach niz slabe, szkicowe feature/search paths.
- To przybliza GPU do CPU reference w warstwie semantyki kreski:
  - mocne contour paths powinny mniej wygladac jak przypadkowe posklejane odcinki,
  - slabsze feature/search paths nadal zachowuja bardziej szkicowy charakter.
- To porzadkuje architekture:
  - `compact_owners` zostaje etapem wyboru lokalnych polaczen,
  - `connect_paths` zostaje etapem budowy jawnego stanu path,
  - `relax_path_owners` zostaje etapem propagacji ownera/path id,
  - `emit_path_segments` czyta juz ten stan zamiast samemu zgadywac wszystko od zera.
- Statystyki GPU NPR zostaly rozszerzone:
  - `frame_jobs`,
  - `projected_vertices_capacity`,
  - `visible_segments_capacity`,
  - `endpoint_heads_capacity`,
  - `endpoint_entries_capacity`,
  - `path_links_capacity`,
  - `path_states_capacity`,
  - `path_segments_capacity`,
  - `stroke_segments_capacity`,
  - `debug_mode`.
- Te pola przechodza juz do `RenderFrameStats`, wiec dalszy tuning GPU path graphu mozna opierac na konkretnych liczbach runtime, a nie tylko na jednym zbiorczym `buffer_capacity_bytes`.

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
- Overlay jest teraz karmiony live przez `RenderFrameStatsService -> Rhai RuntimeApi`, wiec scena pokazuje:
  - liczbe meshy GPU/CPU,
  - liczbe path,
  - edges/triangles dla GPU,
  - pojemnosci `path_states/path_segments/stroke_segments`,
  - aktywny `npr_debug_mode`.

Glowne pliki:

- `crates/3d/mesh/src/lib.rs`
- `crates/3d/mesh/src/scene_command.rs`
- `crates/scripting/rhai/src/bindings/runtime.rs`
- `crates/scripting/rhai/src/bindings/world_root.rs`
- `crates/scripting/rhai/src/runtime/plugin.rs`
- `crates/scripting/rhai/src/runtime/script_runtime/constructors.rs`
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
  - endpoint bins,
  - owner compaction,
  - stroke build.
- GPU renderuje kreski bez budowania CPU `NprFaceVisibilityBuffer`.
- GPU nie powinno generowac CPU `Vec<NprStrokeSegmentVertex>` dla `gpu_realtime`.
- GPU dispatcher wykonuje teraz:
  - `classify_edges`,
  - `build_endpoint_bins`,
  - `compact_owners`,
  - `emit_path_segments`,
  - `build_strokes`.
- Dodano GPU endpoint buffers:
  - `endpoint_heads`,
  - `endpoint_entries`.
- Dodano GPU path buffer:
  - `path_segments`.
- `compact_owners` korzysta teraz z endpoint bucketow budowanych per frame w screen space,
  zamiast polegac wylacznie na statycznych `next_a/next_b`.
- `build_strokes` zaczal korzystac z `path_links` jako zrodla:
  - owner edge,
  - start/end continuation,
  - connected start/end flags.
- `emit_path_segments` wydziela teraz osobny krok:
  - owner edge -> path segment,
  - path flags -> segment metadata,
  - path length / importance -> metrics.
- `emit_path_segments` czyta teraz tez `path_states`,
  wiec `path_id` i owner path sa juz rozdzielone od samego `path_links`.
- `emit_path_segments` kompaktuje teraz tylko realnie istniejace segmenty do zwartego bufora przez atomic counter,
  zamiast polegac wylacznie na sztywnych slotach.
- `emit_path_segments` robi juz prosty `path walk` po `path_links`:
  - rozszerza owner segment w obie strony,
  - akumuluje chain length,
  - aktualizuje final start/end dla `path_segments`.
- Walk jest teraz bardziej konserwatywny:
  - odrzuca kontynuacje ponizej `max_chain_angle_degrees`,
  - odrzuca za duze skoki depth,
  - odrzuca agresywne zmiany lokalnej skali segmentu przy dalszych hopach.
- `emit_path_segments` zapisuje juz bardziej stabilna tozsamosc:
  - `path_id` jest oparty o skwantowane konce walked chainu, kind, hop count i bucket dlugosci,
  - `path_id` jest juz kierunkowo kanoniczny, wiec odwrocenie chainu nie powinno zmieniac identyfikatora,
  - `path_id` nie zalezy juz bezposrednio od owner edge,
  - `path.z` niesie podstawowa informacje o liczbie hopow chainu.
- `emit_path_segments` emituje teraz dwa segmenty na walked chain:
  - segment A: start outer,
  - segment B: start middle,
  - segment C: start inner,
  - segment D: owner first half,
  - segment E: owner second half,
  - segment F: end inner,
  - segment G: end middle,
  - segment H: end outer.
- To daje lepsze `path_t` dla `build_strokes` niz wczesniejszy pojedynczy owner segment albo podzial na tylko 6 czesci owner-centric.
- Kazdy z 4 segmentow ma teraz wlasne lokalne `connected_start/connected_end`,
  zamiast odziedziczyc flagi calej sciezki 1:1.
- `build_strokes` czyta juz `path_segments` jako glowny input finalnej kreski.
- `build_strokes` czyta tylko rzeczywiscie wyemitowane `path_segments`, nie caly teoretyczny bufor slotow.
- `build_strokes` zaczal uzywac `path_id` i `path_t` do stylizacji.
- `build_strokes` ma juz path-level endpoint lock:
  - wobble jest tlumione przy koncach path,
  - tangent drift jest tlumiony przy koncach path,
  - overshoot jest tlumiony przy koncach path.
- Path-level endpoint lock liczy sie juz od prawdziwej dlugosci path, a nie od globalnego `max_render_length_px`.
- `build_strokes` nie traktuje juz kazdego `path_segment` jak calej sciezki:
  - geometria/stroke split opiera sie na lokalnej dlugosci segmentu,
  - `path_length` zostaje zachowane osobno dla semantyki stylu.
- Wewnetrzne laczenia segmentow sa juz oznaczane jako connected lokalnie,
  wiec taper/overshoot nie sa stosowane tak agresywnie w srodku chainu.
- Dropout w GPU nie jest juz tylko segment-local random:
  - uzywa `path_id`,
  - uzywa `path_t` cell,
  - jest bardziej spójny wzdluz calej sciezki.
- `emit_path_segments` nie preferuje juz `state.path_id` z lokalnego edge-state.
  Finalna kreska dostaje teraz kanoniczne `computed_path_id` wyliczane z:
  - kind,
  - skanonizowanych endpointow path,
  - hop count,
  - dlugosci path.
- `connect_paths` i `relax_path_owners` nie mieszaja juz `edge_id` do `path_id`.
  To zmniejsza ryzyko, ze identyfikacja sciezki bedzie skakac przy zmianie ownera
  albo przy przejsciu miedzy sasiednimi edge tego samego chainu.
- `relax_path_owners` propaguje teraz tez `segment_count` dalej niz tylko bezposredni sasiedzi.
  Dzięki temu:
  - `path_coherence`,
  - `importance`,
  - width/alpha modulation,
  - search gating
  lepiej odrozniaja prawdziwie dluzsze chainy od lokalnych 2-3 edge fragmentow.
- `emit_path_segments` ma teraz gestszy owner split:
  - dla krotszych ownerow zostaje lekki podzial,
  - dla dluzszych owner chainow srodek path jest dzielony na 4 segmenty zamiast 2.
  To daje lepsze `path_t0/path_t1` dla:
  - taper,
  - pressure,
  - alpha,
  - dropout,
  bo finalna kreska nie dostaje juz tak grubego, zbyt uproszczonego srodka sciezki.
- `build_strokes` dalej nie jest pelnym path rendererem, bo `path_t/path walk` sa jeszcze uproszczone.
- GPU frame buffers maja teraz tez osobny bufor `path_states`.

Glowne pliki:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_face_id.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_project_vertices.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_classify_edges.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_build_endpoint_bins.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_compact_owners.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_connect_paths.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_emit_path_segments.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_build_strokes.wgsl`

### 1.7. Debug GPU NPR

- Dodano jawny `debug_mode` do `NprGpuRealtimeTuning3d`.
- `debug_mode` przechodzi przez:
  - render-api,
  - YAML scene/preset hydration,
  - mesh runtime service,
  - Rhai bindings,
  - playground scene.
- Scena `comic-lines` ma teraz:
  - `V` jako cycle debug mode,
  - `7/8/9/0` jako szybkie skoki do wybranych widokow,
  - reaplikacje debug mode po zmianie presetu/modelu.
- `debug_mode` jest tez czescia presetow GPU, wiec stan debugowania mozna zapisac jawnie w YAML.

### 1.8. Codemap / tooling

- `amigo-codemap` dostal szybsze stale handling, incremental refresh, persistent anchor cache, coverage dla `concat.zip`, daemon auto-start i lepsze `open-set` / `change-plan`.
- `amigo-symbol-explorer` indeksuje teraz `wgsl`.

Glowne pliki:

## 2. Co zostalo do domkniecia

### 2.1. Path graph GPU

- Obecny GPU path graph jest juz wyraznie lepszy od wersji owner-edge-only, ale nadal jest uproszczony.
- Trzeba dalej poprawic przejscie od `path_links` do finalnej semantycznej sciezki tak, aby GPU dalo ten sam obrys i te same wewnetrzne linie co CPU reference.

Glowne pliki:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_compact_owners.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_emit_path_segments.wgsl`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_build_strokes.wgsl`
- `crates/engine/render-wgpu/src/renderer/npr/gpu_realtime.rs`

### 2.2. Parytet presetow CPU/GPU

- Nie wszystkie presety daja jeszcze rownie czytelny wynik na GPU jak na CPU.
- Trzeba doprowadzic do tego, zeby preset zmienial charakter kreski, a nie tylko gestosc losowych chainow.

Glowne pliki:

- `crates/engine/render-wgpu/src/renderer/npr/style.rs`
- `crates/engine/render-wgpu/src/renderer/shaders/npr_build_strokes.wgsl`
- `mods/playground-npr/scenes/comic-lines/npr-presets/*.yml`

### 2.3. Stabilnosc temporalna

- Efekt "kreska zostaje i wraca" jest juz wystawiony jako feature, ale dalej wymaga dostrojenia pod GPU.
- Chodzi o to, zeby wlaczenie temporal bylo kontrolowane i czytelne, a nie zeby maskowalo bledy grafu path.

Glowne pliki:

- `crates/engine/render-wgpu/src/renderer/shaders/npr_build_strokes.wgsl`
- `mods/playground-npr/scenes/comic-lines/scene.rhai`
- `mods/playground-npr/scenes/comic-lines/scene.yml`

### 2.4. Debug i diagnostyka

- Statystyki sa juz bogatsze, ale trzeba je nadal wykorzystac do szybkiego porownywania CPU vs GPU.
- Finalnie overlay powinien pomagac stroic path graph i preset, a nie tylko wyswietlac surowe liczby.

Glowne pliki:

- `crates/engine/render-api/src/stats.rs`
- `crates/apps/app/src/render_runtime.rs`
- `mods/playground-npr/scenes/comic-lines/scene.rhai`

## 3. Definicja "gotowe"

Za realne domkniecie `npr_v2` uznajemy dopiero sytuacje, w ktorej:

- GPU i CPU daja bardzo podobny obrys i podobna logike linii wewnetrznych dla tych samych presetow.
- GPU nie pokazuje przypadkowych dlugich kresek, prostokatow ani niestabilnych chainow zaleznych od kata kamery.
- Preset na GPU zmienia charakter kreski w sposob przewidywalny i porownywalny do CPU reference.
- Overlay/debug stats wystarczaja, zeby roznice CPU/GPU diagnozowac bez zgadywania.

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
- Debug GPU mozna teraz wymuszac jawnie w danych i runtime, bez mieszania tego z samym `camera.final`.
- Ostatni etap endpoint bins kompiluje sie poprawnie dla:
  - `amigo-render-wgpu`,
  - `amigo-app`.
- Ostatni etap `emit_path_segments -> build_strokes(path_segments)` tez kompiluje sie poprawnie dla:
  - `amigo-render-wgpu`,
  - `amigo-app`.

## 3. Znane problemy

### 3.1. GPU realtime nie jest jeszcze wiernym odpowiednikiem CPU reference

Najwazniejszy problem: GPU nadal renderuje visible edge segments bardziej niz prawdziwe stroke paths.
Efekt: niektore presety pokazuja dlugie, zwykle kreski albo linie, ktore tylko przyblizaja ksztalt modelu, ale nie maja tej samej charakterystyki obrysu i detalu co CPU.

### 3.2. Brak pelnego GPU path model

Brakuje GPU odpowiednika:

- stabilnego path walk,
- path ids,
- path-level simplification,
- path-level pressure,
- path-level humanization,
- dropout mask na poziomie stroke path,
- search/correction passes zgodnych z CPU.

To jest glowna roznica wizualna miedzy CPU i GPU.
Obecny stan jest juz lepszy niz czyste `visible_segments`, bo `endpoint bins`, `path_links`, owner compaction i `path_segments` sa aktywne, ale to nadal nie jest pelny `NprStrokePath`.

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
- debug widoki parity dla owner/face id.

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

2. Rozszerzyc obecny compact/path link layer do prawdziwego GPU endpoint binning.
   - Status: czesciowo zrobione.
   - Endpoint quantization w screen space juz istnieje.
   - Biny per line kind juz istnieja.
   - Brakuje jeszcze pelnego przejscia z endpoint entries do stabilnego path graphu.

3. Rozszerzyc obecny owner/path walk do prawdziwego GPU path walk.
   - Walk po wlascicielach edge/fragment.
   - Limity `max_terminal_walk_edges` i `max_chained_walk_edges`.
   - Rejection po angle/depth/kind.
   - Status: czesciowo zrobione.
   - `path_links` i owner compaction juz korzystaja z endpoint bucketow,
     a finalny render jest juz budowany z osobnego `path_segments` bufora.
   - Jest juz prosty walk owner->neighbor po `path_links`.
   - Lokalny owner graph jest juz stabilniejszy dzieki kanonicznemu wyborowi ownera w kompaktowalnym chainie.
   - Polaczenia w `path_links` sa juz tez filtrowane przez wzajemnosc wyboru sasiedniego edge.
   - Brakuje jeszcze stabilnego `path_id`, wielosegmentowego graph walku i path-level `t`.

4. Ujednolic stylizacje CPU/GPU.
   - Wydzielic wspolny model preset -> resolved style.
   - GPU ma uzywac tych samych semantyk co CPU.

5. Zrobic path-level humanization w GPU.
   - Noise po `path_id + t`.
   - Endpoint lock.
   - Pressure/alpha curves po arc length.
   - Status: czesciowo zrobione.
   - Obecny `build_strokes` czyta juz `path_segments`, uzywa `path_id` i `path_t`.
   - `path_t` nie jest juz sztywne `0/0.5/1`, bo walked chain jest dzielony na 4 segmenty.
   - Endpoint lock per path jest juz czesciowo aktywny.
   - `path_id` jest stabilniejsze i nie zalezy juz od owner edge index ani od kierunku chainu.
   - Dropout jest juz bardziej path-level niz edge-level.
   - Nadal brakuje prawdziwego `t` po pelnym wielosegmentowym luku i bardziej zaawansowanego path graphu.

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
   - Status: czesciowo zrobione.
   - `face-id/depth` jest juz globalne na poziomie calej ramki dla wszystkich GPU meshy.
   - `face_id_base` eliminuje kolizje trojkatow miedzy meshami.
   - Zostaje dalsze dopracowanie samplingu owner/debug parity.

9. Dodac debug mode do YAML.
   - Status: bazowo zrobione.
   - `gpu_realtime_tuning.debug_mode` przechodzi juz przez YAML, runtime i scene script.
   - Do rozszerzenia zostaja tylko dodatkowe widoki parity, jesli beda jeszcze potrzebne.

10. Dodac parity preset audit.
    - Dla kazdego presetu porownac CPU/GPU.
    - Oznaczyc, ktore pola jeszcze nie sa wspolne.

11. Zrobic cleanup po doprowadzeniu parity.
    - Usunac tymczasowe hinty segmentowe, ktore dubluja path ownership.
    - Nie zostawiac `v2`, `legacy`, `hybrid`, `auto`.

## 5. Minimalny nastepny etap

Najkrotsza droga do widocznej poprawy:

1. Naprawic face-id / owner sampling.
2. Domknac endpoint bins do pelnego path graphu.
3. Zrobic pierwszy GPU path walk bez zaawansowanej humanizacji.
4. Renderowac path segments zamiast raw visible segments.
5. Dopiero potem przenosic pressure/dropout/search z CPU na GPU.

Do tego momentu GPU moze miec wyzszy FPS, ale nie bedzie wygladac jak CPU reference.
