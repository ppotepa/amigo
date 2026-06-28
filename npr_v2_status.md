# Amigo NPR V1 - status wdrozenia

Status: aktualny zapis tego, co zostalo wdrozone i co zostaje do naprawy po pierwszej implementacji `gpu_realtime`.
Powiazany plan: `npr_v2.md`.

## 0. Postep globalny

- Szacowany postep bazowego routingu/pipeline NPR V1: okolo 99%.
- Szacowany postep `akira` / character manga ink V1: okolo 85%.
- Zrobione: kontrakt, YAML, routing CPU/GPU, debug mode, endpoint bins, owner compaction, connect/relax path owners, `path_segments`, path-level lock/dropout foundation.
- Zostalo: domkniecie parytetu wizualnego CPU/GPU, mocniejszy graph walk i stabilniejsze `path_t/path_id`, dalsze dopasowanie stylizacji do `cpu_reference` oraz domkniecie character manga ink w ramach V1: semantic line roles, importance score, line budget, pelniejsze material black fills i authored ink guides.
- Usunieta blokada startu `gpu_realtime` na czesci adapterow WGPU: compute bind group zostal rozbity na osobne layouty per pass, zamiast jednego ukladu przekraczajacego limit 8 storage bufferow na shader stage.
- Naprawiony drugi blad walidacji WGPU po rozbiciu layoutow: shadery GPU NPR deklaruja teraz `read` dla buforow, ktore sa tylko czytane, zamiast wymagac `read_write` niezgodnego z layoutem pipeline'u.
- Naprawione uzycie `indirect_args`: licznik `path_segments` zostal przeniesiony poza pierwsze cztery pola `DrawIndirectArgs`, zeby compute pass nie nadpisywal `first_vertex` przed `draw_indirect`.
- Offscreen UI upload nie uzywa juz `create_buffer_init` dla `amigo-offscreen-ui-color-vertices` / `amigo-offscreen-ui-texture-vertices`; bufor jest tworzony jawnie i wypelniany przez `queue.write_buffer`, co omija panic w `get_mapped_range`.
- GPU NPR command buffer jest teraz submitowany razem z offscreen render command bufferem po CPU uploadach. Usuwa to startup panic `Queue::write_buffer: Buffer with 'amigo-offscreen-ui-color-vertices' label is invalid`, ktory wynikal z submitu GPU NPR przed kolejnymi uploadami UI.
- Presety NPR moga juz deklarowac `pipeline` z osobnymi strategiami `candidate/path/stroke/fill/hatching/budget/temporal`; `akira` i `akira_cpu_reference` uzywaja tego kontraktu jako pierwsze presety pod character manga ink.
- Strategie `pipeline` sa juz kodowane do uniformow GPU (`pipeline0`, `pipeline1`), a `build_strokes` uzywa pierwszego zachowania `akira_ink` / character budget do wzmocnienia silhouette i przygaszenia technicznych feature/contact lines.
- `path_strategy` steruje juz emission: `direct_visible_segments` zachowuje prosty segment-local path, a `stable_stroked_paths` uruchamia GPU path walk.
- Default GPU Comic i Akira V1 uzywaja teraz `stable_stroked_paths` z konserwatywnym `max_chained_walk_edges: 1`. Kontrolowane offscreen smoke testy potwierdzaja widoczny ink output oraz zgodnosc regionu GPU/CPU reference dla defaultu.
- Sciezka `stable_stroked_paths` dispatchuje path graph MVP miedzy `classify_edges` i `emit_path_segments`: `clear_endpoint_heads`, `build_endpoint_bins`, `compact_owners`, `connect_paths`, dwie iteracje `relax_path_owners`, a dopiero potem `emit_path_segments` i `build_strokes`. Path graph jest aktywny dla wybranych presetow, ale nadal wymaga dalszego strojenia przed luzowaniem chain budgetu.
- `endpoint_heads` sa czyszczone osobnym compute passem `npr_clear_endpoint_heads`, a nie w tym samym dispatchu, ktory wpisuje endpointy. Usuwa to race condition pomiedzy atomic clear i atomic insert.
- Uniform GPU niesie teraz realne `edge_count`, `vertex_count` i `triangle_count` aktualnego joba. Shadery `build_endpoint_bins`, `compact_owners`, `emit_path_segments` i `build_strokes` odcinaja `visible_segments` po aktywnym `edge_count`, zamiast traktowac pojemnosc bufora jako liczbe poprawnych edge'y. To zmniejsza ryzyko losowych linii po zmianie modelu, presetow albo po alokacji wiekszego bufora.
- `character_semantic` / character budget wplywa juz przed stroke build: klasyfikacja GPU podnosi progi feature/contact i chroni silhouette nizszym progiem dlugosci.
- Bufor finalnych `stroke_segments` skaluje sie juz z liczba slotow `path_segments` na edge, wiec odblokowany GPU path walk nie ucina wiekszosci wygenerowanych segmentow przez zbyt mala pojemnosc outputu.
- `material_black_mass` ma pierwszy jawny V1 hook: `NprLineSettings3d.black_mass_material_ids` przechodzi z YAML, jest kodowane do GPU bitmaski i render loop rysuje czarny fill dla wskazanych material IDs bez zgadywania po nazwie modelu.
- `black_mass_material_ids` jest traktowane jako modelowy override: `apply_npr_preset` zachowuje istniejaca liste, gdy preset jej nie definiuje. Akira preset nie zawiera juz globalnych material IDs, a Khronos Male deklaruje je w swoim `Mesh3D.npr`.
- Doszla druga jawna rola materialowa V1: `ink_detail_material_ids`. Khronos Male deklaruje material IDs twarzy/oczu/brwi jako detail ink, GPU classify obniza dla nich prog feature/seam, a CPU reference uzywa zgodnego lokalnego progu dlugosci bez luzowania calego modelu.
- Khronos Male ma test regresji statycznego importu GLB: loader musi wczytac materialy body, hair i face/eye detail, zeby uniknac regresji typu "widac tylko twarz" wynikajacej z czesciowego importu geometrii.
- GPU endpoint binning zapisuje teraz jawny `endpoint_vertex`, a `compact_owners` laczy kandydatow tylko wtedy, gdy endpointy reprezentuja ten sam source vertex. Partial visibility endpoints dostaja invalid vertex i nie trafiaja do endpoint bins. To ogranicza fałszywe dlugie chainy wynikajace z samej bliskosci ekranowej bez fizycznego polaczenia w geometrii.
- Dla primary contour (`silhouette` / `boundary`) endpoint vertex ma mala tolerancje widocznosci przy prawie pelnych runach (`t` blisko 0/1), co odzyskuje czesc ciaglosci obrysu bez luzowania feature/seam/crease lines.
- Naprawiony live crash hosted WGPU po kilku klatkach (`Queue::write_buffer: amigo-offscreen-ui-color-vertices is invalid`). Przyczyna byla wczesniejsza utrata/invalidacja stanu GPU przez zbyt kosztowny `compact_owners` endpoint bucket scan i zbyt szeroki indirect draw budget, a UI buffer byl tylko pierwszym miejscem raportowania bledu.
- `compact_owners` ma teraz twardy limit skanowania endpoint bucketu (`MAX_ENDPOINT_BUCKET_SCAN`), wiec compute pass jest bounded i nie moze wejsc w ekstremalnie drogi linked-list traversal.
- `build_strokes` jest domkniety dodatkowym GPU pass `npr_clamp_indirect_args`, ktory przycina licznik instancji draw do realnej pojemnosci `stroke_segments` bez readbacku i bez CPU fallbacku.
- GPU `stroke_segments` capacity jest teraz budzetowane per `NprBudgetStrategy3d`, zamiast traktowac surowy iloczyn `edge_count * path_slots * passes * segments` jako realny draw budget. Dla Soldiera default spadl z okolo 43 MB do okolo 4.8 MB bufora stroke instances.
- GPU path quality zostalo dodatkowo zaostrzone dla detali: `compact_owners` wymaga teraz mocniejszego alignmentu, bardziej zbalansowanych dlugosci i mniejszego depth gap dla `crease/seam/feature/contact`, ale nie zaostrza `silhouette/boundary`.
- `build_strokes` ma pierwszy runtime line suppression dla short/low-coherence detail segments w `akira_ink` i character budget. Sylwetka i boundary sa chronione, a techniczne krotkie feature/contact lines bez kontekstu sciezki sa odrzucane przed generowaniem passow.
- `sparse_character_hatching` nie jest juz martwym polem kontraktu: GPU `build_strokes` umie dodac jeden krotki, deterministyczny hatch-pass dla wybranych wewnetrznych feature lines, gdy preset jawnie ustawi `pipeline.hatching_strategy: sparse_character_hatching`. Pojemnosc `stroke_segments` uwzglednia ten dodatkowy pass.
- CPU reference ma zgodny V1 odpowiednik sparse character hatching: `build_npr_stroke_pass_plan` dodaje krotki `Hatch` pass tylko dla wewnetrznych feature lines w character/akira budget, z deterministycznym zakresem `active_t0/active_t1`. Akira i Akira CPU Reference maja teraz wlaczone `sparse_character_hatching`.

