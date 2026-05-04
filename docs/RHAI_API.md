# Rhai API Structure

## 1. Goal

Amigo should treat `Rhai` as a small domain-oriented scripting framework, not as a bag of flat helper functions.

The scripting surface should:

1. feel like a stable mini-framework
2. be split by responsibility and domain
3. follow `SRP`
4. stay backend-agnostic
5. expose engine contracts, not raw Rust services
6. support mod-first scene workflows

The desired shape is:

```rhai
fn update(dt) {
    let square = world.entities.named("playground-2d-square");

    if world.input.down("Left") {
        square.rotate_2d(-2.0 * dt);
    }

    if world.input.pressed("Up") {
        world.scene.select("hello-world-spritesheet");
    }
}
```

---

## 2. Design Principles

### 2.1 One root object

Scripts should work through one root object:

1. `world`

This keeps the API discoverable and prevents uncontrolled growth of flat globals.

### 2.2 Domain-first layout

`world` is split into small domains, each with one clear responsibility.

### 2.3 Handles instead of raw service access

Scripts should operate on lightweight references such as:

1. `EntityRef`
2. `SceneRef`
3. `AssetRef`

These are scripting handles, not direct engine pointers.

### 2.4 Query and command separation

A script API should clearly distinguish:

1. query operations
2. mutation/command operations

Examples:

1. `world.scene.current_id()` is a query
2. `world.scene.select("foo")` is a command

### 2.5 Runtime-safe facade

The `Rhai` layer should never expose:

1. `wgpu`
2. `winit`
3. mutexes
4. raw runtime services
5. crate internals

It only exposes stable engine-facing contracts.

### 2.6 Mod-first scene flow

Scenes belong to mods.
Scene switching is an engine feature and can be triggered by:

1. YAML scene transitions
2. `Rhai` script calls

Both paths must converge to the same runtime command model.

---

## 3. Root API Shape

The root scripting object should be:

```rhai
world
```

Initial top-level domains:

1. `world.scene`
2. `world.entities`
3. `world.input`
4. `world.assets`
5. `world.mod`
6. `world.time`
7. `world.dev`
8. `world.runtime`
9. `world.sprite2d`
10. `world.text2d`
11. `world.mesh3d`
12. `world.material3d`
13. `world.text3d`

This is intentionally broader than the first implementation slice.
Not every domain has to be fully implemented immediately, but this should be the target structure.

---

## 4. Domain Responsibilities

### 4.1 `world.scene`

Responsibility:

1. active scene state
2. scene transitions
3. scene-level control

Proposed API:

```rhai
world.scene.current_id() -> String
world.scene.current() -> SceneRef
world.scene.select(scene_id) -> bool
world.scene.reload() -> ()
world.scene.has(scene_id) -> bool
world.scene.available() -> Array
world.scene.transitions() -> Array
```

Notes:

1. `select(scene_id)` should route to `SceneCommand::SelectScene`
2. `reload()` should route to `SceneCommand::ReloadActiveScene`
3. `available()` should reflect scenes exposed by the active root mod

### 4.2 `world.entities`

Responsibility:

1. entity lookup
2. lightweight entity creation
3. entity existence checks

Proposed API:

```rhai
world.entities.named(name) -> EntityRef
world.entities.try_named(name) -> EntityRef
world.entities.exists(name) -> bool
world.entities.create(name) -> EntityRef
world.entities.count() -> i64
world.entities.names() -> Array
```

Notes:

1. `named(name)` can return an invalid handle if the entity does not exist yet
2. the handle should expose `exists()` so scripts do not need to compare against `null`
3. `create(name)` is the canonical Rhai entrypoint because `spawn` collides with reserved syntax in method position

### 4.3 `world.input`

Responsibility:

1. input queries only

Proposed API:

```rhai
world.input.down(key) -> bool
world.input.pressed(key) -> bool
world.input.released(key) -> bool
world.input.axis(name) -> f32
world.input.keys() -> Array
```

Notes:

1. no input mutation from scripts
2. named axes can come later

### 4.4 `world.assets`

Responsibility:

1. asset state queries
2. reload requests
3. asset filtering by mod/domain/tag

Proposed API:

