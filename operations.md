# Operations

Lekki dziennik pracy. Najnowsze wpisy na gorze.

Format:
- Task: co robimy.
- Ops: uzyte narzedzia/komendy.
- Files: najwazniejsze pliki.
- Verify: build/test/check albo `docs only`.
- Tokens: szacunek `used` i `saved`.

## 2026-05-06

### Editor Mode Camera And Draft Alignment
- Task: domknac investigate bounding box vs preview: snapshot bierze Camera2D z dokumentu, overlay/input mapuja przez scene camera, Text2D bounds sa liczone jak obecny renderer tekstu, a drag dostaje tymczasowy draft proxy w overlay zamiast udawac live render obiektu.
- Ops: `rg`, `Get-Content`, `apply_patch`, `cargo fmt -p amigo-editor`, `npm run build`, `npm test`, `cargo check -p amigo-editor`, `cargo test -p amigo-editor document_snapshot`, `cargo test -p amigo-editor coordinates`, `cargo test -p amigo-editor overlay`, `cargo test -p amigo-editor gizmos`, `cargo run -p amigo-codemap -- scan`, `git diff --check`.
- Files: `src/features/scenes/editor/{SceneEditorCanvas,sceneEditorModel,sceneEditorTypes}.tsx`, `src-tauri/src/editor_mode/{document_snapshot,renderer,overlay}.rs`.
- Verify: `npm run build`, `npm test` 7/7 files 22/22 tests, `cargo check -p amigo-editor` z istniejacymi warningami szkieletow editor-mode, `document_snapshot` 7/7, `coordinates` 3/3, `overlay` 2/2, `gizmos` 5/5, `amigo-codemap scan`, `git diff --check` tylko CRLF warnings.
- Tokens: used ~3500, saved ~35-45% przez zawężenie do camera/text-bounds/draft proxy zamiast ruszania rotate/scale/rect albo pelnego session render pipeline.

## 2026-05-05

### Editor Mode Coordinate Alignment Pass
- Task: rozpoczac stabilizacyjny coordinate alignment: jawne scene/frame pointer DTO, backendowy `EditorCoordinateMapper`, overlay przez mapper zamiast recznego wzoru, debug origin/pointer, `canUndo/canRedo`, klikalne docki nad artboardem, neutralna backend camera dla SVG wrappera i raw frame pointer debug.
- Ops: `rg`, `Get-Content`, `apply_patch`, `cargo fmt -p amigo-editor`, `npm run build`, `npm test`, `cargo check -p amigo-editor`, `cargo test -p amigo-editor coordinates`, `cargo test -p amigo-editor overlay`, `cargo test -p amigo-editor gizmos`, `cargo run -p amigo-codemap -- scan`, `git diff --check`.
- Files: `src/api/dto.ts`, `src/features/scenes/editor/{SceneEditorCanvas.tsx,SceneEditorToolbar.tsx,scene-editor.css}`, `src-tauri/src/editor_mode/{coordinates,dto,input,overlay,renderer,session,mod}.rs`, `src-tauri/src/commands/editor_mode.rs`.
- Verify: `npm run build`, `npm test` 7/7 files 22/22 tests, `cargo check -p amigo-editor` z istniejacymi warningami szkieletow editor-mode, `cargo test -p amigo-editor coordinates` 3/3, `overlay` 2/2, `gizmos` 5/5, `amigo-codemap scan`, `git diff --check` tylko CRLF warnings.
- Tokens: used ~8200, saved ~45-55% przez zawężenie do alignment/debug DTO i UI event routing bez ruszania rotate/scale/rect i pelnego render pipeline.

### Editor Mode Tool Gizmo State
- Task: zaadaptowac patchset tool/gizmo do aktualnego `EditorModeSession` bez przywracania starego live/DOM overlay; dodac DTO selection/toolState/gizmos, backendowe generowanie gizmos ze snapshotu, selection przez pointerDown i tool `rect`.
- Ops: `rg`, `Get-Content`, `apply_patch`, `cargo fmt -p amigo-editor`, `npm run build`, `npm test`, `cargo check -p amigo-editor`, `cargo test -p amigo-editor gizmos`, `cargo run -p amigo-codemap -- scan`.
- Files: `src/api/dto.ts`, `src/features/scenes/editor/{SceneEditorCanvas.tsx,SceneEditorHud.tsx,sceneEditorTypes.ts}`, `src/main-window/MainEditorWindow.tsx`, `src-tauri/src/editor_mode/{dto,gizmos,input,document_snapshot,snapshot,document_patch,mod}.rs`, `src-tauri/src/commands/editor_mode.rs`.
- Verify: `npm run build`, `npm test` 7/7 files 22/22 tests, `cargo check -p amigo-editor`, `cargo test -p amigo-editor gizmos` 3/3, `amigo-codemap scan`.
- Tokens: used ~9000, saved ~45-60% przez dopasowanie patchsetu do obecnej architektury `EditorFrameDto` zamiast reaktywowania usunietego live session/DOM gizmo flow.

### Editor Overlay Pass
- Task: zaaplikowac `editor-overlay-pass.patch`, dodac tymczasowy SVG overlay pass nad image-url preview frame i podpiac go w `render_editor_mode_frame`.
- Ops: `git apply`, `cargo fmt -p amigo-editor`, `npm run build`, `npm test`, `cargo check -p amigo-editor`, `cargo test -p amigo-editor gizmos`, `cargo test -p amigo-editor overlay`, `cargo run -p amigo-codemap -- scan`.
- Files: `src-tauri/src/editor_mode/{overlay,renderer,mod}.rs`.
- Verify: `npm run build`, `npm test` 7/7 files 22/22 tests, `cargo check -p amigo-editor`, `cargo test -p amigo-editor gizmos` 5/5 filtered, `cargo test -p amigo-editor overlay` 2/2, `amigo-codemap scan`.
- Tokens: used ~2600, saved ~35-45% przez uzycie SVG data-url jako pierwszego checkpointu overlay pass zamiast implementowania raster encode/cache.

### Editor Mode Move Transactions
- Task: dopiac backend selection clear, gizmo handle hit-test, `activeInteraction`, move drag 2D, transaction log, undo/redo toolbar/API oraz save/discard przez YAML patch dla `Transform2`.
- Ops: `Get-Content`, `rg`, `apply_patch`, `cargo fmt -p amigo-editor`, `npm run build`, `npm test`, `cargo check -p amigo-editor`, `cargo test -p amigo-editor gizmos`, `cargo test -p amigo-editor overlay`, `cargo run -p amigo-codemap -- scan`.
- Files: `src/app/editorStore.tsx`, `src/api/editorApi.ts`, `src/main-window/{MainEditorWindow,workspaceRuntimeServices}.tsx`, `src/features/scenes/editor/{SceneEditorToolbar,SceneEditorWorkbench}.tsx`, `src-tauri/src/editor_mode/{input,session,transaction,document_patch,gizmos}.rs`, `src-tauri/src/commands/{editor_mode,mod}.rs`, `src-tauri/src/lib.rs`.
- Verify: `npm run build`, `npm test` 7/7 files 22/22 tests, `cargo check -p amigo-editor`, `cargo test -p amigo-editor gizmos` 5/5 filtered, `cargo test -p amigo-editor overlay` 2/2, `git diff --check`, `amigo-codemap scan`.
- Tokens: used ~8500, saved ~45-60% przez compile-driven adaptacje patchsetu do aktualnego `EditorModeSession` i istniejacego YAML transform command.

### Scene Editor Snapshot Bridge
- Task: zaadaptowac paczke Editor Mode do aktualnego `features/scenes/editor`: dodac frontend DTO/API, Tauri fallback snapshot/commands, podpiac snapshot do scene editora i dodac selected entity/transform widgets w prawym `SceneContextDock`.
- Ops: `amigo-codemap open-set`, `tauri-commands`, `patch-check`, `patch-apply --write`, `apply_patch` po wykryciu bug-przypadku `0 hunks`, `npm run build`, `npm test`, `cargo check -p amigo-editor`, `cargo fmt -p amigo-editor`.
- Files: `src/api/dto.ts`, `src/api/editorApi.ts`, `src/main-window/MainEditorWindow.tsx`, `src/main-window/workspaceRuntimeServices.ts`, `src/features/scenes/editor/*`, `src/features/scenes/context/*`, `src-tauri/src/editor_mode/*`, `src-tauri/src/commands/editor_mode.rs`.
- Verify: `npm run build`, `npm test` 3/3 files 13/13 tests, `cargo check -p amigo-editor`.
- Tokens: used ~9000, saved ~45-60% przez adaptacje batchy do istniejacego edytora zamiast tworzenia rownoleglego `features/scene-editor`; wykryto tez konieczna poprawke w codemap `patch-apply` dla niepustych patchy z `0 hunks`.

