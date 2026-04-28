# Amigo Architecture

## 1. Purpose

Amigo is a modular Rust engine workspace built around one core rule:

1. the runtime always boots `core`
2. the user selects one `root mod`
3. the engine resolves dependencies for that root mod
4. the user starts one scene exposed by that mod

The project is intentionally `mod-first`, `contract-first`, and `plugin-driven`.
Most systems are designed as small crates that expose stable services and queues instead of large shared globals.

---

## 2. Top-Level Shape

The repository is a Cargo workspace split by responsibility:

```text
crates/
  foundation/
    core
    math
  engine/
    runtime
    assets
    hot-reload
    modding
    render-api
    render-wgpu
    scene
  platform/
    app-host-api
    app-host-winit
    file-watch-api
    file-watch-notify
    input-api
    input-winit
    window-api
    window-winit
  scripting/
    api
    rhai
  2d/
    sprite
    text
  3d/
    mesh
    material
  apps/
    app
    launcher

mods/
  core
  core-game
  playground-2d
  playground-3d

config/
  launcher.toml
```

The workspace default member is `amigo-launcher`, so the launcher is the default entry point for humans.

---

## 3. Architectural Principles

### 3.1 Mod-first runtime

Gameplay and tools are modeled as mods. The engine should not hardcode project content into the app binary.

### 3.2 Root-mod startup

The normal startup model is:

1. load `core`
2. load one selected `root mod`
3. resolve and load its dependencies
4. choose one startup scene from that root mod

### 3.3 Small crates, explicit boundaries

Each crate owns one responsibility and communicates through typed services, queues, and manifests.

### 3.4 Stable contracts before feature depth

The code prefers durable contracts such as:

1. services in the runtime registry
2. script command queues
3. scene command queues
4. asset catalog states
5. hot-reload watch descriptors

### 3.5 Platform isolation

`platform/*` crates hide backend details such as `winit` or `notify`.
Higher layers depend on engine-facing contracts, not directly on backend implementation details.

---

## 4. Foundation Layer

### 4.1 `amigo-core`

`crates/foundation/core`

This crate contains shared engine-level concepts used across the workspace:

1. `AmigoResult` and engine errors
2. typed IDs
3. launch selection data
4. runtime diagnostics snapshots

It is intentionally small and backend-agnostic.

### 4.2 `amigo-math`

`crates/foundation/math`

This crate contains simple math/data structures used by scene and domain crates:

1. vectors
2. transforms
3. colors

---

## 5. Runtime Composition Model

### 5.1 `amigo-runtime`

`crates/engine/runtime`

This is the composition root for services.

Core concepts:

1. `RuntimePlugin`
2. `ServiceRegistry`
3. `Runtime`
4. `RuntimeBuilder`

Every plugin registers typed services into the runtime.
Services are resolved by type, stored behind `Arc`, and are globally available to the runtime after bootstrap.

### 5.2 Why this matters

This keeps crate boundaries clear:

1. systems do not import each other through large mutable singletons
2. systems ask for explicit services
3. higher-level orchestration happens in `amigo-app`, not in random crates

---

## 6. App and Launcher

### 6.1 `amigo-launcher`

`crates/apps/launcher`

The launcher is a host-side control surface, not the engine.

Responsibilities:

1. load `config/launcher.toml`
2. expose profiles in a TUI
3. expose root mods and scenes
4. validate profiles before launch
5. launch `amigo-app` either in-process or as a release binary

Current TUI model:

1. top profile bar
2. left panel with root mods
3. right panel with scenes
4. bottom diagnostics/status area

Behavior:

1. `Enter` on a profile selects the profile
2. `Enter` on a mod launches that mod using its first declared scene
3. `Enter` on a scene launches that exact scene

### 6.2 `amigo-app`

`crates/apps/app`

This is the actual runtime bootstrap layer.
It is responsible for wiring plugins, loading mods, resolving scenes, running scripts, stabilizing queues, and optionally opening the hosted window.

Main CLI shape:

```text
amigo-app --mod=<root-mod> --scene=<scene-id> [--hosted] [--dev] [--mods-root=mods]
```

`--mods` still exists as a low-level override, but normal flow is `--mod + --scene`.

### 6.3 Bootstrap flow