## 0.2. Stan wprost

### Domkniete

- Kontrakt `gpu_realtime` / `cpu_reference`.
- Domyslny GPU runtime bez `auto` i bez `hybrid`.
- Routing YAML -> scene hydration -> mesh runtime -> render loop.
- Playground controls dla presetow, modeli, strategii i debug mode.
- Kontrakt preset-level pipeline strategies w `NprLineSettings3d`.
- Kodowanie preset-level pipeline strategies do uniformow GPU.
- Podstawowy GPU pipeline:
  - `face-id`,
  - `project_vertices`,
  - `classify_edges`,
  - `endpoint bins`,
  - `owner compaction`,
  - `connect_paths`,
  - `relax_path_owners`,
  - `emit_path_segments`,
  - `build_strokes`.

### Dziala, ale nie ma jeszcze pelnego parytetu

- `stable_stroked_paths` jako aktywny V1 runtime dla defaultowego viewportu i Akiry przy konserwatywnym chain budget.
- GPU path walk i identyfikacja sciezki w `stable_stroked_paths`; sciezka jest zaimplementowana i przechodzi kontrolowane offscreen smoke testy dla defaultu/Akiry.
- GPU `path_t` i `path_id`.
- GPU dropout, endpoint lock, taper i search shaping.
- Zgodnosc presetow CPU/GPU.
- Temporal/path smoothing w ujeciach granicznych.