### Scene Editor Canvas Engines
- Task: rozdzielic aktualna implementacje 2D canvas od placeholderow 2.5D/3D i dodac `canvasKind` w snapshot DTO/backend fallback.
- Ops: `Get-Content`, `apply_patch`, `npm run build`, `npm test`, `cargo check -p amigo-editor`, `cargo fmt -p amigo-editor`.
- Files: `src/api/dto.ts`, `src/features/scenes/editor/sceneEditorTypes.ts`, `SceneEditorWorkbench.tsx`, `SceneEditorToolbar.tsx`, `src/features/scenes/editor/canvas/*`, `src/features/scenes/editor/scene-editor.css`, `src-tauri/src/editor_mode/{dto,snapshot}.rs`, `src-tauri/src/commands/editor_mode.rs`.
- Verify: `npm run build`, `npm test` 3/3 files 13/13 tests, `cargo check -p amigo-editor`.
- Tokens: used ~4200, saved ~40-55% przez adaptacje do istniejacego `features/scenes/editor` zamiast tworzenia rownoleglego `features/scene-editor`.

### Scene Editor Fallback Layout Guard
- Task: usunac falszywe fallback bounds/kwadraty z edytora sceny, dodac `layoutSource`, wylaczyc viewport picking/drag bez realnego snapshotu i przestac udawac sukces transform command.
- Ops: `Get-Content`, `apply_patch`, `npm run build`, `npm test`, `cargo check -p amigo-editor`, `cargo fmt -p amigo-editor`.
- Files: `src/api/dto.ts`, `src/features/scenes/editor/sceneEditorTypes.ts`, `sceneEditorModel.ts`, `SceneEditorCanvas.tsx`, `SceneEditorHud.tsx`, `scene-editor.css`, `src-tauri/src/editor_mode/{dto,snapshot}.rs`, `src-tauri/src/commands/editor_mode.rs`.
- Verify: `npm run build`, `npm test` 3/3 files 13/13 tests, `cargo check -p amigo-editor`.
- Tokens: used ~4800, saved ~45-60% przez jasne rozdzielenie realnego snapshotu od fallbacku zamiast debugowania falszywych klikow/dragow na kanwie.

### Startup Dialog Mod Selection Race
- Task: naprawic startup dialog, w ktorym klikniecie moda bywalo nadpisywane przez stary load albo kolejny scan i wracalo do poprzedniego zaznaczenia.
- Ops: `target/debug/amigo-codemap.exe find`, `target/debug/amigo-codemap.exe scope`, `Get-Content`, `apply_patch`, `npm test`, `npm run build`.
- Files: `crates/apps/amigo-editor/src/app/editorStore.tsx`, `crates/apps/amigo-editor/src/startup/StartupDialog.tsx`, `operations.md`.
- Verify: `npm test` 12/12, `npm run build`.
- Tokens: used ~4200, saved future ~35-45% przez szybkie zawężenie do startup/store zamiast ręcznego śledzenia całego flow mod selection.

