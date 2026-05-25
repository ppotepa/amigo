# Plan: architektoniczne naprawy po review `concat-output-rec6v`

Ten plik zastepuje poprzedni szeroki baseline. Jest lista konkretnych prac wynikajacych z review architektury. Kazdy punkt ma blad, dowod, operacje, walidacje i zakres, ktorego nie ruszac.

## Zasady wykonania

- Start i koniec kazdego punktu: `git status --short`.
- Najpierw owner crate, potem downstream.
- Nie uzywac `cargo check --workspace` jako pierwszej walidacji.
- Nie dodawac `legacy`, `v2`, `v3`, `compat`, shimow ani cichych fallbackow.
- Nie traktowac `concat-output-rec6v.txt` jako source file. To tylko artefakt review.
- Kazdy punkt ma byc osobnym, malym patchem albo jawnie opisanym etapem.

## Kolejnosc prac

1. Guardy i stan operacyjny: punkty 4, 7, 8.
2. Granica app/runtime: punkty 1, 2.
3. Render contracts i renderer heuristics: punkty 3, 5.
4. Hotspoty utrzymywalnosci: punkty 6, 9.
5. Sekwencjonowanie i finalne invariant checks: punkt 10.

## Agenty i rownoleglosc

Kazdy agent pracuje w rozlacznym write-seciku. Agent nie jest sam w repo: nie wolno mu revertowac cudzych zmian, przepisywac plikow spoza zakresu ani wykonywac workspace-wide checkow jako pierwszej walidacji.

| Agent | Zakres | Write set | Punkty | Walidacja owner |
| --- | --- | --- | --- | --- |
| A0 Plan/Coordinator | utrzymanie `plan.md`, kolejnosc, finalne raporty | `plan.md` | 8, 10 | `git diff --check -- plan.md` |
| A1 Runtime Facade | usuniecie publicznej fasady plugin crate'ow | `crates/runtime/bundles/src/lib.rs`, `crates/runtime/bundles/src/plugin_crates.rs`, najblizsze importy w `crates/runtime/bundles/src/**` | 1, 7 | `cargo check -p amigo-runtime-bundles` |
| A2 App Boundary | app nie zna domain plugin/services | `crates/apps/app/Cargo.toml`, `crates/apps/app/src/scene_preview.rs`, `crates/apps/app/tests/architecture.rs` | 2, 7 | `cargo check -p amigo-app` |
| A3 Render Guard | arch guardy dla renderer guessing | `crates/engine/render-wgpu/tests/architecture_regressions.rs` | 3, 7 | `cargo test -p amigo-render-wgpu architecture` |
| A4 Plugin Docs | przywrocenie znaczeniowych docs dotknietych pluginow | `plugins/**/docs/*.md`, plugin-level deleted `*.md` only | 4 | `git diff --check -- plugins` |
| A5 PostFX Registry | descriptor/executor registry i invariants | `crates/engine/render-api/src/post_fx_model/**`, `crates/engine/render-api/src/tests.rs`, `crates/engine/render-wgpu/src/renderer/service/post_fx/**` | 5 | `cargo check -p amigo-render-api`; `cargo check -p amigo-render-wgpu` |
| A6 Maintainability Split | podzial najwiekszych hotspotow bez zmiany behavior | `crates/engine/scene/src/component_metadata.rs`, `plugins/camera/camera-core/src/runtime/service.rs`, `plugins/camera/camera-profiles/src/runtime/profiles.rs` | 6, 9 | owner crate checks per file |

Pierwsze rownolegle uruchomienie:
- Start: A1, A3, A4, A5.
- Wstrzymac A2 do zakonczenia A1, bo moze potrzebowac helpera w `runtime/bundles`.
 - A6 wykonywac dopiero po A1-A5, jako osobny rownolegly etap na rozdzielonych write setach.
- A0 wykonuje lokalnie aktualizacje planu i integracje wynikow.

Status wykonania:
- A0: done - `plan.md` zawiera podzial agentow i wynik pierwszych fal.
- A1: done - usunieto publiczny `plugin_crates`, dodano runtime-bundles guard.
- A2: done - app preview korzysta z neutralnego helpera w `runtime/bundles`, app nie ma direct camera plugin deps.
- A3: done - render-wgpu architecture guard przechodzi po usunieciu procedural material guessing.
- A4: done - `plugins/**/docs/*.md` nie sa juz usuniete.
- A5: done - PostFX ma descriptor registry i executor invariant.
- A6: done - wydzielono `component_metadata/model.rs`, `camera-core/runtime/service/focus_transition.rs` i `camera-profiles/runtime/profiles/catalog.rs`.