### Nadal do zrobienia

- Dalsza walidacja `stable_stroked_paths` na innych modelach, presetach i katach kamery przed zwiekszeniem `max_chained_walk_edges` powyzej 1.
- Dalsze ograniczenie falszywych dlugich chainow dla niektorych presetow i katow kamery.
- Lepsza zgodnosc GPU z CPU dla pressure/alpha/humanization/search.
- Cleanup tymczasowych hintow `next_*` / `alt_next_*` po pelnym przejsciu na mocniejszy path graph.
- Ostateczne strojenie budzetow, limitow i debug stats pod GPU realtime.
- Prawdziwe animacje skinned GLB dla Khronos Male nie sa wykonywane runtime'owo w V1; obecny loader renderuje statyczna geometrie mesha. HUD i log sceny opisuja to teraz jawnie jako `skinning unsupported in V1`, zeby scena nie udawala gotowego clip/skinning evaluation.
- Character NPR dla `akira` jest czescia zakresu V1, ale nie jest jeszcze wykonany runtime'owo:
  - runtime execution strategii `akira_ink` i `budget` ma pierwszy etap w GPU stroke shaderze;
  - runtime execution strategii `character_semantic` ma pierwszy etap w GPU candidate filtering;
  - runtime execution strategii `material_black_mass` ma pierwszy etap oparty o jawne `black_mass_material_ids`;
  - runtime execution strategii `stable_arc_length` nie jest jeszcze pelne;
  - semantic roles dla postaci (`FaceDetail`, `HairMassBoundary`, `MuscleForm`, `ClothFold`);
  - importance scoring i line budget/suppression;
  - bogatsze material ink roles poza prostym `black_mass_material_ids` / `ink_detail_material_ids`;
  - authored `ink_guides` dla twarzy, wlosow, miesni i fald ubrania;
  - apparent ridges / lepsze suggestive contours;
  - dalsze strojenie sparse character hatching zamiast globalnego halftone.

