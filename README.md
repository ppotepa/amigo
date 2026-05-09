# Amigo

Amigo is a mod-first Rust + Tauri monorepo for a 2D/3D engine, desktop editor, and launcher.

## Repository map (short)

```text
crates/
  apps/
    app/                 Runtime host / game application
    launcher/            TUI launcher
    amigo-editor/        Desktop editor (Tauri + Vite)

  foundation/            Core shared types and utilities
  engine/                Scene, runtime, input, audio, rendering APIs
  platform/              Host backends and platform adapters
  scripting/             Rhai integration and scripting interfaces
  2d/                    2D rendering/domain crates
  3d/                    3D rendering/domain crates
  ui/                    Shared UI runtime abstractions
  audio/                 Audio API and implementations
  tools/                 Developer tools

mods/
  core, core-game, playground-2d, playground-3d, ...
```

### Relevant editor paths

- `crates/apps/amigo-editor/src/workbench/target-view/` — target contract host, contract registry, and rendering slots
- `crates/apps/amigo-editor/src/workbench/layout/` — slot/tab/split layout primitives
- `crates/apps/amigo-editor/src/workbench/widgets/` — generic workbench widgets
- `crates/apps/amigo-editor/src/features/scene/target/` — scene target contract/model/actions
- `crates/apps/amigo-editor/src/features/entity/target/` — scene-entity target contract/model/actions
- `crates/apps/amigo-editor/src/features/target-panel/` — target panel component used by workbench

Legacy UI paths (`features/scenes/context`, `ui/context-dock`, `context.panel` entrypoints) are being migrated to this target-view stack.

## Running the project

### 1. Build workspace

From repo root:

```powershell
cargo build --workspace
```

### 2. Run launcher + runtime

```powershell
# launcher profile menu
cargo run -p amigo-launcher

# direct launch example (hosted mode)
cargo run -p amigo-launcher -- --hosted --mod=playground-2d --scene=basic-scripting-demo

# direct runtime (without launcher)
cargo run -p amigo-app -- --hosted --mods-root mods --mod=playground-2d --scene=basic-scripting-demo
```

Launcher flags of interest:

- `--mod=<mod-id>` — module id (`playground-2d`, `core-game`, `core`)
- `--scene=<scene-id>` — scene name
- `--headless` — run without window
- `--profile=<id>` — launch profile from `config/launcher.toml`

### 3. Run editor

From `crates/apps/amigo-editor`:

```powershell
npm install
npm run dev            # frontend only
npm run tauri:dev      # full desktop shell (recommended)
```

Useful editor commands after change:

```powershell
npm run build --prefix crates/apps/amigo-editor
npm run test --prefix crates/apps/amigo-editor
```

## Recommended day-to-day commands

```powershell
# full Rust verification
cargo test -p amigo-editor
cargo test -p amigo-scene component_descriptors

# editor UI tests / build
npm run build --prefix crates/apps/amigo-editor
npm run test --prefix crates/apps/amigo-editor -- targetViewRegistry
```

## Good first files to explore

- `crates/apps/launcher/src/main.rs` — launcher argument parsing
- `config/launcher.toml` — launcher profile defaults
- `crates/apps/app/src/main.rs` — bootstrap settings for app runtime
- `crates/engine/scene/` — scene model and component descriptors
- `crates/engine/runtime/` — runtime flow orchestration
- `crates/engine/render-wgpu/` — renderer backend
- `crates/apps/amigo-editor/src/workbench` — host/slots/widget stack
- `crates/apps/amigo-editor/src/features/scene/target` — first target contract migration slice

## Architecture snapshot

Right now editor architecture is migrating from legacy context docking (`ContextDock` / `SceneContext` / `TargetContext`) to the new contract-driven flow:

```text
layout -> TargetViewHost -> resolveTargetContract -> target contract (scene/entity/asset/component/file) -> slots -> tabs -> widgets
```

This README is the short reference for current startup and project structure.