Focused changeset do review/staging:
- `plan.md`
- `crates/apps/app/Cargo.toml`
- `crates/apps/app/src/scene_preview.rs`
- `crates/apps/app/tests/architecture.rs`
- `crates/runtime/bundles/src/lib.rs`
- `crates/runtime/bundles/src/offscreen_runtime_frame.rs`
- `crates/runtime/bundles/src/render_extractor_bridges/visual_2d_items.rs`
- `crates/runtime/bundles/src/render_session.rs`
- `crates/runtime/bundles/src/runtime_summary.rs`
- `crates/runtime/bundles/tests/architecture_regressions.rs`
- `crates/engine/render-api/src/post_fx_model/render_descriptor.rs`
- `crates/engine/render-api/src/tests.rs`
- `crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs`
- `crates/engine/render-wgpu/src/renderer/service/render/visual_source_buffer_pass/procedural_material.rs`
- `crates/engine/render-wgpu/tests/architecture_regressions.rs`
- `crates/engine/scene/src/component_metadata.rs`
- `crates/engine/scene/src/component_metadata/model.rs`
- `plugins/camera/camera-core/src/runtime/service.rs`
- `plugins/camera/camera-core/src/runtime/service/focus_transition.rs`
- `plugins/camera/camera-profiles/src/runtime/profiles.rs`
- `plugins/camera/camera-profiles/src/runtime/profiles/catalog.rs`
- `plugins/*/*/docs/*.md`

Nie stagingowac w tym changesecie:
- legacy root plugin markdown: `plugins/*-*.md`
- repo/root docs deletions z audytu i snapshotow
- `mods/**`, `crates/2d/**`, `crates/3d/**`, scripting/devtools/editor changes spoza tej listy
- untracked runtime files poza `offscreen_runtime_frame.rs`

## Pozostale prace: plany dla agentow

Te plany dotycza tego, co zostalo po A0-A6. Kazdy agent musi zaczac od `git status --short`, uzyc `amigo-codemap` albo waskiego `rg`, pracowac tylko w swoim write setcie i zakonczyc owner validation. Nie uruchamiac wszystkiego naraz, jesli write sety sie przecinaja.

| Agent | Cel | Write set | Blokady | Walidacja |
| --- | --- | --- | --- | --- |
| B0 Focused Staging Auditor | przygotowac staging/review tylko A0-A6 bez cudzych zmian | tylko lista `Focused changeset do review/staging` | nie stagingowac `plugins/*-*.md`, root docs deletions, `mods/**`, nowych runtime files poza lista | `git diff --cached --name-status`; targeted checks z A0-A6 |
| B1 Documentation Canonicalizer | zdecydowac, ktore skasowane docs sa legacy, a ktore trzeba przywrocic/promowac | root docs, `docs/architecture/**`, `reference/**`, `workflows/**`, `templates/**`, `crates/*.md`, `plugins/*-*.md` | nie ruszac `plugins/**/docs/*.md` z A4 bez osobnej decyzji | `git diff --check -- docs reference workflows templates crates plugins *.md` |
| B2 Generated/Snapshot Cleanup | usunac albo zignorowac review/generated artifacts po przeniesieniu wiedzy do `plan.md` | `.amigo/codemap.coverage.generated.md`, `codemap.index.md`, `concat-output-*.txt`, `concat-output-*.zip`, `plan.txt` | nie kasowac source docs; nie traktowac concat jako source | `git status --short`; `rg -n "concat-output|plan.txt" plan.md README.md docs AGENTS.md` |
| B3 Runtime/Render Contract Split | przejrzec nowe untracked contract files i zdecydowac, czy sa czescia migracji | `crates/runtime/bundles/src/event_pipeline.rs`, `crates/runtime/bundles/src/render_scene_view.rs`, `crates/engine/render-api/src/commands_2d.rs`, `commands_3d.rs`, `render_scene_view.rs` | nie wciagac do A0-A6 staging bez review API | `cargo check -p amigo-render-api`; `cargo check -p amigo-runtime-bundles` |
| B4 Plugin Metadata Providers | doprowadzic nowe plugin metadata files do finalnej granicy provider registration | `plugins/gfx/tilemap-2d/src/scene/metadata.rs`, `plugins/vfx/particles-2d/src/scene/metadata.rs`, najblizsze `scene/mod.rs`, provider tests | nie duplikowac metadata w engine scene | `cargo check -p amigo-tilemap-2d-plugin`; `cargo check -p amigo-particles-2d-plugin` |
| B5 Plugin Source Review | rozdzielic szerokie zmiany w plugin source na domenowe patche | `plugins/**/src/**`, z wykluczeniem plikow juz w A6/B4 | nie mieszac camera, gfx, lighting, postfx, vfx w jednym commicie | owner `cargo check -p <plugin-crate>` per domena |
| B6 Engine/App Residual Review | sprawdzic pozostale zmiany w app/engine poza focused changeset | `crates/apps/app/src/**`, `crates/engine/**`, z wykluczeniem A0-A6 i B3 | nie zmieniac app boundary bez testu architektury | `cargo check -p <owner-crate>`; `cargo test -p amigo-app --test architecture` jesli app |
| B7 Mods Authored Data Review | sprawdzic, czy zmiany w `mods/**` sa authored data migration, czy przypadkowy churn | `mods/**` | mods nie moga definiowac engine behavior przez renderer hacks | scene/preset targeted validation; `git diff --check -- mods` |
| B8 Final Integration Validator | po B0-B7 uruchomic koncowy zestaw invariantow i spisac ryzyka | brak write set poza `plan.md` raportem | nie robic workspace-wide check zanim owner crates sa zielone | targeted `rg`; owner checks; wybrane downstream tests |