The bootstrap path in `amigo-app` looks like this:

1. build `BootstrapOptions`
2. derive `LaunchSelection`
3. construct `RuntimeBuilder`
4. register engine/platform/domain/scripting plugins
5. validate root mod and scene against the loaded mod catalog
6. load the selected scene document if present
7. queue document hydration
8. execute enabled mod scripts
9. load the active scene script if present
10. stabilize the runtime until queues and asset work settle
11. emit a runtime summary

Headless mode prints a summary.
Hosted mode opens the `winit` host and keeps the runtime alive.

---

## 7. Platform Layer

### 7.1 Window and input

Contracts:

1. `platform/window-api`
2. `platform/input-api`

Current backends:

1. `platform/window-winit`
2. `platform/input-winit`

The engine sees abstract window/input service info and normalized input events.

### 7.2 App host

Contracts:

1. `platform/app-host-api`

Backend:

1. `platform/app-host-winit`

This layer owns the hosted app lifecycle:

1. create the window
2. run the event loop
3. dispatch lifecycle events
4. allow dev-mode interaction while the runtime stays alive

### 7.3 File watch

Contracts:

1. `platform/file-watch-api`

Backend:

1. `platform/file-watch-notify`

This layer handles native file notifications.
If native notifications are unavailable, the engine can still fall back to the polling path from `engine/hot-reload`.

---

## 8. Rendering Layer

### 8.1 `engine/render-api`

This crate defines backend-facing render information and initialization reporting.

### 8.2 `engine/render-wgpu`

Current rendering backend is `wgpu`.

Current role:

1. bootstrap backend information
2. expose renderer service state
3. support the hosted path

Important limitation:

The project is not yet a full render graph or full resource-lifetime engine.
The current architecture is ready for that evolution, but it is still in a staged foundation/hardening phase.

---

## 9. Modding Layer

### 9.1 Manifest model

`engine/modding` loads `mod.toml` files into:

1. `ModManifest`
2. `ModSceneManifest`
3. `DiscoveredMod`
4. `ModCatalog`

Each mod can declare:

1. metadata
2. dependencies
3. capabilities
4. an optional `[scripting]` block for `mod.rhai`
5. scenes

### 9.2 Resolution model

The mod catalog resolves dependencies before runtime boot.

Rules:

1. dependency order is deterministic
2. missing dependencies are hard errors
3. dependency cycles are hard errors
4. `core` is the universal base mod

### 9.3 Runtime semantics

The launcher and app do not treat startup as “run an arbitrary bag of mods”.
They treat startup as:

1. one selected root mod
2. automatic dependency resolution
3. one selected scene from that root mod

That keeps mod startup understandable and predictable.

### 9.4 Scene-centric content layout

The canonical content layout is scene-folder based:

```text
mods/
  some-mod/
    mod.toml
    scripts/
      mod.rhai
    scenes/
      some-scene/
        scene.yml
        scene.rhai
```

Each `[[scenes]]` entry points at a scene root folder through `path`.

Defaults:

1. scene document: `path/scene.yml`
2. scene codebehind: `path/scene.rhai`

Scene-level `document` and `script` are optional overrides, but the scene folder remains the ownership boundary.

### 9.5 Mod and scene scripting model

`mod.rhai` is optional and configured through:

1. `scripting.mod_script`
2. `scripting.mod_script_mode = disabled | bootstrap | persistent`

Intended usage:

1. `disabled`: content-only mod
2. `bootstrap`: one-shot mod bootstrap work
3. `persistent`: shared state or routing across scenes

`scene.rhai` is scene-local codebehind and is loaded only for the active scene.

This means:

1. no shared `if active_scene == ...` branches
2. each scene owns its own behavior
3. cross-scene state belongs in persistent `mod.rhai`, not in scene scripts

---

## 10. Scene Layer

### 10.1 `engine/scene`

This crate is the central scene orchestration layer.

Important concepts:

1. `SceneKey`
2. `SceneService`
3. `SceneCommand`
4. `SceneEvent`
5. scene documents
6. hydration plans
7. hydrated scene snapshot/state

### 10.2 Scene service

`SceneService` owns:

1. selected scene
2. named entities
3. basic spawn/clear/select operations