```rhai
world.assets.get(key) -> AssetRef
world.assets.has(key) -> bool
world.assets.by_mod(mod_id) -> Array
world.assets.reload(key) -> ()
world.assets.pending() -> Array
world.assets.loaded() -> Array
world.assets.prepared() -> Array
world.assets.failed() -> Array
```

### 4.5 `world.mod`

Responsibility:

1. active mod metadata
2. root mod scene visibility
3. capability queries

Proposed API:

```rhai
world.mod.current_id() -> String
world.mod.scenes() -> Array
world.mod.has_scene(scene_id) -> bool
world.mod.capabilities() -> Array
world.mod.loaded() -> Array
```

### 4.6 `world.time`

Responsibility:

1. frame timing
2. runtime time view

Proposed API:

```rhai
world.time.delta() -> f32
world.time.elapsed() -> f32
world.time.frame() -> i64
```

### 4.7 `world.dev`

Responsibility:

1. script-visible diagnostics
2. script event emission
3. console/dev integration

Proposed API:

```rhai
world.dev.log(text) -> ()
world.dev.warn(text) -> ()
world.dev.event(topic) -> ()
world.dev.event(topic, payload) -> ()
world.dev.command(line) -> ()
world.dev.refresh_diagnostics(mod_id) -> bool
```

Notes:

1. `world.dev.event(...)` should route to `ScriptEventQueue`
2. `world.dev.command(...)` should route to `DevConsoleQueue`

### 4.8 `world.runtime`

Responsibility:

1. engine/runtime diagnostics
2. backend/capability visibility

Proposed API:

```rhai
world.runtime.window_backend() -> String
world.runtime.input_backend() -> String
world.runtime.render_backend() -> String
world.runtime.script_backend() -> String
world.runtime.capabilities() -> Array
world.runtime.plugins() -> Array
world.runtime.services() -> Array
world.runtime.dev_mode() -> bool
```

### 4.9 Domain-specific content APIs

These APIs should own only domain-specific behavior.
They should not absorb generic scene/entity/runtime responsibilities.

#### `world.sprite2d`

```rhai
world.sprite2d.frame(entity_name) -> i64
world.sprite2d.set_frame(entity_name, frame) -> bool
world.sprite2d.advance(entity_name, dt) -> bool
world.sprite2d.queue(entity_name, texture_key, width, height) -> bool
```

#### `world.text2d`

```rhai
world.text2d.queue(entity_name, content, font_key, width, height) -> bool
```

#### `world.mesh3d`

```rhai
world.mesh3d.queue(entity_name, mesh_key) -> bool
```

#### `world.material3d`

```rhai
world.material3d.bind(entity_name, label, material_key) -> ()
```

#### `world.text3d`

```rhai
world.text3d.queue(entity_name, content, font_key, size) -> bool
```

---

## 5. Handle Types

The API should expose handle objects rather than raw strings everywhere.

### 5.1 `EntityRef`

Purpose:

1. represent a scene entity by stable script-facing identity
2. centralize entity-oriented methods

Proposed API:

```rhai
let entity = world.entities.named("playground-2d-square");

entity.name() -> String
entity.exists() -> bool
entity.rotate_2d(delta) -> bool
entity.rotate_3d(x, y, z) -> bool
entity.translate_2d(x, y) -> bool
entity.translate_3d(x, y, z) -> bool
entity.transform() -> Map
```

Notes:

1. `EntityRef` should store only a stable lookup key such as entity name or id
2. methods resolve through `SceneService` when called
3. an invalid `EntityRef` must fail safely

### 5.2 `SceneRef`

Purpose:

1. represent a scene identity
2. surface scene-specific metadata and commands

Proposed API:

```rhai
let scene = world.scene.current();

scene.id() -> String
scene.select() -> bool
scene.reload() -> ()
scene.transitions() -> Array
```

### 5.3 `AssetRef`

Purpose:

1. represent an asset key
2. group asset queries

Proposed API:

```rhai
let asset = world.assets.get("playground-2d/images/square");

asset.key() -> String
asset.exists() -> bool
asset.state() -> String
asset.kind() -> String
asset.label() -> String
asset.path() -> String
asset.reload() -> ()
```