Status drugiej fali:
- B0: done - indeks pusty, focused changeset A0-A6 przechodzi `git diff --check`; nowych plikow nie stagingowano.
- B1: done - `PROJECT.md` i `THIRD_PARTY_NOTICES.md` przywrocone jako canonical docs; `crates/README.md`, `plugins/README.md`, `templates/plugin/README.md` uproszczone; root/docs/reference/workflow/template snapshots final-delete poza plugin-local docs.
- B2: done - generated/review artifacts pozostaja poza focused staging; `concat-output-rec6v.*` zostaje untracked do czasu potwierdzenia; `codemap.index.md` zostaje jako human-readable source doc.
- B3: done - `commands_2d.rs`, `commands_3d.rs`, `render_scene_view.rs`, `event_pipeline.rs` sa finalnymi contract/bridge files; ryzyko: runtime camera selection nadal ma entity-name lookup.
- B4: done - tilemap i particles metadata providers sa finalne plugin-owned files; dodano narrow tests rejestracji.
- B5: done - plugin source rozpisany per rodzina; pierwsza kolejka: camera, potem gfx/lighting/postfx/vfx/gameplay.
- B6: done - app/engine residual review rozpisany per grupa; owner checks dla app, scene, render-api, render-wgpu, devtools/editor, scripting/audio/ui sa zielone.
- B7: done - `mods/**` to authored-data-only migration component `type:` na fully-qualified plugin IDs; brak nowych render hackow; app particle preset test zablokowany przez unrelated app compile errors.
- B8: done - finalne architecture/render descriptor tests przechodza; residuale zostaly spisane ponizej.

Trzecia fala residuali:
- R1 Runtime exports: done - `render_packet_services.rs` i `two_d.rs` nie maja juz szerokich `pub use amigo_*`; runtime internals importuja owner crates bez przywracania fasady.
- R2 App tests/runtime helpers: done - app callsites i test/support paths importuja typy domenowe z owner crates albo neutralnych kontraktow, nie z runtime-bundles facade.
- R3 Renderer plate relight: done - usunieto beacon lookup po `owner_entity() == source.owner` i `BeaconLight2D`; renderer uzywa jawnych danych z `LightSource2dCommon`.
- R4 Codemap docs: done - `codemap.index.md` zostaje jako canonical human-readable codemap taxonomy guide; nie jest generated artifact.
- R5 App particle preset test: done - app compile errors po brakujacym `amigo_focus_depth_plugin` re-export i `SceneEntityId` mismatch sa usuniete; preset test przechodzi.

Trzecia fala changeset do review/staging:
- `codemap.index.md`
- `crates/apps/app/Cargo.toml`
- `crates/apps/app/src/assets/mod.rs`
- `crates/apps/app/src/host_runtime.rs`
- `crates/apps/app/src/particle_presets.rs`
- `crates/apps/app/src/render_runtime.rs`
- `crates/apps/app/src/render_runtime/tests.rs`
- `crates/apps/app/src/scene_runtime/ui_support.rs`
- `crates/apps/app/src/script_runtime/test_helpers.rs`
- `crates/apps/app/src/tests/bootstrap_tests.rs`
- `crates/apps/app/src/tests/**`
- `crates/runtime/bundles/src/event_pipeline.rs`
- `crates/runtime/bundles/src/render_packet_services.rs`
- `crates/runtime/bundles/src/render_session.rs`
- `crates/runtime/bundles/src/runtime_service_types.rs`
- `crates/runtime/bundles/src/two_d.rs`
- `crates/runtime/bundles/tests/architecture_regressions.rs`
- `crates/engine/render-wgpu/src/renderer/service/render/plate_relight.rs`
- `crates/engine/render-wgpu/tests/architecture_regressions.rs`

Pozostale ryzyka po trzeciej fali:
- `concat-output-rec6v.*` nadal jest untracked review artifact; nie stagingowac, dopoki nie zapadnie decyzja o usunieciu.
- B3 wskazal runtime camera selection entity-name lookup jako osobne ryzyko; nie bylo w zakresie R1-R5.

### B0 Focused Staging Auditor

Operacje:
- READ `Focused changeset do review/staging`.
- READ `git status --short -- <focused paths>`.
- ADD do indeksu tylko pliki z focused changeset, jesli staging jest celem tej rundy.
- VERIFY `git diff --cached --name-status` nie zawiera root docs deletions, `mods/**`, `plugins/*-*.md`, ani untracked runtime files spoza listy.