This is not a full ECS.
It is a lightweight scene registry and command target.

### 10.3 Scene commands

The runtime does not mutate everything directly.
It pushes work through `SceneCommand`, for example:

1. select scene
2. reload active scene
3. clear entities
4. queue 2D sprite/text work
5. queue 3D mesh/material work

### 10.4 Scene documents

Scenes can be described by YAML files referenced from `mod.toml`.

Current scene document pipeline:

1. select mod scene
2. resolve YAML path
3. load `SceneDocument`
4. build `SceneHydrationPlan`
5. queue entity spawns and domain-specific scene commands

### 10.5 Rehydration

Active scene reload uses the same queue-driven path as first load.

That means:

1. explicit reload
2. file-driven reload
3. scene selection

all converge on the same runtime orchestration path.

---

## 11. Asset Layer

### 11.1 `engine/assets`

This crate owns the asset catalog and state transitions.

Main concepts:

1. `AssetKey`
2. `AssetManifest`
3. `AssetLoadRequest`
4. `AssetCatalog`
5. `LoadedAsset`
6. `PreparedAsset`

### 11.2 Asset states

The current asset pipeline is intentionally explicit:

1. `registered`
2. `pending`
3. `loaded`
4. `prepared`
5. `failed`

### 11.3 What “prepared” means today

Prepared assets are still CPU-side placeholders or metadata-level prepared values.
This is already useful for:

1. diagnostics
2. scripting visibility
3. stable runtime contracts
4. future renderer preparation

### 11.4 Asset ownership

Assets are usually sourced from:

1. `engine`
2. `mod:<mod-id>`
3. filesystem roots
4. generated content

Mod asset paths are resolved relative to the owning mod root.

### 11.5 Reload model

Asset reload goes back through the same stable path:

1. request reload
2. clear loaded/prepared/failed state
3. queue pending request again
4. resolve filesystem path again
5. load and prepare again

---

## 12. Scripting Layer

### 12.1 `scripting/api`

This crate defines the engine-facing scripting contracts:

1. `ScriptRuntime`
2. `ScriptRuntimeService`
3. `ScriptCommand`
4. `ScriptEvent`
5. `ScriptCommandQueue`
6. `ScriptEventQueue`
7. `DevConsoleQueue`
8. `DevConsoleState`

### 12.2 `scripting/rhai`

This is the current script backend.

Responsibilities:

1. execute enabled `Rhai` mod scripts
2. execute the active scene's `scene.rhai`
3. expose safe helper functions to scripts
4. translate script calls into queue-driven engine commands
5. expose diagnostics/runtime information back to scripts

The target shape of the scripting surface is described in:

1. `docs/RHAI_API.md`

### 12.3 Script lifecycle

Current lifecycle hooks are designed around the active scene plus optional persistent mod scripts:

1. `on_enter()`
2. `update(dt)`
3. `on_event(topic, payload)`
4. `on_exit()`

At runtime:

1. persistent mod scripts stay loaded across scene switches
2. scene scripts are unloaded and reloaded with the active scene
3. both surfaces use the same safe `world.*` API

### 12.4 Why queues are important

Scripts do not directly mutate every engine subsystem.
Instead they queue work such as:

1. scene selection
2. scene reload
3. asset reload
4. spawn 2D or 3D domain objects
5. console commands

Then `amigo-app` consumes those queues during stabilization or hosted runtime updates.

---

## 13. Domain Layer

### 13.1 2D

Crates:

1. `2d/sprite`
2. `2d/text`

These own domain-specific scene services and records for:

1. sprite entities
2. text entities

They do not own global bootstrap or mod resolution logic.
They consume scene commands produced by scene hydration or scripting.

### 13.2 3D

Crates:

1. `3d/mesh`
2. `3d/material`

These do the same for 3D-oriented placeholder domain state:

1. mesh entities
2. material bindings

### 13.3 Current maturity

These domains are real enough to support:

1. scene-document hydration
2. script-driven queueing
3. asset registration
4. runtime summaries

They are not yet a full gameplay/object framework.

---

## 14. Hot Reload Layer

### 14.1 `engine/hot-reload`

This crate tracks watch targets and change detection.

Important concepts:

