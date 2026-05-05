# Operations

Lekki dziennik pracy. Najnowsze wpisy na gorze.

Format:
- Task: co robimy.
- Ops: uzyte narzedzia/komendy.
- Files: najwazniejsze pliki.
- Verify: build/test/check albo `docs only`.
- Tokens: szacunek `used` i `saved`.

## 2026-05-05

### Codemap Refactor Reports
- Task: dodac operacyjne raporty `amigo-codemap`, fixture/snapshot testy oraz opis workflow pracy z nowymi komendami.
- Ops: `amigo-codemap verify-plan`, `stale`, `impact`, `fallout`, `move-plan`, `dup`, `tauri-commands`, `service-shape`, `registry-check`, `operations-summary`, `commit-summary`, `apply_patch`, `cargo test`, `cargo build`.
- Files: `crates/tools/amigo-codemap/src/report/*`, `crates/tools/amigo-codemap/src/{cli,main}.rs`, `crates/tools/amigo-codemap/tests/*`, `crates/tools/amigo-codemap/README.md`, `AMIGO_WORKFLOW.md`, `crates/tools/amigo-codemap/PR_SPLIT.md`.
- Verify: `cargo test -p amigo-codemap` 54/54, `cargo build -p amigo-codemap`.
- Tokens: used ~22000, saved ~60-70% przez przeniesienie powtarzalnych rg/build-log/registry/Tauri checks do raportow codemap.

### Final Cleanup Pass
- Task: dosprzatac backend helpery po splitcie, przepiac project node actions na registry i uproscic drobne visual maps.
- Ops: `amigo-codemap scope`, `rg`, `apply_patch`, `npm test`, `npm run build`, `cargo test -p amigo-editor --lib`, `amigo-codemap compact`.
- Files: `src-tauri/src/commands/shared.rs`, `src-tauri/src/commands/{mods,cache,project_files,project_tree,mod}.rs`, `features/project/projectNodeActions.ts`, `main-window/MainEditorWindow.tsx`, `features/tasks/TaskTable.tsx`, `features/events/eventFormatters.ts`.
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