Nie zmieniac:
- Nie poprawiac kodu w trakcie staging review.
- Nie uzywac szerokiego `git add .`.
- Nie cofac cudzych zmian.

Walidacja:
- `git diff --cached --name-status`
- `git diff --cached --check`
- Powtorzyc zielone checks z A0-A6, jesli staging ma isc do commita.

### B1 Documentation Canonicalizer

Operacje:
- READ status usunietych docs w grupach: root, `docs/architecture`, `reference`, `workflows`, `templates`, crate-level `*.md`, legacy plugin-level `plugins/*-*.md`.
- DECIDE per grupa: final delete, restore, albo przeniesienie tresci do canonical docs.
- MODIFY tylko canonical docs, jezeli tresc ma zostac zachowana.
- DELETE legacy snapshots tylko z jasna notatka w `plan.md`.

Nie zmieniac:
- Nie ruszac `AGENTS.md` bez osobnej potrzeby.
- Nie przywracac placeholderow tylko po to, zeby status byl czysty.
- Nie mieszac docs cleanup z runtime behavior.

Walidacja:
- `git diff --check -- docs reference workflows templates crates plugins *.md`
- `git status --short | rg "^( D| M|\\?\\?) (docs|reference|workflows|templates|crates/.+\\.md|plugins/.+\\.md|.*\\.md)"`

### B2 Generated/Snapshot Cleanup

Operacje:
- READ references do `concat-output-*`, `codemap.index.md`, `.amigo/codemap.coverage.generated.md`, `plan.txt`.
- DELETE albo zostawic untracked review artifacts zgodnie z decyzja B1.
- UPDATE `plan.md`, jesli artifact ma zostac poza stagingiem.

Nie zmieniac:
- Nie usuwac `concat-output-rec6v.txt` zanim wszystkie wnioski sa juz w `plan.md`.
- Nie stagingowac generated codemap output razem z source changes.

Walidacja:
- `rg -n "concat-output|codemap.index|plan.txt" plan.md README.md docs AGENTS.md`
- `git status --short -- .amigo codemap.index.md concat-output-*.txt concat-output-*.zip plan.txt`

### B3 Runtime/Render Contract Split

Operacje:
- READ nowe pliki: `commands_2d.rs`, `commands_3d.rs`, `render_scene_view.rs`, `event_pipeline.rs`.
- TRACE exports w `crates/engine/render-api/src/lib.rs` i `crates/runtime/bundles/src/lib.rs`.
- DECIDE czy to finalne contract files, czy staging leftover.
- ADD tests/invariants tylko dla public contract boundaries.

Nie zmieniac:
- Nie dodawac app-side domain wiring.
- Nie robic fallbackow dla brakujacych contributions.
- Nie laczyc z PostFX registry zmianami, ktore juz sa w A5.

Walidacja:
- `cargo check -p amigo-render-api`
- `cargo check -p amigo-runtime-bundles`
- targeted `rg -n "pub mod commands_2d|render_scene_view|event_pipeline" crates/engine/render-api crates/runtime/bundles`

### B4 Plugin Metadata Providers

Operacje:
- READ `plugins/gfx/tilemap-2d/src/scene/metadata.rs` i `plugins/vfx/particles-2d/src/scene/metadata.rs`.
- TRACE provider registration do plugin entrypoints.
- MODIFY plugin scene modules, jesli metadata provider nie jest finalnie eksportowany.
- ADD narrow tests dla descriptor/provider registration.

Nie zmieniac:
- Nie przenosic plugin-specific metadata do `crates/engine/scene`.
- Nie dodawac placeholder docs bez tresci.

Walidacja:
- `cargo check -p amigo-tilemap-2d-plugin`
- `cargo check -p amigo-particles-2d-plugin`
- targeted tests, jesli istnieja provider tests.

### B5 Plugin Source Review

Operacje:
- Split status na rodziny: camera, gfx, lighting, materials, postfx, vfx, gameplay, devtools.
- Dla kazdej rodziny: READ tylko touched files i najblizsze tests.
- MODIFY tylko jesli patch jest niezbedny do zielonego owner checku.
- REPORT osobne ryzyka per plugin family.

Nie zmieniac:
- Nie commitowac wszystkich pluginow jako jeden patch.
- Nie robic formatting-only cleanup.
- Nie wykonywac efektow innego pluginu bez contracts.

Walidacja:
- `cargo check -p <plugin-crate>` per plugin family.
- targeted `cargo test -p <plugin-crate> <filter>` jesli behavior sie zmienil.

### B6 Engine/App Residual Review