### Akira V1 status

- Zrobione:
  - plikowy preset `akira` i `akira_cpu_reference`;
  - line-only tusz: czarna kreska, mocna sylwetka, cienkie feature lines, brak search/dropout;
  - deklaracja pipeline: `character_semantic`, `stable_stroked_paths`, `akira_ink`, `material_black_mass`, `stable_arc_length`;
  - propagacja pipeline do GPU uniformow;
  - pierwsze shaderowe zachowanie `akira_ink`: mocniejsza silhouette, slabsze feature/contact, mniej wobble detali;
  - `akira` V1 uzywa teraz `stable_stroked_paths` z konserwatywnym `max_chained_walk_edges: 1`;
  - `character_semantic` w GPU classify filtruje slabe feature/contact przy zachowaniu silhouette;
  - `sparse_character_hatching` jest wlaczone w Akirze jako V1 stroke-level feature, z odpowiednikiem w CPU reference;
  - capacity GPU stroke output jest policzone od liczby slotow path walk, nie tylko od liczby surowych edge'y;
  - `black_mass_material_ids` dla Khronos Male sa zapisane przy modelu, nie w globalnym presecie: hair `4,5`, mouth/iris/brow/lash/eyeline `6,7,11,12,13`;
  - `ink_detail_material_ids` dla Khronos Male sa zapisane przy modelu: mouth/iris/brow/lash/eyeline `6,7,11,12,13`;
  - render loop rysuje czarne masy tylko gdy preset deklaruje `pipeline.fill_strategy: material_black_mass` i niepusta liste material IDs;
  - GPU classify i CPU reference uzywaja detail material role do zachowania krotszych linii twarzy/oczu/brwi;
  - test loadera Khronos Male potwierdza, ze statyczny import obejmuje body, hair i face/eye detail material IDs;
  - rejestracja w scenie i HUD/preset switching.
- Do zrobienia w tym samym V1:
  - pelniejsze wykonanie `character_semantic` w runtime;
  - dalsze strojenie material black masses dla wlosow/oczu/cieni;
  - `ink_guides` i role linii dla twarzy/wlosow/ubran;
  - line budget/importance scoring;
  - dalsze strojenie sparse character hatching po porownaniu z realnym viewportem.

### Ostatnia weryfikacja runtime