---

## 6. Lifecycle Model

Top-level execution should become more structured than a single `update(dt)`.

Recommended lifecycle:

```rhai
fn on_enter() {}
fn update(dt) {}
fn on_event(topic, payload) {}
fn on_exit() {}
```

Rules:

1. top-level script body is for declarations only
2. `on_enter()` runs once when the mod/scene script becomes active
3. `update(dt)` runs every tick
4. `on_event(...)` is the engine-level event hook for script-visible events
5. `on_exit()` runs before the scene is replaced

This keeps scripts deterministic and avoids overloading the top-level body.

---

## 7. Scene Transition Model

Scene transitions should be a first-class engine concept.

There must be two entry paths:

### 7.1 YAML transitions

Example:

```yaml
transitions:
  - id: cutscene-end
    to: main
    when:
      kind: script_event
      topic: cutscene.finished
```

Supported direction:

1. `after_seconds`
2. `script_event`

Planned direction:

1. `on_animation_end`
2. `on_entity_missing`
3. `on_condition`

### 7.2 Script-driven transitions

Example:

```rhai
if world.input.pressed("Enter") {
    world.scene.select("main");
}
```

Both YAML and script-driven transitions must converge to:

1. `SceneCommand::SelectScene`

This is the key architectural rule.

---

## 8. Commands vs Queries

To keep the API coherent:

1. queries should return data only
2. commands should route through engine queues or safe services
3. domains should not mix unrelated responsibilities

Examples:

Good:

1. `world.assets.get(key).state()`
2. `world.scene.select("foo")`
3. `entity.rotate_2d(delta)`

Bad:

1. `world.rotate_playground_square_only(...)`
2. `world.wgpu.draw_sprite(...)`
3. `world.scene_and_assets_reload_everything()`

---

## 9. API Policy

Amigo exposes one scripting style only:

1. `world.*`
2. lightweight handles such as `EntityRef` and `AssetRef`
3. lifecycle hooks such as `on_enter`, `update`, `on_event`, `on_exit`

The engine should not expose:

1. flat global helper functions
2. compatibility aliases for old bindings
3. backend-specific script entrypoints

That keeps the scripting surface small, predictable, and easy to evolve without duplicated paths.

---

## 10. Rust-side Implementation Structure

The Rust implementation should also follow SRP.

Target structure:

```text
crates/scripting/api/src/
  lib.rs
  facade/
    world.rs
    scene.rs
    entities.rs
    input.rs
    assets.rs
    mod_api.rs
    time.rs
    debug.rs
    runtime.rs
    sprite2d.rs
    text2d.rs
    mesh3d.rs
    material3d.rs
    text3d.rs
  value/
    entity_ref.rs
    scene_ref.rs
    asset_ref.rs
  lifecycle/
    hooks.rs
```

And in the backend:

```text
crates/scripting/rhai/src/
  lib.rs
  bindings/
    world_root.rs
    scene.rs
    entities.rs
    input.rs
    assets.rs
    mod_api.rs
    time.rs
    debug.rs
    runtime.rs
    sprite2d.rs
    text2d.rs
    mesh3d.rs
    material3d.rs
    text3d.rs
  handles/
    entity_ref.rs
    scene_ref.rs
    asset_ref.rs
```

Meaning:

1. `scripting/api` owns the conceptual contract
2. `scripting/rhai` owns binding and registration details
3. each binding module registers one domain only
4. there is no parallel flat binding layer

---

## 11. Immediate Implementation Slice

The recommended implementation order is still incremental:

1. add `world`
2. add `world.scene`
3. add `world.entities`
4. add `world.input`
5. add `world.dev`
6. add `EntityRef`
7. add remaining content/runtime domains on top of the same root model

The important constraint is that every new feature lands directly on the domain API, not through a temporary global helper.

---

## 12. Short Version

The correct long-term shape is:

1. `world` as the single root
2. domain objects under `world`
3. lightweight handles such as `EntityRef`
4. scene transitions as an engine feature
5. commands routed through engine queues
6. queries exposed as safe snapshots
7. no flat helpers, no duplicate binding path

That is the intended scripting foundation for Amigo.