Operacje:
- Split engine/app status na: app host, scene/hydration, render-api, render-wgpu, devtools/editor, scripting/audio/ui.
- Dla kazdej grupy uruchomic `amigo-codemap open-set` albo waskie `rg`.
- ADD/MODIFY architecture tests tylko tam, gdzie istnieje boundary risk.
- REPORT pliki, ktore sa poza A0-A6 i wymagaja osobnej decyzji.

Nie zmieniac:
- Nie uruchamiac `cargo check --workspace`.
- Nie rozbijac duzych plikow bez owner-crate green.
- Nie ruszac `apps/app` domain deps bez `cargo test -p amigo-app --test architecture`.

Walidacja:
- `cargo check -p <owner-crate>`
- `cargo test -p amigo-app --test architecture`, jesli app jest dotkniete.
- `cargo test -p amigo-render-wgpu architecture`, jesli render-wgpu jest dotkniete.

### B7 Mods Authored Data Review

Operacje:
- READ status `mods/**` wedlug mod family.
- VERIFY czy zmiany sa authored content only: scenes, presets, routes, assets.
- REPORT kazde podejrzenie engine behavior ukrytego w authored data.
- MODIFY tylko dane modow, bez engine code.

Nie zmieniac:
- Nie przenosic engine semantics do mod YAML.
- Nie mieszac mod data z plugin/runtime refactor.

Walidacja:
- `git diff --check -- mods`
- targeted scene/preset loader tests, jesli istnieja dla dotknietego moda.

### B8 Final Integration Validator

Operacje:
- RUN final targeted invariant checks z `Final acceptance checklist`.
- RUN owner checks dla wszystkich staged groups.
- RUN first downstream checks tylko po zielonych owner crates.
- MODIFY `plan.md` z koncowym statusem i ryzykami.

Nie zmieniac:
- Nie naprawiac nowych bledow szeroko; otworzyc nowy plan dla konkretnego owner crate.
- Nie raportowac partial jako complete.

Walidacja:
- `git diff --check`
- `cargo test -p amigo-app --test architecture`
- `cargo test -p amigo-render-wgpu architecture`
- `cargo test -p amigo-runtime-bundles runtime_bundles_do_not_publicly_reexport_plugin_crates`
- `cargo test -p amigo-render-api render_descriptor --lib`

## 1. Runtime bundles jako zbyt szeroka fasada domen

Blad:
- `crates/runtime/bundles` publicznie reexportuje plugin crate'y i staje sie fasada domen zamiast warstwa kompozycji runtime.

Dowod:
- `crates/runtime/bundles/src/plugin_crates.rs`
- `crates/runtime/bundles/src/lib.rs`

Operacje:
- READ `crates/runtime/bundles/src/lib.rs` - potwierdzic `pub mod plugin_crates` i `pub use plugin_crates::*`.
- READ `crates/runtime/bundles/src/plugin_crates.rs` - spisac publiczne reexporty.
- MODIFY `crates/runtime/bundles/src/lib.rs` - usunac publiczny export `plugin_crates`.
- MODIFY moduly w `crates/runtime/bundles/src` - zamienic `crate::amigo_*` na bezposrednie importy crate'ow tylko w miejscach wewnetrznie potrzebnych.
- DELETE `crates/runtime/bundles/src/plugin_crates.rs` po usunieciu ostatniego uzycia.

Nie zmieniac:
- Nie przenosic semantyki domen do `runtime/bundles`.
- Nie dodawac nowego shim module z inna nazwa.
- Nie zmieniac runtime plugin registration order, chyba ze test wymusi jawna korekte.

Walidacja:
- `rg -n "plugin_crates|crate::amigo_|pub use .*plugin" crates/runtime/bundles`
- `cargo check -p amigo-runtime-bundles`

## 2. `apps/app` nadal zna domeny bezposrednio

Blad:
- App host zalezy od domenowych pluginow i wymaga domenowych services w preview path.

Dowod:
- `crates/apps/app/Cargo.toml`
- `crates/apps/app/src/scene_preview.rs`

Operacje:
- READ `crates/apps/app/Cargo.toml` - potwierdzic bezposrednie zaleznosci `amigo-camera-*-plugin`.
- READ `crates/apps/app/src/scene_preview.rs` - spisac `required::<...SceneService>()` i domenowe typy importowane z `amigo_runtime_bundles`.
- ADD/MODIFY neutralny helper w `crates/runtime/bundles` albo `crates/engine/editor-session` dla scene preview service requirements.
- MODIFY `crates/apps/app/src/scene_preview.rs` - app ma wolac jeden neutralny helper i nie znac domenowych service typow.
- MODIFY `crates/apps/app/Cargo.toml` - usunac bezposrednie zaleznosci app od camera/plugin crates, jezeli po przeniesieniu nie sa potrzebne.
- MODIFY `crates/apps/app/tests/architecture.rs` - test ma blokowac `amigo-*-plugin` w app poza jawnie dozwolonym `amigo-plugin-api`.