1. `SceneDocumentWatch`
2. `AssetWatch`
3. `HotReloadWatchDescriptor`
4. `HotReloadChange`
5. `HotReloadService`

### 14.2 What it watches

Today it watches:

1. the active scene document
2. registered asset files that resolve to real paths

### 14.3 How reload enters the runtime

Hot reload does not bypass engine logic.

Changes become:

1. `SceneCommand::ReloadActiveScene`
2. asset reload requests through `AssetCatalog`

So file changes use the same path as explicit reload commands.

### 14.4 Native watcher plus fallback

The engine can:

1. drain native file events from `notify`
2. map them back to watched scene/asset descriptors
3. fall back to polling if no native watcher event is available

---

## 15. Hosted Dev Mode

Hosted mode uses the app host layer and keeps the runtime alive.

Current dev-host behavior:

1. open a real `winit` window
2. keep runtime state alive
3. process runtime queues incrementally
4. allow scene switching while running
5. surface console output through runtime summaries and dev diagnostics

This is the beginning of the dev shell path, not the final editor.

`core-game` is currently the first dev-oriented root mod.

---

## 16. Mods in This Repository

### 16.1 `mods/core`

Base runtime/content layer.
This should always be present.

### 16.2 `mods/core-game`

Current dev-shell style root mod.
It is where diagnostics, console-oriented behavior, and dev startup live today.

### 16.3 `mods/playground-2d`

Validation content for the 2D pipeline:

1. scenes
2. assets
3. script hooks
4. YAML scene documents

### 16.4 `mods/playground-3d`

Validation content for the 3D pipeline:

1. mesh/material assets
2. scenes
3. YAML scene documents

---

## 17. End-to-End Flow

### 17.1 Normal launcher flow

1. user opens `amigo-launcher`
2. selects profile
3. selects root mod or scene
4. launcher validates the profile
5. launcher launches `amigo-app`
6. app loads `core + root mod + dependencies`
7. app resolves scene document and runs scripts
8. runtime stabilizes queues
9. runtime enters headless or hosted mode

### 17.2 Headless flow

Used for diagnostics, smoke runs, and tests.
The app prints a bootstrap summary showing:

1. selected backends
2. loaded mods
3. active scene
4. loaded/prepared assets
5. watched reload targets
6. processed commands/events

### 17.3 Hosted flow

Used for interactive runtime work.
The app opens a window, retains runtime services, and keeps reacting to input and reload changes.

---

## 18. What Is Real Today vs What Is Still Foundational

### Real today

1. workspace architecture
2. runtime plugin/service model
3. launcher profiles and TUI
4. root-mod startup model
5. mod discovery and dependency resolution
6. scene manifests and YAML scene documents
7. scene command queueing
8. Rhai scripting integration
9. asset catalog with explicit states
10. hot reload watch pipeline
11. first 2D and 3D validation slices
12. headless CLI integration tests

### Still foundational

1. full renderer lifecycle hardening
2. richer in-window developer UI
3. deeper asset decoding/preparation
4. broader gameplay-level systems
5. a more advanced world/entity model than the current lightweight scene registry

---

## 19. How To Read the Codebase

If you want to understand the repo quickly, follow this order:

1. `Cargo.toml`
2. `crates/apps/launcher`
3. `crates/apps/app`
4. `crates/engine/runtime`
5. `crates/engine/modding`
6. `crates/engine/scene`
7. `crates/engine/assets`
8. `crates/scripting/api`
9. `crates/scripting/rhai`
10. `crates/platform/*`
11. `mods/*`

That path mirrors the real runtime flow from user action to engine state.

---

## 20. Current Mental Model

The shortest correct description of Amigo today is:

`launcher` selects a profile, root mod, and scene.
`app` builds a plugin-based runtime.
`modding` resolves `core + root mod + dependencies`.
`scene` loads YAML scene documents and turns them into queue-driven commands.
`scene` also owns engine-level scene transitions, which can be declared in YAML or triggered from scripts.
`scripting/rhai` can push additional commands, diagnostics, and direct scene jumps into the runtime.
`assets` manages file-backed asset states.
`2d/*` and `3d/*` consume scene commands into domain state.
`hot-reload` watches scene and asset files and routes changes back through the same engine contracts.

That is the current architecture.