### Codemap Usage Docs
- Task: dopisac praktyczna dokumentacje jak uzywac `command-map`, `append-plan` i `copy-plan`, a nie tylko liste komend.
- Ops: `target/debug/amigo-codemap.exe command-map copy-plan`, `target/debug/amigo-codemap.exe command-map append-plan`, `target/debug/amigo-codemap.exe scope copy_plan --root crates/tools/amigo-codemap`, `apply_patch`.
- Files: `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `operations.md`.
- Verify: docs only.
- Tokens: used ~2200, saved future ~30-40% przy pytaniach "jak tego uzywac" bez recznego tlumaczenia workflow za kazdym razem.

### Codemap Copy Plan
- Task: dodac `copy-plan` do planowania kopiowania wzorcow z donor file do targetu z rename hotspots i mirrored companion files.
- Ops: `target/debug/amigo-codemap.exe command-map append-plan`, `target/debug/amigo-codemap.exe command-map open-set`, `target/debug/amigo-codemap.exe command-map slice`, `target/debug/amigo-codemap.exe scope ... --root crates/tools/amigo-codemap`, `apply_patch`, `cargo fmt`, `cargo test copy_plan`, `cargo test command_map`, `cargo build`, smoke `target/debug/amigo-codemap.exe command-map copy-plan`, smoke `target/debug/amigo-codemap.exe copy-plan ...`.
- Files: `crates/tools/amigo-codemap/src/cli.rs`, `crates/tools/amigo-codemap/src/main.rs`, `crates/tools/amigo-codemap/src/report/command_map.rs`, `crates/tools/amigo-codemap/src/report/file_ops/copy_plan.rs`, `crates/tools/amigo-codemap/src/report/file_ops/mod.rs`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `operations.md`.
- Verify: `cargo test copy_plan`, `cargo test command_map`, `cargo build`, smoke `command-map copy-plan`, smoke `copy-plan`.
- Tokens: used ~6500, saved future ~45-60% przy copy-driven taskach i rozwijaniu samego codemap bez recznego szukania donor files i rename hotspots.

### Codemap Refactor Reports
- Task: dodac operacyjne raporty `amigo-codemap`, fixture/snapshot testy oraz opis workflow pracy z nowymi komendami.
- Ops: `amigo-codemap verify-plan`, `stale`, `impact`, `fallout`, `move-plan`, `dup`, `tauri-commands`, `service-shape`, `registry-check`, `operations-summary`, `commit-summary`, `apply_patch`, `cargo test`, `cargo build`.
- Files: `crates/tools/amigo-codemap/src/report/*`, `crates/tools/amigo-codemap/src/{cli,main}.rs`, `crates/tools/amigo-codemap/tests/*`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `crates/tools/amigo-codemap/PR_SPLIT.md`.
- Verify: `cargo test -p amigo-codemap` 54/54, `cargo build -p amigo-codemap`.
- Tokens: used ~22000, saved ~60-70% przez przeniesienie powtarzalnych rg/build-log/registry/Tauri checks do raportow codemap.

### Final Cleanup Pass
- Task: dosprzatac backend helpery po splitcie, przepiac project node actions na registry i uproscic drobne visual maps.
- Ops: `amigo-codemap scope`, `rg`, `apply_patch`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`, `amigo-codemap compact`.
- Files: `src-tauri/src/commands/shared.rs`, `src-tauri/src/commands/{mods,cache,project_files,project_tree,mod}.rs`, `features/project legacy node-actions file`, `main-window/MainEditorWindow.tsx`, `features/tasks/TaskTable.tsx`, `features/events/eventFormatters.ts`.
- Verify: `npm test` 2/2, `npm run build`, `cargo test -p amigo-editor --lib` 8/8.
- Tokens: used ~9000, saved ~50-60% przez codemapowe znalezienie duplikatow i malych hotspotow zamiast recznego sweepu.

### Final Selection And Commands Split
- Task: domknac `resolvedSelection` w inspector/properties i rozciac backend `commands/mod.rs` na domenowe moduly z cienkimi wrapperami Tauri.
- Ops: `amigo-codemap scope`, `rg`, `apply_patch`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`, `amigo-codemap compact`.
- Files: `features/inspector/*`, `main-window/MainEditorWindow.tsx`, `main-window/workspaceRuntimeServices.ts`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/commands/{mods,session,project_tree,project_files,assets,sheets,preview,cache,settings}.rs`.
- Verify: `npm test` 2/2, `npm run build`, `cargo test -p amigo-editor --lib` 8/8.
- Tokens: used ~18000, saved ~60-70% przez codemapowe zawazenie hotspotow i compile-driven split zamiast recznego sweepu po calym backendzie/frontstore.

### Selection Ref Migration
- Task: przelaczyc frontend store i glowne widoki na `selection: EditorSelectionRef` jako zrodlo prawdy.
- Ops: `amigo-codemap refs/scope`, `rg`, `apply_patch`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`, `amigo-codemap compact`.
- Files: `app/store/*`, `app/selectionSelectors.ts`, `app/editorStore.tsx`, `main-window/MainEditorWindow.tsx`, `startup/*`, `settings/SettingsDialog.tsx`.
- Verify: `npm test` 2/2, `npm run build`, `cargo test -p amigo-editor --lib` 8/8.
- Tokens: used ~14000, saved ~55-65% przez codemap refs i compile-driven migration zamiast recznego sledzenia selection po calym UI.

### Editor Store Split Stage 1
- Task: wyniesc `EditorState`, `initialState`, `Action`, `reducer` i podstawowe selektory z `editorStore.tsx`.
- Ops: `amigo-codemap scope`, `rg`, `apply_patch`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`, `amigo-codemap compact`.
- Files: `app/store/editorState.ts`, `app/store/editorActions.ts`, `app/store/editorReducer.ts`, `app/store/editorSelectors.ts`, `app/editorStore.tsx`, `main-window/MainEditorWindow.tsx`.
- Verify: `npm test` 2/2, `npm run build`, `cargo test -p amigo-editor --lib` 8/8.
- Tokens: used ~9000, saved ~50-60% przez codemapowe zawężenie store i compile-driven cleanup.

### WorkspacePanels Removal
- Task: przeniesc legacy `assets` i `project explorer` z `workspacePanels` do `features/*` i usunac plik.
- Ops: `amigo-codemap scope`, `rg`, extract split, `apply_patch`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`, `amigo-codemap compact`.
- Files: `features/assets/*`, `features/project/*`, `features/files/*`, `main-window/workspaceTabs.tsx`, deleted `main-window/workspacePanels.tsx`.
- Verify: `npm test` 2/2, `npm run build`, `cargo test -p amigo-editor --lib` 8/8.
- Tokens: used ~15000, saved ~65-75% przez codemap scope i celowane usuniecie ostatnich importow legacy.

### Scenes Inspector Files Split
- Task: odciac scenes browser/hierarchy, inspector/properties oraz file browser/workspaces od `workspacePanels`.
- Ops: `amigo-codemap scope`, `rg`, `apply_patch`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`, `amigo-codemap compact`.
- Files: `features/scenes/*`, `features/inspector/*`, `features/files/*`, `MainEditorWindow.tsx`, `workspacePanels.tsx`.
- Verify: `npm test` 2/2, `npm run build`, `cargo test -p amigo-editor --lib` 8/8.
- Tokens: used ~12000, saved ~60-70% przez codemapowe wykrycie pozostalych importow i hotspotow.

### Events And Preview Physical Split
- Task: fizycznie przeniesc `events.log` i `scene.preview` z `workspacePanels` do feature files.
- Ops: `amigo-codemap scope`, `apply_patch`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`, `amigo-codemap compact`.
- Files: `features/events/EventTable.tsx`, `features/events/eventFormatters.ts`, `features/scenes/ScenePreviewWorkbench.tsx`, `workspacePanels.tsx`.
- Verify: `npm test` 2/2, `npm run build`, `cargo test -p amigo-editor --lib` 8/8.
- Tokens: used ~6500, saved ~55-65% przez codemapowe wyciecie dwoch zwartych komponentow.

### Full Component Renderer Handoff
- Task: podpiac duze panele przez `features/*`, usunac legacy switch i przygotowac store/backend split scaffolding.
- Ops: `amigo-codemap scope/refs`, `apply_patch`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`, `amigo-codemap compact`.
- Files: `features/events/*`, `features/scenes/*`, `features/inspector/*`, `features/files/*`, `features/assets/*`, `features/project/*`, `builtinComponents.tsx`, `WorkspaceComponentHost.tsx`, `workspacePanels.tsx`, `app/selection*.ts`, `app/store/*`, `src-tauri/src/commands/*`.
- Verify: `npm test` 2/2, `npm run build`, `cargo test -p amigo-editor --lib` 8/8.
- Tokens: used ~17000, saved ~65-75% dzieki codemap i etapowemu wrapper handoff zamiast kopiowania calego `workspacePanels`.

### Feature Renderers And Commands Module
- Task: przeniesc pierwsze panele do `features/*`, podpiac realne renderery i przygotowac backend `commands/mod.rs`.
- Ops: `amigo-codemap scope/refs`, `apply_patch`, `Move-Item`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`.
- Files: `features/project/*`, `features/diagnostics/ProblemsTable.tsx`, `features/tasks/TaskTable.tsx`, `features/cache/CachePanel.tsx`, `builtinComponents.tsx`, `workspacePanels.tsx`, `src-tauri/src/commands/mod.rs`.
- Verify: `npm test` 2/2, `npm run build`, `cargo test -p amigo-editor --lib` 8/8.
- Tokens: used ~9500, saved ~60% przez codemapowe wybranie najprostszych paneli i mechaniczny backend move.

### Main Window And Store Split
- Task: wydzielic layout/toolbox/menu z `MainEditorWindow` i dodac wspolny runner taskow w store.
- Ops: `amigo-codemap brief/scope`, `apply_patch`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`.
- Files: `useWorkspaceLayout.ts`, `WorkspaceResizeHandle.tsx`, `ComponentMenu.tsx`, `toolboxRegistry.ts`, `runEditorTask.ts`, `MainEditorWindow.tsx`, `editorStore.tsx`.
- Verify: `npm test` 2/2, `npm run build`, `cargo test -p amigo-editor --lib` 8/8.
- Tokens: used ~13000, saved ~55-70% przez codemapowe zawężenie hotspotow.

### Workspace Host And File Rules
- Task: dodac `WorkspaceComponentHost`, `WorkspaceRuntimeServices`, `EditorFeature` agregator i przeniesc file workspace rules do `features/files`.
- Ops: `amigo-codemap brief/scope/refs`, `apply_patch`, `npm test`, `npm run build`, `amigo-codemap compact`.
- Files: `WorkspaceComponentHost.tsx`, `workspaceRuntimeServices.ts`, `componentTypes.ts`, `componentHost.tsx`, `componentRegistry.tsx`, `features/editorFeatures.ts`, `features/files/*`, `MainEditorWindow.tsx`.
- Verify: `npm test` 2/2, `npm run build`.
- Tokens: used ~11000, saved ~60-70% przez codemap i re-export kompatybilnosciowy zamiast pelnego czytania `workspacePanels`.

### Typed Registry Cleanup
- Task: zmniejszyc ifologie w registry i uproscic properties panele bez zmiany zachowania.
- Ops: `amigo-codemap brief`, `amigo-codemap scope`, `apply_patch`, `npm test`, `npm run build`.
- Files: `componentRegistry.tsx`, `propertiesTypes.ts`, `propertiesRegistry.tsx`, `src/properties/panels/*`, `src/ui/properties/KeyValueSection.tsx`.
- Verify: `npm test` 2/2, `npm run build`.
- Tokens: used ~5200, saved ~55-65% wzgledem recznego czytania `workspacePanels` i paneli.

### Properties Registry
- Task: przeniesc `References`/`Used By` z asset tree do kontekstowego properties panelu.
- Ops: `amigo-codemap scope`, `amigo-codemap refs`, `apply_patch`, `npm test`, `npm run build`.
- Files: `src/properties/*`, `workspacePanels.tsx`, `assetTreeBuilder.ts`, `assetTreeBuilder.test.ts`.
- Verify: `npm test` 2/2, `npm run build`.
- Tokens: used ~9000, saved future ~50-70% przy dodawaniu nowych properties paneli.

### Asset Relations Buckets
- Task: pogrupowac `References` i `Used By` w asset viewerze po typie celu.
- Ops: `amigo-codemap scope`, `amigo-codemap refs`, `apply_patch`, `npm test`, `npm run build`.
- Files: `crates/apps/amigo-editor/src/assets/assetTreeBuilder.ts`, `crates/apps/amigo-editor/src/assets/assetTreeBuilder.test.ts`.
- Verify: `npm test` 3/3, `npm run build`.
- Tokens: used ~4200, saved ~1800.

### Amigo Codemap Task Views
- Task: dodac male widoki `brief/find/scope/refs/docs/verify` i `changed --group`.
- Ops: `amigo-codemap compact`, `Get-Content`, `apply_patch`, `cargo test -p amigo-codemap`, `cargo build -p amigo-codemap`, `target/debug/amigo-codemap.exe ...`.
- Files: `crates/tools/amigo-codemap/src/*`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `operations.md`.
- Verify: `cargo test -p amigo-codemap` 4/4, smoke `brief`, `changed --group package`, `find`, `scope`, `refs`, `docs`, `verify`.
- Tokens: used ~8500, saved future ~65-80% per navigation task.

### Operations Log
- Task: dodac staly `operations.md` dla kolejnych prac.
- Ops: `amigo-codemap compact`, `Test-Path`, `git status`, `apply_patch`.
- Files: `operations.md`, `AMIGO_WORKFLOW.md`.
- Verify: docs only.
- Tokens: used ~900, saved ~300.

### Asset Tree Indentation
- Task: uproscic root sekcji asset tree, usunac mylacy top-level `Scenes` row i dodac linie prowadzace/wciecia jak w eksploratorach plikow.
- Ops: `target/debug/amigo-codemap.exe find`, `target/debug/amigo-codemap.exe scope`, `target/debug/amigo-codemap.exe verify-plan --changed`, `apply_patch`, `npm test`, `npm run build`.
- Files: `crates/apps/amigo-editor/src/assets/AssetTreePanel.tsx`, `crates/apps/amigo-editor/src/main-window/main-window.css`.
- Verify: `npm test` 3/3, `npm run build`.
- Tokens: used ~5000, saved ~50-60% przez zawężenie do `AssetTreePanel` i `TreeView` zamiast ręcznego czytania całego edytora.

### Debug Source Toggle
- Task: dodac debugowy toggle obok settings i pokazac nazwe pliku zrodlowego komponentu w stopce kazdego panelu w dev mode.
- Ops: `target/debug/amigo-codemap.exe find`, `target/debug/amigo-codemap.exe scope`, `apply_patch`, `npm test`, `npm run build`.
- Files: `crates/apps/amigo-editor/src/main-window/MainEditorWindow.tsx`, `crates/apps/amigo-editor/src/main-window/WorkspaceComponentHost.tsx`, `crates/apps/amigo-editor/src/main-window/main-window.css`, `crates/apps/amigo-editor/src/editor-components/componentTypes.ts`, `crates/apps/amigo-editor/src/editor-components/builtinComponents.tsx`, `crates/apps/amigo-editor/src/vite-env.d.ts`.
- Verify: `npm test` 3/3, `npm run build`.
- Tokens: used ~6500, saved ~45-55% przez zawężenie do hosta komponentow i titlebara zamiast czytania calego workspace UI.

### Global Debug Source Overlay
- Task: wyciagnac wspolny overlay debugowy i podpiac go do startupu, standalone windows, startup panels i hosta komponentow workspace.
- Ops: `target/debug/amigo-codemap.exe find`, `target/debug/amigo-codemap.exe scope`, `apply_patch`, `npm test`, `npm run build`.
- Files: `src/debug/debugSource.tsx`, `src/debug/debug-source.css`, `src/main-window/WorkspaceComponentHost.tsx`, `src/main-window/MainEditorWindow.tsx`, `src/startup/StartupDialog.tsx`, `src/startup/ModsPanel.tsx`, `src/startup/ScenePreviewWorkspace.tsx`, `src/startup/ModInspectorPanel.tsx`, `src/theme/ThemeControllerWindow.tsx`, `src/settings/SettingsWindow.tsx`, `src/settings/ModSettingsWindow.tsx`, `src/App.tsx`, `src/editor/EditorWorkspace.tsx`.
- Verify: `npm test` 3/3, `npm run build`.
- Tokens: used ~9000, saved ~55-65% przez codemap scope zamiast recznego szukania wszystkich route/window i paneli.

### Codemap Command Map And Append Plan
- Task: dodac `command-map` do rozwoju samego amigo-codemap oraz `append-plan` pod additive file-ops i token savings.
- Ops: `target/debug/amigo-codemap.exe changed --group package`, `target/debug/amigo-codemap.exe docs`, `apply_patch`, `cargo fmt -p amigo-codemap`, `cargo test -p amigo-codemap parses_command_map_query`, `cargo test -p amigo-codemap append_plan`, `cargo build -p amigo-codemap`, smoke `target/debug/amigo-codemap.exe command-map append-plan`, smoke `target/debug/amigo-codemap.exe append-plan ...`.
- Files: `crates/tools/amigo-codemap/src/cli.rs`, `crates/tools/amigo-codemap/src/main.rs`, `crates/tools/amigo-codemap/src/report/command_map.rs`, `crates/tools/amigo-codemap/src/report/mod.rs`, `crates/tools/amigo-codemap/src/report/file_ops/append_plan.rs`, `crates/tools/amigo-codemap/src/report/file_ops/mod.rs`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `operations.md`.
- Verify: targeted `cargo test -p amigo-codemap` filters passed, `cargo build -p amigo-codemap`, smoke `command-map`, smoke `append-plan`. Full `cargo test -p amigo-codemap` still shows existing snapshot newline failures outside this change.
- Tokens: used ~8000, saved future ~55-70% przy rozwijaniu nowych komend codemap i additive file-ops bez ręcznego szukania po CLI/report/docs.

### Sidescroller Descriptor-First Asset Render Fix
- Task: naprawic regres w `playground-sidescroller`, gdzie sprite gracza renderowal caly spritesheet zamiast pojedynczej klatki, a tileset platform byl szary przez brak poprawnego resolve atlasu descriptor-first.
- Ops: `target/debug/amigo-codemap.exe find`, `target/debug/amigo-codemap.exe scope`, `apply_patch`, `cargo test -p amigo-assets parser -- --nocapture`, `cargo test -p amigo-render-wgpu -- --nocapture`, `cargo test -p amigo-app playground_sidescroller_bootstraps_and_prepares_tile_and_sprite_assets -- --nocapture`, `cargo test -p amigo-app interactive_host_handler_advances_sidescroller_sprite_frames -- --nocapture`, `cargo test -p amigo-app render_runtime -- --nocapture`.
- Files: `crates/apps/app/src/app_helpers.rs`, `crates/apps/app/src/scene_runtime/handlers/tilemap2d.rs`, `crates/apps/app/src/tests/scene_loading_tests/twod.rs`, `crates/engine/assets/src/prepare.rs`, `crates/engine/assets/src/tests/parser.rs`, `crates/engine/render-wgpu/src/renderer/assets.rs`, `crates/engine/render-wgpu/src/renderer/service/texture_batches.rs`, `crates/engine/render-wgpu/src/renderer/tests.rs`.
- Verify: targeted parser, renderer, bootstrap i runtime tests passed.
- Tokens: used ~11000, saved ~50-60% przez zawężenie do descriptor-first asset pipeline zamiast ręcznego czytania całego runtime/render stack.

### YAML Source View Entry Points
- Task: dodac wspolny `YamlSourceRef`, akcje `showYamlView`, przyciski `Show YAML View` w scene preview/properties i usunac `scene.yml`/`scene.rhai` jako dzieci scen w semantic project tree, takze po merge z backendowym `projectStructureTree`.
- Ops: `target/debug/amigo-codemap.exe find`, `target/debug/amigo-codemap.exe changed --group package`, `target/debug/amigo-codemap.exe verify-plan --changed`, `apply_patch`, `npm test`, `npm run build`.
- Files: `crates/apps/amigo-editor/src/features/files/yamlSourceRefs.ts`, `crates/apps/amigo-editor/src/features/files/ShowYamlButton.tsx`, `crates/apps/amigo-editor/src/main-window/workspaceRuntimeServices.ts`, `crates/apps/amigo-editor/src/main-window/MainEditorWindow.tsx`, `crates/apps/amigo-editor/src/features/scenes/ScenePreviewWorkbench.tsx`, `crates/apps/amigo-editor/src/features/inspector/InspectorPanel.tsx`, `crates/apps/amigo-editor/src/properties/*`, `crates/apps/amigo-editor/src/features/project/ProjectExplorerPanel.tsx`, `crates/apps/amigo-editor/src/features/project/projectTreeModel.ts`, `crates/apps/amigo-editor/src/features/project/projectTreeModel.test.ts`, `crates/apps/amigo-editor/src/app/editorEvents.ts`.
- Verify: `npm test` 3/3 files, 13/13 tests; `npm run build`.
- Tokens: used ~9000, saved ~45-60% przez codemapowe zawężenie do workspace services/properties/project explorer zamiast czytania całego editor UI.

### Asset Explorer Descriptor Node Removal
- Task: usunac sztuczne dzieci `Descriptor` z Asset Explorer, zeby skrypty, sceny i `scene.rhai` nie pokazywaly descriptorow jako osobnych pozycji po dodaniu `Show YAML View`.
- Ops: `target/debug/amigo-codemap.exe find`, `target/debug/amigo-codemap.exe scope AssetTree`, `apply_patch`, `npm test`, `npm run build`.
- Files: `crates/apps/amigo-editor/src/assets/assetTreeBuilder.ts`, `crates/apps/amigo-editor/src/assets/assetTreeBuilder.test.ts`, `operations.md`.
- Verify: `npm test` 3/3 files, 13/13 tests; `npm run build`.
- Tokens: used ~2500, saved ~50% przez codemapowe zawężenie do buildera asset tree zamiast szukania w panelach renderujacych.

### Single Scene Context Activation
- Task: ustawic scene navigation jako pojedynczy aktywny kontekst edytora: klik sceny przelacza staly `scene-preview`, prawy dock na `Scene Hierarchy`, a YAML/Rhai otwieraja sie tylko przez Files albo `Show YAML View`.
- Ops: `target/debug/amigo-codemap.exe find`, `target/debug/amigo-codemap.exe scope MainEditorWindow`, `apply_patch`, `npm test`, `npm run build`.
- Files: `crates/apps/amigo-editor/src/main-window/MainEditorWindow.tsx`, `crates/apps/amigo-editor/src/main-window/workspaceRuntimeServices.ts`, `crates/apps/amigo-editor/src/editor-components/builtinComponents.tsx`, `crates/apps/amigo-editor/src/features/project/ProjectExplorerPanel.tsx`, `crates/apps/amigo-editor/src/features/scenes/ScenesBrowserPanel.tsx`, `crates/apps/amigo-editor/src/features/assets/AssetBrowserPanel.tsx`, `crates/apps/amigo-editor/src/app/editorEvents.ts`, `operations.md`.
- Verify: `npm test` 3/3 files, 13/13 tests; `npm run build`.
- Tokens: used ~7500, saved ~45-60% przez codemapowe zawężenie do scene navigation entrypointow i service shape.

### Scene Context Scripts And Source Split
- Task: dodac prawy `Scene Context` z domyslna zakladka `Scripts`, rozdzielic `Show YAML View` od `Open Script`, przeniesc Files/Scripts do bottom docka i usunac scripts jako domenowy bucket z Asset Explorer.
- Ops: `target/debug/amigo-codemap.exe find`, `target/debug/amigo-codemap.exe scope dto`, `apply_patch`, `npm test`, `npm run build`.
- Files: `src/features/files/yamlSourceRefs.ts`, `src/features/scenes/sceneContextModel.ts`, `src/features/scenes/SceneContextPanel.tsx`, `src/features/scenes/ScenePreviewWorkbench.tsx`, `src/features/files/ScriptsBrowserPanel.tsx`, `src/features/assets/AssetBrowserPanel.tsx`, `src/features/assets/assetBrowserModel.ts`, `src/features/assets/assetBrowserModel.test.ts`, `src/assets/assetTreeBuilder.ts`, `src/editor-components/builtinComponents.tsx`, `src/main-window/MainEditorWindow.tsx`, `src/main-window/workspaceRuntimeServices.ts`, `src/app/editorEvents.ts`, `src/dock/dockRegistry.tsx`.
- Verify: `npm test` 3/3 files, 13/13 tests; `npm run build`.
- Tokens: used ~9000, saved ~45-60% przez codemapowe zawężenie do DTO, scene panels, asset browser model i main window services.

### Codemap Patch Check Apply
- Task: dodac do `amigo-codemap` komendy `patch-check` i `patch-apply --write`, ktore przyjmuja unified diff z pliku albo stdin, dry-runuja hunki i opcjonalnie stosuja je do plikow workspace.
- Ops: `target/debug/amigo-codemap.exe command-map append-plan`, `target/debug/amigo-codemap.exe command-map copy-plan`, `apply_patch`, `cargo fmt -p amigo-codemap`, targeted cargo tests, `cargo build -p amigo-codemap`, smoke `patch-check`, smoke `patch-apply --write`.
- Files: `crates/tools/amigo-codemap/src/cli.rs`, `crates/tools/amigo-codemap/src/main.rs`, `crates/tools/amigo-codemap/src/report/file_ops/patch_apply.rs`, `crates/tools/amigo-codemap/src/report/file_ops/mod.rs`, `crates/tools/amigo-codemap/src/report/command_map.rs`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `operations.md`.
- Verify: `cargo test -p amigo-codemap patch_apply`, `cargo test -p amigo-codemap parses_patch_apply_write`, `cargo test -p amigo-codemap command_map`, `cargo build -p amigo-codemap`, smoke patch dry-run/write on temp file.
- Tokens: used ~7000, saved future ~50-70% przy stosowaniu gotowych unified diffow bez recznego przepisywania hunkow.

### Scene Context Dock Patch Batches
- Task: przekonwertowac opisowy markdown patch na cztery czyste unified diff batche i zastosowac je przez `amigo-codemap patch-check`/`patch-apply`.
- Ops: `target/debug/amigo-codemap.exe open-set SceneContextPanel`, `append-plan`, `copy-plan`, `patch-preview`, `patch-check`, `patch-apply --write`, `fallout`.
- Files: `crates/apps/amigo-editor/src/ui/context-dock/*`, `crates/apps/amigo-editor/src/features/files/sourceRefs.ts`, `scriptSourceRefs.ts`, `OpenScriptButton.tsx`, `crates/apps/amigo-editor/src/features/scenes/context/*`, `crates/apps/amigo-editor/src/editor-components/builtinComponents.tsx`, `componentInstances.ts`, `src/main-window/MainEditorWindow.tsx`, `src/main.tsx`.
- Verify: `npm run build` przez `fallout` errors 0; `npm test` przez `fallout` errors 0.
- Tokens: used ~8500, saved ~50-65% przez batchowanie patcha i automatyczne odrzucenie nieczystych fragmentow zamiast recznego przepisywania calego diffu.

### Scene Context Categorized Trees
- Task: uporzadkowac prawy `SceneContextDock`: dodac scrollowane body widgetow oraz zamienic plaskie listy Scripts/Entities na kategoryzowane drzewa.
- Ops: `target/debug/amigo-codemap.exe patch-check`, `patch-apply --write`, `slice SceneScriptsWidget`, generated clean diff dla problematycznego hunka, `fallout`.
- Files: `crates/apps/amigo-editor/src/ui/context-dock/ContextWidget.tsx`, `ContextRow.tsx`, `ContextTree.tsx`, `context-dock.css`, `crates/apps/amigo-editor/src/features/scenes/context/sceneContextIcons.tsx`, `SceneScriptsWidget.tsx`, `SceneEntitiesWidget.tsx`, `operations.md`.
- Verify: `npm run build` przez `fallout` errors 0; `npm test` przez `fallout` errors 0.
- Tokens: used ~4500, saved ~45-60% przez podzial na clean batche i wygenerowanie poprawnego full-file hunka tylko dla `SceneScriptsWidget`.

### Scene Editor Stage 1
- Task: zastapic preview-only workspace frontendowym `Scene Editor` z artboardem, zoomem, trybami, overlayem encji, lokalnym dragowaniem i wspolnym zaznaczaniem z prawym panelem.
- Ops: `target/debug/amigo-codemap.exe open-set ScenePreviewWorkbench`, `copy-plan`, `patch-preview`, `patch-check`, `patch-apply --write`, `fallout`.
- Files: `crates/apps/amigo-editor/src/features/scenes/editor/*`, `crates/apps/amigo-editor/src/features/scenes/ScenePreviewWorkbench.tsx`, `crates/apps/amigo-editor/src/main.tsx`, `operations.md`.
- Verify: `npm run build` przez `fallout` errors 0; `npm test` przez `fallout` errors 0.
- Tokens: used ~7000, saved ~45-60% przez patch batch na 16 plikow i codemapowe sprawdzenie kontekstu zamiast recznego wklejania calego edytora.

### Codemap Next Navigation Features

- Task: dodać next featureset dla LLM workflow: multi-line signatures, symbol-aware ops, trace/open-set/impact improvements, change-plan, explain-file/neighbors/api/component/Tauri/callsite reports, todo/risk indexing.
- Files: `crates/tools/amigo-codemap/src/model.rs`, `test_support.rs`, `scan/signature.rs`, `scan/symbols.rs`, `report/file_ops/slice.rs`, `report/file_ops/symbol_ops.rs`, `report/file_ops/ops_plan.rs`, `report/change_plan.rs`, `report/explain_file.rs`, `report/neighbors.rs`, `report/api_surface.rs`, `report/component_graph.rs`, `report/tauri_graph.rs`, `report/callsite_candidates.rs`, `report/todo_index.rs`, `report/risk_index.rs`, `cli.rs`, `main.rs`, `report/mod.rs`, `report/command_map.rs`, `README.md`, `AMIGO_WORKFLOW.md`.
- Verify: `cargo fmt -p amigo-codemap`, `cargo test -p amigo-codemap --no-run`, `cargo test -p amigo-codemap`, `cargo build -p amigo-codemap`, smoke: `trace`, `signature`, `slice --symbol`, `open-set --why`, `change-plan`.
- Tokens: used ~14000, saved future ~50-70% przez signature/slice/open-set/change-plan przed otwieraniem źródeł.

### Codemap Documentation Refresh

- Task: rozwinąć dokumentację `amigo-codemap 0.1` i root workflow o praktyczny codemap-first sposób pracy w repo, command reference, ops-plan, anchors, workset i zasady weryfikacji.
- Ops: `Get-Content README/AMIGO_WORKFLOW`, dokumentacja z planu użytkownika, `apply_patch`/doc rewrite, smoke komend codemap.
- Files: `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `operations.md`.
- Verify: docs review, `target/debug/amigo-codemap.exe command-map change-plan`, `target/debug/amigo-codemap.exe signature scan_symbols --limit 1`.
- Tokens: used ~5000, saved future ~40-60% przez spójny workflow zamiast powtarzania instrukcji w kolejnych taskach.

### Codemap Symbols File Metadata

- Task: dodać `symbols --file <path> --metadata`, żeby listować metody/symbole jednego pliku razem z params, returns, generics, owner, tags, confidence i range.
- Ops: `explain-file`, `signature`, `apply_patch`, `cargo fmt -p amigo-codemap`, targeted tests, smoke `symbols --file`.
- Files: `crates/tools/amigo-codemap/src/cli.rs`, `crates/tools/amigo-codemap/src/main.rs`, `crates/tools/amigo-codemap/src/report/symbols.rs`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `operations.md`.
- Verify: `cargo test -p amigo-codemap parses_symbols_file_metadata`, `cargo build -p amigo-codemap`, smoke `target/debug/amigo-codemap.exe symbols --file crates/tools/amigo-codemap/src/scan/symbols.rs --metadata --limit 5`.
- Tokens: used ~4000, saved future ~30-50% przy analizie pojedynczych dużych plików bez pełnego `Get-Content`.

### Codemap Ops First Workflow

- Task: rozwinąć `amigo-codemap` pod ops-first implementacje: `ops-skeleton`, `range-for-symbol`, `replace_between_anchors`, opcjonalne `task` w planie i mocniejszy safety output `ops-check`.
- Ops: `range-for-symbol`, `ops-skeleton`, temp `replace_between_anchors` smoke, `apply_patch`, `cargo fmt -p amigo-codemap`, targeted tests, `cargo build -p amigo-codemap`.
- Files: `crates/tools/amigo-codemap/src/cli.rs`, `src/main.rs`, `src/report/file_ops/ops_plan.rs`, `src/report/file_ops/ops_skeleton.rs`, `src/report/file_ops/range_for_symbol.rs`, `src/report/file_ops/mod.rs`, `src/report/command_map.rs`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `operations.md`.
- Verify: `cargo test -p amigo-codemap parses_ops_skeleton_write_out`, `cargo test -p amigo-codemap parses_range_for_symbol_query`, `cargo build -p amigo-codemap`, smoke `range-for-symbol`, `ops-skeleton`, `ops-check`, `ops-apply` on temp anchor plan.
- Tokens: used ~7000, saved future ~40-65% przy średnich/dużych implementacjach przez plan.yml zamiast opisowego prose + ręcznego line hunting.

### Codemap YAML Ops Workflow Upgrade

- Task: domknąć YAML-first workflow dla `amigo-codemap`: inline `--yaml`, stdin `--from -`, `ops-schema`, `ops-split`, `ops-verify`, `ops-summary`, `anchor-range`, strict safety report, backup/stop-on-error i dokumentację.
- Ops: `ops-schema`, `ops-check --yaml`, `ops-preview`, `ops-verify`, `ops-summary`, `anchor-range`, `command-map`, `apply_patch`, docs update.
- Files: `crates/tools/amigo-codemap/src/cli.rs`, `src/main.rs`, `src/report/file_ops/ops_plan.rs`, `src/report/file_ops/ops_schema.rs`, `src/report/file_ops/ops_reports.rs`, `src/report/file_ops/anchor_range.rs`, `src/report/file_ops/mod.rs`, `src/report/command_map.rs`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `operations.md`.
- Verify: `cargo fmt -p amigo-codemap`, `cargo test -p amigo-codemap --no-run`, targeted parser/catalog tests, `cargo test -p amigo-codemap`, `cargo build -p amigo-codemap`, smoke `ops-schema`, inline `ops-check`, `ops-verify`, `ops-summary`.
- Tokens: used ~9000, saved future ~45-70% przez wykonywalne YAML plany zamiast ręcznego przepisywania instrukcji i szukania linii.

### Shared Tree Connector And Toggle Fix

- Task: naprawić shared `TreeView`, gdzie prowadnice linii rozjeżdżały się przez podwójne liczenie indentu, a root/nody mogły nie zwijać się stabilnie przez auto-expand effect.
- Ops: analiza shared `TreeView`, UI Document adapter/styles, Project/Asset tree compatibility styles, patch layout/toggle/expansion, korekta punktu kotwiczenia prowadnic ze środka strzałki na środek ikony itemu.
- Files: `crates/apps/amigo-editor/src/ui/tree/TreeView.tsx`, `src/ui/tree/tree-view.css`, `src/ui/tree/useTreeExpansion.ts`, `src/editors/ui-document/ui-document-structure-dock.css`, `src/main-window/styles/project-tree.css`, `src/main-window/styles/asset-tree.css`, `operations.md`.
- Verify: `npm test -- --run treeTypes uiNodeCapabilities`, `npm run build`.
- Tokens: used ~5000, saved future ~20-35% przez naprawę shared componentu zamiast osobnych obejść w każdym drzewie.

### Codemap Anchor Taxonomy Index

- Task: dodać centralną taksonomię i indeks anchorów `@codemap`, komendy `taxonomy`, `anchors`, `anchor-check` oraz integrację anchorów z `trace`, `open-set`, `change-plan`, `neighbors`.
- Ops: `open-set codemap_tags/open_set/neighbors/command_map`, model/parser/report patches, generated anchor index, smoke nowych komend.
- Files: `codemap.index.md`, `.amigo/codemap.taxonomy.yml`, `.amigo/codemap.anchors.generated.json`, `.amigo/codemap.coverage.generated.md`, `crates/tools/amigo-codemap/src/model.rs`, `scan/codemap_tags.rs`, `taxonomy.rs`, `report/anchors.rs`, `report/anchor_check.rs`, `report/taxonomy_report.rs`, `report/trace.rs`, `report/file_ops/open_set.rs`, `report/change_plan.rs`, `report/neighbors.rs`, `cli.rs`, `main.rs`, `README.md`, `AMIGO_WORKFLOW.md`.
- Verify: `cargo test -p amigo-codemap` 128 passed, `cargo build -p amigo-codemap`, `taxonomy`, `anchors --write`, `anchor-check`, `trace codemap-report-tauri-graph`, `open-set ui-document --why`, `change-plan editor-mode`.
- Tokens: used ~12000, saved future ~50-70% przez generated file-level anchors i scoring domen/rol przed otwieraniem plików.

### Codemap Fast Snapshot Cache

- Task: dodać szybki runtime snapshot cache dla `amigo-codemap`, żeby raportowe komendy nie skanowały pełnego repo przy każdym uruchomieniu.
- Motivation: częste komendy typu `trace`, `open-set`, `impact`, `signature`, `files`, `changed` robiły pełny scan i były za wolne przy dużym repo.
- Added: `.amigo/codemap.snapshot.json`, `snapshot_store.rs`, `refresh`, `status`, `--no-cache`, `watch --write` zapisujący compact output i fast snapshot.
- Files: `crates/tools/amigo-codemap/src/model.rs`, `snapshot_store.rs`, `cache.rs`, `watch.rs`, `cli.rs`, `main.rs`, `report/command_map.rs`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`.
- Verify: `cargo fmt -p amigo-codemap`, `cargo test -p amigo-codemap`, `cargo build -p amigo-codemap`, `refresh`, `status`, `trace codemap`, `open-set codemap --why --limit 10`.
- Tokens: used ~9000, saved future ~40-65% przez cache snapshotu zamiast pełnych skanów na każdej komendzie.

### Codemap Live Git Changes

- Task: dodać live-git summary do `amigo-codemap`, żeby zastąpić szumne `git status --short` i `git diff --stat`.
- Motivation: snapshotowe `changed` może być stale, a ręczne git outputy są zbyt długie dla agenta i mieszają generated/submodule/CRLF warnings z realnym zakresem zmian.
- Added: `changes`, `changes --compact`, `changes --hide-generated`, `changes --group domain|package|status`, `changes --warnings`, `commit-plan`.
- Files: `crates/tools/amigo-codemap/src/cli.rs`, `main.rs`, `report/live_changes.rs`, `report/mod.rs`, `report/command_map.rs`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`.
- Verify: `cargo fmt -p amigo-codemap`, `cargo test -p amigo-codemap`, `cargo build -p amigo-codemap`, `changes --compact --hide-generated`, `changes --group domain`, `commit-plan --compact`.
- Tokens: used ~6500, saved future ~25-45% przez live compact dirty-state summary bez wielokrotnego ręcznego status/diff.

### Codemap Workflow Benchmark Protocol

- Task: dodać do README `amigo-codemap` kontrolowany benchmark porównujący codemap-first flow ze standardowym `rg`/`Get-Content` flow.
- Motivation: potrzebujemy powtarzalnego sposobu mierzenia, ile plików/linii/tokenów codemap oszczędza przy małych, średnich i dużych zadaniach.
- Added: protokół 3 zadań, metryki, reset working tree między przebiegami, helper `Run-Measured`, flows codemap-first i standard, szablon raportu wyników.
- Files: `crates/tools/amigo-codemap/README.md`, `.amigo/codemap.anchors.generated.json`, `operations.md`.
- Verify: `target/debug/amigo-codemap.exe anchors --write`, `target/debug/amigo-codemap.exe anchor-check`.
- Tokens: used ~2500, saved future ~20-35% przy ocenie i dokumentowaniu ROI codemap workflows.

### Workspace Surface Migration Cleanup

- Task: domknąć semantykę `editor/viewer = center surface + dock profile + optional detached workspace`.
- Motivation: center-tab components still advertised direct `window` placement while the actual detach flow is workspace-based; UI Document structure dock also had a legacy local `Inspector` name for node actions.
- Added: P0/P1 anchors for component surface types/helpers, workspace tab strip/detach action, right dock split, UI Document structure/actions/preview styles.
- Files: `crates/apps/amigo-editor/src/editor-components/**`, `src/main-window/**`, `src/app/editorEvents.ts`, `src/editors/ui-document/**`, `codemap.index.md`, `.amigo/codemap.taxonomy.yml`.
- Verify: codemap stale scan for window semantics, legacy UI inspector naming, and right dock split naming; `target/debug/amigo-codemap.exe registry-check components --limit 100`, `target/debug/amigo-codemap.exe verify-plan --changed`, `npm test`, `npm run build`, `target/debug/amigo-codemap.exe anchors --write`, `target/debug/amigo-codemap.exe anchor-check`.

### UI Document Simple Realtime Preview MVP

- Task: dodać frontendowy przełącznik Simple/Realtime preview oraz realny `actionTarget` dla linków UI.
- Added: `actionTarget` w frontend/backend DTO, YAML patch `action_event/action_target`, edycja pól w `UiNodePropertiesPanel`, Simple HTML artboard z faktycznego `document.root`, runtime-looking Realtime shell i Screen Links z DTO.
- Files: `crates/apps/amigo-editor/src/api/dto.ts`, `src-tauri/src/dto.rs`, `src-tauri/src/commands/project_tree.rs`, `src-tauri/src/editor_mode/ui_node_patch.rs`, `src/properties/panels/UiNodePropertiesPanel.tsx`, `src/editors/ui-document/*preview*`.
- Verify: `npm run build`, `npm test`, `cargo test -p amigo-editor patches_button_action_target_inside_ui_document`, `cargo build -p amigo-editor`, `target/debug/amigo-codemap.exe anchors --write`, `target/debug/amigo-codemap.exe anchor-check`.

### Shared Tree Legacy Renderer Removal

- Task: usunąć legacy tree/list renderery z asset/file/context pobocznych widoków i zostawić shared `TreeView` jako jedyny aktywny renderer hierarchii.
- Added: explicit shared tree guide anchor offset, context tree adapter on shared `TreeView`, root-consistent `ProjectFileTree`, asset browser locked to `AssetTreePanel`, removal of legacy `FolderView` and stale tree/list CSS selectors.
- Files: `crates/apps/amigo-editor/src/ui/tree/*`, `src/features/files/*Tree*`, `src/features/assets/AssetBrowserPanel.tsx`, `src/assets/AssetTreePanel.tsx`, `src/ui/context-dock/ContextTree.tsx`, `src/main-window/styles/{asset-tree,asset-tree-status,project-tree}.css`, `src/editor-components/builtin/assetComponents.tsx`.
- Verify: `rg` for legacy tokens, `npm test -- --run treeTypes uiNodeCapabilities assetThumbnailResolver`, `npm run build`, `target/debug/amigo-codemap.exe impact tree-view --limit 30`.

### Editor Target Activation Batch 2

- Task: dodać centralne `activateEditorTarget`, podpiąć `currentEditorTarget` do `WorkspaceRuntimeServices` i przełączyć prawy dock na docelowe źródło selection.
- Ops: `amigo-codemap ops-check/ops-apply --from -`, poprawki `amigo-codemap` dla `replace_file` create-on-missing, `insert_before_text`, `insert_after_text`, `replace_text`, CRLF locatorów i błędnego exit code `ops-check`.
- Files: `crates/apps/amigo-editor/src/editor-targets/*`, `src/main-window/MainEditorWindow.tsx`, `src/main-window/workspaceRuntimeServices.ts`, `src/main-window/hooks/useWorkspaceRuntimeServices.ts`, `src/features/inspector/PropertiesPanel.tsx`, `src/app/editorEvents.ts`, `crates/tools/amigo-codemap/src/report/file_ops/*`, `.amigo/codemap.*.generated.*`.
- Verify: `npm run build`, `npm test`, `target/debug/amigo-codemap.exe trace editor-target-activation`, `trace editor-target-open-routing`, `trace workspace-current-editor-target`, `trace activateEditorTarget`, `trace currentEditorTarget`, `verify-plan --changed`, `anchors --write`, `anchor-check`, `cargo test -p amigo-codemap`.
- Tokens: used ~9000, saved ~35-55% przez rozszerzenie ops CLI zamiast ręcznego przepisywania kolejnych batchy.

### Editor Target Left Panels Batch 3

- Task: przepiąć pierwsze lewe panele (`ProjectExplorerTree`, `ProjectExplorerPanel`, `ProjectFileTree`, `FilesBrowserPanel`, `AssetTreePanel`, `AssetBrowserPanel`) na `activateEditorTarget`.
- Ops: `amigo-codemap ops-check/ops-apply --from -` dla pełnych replace plików, ręczny focused patch dla paneli, gdzie istniejący kod odbiegał od planowanych locatorów.
- Files: `crates/apps/amigo-editor/src/features/files/ProjectFileTree.tsx`, `src/features/files/FilesBrowserPanel.tsx`, `src/features/project/ProjectExplorerTree.tsx`, `src/features/project/ProjectExplorerPanel.tsx`, `src/assets/AssetTreePanel.tsx`, `src/features/assets/AssetBrowserPanel.tsx`, `.amigo/codemap.*.generated.*`.
- Verify: `trace files-browser-target-wiring`, `trace project-explorer-shared-tree`, `trace asset-shared-tree-section`, `trace activateEditorTarget`, `stale --patterns ...`, `verify-plan --changed`, `npm test`, `npm run build`, `anchors --write`, `anchor-check`.
- Tokens: used ~8500, saved ~25-40% przez batchowe replace i stale scan zamiast ręcznego przeglądu każdego callsite.

### Editor Target Scene UI Diagnostics Batch 4

- Task: przepiąć `SceneHierarchyPanel`, `SceneHierarchyTree`, `UiDocumentStructureDock`, `DiagnosticsPanel` i `ProblemsTable` na `EditorTarget`.
- Ops: `amigo-codemap ops-check/ops-apply --from -` dla scene/diagnostics replace oraz UI structure text ops, jeden focused TS prop-type fallout fix.
- Files: `crates/apps/amigo-editor/src/features/scenes/SceneHierarchyPanel.tsx`, `src/features/scenes/SceneHierarchyTree.tsx`, `src/editors/ui-document/UiDocumentStructureDock.tsx`, `src/features/diagnostics/DiagnosticsPanel.tsx`, `src/features/diagnostics/ProblemsTable.tsx`, `.amigo/codemap.*.generated.*`.
- Verify: `trace scene-hierarchy-target-tree`, `trace scene-hierarchy-target-mapper`, `trace ui-document-structure-dock`, `trace diagnostics-panel-target-wiring`, `trace problems-table-target-wiring`, `stale --patterns ...`, `verify-plan --changed`, `npm test`, `npm run build`, `anchors --write`, `anchor-check`.
- Tokens: used ~6500, saved ~25-35% przez centralny target mapper i stale scan.

### Editor Target Migration Finalization Batch 5

- Task: domknąć migrację `EditorTarget` przez ukrycie legacy open/select API za `targetBridge`.
- Ops: `WorkspaceRuntimeServices` expose tylko `currentEditorTarget`, `activateEditorTarget`, `targetBridge`; `editorTargetActivation` używa bridge; poboczne scene/scripts/inspector/UI editor callsite'y przepięte na `activateEditorTarget` albo `targetBridge`.
- Files: `crates/apps/amigo-editor/src/main-window/{workspaceRuntimeServices,MainEditorWindow}.tsx?`, `src/editor-targets/editorTargetActivation.ts`, `src/features/{scenes,files,inspector,project}/**`, `src/editors/ui-document/UiDocumentEditor.tsx`, `.amigo/codemap.*.generated.*`.
- Verify: `stale` public legacy service fields clean, `trace editor-target-*`, `registry-check properties/components`, `verify-plan --changed`, `npm test`, `npm run build`, `anchors --write`, `anchor-check`.
- Tokens: used ~6000, saved ~25-40% przez stale-check + focused fallout patches zamiast ręcznego przechodzenia całego service baga.

### Editor Target Final Cleanup

- Task: usunąć duplikat project-node routing po migracji `EditorTarget MVP`.
- Ops: usunięty legacy `PROJECT_NODE_ACTIONS` path, `handleProjectNodeActivated`, `WorkspaceProjectNodeRef`, event `ProjectTreeNodeActivated`; `ProjectNodeContextMenu` używa teraz `onActivateNode(..., "open")`; `ProjectNodeActionStrip` otwiera przez target-aware callback; codemap `project-actions` registry wskazuje nowy target routing.
- Files: `crates/apps/amigo-editor/src/main-window/MainEditorWindow.tsx`, `src/main-window/workspaceRuntimeServices.ts`, `src/features/project/ProjectExplorerPanel.tsx`, `src/features/project/projectNodeActions.ts`, `src/app/editorEvents.ts`, `crates/tools/amigo-codemap/src/report/registry.rs`, `.amigo/codemap.*.generated.*`.
- Verify: `stale PROJECT_NODE_ACTIONS/projectNodeActions/handleProjectNodeActivated`, `stale services.* legacy select/open`, `npm test`, `npm run build`, `cargo fmt/test/build -p amigo-codemap`, `anchors --write`, `anchor-check`.
- Tokens: used ~3500, saved ~10-20% przez usunięcie równoległego project-node action flow.

### Codemap Range For Lines

- Task: dodać `range-for-lines` generujące bezpieczne YAML ops dla konkretnych linii.
- Ops: nowa komenda CLI, dispatch, `file_ops/range_for_lines.rs`, command-map descriptor, przykłady `replace_range/delete_range` w `ops-schema`, dokumentacja workflow.
- Files: `crates/tools/amigo-codemap/src/{cli,main}.rs`, `src/report/file_ops/{mod,range_for_lines,ops_schema}.rs`, `src/report/command_map.rs`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`.
- Verify: `cargo fmt -p amigo-codemap --check`, `cargo test -p amigo-codemap range_for_lines`, `cargo test -p amigo-codemap`, `cargo build -p amigo-codemap`, smoke `range-for-lines ... --yaml-op replace_range/delete_range`, `command-map range-for-lines`.
- Tokens: used ~4500, saved future ~20-35% przy line-based codemap YAML ops.

### Codemap Ops Mutation API

- Task: rozszerzyć istniejący `version: 1` ops-plan bez tworzenia legacy/v2 formatu.
- Ops: `content_root`, `content_from`, `copy_file`, `move_file`, `rename_file`, `create_dir`, `delete_dir`, path safety, `deny_unknown_fields`, walidacja `version == 1`, strict apply, non-zero failed apply, sekwencyjne `ops-check` dla podstawowych FS ops.
- Files: `crates/tools/amigo-codemap/src/report/file_ops/{ops_plan,ops_reports,ops_schema}.rs`, `crates/tools/amigo-codemap/src/main.rs`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`.
- Verify: `cargo fmt -p amigo-codemap`, `cargo check -p amigo-codemap`, `cargo build -p amigo-codemap`, smoke `content_from + create_dir + copy_file + move_file`, `ops-schema --example copy_file`, `ops-schema --example create_file`.
- Tokens: used ~6500, saved future ~30-50% przez sidecar code files i realne FS ops zamiast dużych inline YAML bloków.

### Editor Target Context Profiles

- Task: dodać typed `primary/secondary` registry dla paneli kontekstowych targetów.
- Ops: nowe typy `TargetPanelComponent`, `TargetPanelInput`, normalizacja pojedynczy komponent albo tablica, MVP panele target details/actions/diagnostics/source/properties, registry `EDITOR_TARGET_CONTEXT_PROFILES`.
- Files: `crates/apps/amigo-editor/src/editor-targets/editorTargetContextTypes.ts`, `editorTargetContextPanels.tsx`, `editorTargetContextProfiles.ts`, `editor-targets/index.ts`.
- Verify: `npm run build`, `npm test`, `anchors --write`, `anchor-check`.
- Tokens: used ~2500, saved future ~15-25% przez centralny profile registry zamiast dock string routing.

### Editor Target Context Primary + UI Bindings

- Task: renderować `contextProfile.primary` w prawym górnym properties panelu i dodać read-only `UiModelBindings` context.
- Ops: wspólny `TargetContextPanelList`, `PropertiesPanel` jako renderer primary, `TargetContextPanel` jako renderer secondary, frontendowy parser YAML `UiModelBindings`, `UiBindingsPanel` dla `uiDocument`/`uiNode`.
- Files: `crates/apps/amigo-editor/src/features/{inspector,target-context}/**`, `src/editor-targets/editorTargetContextProfiles.ts`.
- Verify: `npm run build`, `npm test`, `trace target-context-panel-list`, `trace target-context-primary-renderer`, `trace ui-bindings-model`, `trace ui-bindings-panel`, `verify-plan --changed`, `anchors --write`, `anchor-check`.
- Tokens: used ~3000, saved future ~15-25% przez użycie profile component refs zamiast osobnego dock routing dla bindingów.