Nie zmieniac:
- Nie przenosic bootstrapu app do runtime bundles.
- Nie dodawac app-side fallbackow dla brakujacych services.
- Nie zmieniac publicznego API preview bez dopasowania testow app.

Walidacja:
- `rg -n "amigo-.*-plugin|PostFx2dService|MaterialSceneService|MeshSceneService|Light.*SceneService|Ui.*Service" crates/apps/app`
- `cargo check -p amigo-app`

## 3. Renderer rekonstruuje intencje z prymitywow i nazw

Blad:
- WGPU visual-source path dopasowuje `RenderPrimitive2d` i `owner_entity()` zamiast uzywac explicit coverage/target.

Dowod:
- `crates/engine/render-wgpu/src/renderer/service/render/visual_source_buffer_pass/procedural_material.rs`

Operacje:
- READ `procedural_material.rs` - miejsca z `RenderPrimitive2d::`, `owner_entity() ==`, `CameraOpticalRenderTargetPlan::for_visual_kind_name`.
- MODIFY kontrakty w `amigo-render-api` / `amigo-camera` - coverage/candidate ma niesc jawny target/buffer identity potrzebny backendowi.
- MODIFY `procedural_material.rs` - backend ma konsumowac explicit candidate targets.
- ADD diagnostyke dla brakujacego targetu: skip + jawny reason, bez proxy fallbacku.
- ADD tests w `render-wgpu`: explicit target renderuje, brak targetu nie zgaduje po nazwie entity, unsupported coverage jest raportowane.

Nie zmieniac:
- Nie dodawac kolejnych map po nazwach entity.
- Nie branchowac na nowe warianty `RenderPrimitive2d` w tym pass.
- Nie wprowadzac renderer-side domain policy.

Walidacja:
- `rg -n "RenderPrimitive2d::|owner_entity\\(\\).*==" crates/engine/render-wgpu/src/renderer/service/render`
- `cargo check -p amigo-render-api`
- `cargo check -p amigo-render-wgpu`

## 4. Usuniete docs pluginow przy aktywnych zmianach

Blad:
- Wiele dotknietych pluginow ma usuniete `docs/pipeline.md`, `docs/contributions.md`, `docs/diagnostics.md`.

Dowod:
- `git status --short | rg "^ D plugins/.*/docs/"`

Operacje:
- READ liste usunietych docs pluginow z `git status`.
- ADD/RESTORE minimalne, znaczeniowe docs dla kazdego dotknietego pluginu: `pipeline.md`, `contributions.md`, `diagnostics.md`.
- MODIFY docs tylko dla pluginow zmienianych w aktualnym worktree.
- DELETE stare placeholdery tylko jezeli sa zastapione sensowna trescia.

Nie zmieniac:
- Nie robic docs dla nietknietych pluginow tylko dla symetrii.
- Nie dodawac jednozdaniowych placeholderow.
- Nie przenosic canonical docs do concat snapshotow.

Walidacja:
- `git status --short | rg "^ D plugins/.*/docs/"`
- `git diff --check`

## 5. PostFX ma centralne listy efektow i executorow

Blad:
- Dodanie efektu wymaga edycji centralnego `for_kind` i centralnego executor registry.

Dowod:
- `crates/engine/render-api/src/post_fx_model/render_descriptor.rs`
- `crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs`

Operacje:
- READ `render_descriptor.rs` - `PostFxRenderDescriptor::for_kind`.
- READ `registry.rs` - `default_wgpu_screen_effect_executors`.
- MODIFY `amigo-render-api` - descriptor definitions maja byc rejestrowane przez jeden jawny descriptor registry/table, bez matcha po stringu w live path.
- MODIFY `amigo-render-wgpu` - executor registry ma walidowac executor id z descriptor registry.
- ADD error dla brakujacego executora.
- MODIFY tests - invariant: kazdy `PostFx2d` ma descriptor, kazdy descriptor wymagajacy executor ma executor.

Nie zmieniac:
- Nie dodawac `PostFx2dV2`.
- Nie dodawac centralnego renderer switcha po effect kind.
- Nie ukrywac brakujacego executora copy-through fallbackiem poza jawnie zadeklarowanymi passthrough efektami.

Walidacja:
- `rg -n "match kind|match descriptor.executor_id|CopyThroughExecutor" crates/engine/render-api crates/engine/render-wgpu/src/renderer/service/post_fx`
- `cargo check -p amigo-render-api`
- `cargo check -p amigo-render-wgpu`

## 6. Ifologia i zbyt duze pliki decyzyjne

Blad:
- Kilka plikow laczy za duzo odpowiedzialnosci i lokalnych decyzji.

Dowod:
- `crates/apps/app/src/scene_runtime/mod.rs`
- `plugins/camera/camera-core/src/runtime/service.rs`
- `plugins/camera/camera-profiles/src/runtime/profiles.rs`
- `crates/engine/render-wgpu/src/renderer/service/render/plate_relight.rs`