- `cargo test -p amigo-app playground_npr_preview_renders_gpu_and_cpu_reference_default_gpu_comic -- --ignored --nocapture` przechodzi na lokalnym WGPU adapterze.
- `cargo test -p amigo-app playground_npr_preview_renders_paper_and_ink_edges -- --ignored --nocapture` przechodzi na lokalnym WGPU adapterze.
- `cargo test -p amigo-app playground_npr_preview_renders_stable_stroked_paths_gpu_comic -- --ignored --nocapture` przechodzi na lokalnym WGPU adapterze.
- `cargo test -p amigo-app playground_npr_preview_renders_akira_gpu_preset -- --ignored --nocapture` przechodzi na lokalnym WGPU adapterze.
- `cargo test -p amigo-scene compiled_playground_npr_scene_registers_file_backed_npr_presets` przechodzi.
- `cargo test -p amigo-render-wgpu npr` przechodzi.
- `cargo test -p amigo-app npr` przechodzi; cztery testy offscreen pozostaja ignorowane w zwyklym filtrze, bo wymagaja lokalnego adaptera i readbacku.
- Headless smoke dla sceny `playground-npr/comic-lines` przechodzi jako czesc `cargo test -p amigo-app npr`.
- `cargo test -p amigo-render-api`, `cargo test -p amigo-3d-mesh`, `cargo test -p amigo-input-actions`, `cargo test -p amigo-input-api`, `cargo test -p amigo-input-winit` i `cargo test -p amigo-scripting-rhai` przechodza po ostatniej walidacji.
- `cargo test -p amigo-render-wgpu` i `cargo test -p amigo-scene` przechodza jako pelne wlascicielskie crate checks.
- `cargo build -p amigo-launcher` przechodzi.
- Hosted smoke `target\debug\amigo-launcher.exe --profile playground-npr --hosted` przezywa kontrolowane 12 sekund na lokalnym adapterze WGPU bez panic/validation error; proces zostal zakonczony recznie po czasie testu.
- Po dodatkowym zaostrzeniu `compact_owners` i `build_strokes` powtorzono `cargo test -p amigo-render-wgpu npr`, `cargo test -p amigo-app playground_npr_preview -- --ignored --nocapture --test-threads=1` oraz hosted smoke 12 sekund; wszystkie przeszly.
- Pelne `cargo test -p amigo-app` nadal nie jest zielona bramka dla tej paczki: padaja niezalezne od NPR testy hot-reload/scen innych playgroundow oraz jeden stary invariant zostal juz zaktualizowany na obecny `submit_wgpu_frame_render_request` flow. Zakres NPR jest walidowany filtrami `npr` i offscreen smoke testami powyzej.

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
- `compact_owners` wymaga teraz zgodnosci topologicznego endpoint vertex:
  - endpoint entry niesie `endpoint_vertex`,
  - endpoint entry jest emitowany tylko dla rzeczywistego poczatku/konca source edge,
  - ucięte przez visibility fragmenty ze srodka edge nie sa juz traktowane jak topologiczny join,
  - `silhouette` i `boundary` moga zachowac endpoint, jezeli widoczny run jest bardzo blisko prawdziwego konca edge,
  - feature/seam/crease pozostaja restrykcyjne i nie dostaja takiej tolerancji,
  - kandydat nie przechodzi scoringu, jezeli dotyka innego source vertex,
  - redukuje to przypadki, w ktorych bliskie ekranowo, ale niepolaczone czesci modelu byly zszywane w jedna kreske.
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
- `walk_path_endpoint` niesie teraz dodatkowy poziom punktu posredniego (`mid_point/mid_length`),
  a `emit_path_segments` potrafi z niego korzystac przy wielohopowych extension chainach.
  To oznacza, ze GPU nie sklada juz dluzszej extension geometrii tylko z:
  - `near`,
  - `penultimate`,
  - `final`,
  ale ma jeszcze jeden wezel posredni do budowy sensowniejszego luku i lepszego `path_t`.
- `build_strokes` potrafi teraz emitowac do 3 ribbon segmentow na pass,
  zamiast sztywnego 1-2 segment split.
  Dla dluzszych i szerszych fragmentow path daje to:
  - wierniejsza aproksymacje krzywizny,
  - mniej brutalny srodkowy zalom,
  - lepsze przejscie width/alpha/taper od startu do konca kreski.
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

Za realne domkniecie bazowego NPR V1 uznajemy dopiero sytuacje, w ktorej:

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
   - Naprawiono blad zasobow GPU: kazdy GPU NPR mesh/job ma teraz osobny uniform buffer.
     Wczesniej jeden globalny uniform buffer byl nadpisywany podczas nagrywania jednego command buffera,
     przez co passy mogly widziec ostatni zestaw uniformow zamiast wlasnego.
   - Dodano trace GPU NPR na stdout:
     pierwsze 4 ramki loguja sie zawsze, a pelny trace wlacza `AMIGO_NPR_GPU_TRACE=1`.
     Log obejmuje jobs, meshe, rozmiary buforow, face-id target, uniform size i ostatni krok przed `write_buffer`.
   - Trace GPU NPR czysci terminal na pierwszej ramce i ma kolorowe poziomy `START`, `INFO`, `JOB`, `STEP`, `ALLOC`, `WRITE`, `OK`.
     Clear mozna wylaczyc przez `AMIGO_NPR_GPU_TRACE_CLEAR=0`, a kolory przez `AMIGO_NPR_GPU_TRACE_COLOR=0` albo `NO_COLOR=1`.
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