Operacje:
- READ metryki linii/if/match dla wskazanych plikow.
- MODIFY `crates/apps/app/src/scene_runtime/mod.rs` - wydzielic host orchestration od domain command dispatch.
- MODIFY `plugins/camera/camera-core/src/runtime/service.rs` - rozdzielic state, focus transitions, debug view, parallax/follow services.
- MODIFY `plugins/camera/camera-profiles/src/runtime/profiles.rs` - oddzielic profile data od resolve/apply logic.
- MODIFY `plate_relight.rs` tylko przez wydzielenie lokalnych helper modules bez zmiany behavior.

Nie zmieniac:
- Nie robic formatting-only splitow.
- Nie zmieniac API publicznego bez testu migracyjnego.
- Nie laczyc wszystkich splitow w jednym patchu.

Walidacja:
- `cargo check -p amigo-app`
- `cargo check -p amigo-camera-core-plugin`
- `cargo check -p amigo-camera-profiles-plugin`
- `cargo check -p amigo-render-wgpu`

## 7. Testy architektoniczne maja dziury

Blad:
- Testy sa glownie string-searchami i nie lapia czesci realnych leakow, np. camera plugin deps w app.

Dowod:
- `crates/apps/app/tests/architecture.rs`
- `crates/apps/app/Cargo.toml`

Operacje:
- READ `crates/apps/app/tests/architecture.rs`.
- MODIFY app architecture tests - blokowac wszystkie direct `*-plugin` deps poza jawnie dozwolonymi engine/plugin-api przypadkami.
- MODIFY render-wgpu architecture tests - dopisac invariant braku `RenderPrimitive2d::` w visual-source pass, nie tylko `world.rs`.
- MODIFY runtime-bundles architecture tests - dopisac invariant braku publicznych plugin crate reexportow.

Nie zmieniac:
- Nie zastapowac testow runtime compile-checkami workspace-wide.
- Nie robic testow, ktore przechodza przez komentarz zamiast importow/Cargo deps, jezeli mozna sprawdzic konkretniejszy plik.
- Nie robic testow zaleznosci od aktualnej kolejnosci prywatnych helperow.

Walidacja:
- `cargo test -p amigo-app architecture`
- `cargo test -p amigo-render-wgpu architecture`
- `cargo test -p amigo-runtime-bundles architecture`

## 8. Worktree i planowanie sa nieczytelne operacyjnie

Blad:
- Repo ma setki zmian, usuniete docs i snapshoty concat obok planu; nie wiadomo, co jest finalnym zrodlem prawdy.

Dowod:
- `git status --short`
- `?? concat-output-rec6v.txt`
- `?? plan.md`

Operacje:
- MODIFY `plan.md` - zastapic poprzedni baseline tym konkretnym planem naprawczym.
- DELETE albo przeniesc poza repo `concat-output-rec6v.txt` i `concat-output-rec6v.zip` po przeniesieniu wnioskow do planu.
- DELETE stare snapshoty tylko jesli nie sa juz potrzebne jako review artifacts.
- RESTORE albo finalnie usunac docs po decyzji z punktu 4, nie zostawiac masowych `D` bez wyjasnienia.

Nie zmieniac:
- Nie traktowac concat snapshotu jako source file.
- Nie robic `git reset --hard`.
- Nie kasowac cudzych zmian bez jawnej decyzji.

Walidacja:
- `git status --short`
- `git diff --check`

## 9. Utrzymywalnosc: centralne pliki sa za duze

Blad:
- Kilka plikow jest naturalnymi bottleneckami zmian.

Dowod:
- `crates/engine/scene/src/component_metadata.rs`
- `plugins/camera/camera-core/src/runtime/service.rs`
- `plugins/camera/camera-profiles/src/runtime/profiles.rs`

Operacje:
- READ tylko symbole/ranges przez `amigo-codemap` albo waskie `rg`.
- ADD `crates/engine/scene/src/component_metadata/model.rs` - przeniesc model, constraints i `ComponentRegistry` bez zmiany publicznych nazw.
- MODIFY `crates/engine/scene/src/component_metadata.rs` - zostawic fabryki descriptorow i re-export modelu.
- ADD `plugins/camera/camera-core/src/runtime/service/focus_transition.rs` - przeniesc helpery focus transition.
- MODIFY `plugins/camera/camera-core/src/runtime/service.rs` - korzystac z helperow bez zmiany publicznego `CameraService`.
- ADD `plugins/camera/camera-profiles/src/runtime/profiles/catalog.rs` - przeniesc builtin lens/film/preset catalog.
- MODIFY `plugins/camera/camera-profiles/src/runtime/profiles.rs` - zachowac publiczne `BUILTIN_*_2D` jako aliasy na katalog.
- RUN focused tests dla zachowan dotknietych splitem.

Nie zmieniac:
- Nie zmieniac nazw publicznych typow tylko dla estetyki.
- Nie mieszac splitu plikow z refaktorem behavior.
- Nie czytac ani przerabiac calego pliku, jesli wystarczy range/symbol.

Walidacja:
- `cargo check -p amigo-scene`
- `cargo check -p amigo-camera-core-plugin`
- `cargo check -p amigo-camera-profiles-plugin`

## 10. Zbyt duza trudnosc projektu bez sekwencjonowania

Blad:
- Rownolegle zmiany w wielu warstwach zwiekszaja ryzyko regresji i konfliktow.

Dowod:
- `amigo-codemap brief`: ok. 91 packages, 2239 files, szeroki dirty worktree.
- `git status --short`: zmiany w app, runtime, engine, plugins, mods, docs.

Operacje:
- MODIFY `plan.md` - utrzymac kolejnosc wykonania z sekcji `Kolejnosc prac`.
- Kazdy etap zaczyna i konczy `git status --short`.
- Kazdy etap ma owner-crate validation przed downstream.
- Po kazdym etapie uruchomic narrow rg invariant z punktow powyzej.

Nie zmieniac:
- Nie uruchamiac `cargo check --workspace` jako pierwszej walidacji.
- Nie laczyc PostFX registry migration z app boundary cleanup w jednym patchu.
- Nie mieszac docs cleanup z behavior refactor, poza niezbednymi docs dla dotknietych pluginow.

Walidacja:
- Per etap: targeted `rg`, potem `cargo check -p <owner-crate>`.
- Po zielonych owner crates: pierwszy downstream crate.
- Finalnie: `cargo test -p amigo-app architecture`, `cargo test -p amigo-render-wgpu architecture`, `cargo test -p amigo-runtime-bundles architecture`.

## Final acceptance checklist

## Czwarta fala: finalne agenty i domkniecie krytykow

Cel:
- Domknac ostatnie 8-10% planu krytycznego po `SceneAssetDependency`, PostFX diagnostics, particles decoupling i `RenderSourceId`.

Agenty:
- C1 Render Heuristics Auditor - read-only audit `owner_entity` / `component_kind` w runtime/render path.
  - Write set: brak.
  - Walidacja: narrow `rg` pod `crates/engine/render-wgpu`, `crates/runtime/bundles`, `plugins`.
  - Wynik: podzial na production execution, diagnostics/stats/debug, tests/support.
- C2 Staging/Review Auditor - read-only lista plikow aktualnej fali.
  - Write set: brak.
  - Walidacja: `git status --short`, `git diff --name-status`.
  - Wynik: pliki staging tej fali vs pliki starszych zmian/mods/Rotten Club.
- C3 Particles Validation Auditor - read-only wskazanie najtanszych testow dla velocity provider bridge.
  - Write set: brak.
  - Walidacja: wyszukac testy `source_velocity`, `inherit_parent_velocity`, runtime bridge.
  - Wynik: komendy i test gaps.
- C0 Local Integrator - implementuje tylko brakujace, male poprawki po wynikach C1-C3.
  - Write set: `plan.md`, minimalne pliki invariantow/testow.
  - Nie zmieniac: `mods/**`, cudzych zmian, stagingu.

Status:
- C1: done - wykryl motion-buffer cache keys po `owner_entity`; C0 przeniosl je na `RenderSourceId` i dodal guard.
- C2: done - przygotowal staging/review split dla aktualnej fali i listy nie-dotykac.
- C3: done - wskazal particles validation gap; C0 dodal provider registry test i uruchomil velocity inheritance tests.
- C0: done - zintegrowal wyniki agentow, uzupelnil guardy i walidacje.

Acceptance tej fali:
- done - `particles-2d` nie ma zaleznosci Cargo ani importow do `shutter-motion`.
- done - renderer lightmap i motion-buffer binduja source przez `RenderSourceId`.
- done - `cargo check -p amigo-app` przechodzi po zmianach.
- done - dodano integracyjny test `TwoDRuntimeBundle` dla bridge motion velocity -> particle source velocity provider.
- done - finalny raport oddziela pozostalosci diagnostyczne od blokujacych runtime heuristics.

- `plan.md` jest jedynym aktualnym planem naprawczym.
- `rg -n "plugin_crates|crate::amigo_|pub use .*plugin" crates/runtime/bundles` nie pokazuje live public facade leakow.
- `rg -n "amigo-.*-plugin|PostFx2dService|MaterialSceneService|MeshSceneService|Light.*SceneService|Ui.*Service" crates/apps/app` nie pokazuje app-domain leakow.
- `rg -n "RenderPrimitive2d::|owner_entity\\(\\).*==" crates/engine/render-wgpu/src/renderer/service/render` nie pokazuje renderer-side visual-source guessing.
- `git status --short | rg "^ D plugins/.*/docs/"` jest puste albo kazde usuniecie docs ma jawna decyzje.
- Architecture tests przechodza dla `amigo-app`, `amigo-render-wgpu`, `amigo-runtime-bundles`.
