# Migration Status

Aktualny snapshot postępu migracji "thin app host".

## Checklist

- [ ] Rozdział 1 — registry/trait cleanup w app  
  Status: częściowo.  
  Blocker: niezgodność HRTB (`for<'a>`) między app registry objects a helperami `register_*` w `amigo-session`.  
  Obecnie lokalne helpery rejestracji w app zostały utrzymane, adaptery traitów usunięte.

- [x] Rozdział 2 — render extractor migration  
  Status: zakończony.  
  Wykonane:
  - domain extraction helpers dodane dla: `2d/composition`, `2d/lighting`, `2d/particles`, `2d/post-fx`, `3d/mesh`, `3d/material`, `3d/text`
  - app extractors przepięte na helpery domenowe
  - direct extraction calls z app (`context.*.commands()/draw_commands()/scene_stack()`) usunięte dla domenowych extractorów
  Verify:
  - `cargo check -p amigo-2d-composition` ✅
  - `cargo check -p amigo-2d-lighting` ✅
  - `cargo check -p amigo-2d-particles` ✅
  - `cargo check -p amigo-2d-post-fx` ✅
  - `cargo check -p amigo-3d-mesh` ✅
  - `cargo check -p amigo-3d-material` ✅
  - `cargo check -p amigo-3d-text` ✅
  - `cargo check -p amigo-app` ✅

- [ ] Rozdział 3 — systems migration  
  Status: w toku.
  - [x] 3.1 przeniesienie `scheduling` do `amigo-session`
  - [x] 3.2 przeniesienie `particles_2d` system do `amigo-2d-particles`
  - [x] 3.3 behavior
  - [~] 3.4 script systems (częściowo: `script_components` przeniesione, `script_update` blocker)
  - [ ] 3.5 UI systems
  - [ ] 3.6 audio system
  Verify (zrealizowane dla 3.1/3.2):
  - `cargo check -p amigo-session` ✅
  - `cargo check -p amigo-2d-particles` ✅
  - `cargo check -p amigo-app` ✅

- [~] Rozdział 4 — script runtime migration  
  Status: w toku.
  - [x] 4.1 asset script command (domain ownership w `amigo-assets`)
  - [x] 4.2 audio script command (domain ownership w `amigo-audio-api`, app tylko host glue)
  - [x] 4.3 ui script command (domain ownership w `amigo-ui`)
  - [~] 4.4 render script command (częściowo: `2d.layered_image`, `2d.light`, `2d.light_group`, `2d.render_layer`)
- [~] Rozdział 5 — dev-console ownership  
  Status: w toku.
  - [x] domain execution dla `composition/layered-image/lighting/particles/post-fx` przeniesione do domen (`src/dev_console.rs`)
  - [x] app command handlers zamienione na adaptery
  - [x] rejestracja adapterów przywrócona w app shell
  - [x] domenowe `RuntimeCapabilityKind::DevConsoleCommand` descriptors obecne dla ww. domen
  - [ ] do domknięcia: końcowe ownership grepy + ewentualne porządki descriptorów host-only w app
- [x] Rozdział 6 — diagnostics/metadata ownership  
  Status: zakończony.
  Wykonane:
  - app diagnostics pozostawione host-only (`runtime.diagnostics.overview`, `runtime.metadata.overview`)
  - domain diagnostics/metadata descriptors dodane w:
    - `2d/particles`
    - `2d/post-fx`
    - `2d/lighting`
    - `2d/composition`
    - `audio/mixer`
    - `ui/core`
    - `scripting/rhai`
  Verify:
  - `cargo check -p amigo-session` ✅
  - `cargo check -p amigo-app` ✅
- [~] Rozdział 7 — final cleanup + docs + global greps + końcowe verify  
  Status: w toku.
  - [x] docs boundary section istnieje w `crates/apps/app/README.md`
  - [x] typed registries section istnieje w `crates/engine/session/README.md`
  - [x] `AGENTS.md` zawiera regułę `app.host` only-for-host
  - [~] global greps uruchomione częściowo (patrz bieżący raport)
  - [ ] pełna końcowa lista targeted verify wg planu

## Ostatnie wykonane zmiany

1. `scheduling`:
- dodano `crates/engine/session/src/scheduling.rs`
- eksport `pub mod scheduling; pub use scheduling::*;` w `crates/engine/session/src/lib.rs`
- `crates/apps/app/src/scheduling.rs` zmienione na re-export z `amigo_session`
- użycia `AppSchedulingService` przepięte na `amigo_session::AppSchedulingService` w wskazanych plikach app

2. `particles_2d` system:
- dodano `crates/2d/particles/src/systems.rs`
- eksport w `crates/2d/particles/src/lib.rs`
- rejestracja systemu w app przepięta na `amigo_2d_particles::tick_particles_2d_world`
- usunięto `crates/apps/app/src/systems/particles_2d.rs`
- dodano zależność `amigo-2d-motion` w `crates/2d/particles/Cargo.toml`

3. `behavior` system:
- dodano `crates/engine/behavior/src/systems.rs`
- skopiowano moduły behavior (`actions/menu/particle_profile/tick/tests`) do `crates/engine/behavior/src/systems/`
- dodano wymagane zależności domenowe do `crates/engine/behavior/Cargo.toml`
- eksport `mod systems; pub use systems::*;` w `crates/engine/behavior/src/lib.rs`
- app `systems/mod.rs` przepięty na `amigo_behavior::tick_behaviors`
- usunięto `pub(crate) mod behavior;` z app systems registry

4. `script systems` (częściowo):
- dodano `crates/scripting/rhai/src/systems.rs` z `tick_script_components`
- eksport `mod systems; pub use systems::*;` w `crates/scripting/rhai/src/lib.rs`
- app `systems/mod.rs` przepięty na `amigo_scripting_rhai::tick_script_components`
- usunięto `pub(crate) mod script_components;` i plik `crates/apps/app/src/systems/script_components.rs`

Blocker 3.4 (`script_update`):
- `tick_active_scripts` w obecnym kształcie zależy od app-owned `crate::scripting_runtime::current_executed_scripts` i `crate::ScriptExecutionRole`.
- Do pełnej migracji potrzeba przenieść execution list API/role model do reusable crate (najlepiej `amigo-session` albo `amigo-scripting-api`) i dopiero wtedy przepiąć `script_update` do `amigo-rhai`.

## Najbliższe kroki

1. Domknąć 3.3 behavior (przeniesienie execution do `amigo-behavior` i przepięcie app registry call-site).
2. Domknąć 3.4 script systems; jeśli `current_executed_scripts` pozostaje app-owned, dodać explicit blocker seam.
3. Kontynuować 3.5 i 3.6 z verify po każdym podrozdziale.


Poniżej plan od aktualnego snapshotu `902734...`.
Nie rób wszystkiego jednym strzałem.
Rób rozdziałami.
Po każdym rozdziale targeted verify.

ROZDZIAŁ 1 — domknij registry/trait cleanup w app

Cel:
trait/registry primitives są w `amigo-session`.
App nie powinno mieć lokalnych helperów rejestrujących, jeśli może użyć generic helperów z session.

1.1 MODIFY:
`crates/apps/app/src/scene_runtime/dispatcher.rs`

Usuń import:

```rust
use std::sync::Arc;
```

Zmień import:

```rust
use amigo_session::SceneCommandHandler;
```

Na:

```rust
use amigo_session::{SceneCommandHandler, register_scene_command_handler};
```

Usuń funkcję z linii 15-23:

```rust
pub(crate) fn register_scene_command_handler<H>(
    registry: &mut SceneCommandHandlerRegistry,
    handler: H,
) where
    H: for<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
        + 'static,
{
    registry.register_arc(Arc::new(handler));
}
```

Zostaw typ:

```rust
pub(crate) type SceneCommandHandlerObject =
    dyn for<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>;

pub(crate) type SceneCommandHandlerRegistry =
    HandlerRegistry<SceneCommandHandlerObject>;
```

1.2 MODIFY:
`crates/apps/app/src/scene_runtime/handlers/mod.rs`

Zmień import z:

```rust
use super::dispatcher::{SceneCommandHandlerRegistry, register_scene_command_handler};
```

Na:

```rust
use super::dispatcher::SceneCommandHandlerRegistry;
use amigo_session::register_scene_command_handler;
```

1.3 MODIFY:
`crates/apps/app/src/script_runtime/mod.rs`

Usuń import:

```rust
use std::sync::Arc;
```

Dodaj do importu `amigo_session`:

```rust
register_script_command_handler,
```

Czyli sekcja ma mieć:

```rust
use amigo_session::{
    runtime_capabilities::{
        RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeCapability,
        RuntimeDomainId, ScriptCommandHandlerContribution, ScriptCommandHandlerDescriptor,
        ScriptCommandProvider, APP_HOST_DOMAIN_ID,
    },
    ScriptCommandHandler,
    RuntimeSession,
    register_script_command_handler,
};
```

Usuń funkcję z linii 40-48:

```rust
pub(super) fn register_script_command_handler<H>(
    registry: &mut ScriptCommandHandlerRegistry,
    handler: H,
) where
    H: for<'a> ScriptCommandHandler<AppScriptCommandContext<'a>, ScriptCommand, ()>
        + 'static,
{
    registry.register_arc(Arc::new(handler));
}
```

1.4 MODIFY:
`crates/apps/app/src/script_runtime/handlers/mod.rs`

Zmień import z:

```rust
use super::{ScriptCommandHandlerRegistry, register_script_command_handler};
```

Na:

```rust
use super::ScriptCommandHandlerRegistry;
use amigo_session::register_script_command_handler;
```

VERIFY:

```powershell
cargo check -p amigo-session
cargo check -p amigo-app
```

GREP:

```powershell
git grep -n "fn register_scene_command_handler\|fn register_script_command_handler\|SceneCommandHandlerAdapter\|ScriptCommandHandlerAdapter" crates/apps/app/src
```

Oczekiwane:
zero lokalnych adapterów i lokalnych register helperów w app.

ROZDZIAŁ 2 — domknij render extractor migration

Cel:
app zostaje render orchestration + packet + WGPU presentation.
Domeny dostarczają extraction helpers.

Już zrobione:
`vector`
`text`
`sprite`
`tilemap`
`layered-image`

Zostało:
`composition`
`lighting`
`particles`
`post-fx`
`3d/material`
`3d/mesh`
`3d/text`

2.1 ADD:
`crates/2d/composition/src/render_extraction.rs`

Zawartość:

```rust
use crate::{
    LightRoute2dCommand, LightRoute2dSceneService, RenderLayer2dCommand,
    RenderLayer2dSceneService,
};

pub struct Composition2dRenderExtractionContext<'a> {
    pub render_layer2d_scene_service: &'a RenderLayer2dSceneService,
    pub light_route2d_scene_service: &'a LightRoute2dSceneService,
}

#[derive(Debug, Default, Clone)]
pub struct Composition2dRenderCommands {
    pub render_layers: Vec<RenderLayer2dCommand>,
    pub light_routes: Vec<LightRoute2dCommand>,
}

pub fn extract_composition2d_render_commands(
    ctx: Composition2dRenderExtractionContext<'_>,
) -> Composition2dRenderCommands {
    Composition2dRenderCommands {
        render_layers: ctx.render_layer2d_scene_service.commands(),
        light_routes: ctx.light_route2d_scene_service.commands(),
    }
}
```

2.2 MODIFY:
`crates/2d/composition/src/lib.rs`

Zmień linie 1-6 na:

```rust
mod model;
mod plugin;
mod render_extraction;
mod runtime_capabilities;
mod scene_command;
mod scene_bridge;
mod service;
```

Dodaj export po `pub use plugin::*;`:

```rust
pub use render_extraction::*;
```

2.3 ADD:
`crates/2d/lighting/src/render_extraction.rs`

Zawartość:

```rust
use crate::{
    GlobalLight2dCommand, GlobalLight2dSceneService, LightGroup2dCommand,
    LightGroup2dSceneService, LightMap2dSceneService, LightMap2dSourceCommand,
};

pub struct Lighting2dRenderExtractionContext<'a> {
    pub global_light2d_scene_service: &'a GlobalLight2dSceneService,
    pub lightmap2d_scene_service: &'a LightMap2dSceneService,
    pub light_group2d_scene_service: &'a LightGroup2dSceneService,
}

#[derive(Debug, Default, Clone)]
pub struct Lighting2dRenderCommands {
    pub global_lights: Vec<GlobalLight2dCommand>,
    pub lightmaps: Vec<LightMap2dSourceCommand>,
    pub light_groups: Vec<LightGroup2dCommand>,
}

pub fn extract_lighting2d_render_commands(
    ctx: Lighting2dRenderExtractionContext<'_>,
) -> Lighting2dRenderCommands {
    Lighting2dRenderCommands {
        global_lights: ctx.global_light2d_scene_service.commands(),
        lightmaps: ctx.lightmap2d_scene_service.commands(),
        light_groups: ctx.light_group2d_scene_service.commands(),
    }
}
```

2.4 MODIFY:
`crates/2d/lighting/src/lib.rs`

Dodaj:

```rust
mod render_extraction;
pub use render_extraction::*;
```

2.5 ADD:
`crates/2d/particles/src/render_extraction.rs`

Zawartość:

```rust
use crate::{Particle2dDrawCommand, Particle2dSceneService};

pub struct Particle2dRenderExtractionContext<'a> {
    pub particle2d_scene_service: &'a Particle2dSceneService,
}

pub fn extract_particle2d_render_commands(
    ctx: Particle2dRenderExtractionContext<'_>,
) -> Vec<Particle2dDrawCommand> {
    ctx.particle2d_scene_service.draw_commands()
}
```

2.6 MODIFY:
`crates/2d/particles/src/lib.rs`

Dodaj:

```rust
mod render_extraction;
pub use render_extraction::*;
```

2.7 ADD:
`crates/2d/post-fx/src/render_extraction.rs`

Zawartość:

```rust
use crate::{PostFx2dService, PostFx2dStack};

pub struct PostFx2dRenderExtractionContext<'a> {
    pub post_fx_service: &'a PostFx2dService,
}

pub fn extract_post_fx2d_render_stack(
    ctx: PostFx2dRenderExtractionContext<'_>,
) -> Option<PostFx2dStack> {
    let stack = ctx.post_fx_service.scene_stack().normalized();
    (!stack.is_empty()).then_some(stack)
}
```

2.8 MODIFY:
`crates/2d/post-fx/src/lib.rs`

Dodaj:

```rust
mod render_extraction;
pub use render_extraction::*;
```

2.9 ADD:
`crates/3d/mesh/src/render_extraction.rs`

Zawartość:

```rust
use amigo_scene::SceneService;

use crate::{MeshDrawCommand, MeshSceneService};

pub struct Mesh3dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub mesh_scene_service: &'a MeshSceneService,
}

pub fn extract_mesh3d_render_commands(
    ctx: Mesh3dRenderExtractionContext<'_>,
) -> Vec<MeshDrawCommand> {
    ctx.mesh_scene_service
        .commands()
        .into_iter()
        .filter(|command| is_entity_render_visible(ctx.scene_service, &command.entity_name))
        .collect()
}

fn is_entity_render_visible(scene_service: &SceneService, entity_name: &str) -> bool {
    scene_service
        .entity_by_name(entity_name)
        .map(|entity| entity.lifecycle.visible)
        .unwrap_or(true)
}
```

2.10 MODIFY:
`crates/3d/mesh/src/lib.rs`

Dodaj:

```rust
mod render_extraction;
pub use render_extraction::*;
```

2.11 ADD:
`crates/3d/material/src/render_extraction.rs`

Zawartość:

```rust
use amigo_scene::SceneService;

use crate::{MaterialDrawCommand, MaterialSceneService};

pub struct Material3dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub material_scene_service: &'a MaterialSceneService,
}

pub fn extract_material3d_render_commands(
    ctx: Material3dRenderExtractionContext<'_>,
) -> Vec<MaterialDrawCommand> {
    ctx.material_scene_service
        .commands()
        .into_iter()
        .filter(|command| is_entity_render_visible(ctx.scene_service, &command.entity_name))
        .collect()
}

fn is_entity_render_visible(scene_service: &SceneService, entity_name: &str) -> bool {
    scene_service
        .entity_by_name(entity_name)
        .map(|entity| entity.lifecycle.visible)
        .unwrap_or(true)
}
```

2.12 MODIFY:
`crates/3d/material/src/lib.rs`

Dodaj:

```rust
mod render_extraction;
pub use render_extraction::*;
```

2.13 ADD:
`crates/3d/text/src/render_extraction.rs`

Zawartość:

```rust
use amigo_scene::SceneService;

use crate::{Text3dDrawCommand, Text3dSceneService};

pub struct Text3dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub text3d_scene_service: &'a Text3dSceneService,
}

pub fn extract_text3d_render_commands(
    ctx: Text3dRenderExtractionContext<'_>,
) -> Vec<Text3dDrawCommand> {
    ctx.text3d_scene_service
        .commands()
        .into_iter()
        .filter(|command| is_entity_render_visible(ctx.scene_service, &command.entity_name))
        .collect()
}

fn is_entity_render_visible(scene_service: &SceneService, entity_name: &str) -> bool {
    scene_service
        .entity_by_name(entity_name)
        .map(|entity| entity.lifecycle.visible)
        .unwrap_or(true)
}
```

2.14 MODIFY:
`crates/3d/text/src/lib.rs`

Dodaj:

```rust
mod render_extraction;
pub use render_extraction::*;
```

2.15 MODIFY:
`crates/apps/app/src/render_runtime/extractors.rs`

Zastąp body `ResolvedComposition2dExtractor::extract`, linie 177-185:

```rust
fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
    let commands = amigo_2d_composition::extract_composition2d_render_commands(
        amigo_2d_composition::Composition2dRenderExtractionContext {
            render_layer2d_scene_service: context.render_layer2d_scene_service,
            light_route2d_scene_service: context.light_route2d_scene_service,
        },
    );

    for command in commands.render_layers {
        packet.push_world_2d_render_layer(command);
    }

    for command in commands.light_routes {
        packet.push_world_2d_light_route(command);
    }
}
```

Zastąp body `ResolvedLighting2dExtractor::extract`, linie 195-207:

```rust
fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
    let commands = amigo_2d_lighting::extract_lighting2d_render_commands(
        amigo_2d_lighting::Lighting2dRenderExtractionContext {
            global_light2d_scene_service: context.global_light2d_scene_service,
            lightmap2d_scene_service: context.lightmap2d_scene_service,
            light_group2d_scene_service: context.light_group2d_scene_service,
        },
    );

    for command in commands.global_lights {
        packet.push_world_2d_global_light(command);
    }

    for command in commands.lightmaps {
        packet.push_world_2d_lightmap(command);
    }

    for command in commands.light_groups {
        packet.push_world_2d_light_group(command);
    }
}
```

Zastąp body `ResolvedMesh3dExtractor::extract`, linie 255-266:

```rust
fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
    for command in amigo_3d_mesh::extract_mesh3d_render_commands(
        amigo_3d_mesh::Mesh3dRenderExtractionContext {
            scene_service: context.scene_service,
            mesh_scene_service: context.mesh_scene_service,
        },
    ) {
        packet.push_world_3d_mesh(command);
    }
}
```

Zastąp body `ResolvedMaterial3dExtractor::extract`, linie 276-287:

```rust
fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
    for command in amigo_3d_material::extract_material3d_render_commands(
        amigo_3d_material::Material3dRenderExtractionContext {
            scene_service: context.scene_service,
            material_scene_service: context.material_scene_service,
        },
    ) {
        packet.push_world_3d_material(command);
    }
}
```

Zastąp body `ResolvedText3dExtractor::extract`, linie 297-308:

```rust
fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
    for command in amigo_3d_text::extract_text3d_render_commands(
        amigo_3d_text::Text3dRenderExtractionContext {
            scene_service: context.scene_service,
            text3d_scene_service: context.text3d_scene_service,
        },
    ) {
        packet.push_world_3d_text(command);
    }
}
```

Zastąp body `ResolvedParticle2dExtractor::extract`, linie 318-321:

```rust
fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
    for command in amigo_2d_particles::extract_particle2d_render_commands(
        amigo_2d_particles::Particle2dRenderExtractionContext {
            particle2d_scene_service: context.particle2d_scene_service,
        },
    ) {
        packet.push_world_2d_particle(command);
    }
}
```

Zastąp body `ResolvedPostFx2dExtractor::extract`, linie 332-336:

```rust
fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
    if let Some(stack) = amigo_2d_post_fx::extract_post_fx2d_render_stack(
        amigo_2d_post_fx::PostFx2dRenderExtractionContext {
            post_fx_service: context.post_fx_service,
        },
    ) {
        packet.set_post_fx_stack(stack);
    }
}
```

VERIFY:

```powershell
cargo check -p amigo-2d-composition
cargo check -p amigo-2d-lighting
cargo check -p amigo-2d-particles
cargo check -p amigo-2d-post-fx
cargo check -p amigo-3d-mesh
cargo check -p amigo-3d-material
cargo check -p amigo-3d-text
cargo check -p amigo-app
```

GREP:

```powershell
git grep -n "context\\..*_scene_service\\.commands()\\|context\\.particle2d_scene_service\\.draw_commands()\\|context\\.post_fx_service\\.scene_stack()" crates/apps/app/src/render_runtime/extractors.rs
```

Oczekiwane:
zero dla domenowych extractorów.
Host overlay extractors mogą zostać.

ROZDZIAŁ 3 — systems migration

Cel:
domain systems do domen.
App zostawia tylko host systems i registry wiring.

Aktualnie już zrobione:
`2d/motion`
`2d/physics`
`engine/scene`

Zostało w app:
`particles_2d`
`audio`
`behavior`
`script_components`
`script_update`
`ui_bindings`
`ui_input`
`scene_transition`

3.1 Najpierw przenieś scheduling service z app do session, bo blokuje `particles_2d`.

ADD:
`crates/engine/session/src/scheduling.rs`

Skopiuj całą zawartość:
`crates/apps/app/src/scheduling.rs`

W nowym pliku zmień widoczności:
`pub(crate)` -> `pub`

Czyli typy mają być publiczne:

```rust
pub struct ResolvedSchedulingOverride
pub struct SchedulingOverrideReport
pub struct ResolvedSchedulingConfig
pub struct SchedulingFrameStats
pub struct AppSchedulingService
```

Metody w `impl AppSchedulingService` też mają być `pub`.

MODIFY:
`crates/engine/session/src/lib.rs`

Dodaj:

```rust
pub mod scheduling;
pub use scheduling::*;
```

MODIFY:
`crates/apps/app/src/scheduling.rs`

Zastąp cały plik:

```rust
pub(crate) use amigo_session::{
    AppSchedulingService, ResolvedSchedulingConfig, ResolvedSchedulingOverride,
    SchedulingFrameStats, SchedulingOverrideReport,
};
```

MODIFY:
`crates/apps/app/src/systems/mod.rs`

Zmień linię 148:

```rust
registry.register(crate::scheduling::AppSchedulingService::default())?;
```

Na:

```rust
registry.register(amigo_session::AppSchedulingService::default())?;
```

MODIFY:
wszystkie użycia:

```rust
crate::scheduling::AppSchedulingService
crate::scheduling::SchedulingOverrideReport
```

Na:

```rust
amigo_session::AppSchedulingService
amigo_session::SchedulingOverrideReport
```

Pliki:

```text
crates/apps/app/src/dev_console/commands/particles.rs
crates/apps/app/src/dev_console/commands/scheduler.rs
crates/apps/app/src/render_runtime.rs
crates/apps/app/src/scene_runtime/mod.rs
crates/apps/app/src/systems/particles_2d.rs
```

VERIFY:

```powershell
cargo check -p amigo-session
cargo check -p amigo-app
```

3.2 Przenieś particles system.

ADD:
`crates/2d/particles/src/systems.rs`

Skopiuj całą zawartość:
`crates/apps/app/src/systems/particles_2d.rs`

W nowym pliku zmień importy na początku.

Było:

```rust
use super::super::*;
use crate::runtime_context::RuntimeContext;
use crate::scheduling::AppSchedulingService;
use amigo_2d_particles::Particle2dEmitterRuntimeInput;
```

Ma być:

```rust
use amigo_core::{AmigoError, AmigoResult};
use amigo_math::{Transform2, Vec2};
use amigo_runtime::{
    EngineJob, EngineLane, EngineSchedulerMode, EngineSchedulingConfig, EngineTaskSystem,
    JobContext, Runtime,
};
use amigo_scene::SceneService;
use amigo_2d_motion::Motion2dSceneService;
use amigo_session::{AppSchedulingService, SchedulingOverrideReport};

use crate::{Particle2dEmitterRuntimeInput, Particle2dSceneService};
```

Dodaj helper po importach:

```rust
fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> AmigoResult<std::sync::Arc<T>> {
    runtime.resolve::<T>().ok_or_else(|| {
        AmigoError::Message(format!(
            "required service `{}` is not registered",
            std::any::type_name::<T>()
        ))
    })
}
```

W funkcji `tick_particles_2d_world` zamień:

```rust
let ctx = RuntimeContext::new(runtime);
let scene_service = ctx.required::<SceneService>()?;
let motion_scene_service = ctx.required::<Motion2dSceneService>()?;
let particle_scene_service = ctx.required::<Particle2dSceneService>()?;
let scheduling = ctx.required::<AppSchedulingService>()?;
let task_system = ctx.required::<EngineTaskSystem>()?;
```

Na:

```rust
let scene_service = required::<SceneService>(runtime)?;
let motion_scene_service = required::<Motion2dSceneService>(runtime)?;
let particle_scene_service = required::<Particle2dSceneService>(runtime)?;
let scheduling = required::<AppSchedulingService>(runtime)?;
let task_system = required::<EngineTaskSystem>(runtime)?;
```

Zamień:

```rust
crate::scheduling::SchedulingOverrideReport
```

Na:

```rust
SchedulingOverrideReport
```

MODIFY:
`crates/2d/particles/src/lib.rs`

Dodaj:

```rust
mod systems;
pub use systems::*;
```

MODIFY:
`crates/apps/app/src/systems/mod.rs`

Zmień linię 224:

```rust
move |runtime| particles_2d::tick_particles_2d_world(runtime, HOST_DELTA_SECONDS),
```

Na:

```rust
move |runtime| amigo_2d_particles::tick_particles_2d_world(runtime, HOST_DELTA_SECONDS),
```

MODIFY:
`crates/apps/app/src/systems/mod.rs`

Usuń linię modułu:

```rust
pub(crate) mod particles_2d;
```

VERIFY:

```powershell
cargo check -p amigo-session
cargo check -p amigo-2d-particles
cargo check -p amigo-app
```

3.3 Przenieś behavior system.

ADD:
`crates/engine/behavior/src/systems.rs`

Przenieś logicznie z:

```text
crates/apps/app/src/systems/behavior.rs
crates/apps/app/src/systems/behavior/*
```

Najbezpieczniej:
skopiuj katalog:

```text
crates/apps/app/src/systems/behavior/
```

Do:

```text
crates/engine/behavior/src/systems/
```

I utwórz:
`crates/engine/behavior/src/systems.rs`

Zawartość:

```rust
mod actions;
mod menu;
mod particle_profile;
mod tick;

pub use tick::*;
```

W przeniesionych plikach usuń `crate::` zależności app.
Jeśli plik używa `crate::particle_presets`, przenieś potrzebną funkcję do `crates/2d/particles` albo zostaw mały adapter w app i zanotuj blocker.
Nie oznaczaj tego jako app.host.

MODIFY:
`crates/engine/behavior/src/lib.rs`

Dodaj:

```rust
mod systems;
pub use systems::*;
```

MODIFY:
`crates/apps/app/src/systems/mod.rs`

Zmień linię 180:

```rust
behavior::tick_behaviors(runtime, HOST_DELTA_SECONDS)
```

Na:

```rust
amigo_engine_behavior::tick_behaviors(runtime, HOST_DELTA_SECONDS)
```

Usuń moduł:

```rust
pub(crate) mod behavior;
```

VERIFY:

```powershell
cargo check -p amigo-engine-behavior
cargo check -p amigo-app
```

3.4 Przenieś script systems.

Target:
`crates/scripting/rhai/src/systems.rs`

ADD:
`crates/scripting/rhai/src/systems.rs`

Przenieś funkcje z:

```text
crates/apps/app/src/systems/script_components.rs
crates/apps/app/src/systems/script_update.rs
```

Eksport:
`crates/scripting/rhai/src/lib.rs`

```rust
mod systems;
pub use systems::*;
```

MODIFY:
`crates/apps/app/src/systems/mod.rs`

Zmień:

```rust
script_components::tick_script_components(runtime, HOST_DELTA_SECONDS)
script_update::tick_active_scripts(runtime, HOST_DELTA_SECONDS)
```

Na:

```rust
amigo_rhai::tick_script_components(runtime, HOST_DELTA_SECONDS)
amigo_rhai::tick_active_scripts(runtime, HOST_DELTA_SECONDS)
```

Usuń moduły:

```rust
pub(crate) mod script_components;
pub(crate) mod script_update;
```

VERIFY:

```powershell
cargo check -p amigo-rhai
cargo check -p amigo-app
```

3.5 Przenieś UI systems.

Target:
`crates/ui/core/src/systems.rs`

ADD:
`crates/ui/core/src/systems.rs`

Przenieś:

```text
crates/apps/app/src/systems/ui_bindings.rs
```

Dla `ui_input.rs`:
jeśli używa `UiInputViewportState` z app, zostaw viewport state w app, ale przenieś czystą logikę do `amigo_ui`.

Dodaj w `amigo_ui`:

```rust
pub struct UiInputSystemContext<'a> {
    pub viewport: Option<amigo_render_wgpu::UiViewportSize>,
    pub ui_input: &'a UiInputService,
    pub ui_scene: &'a UiSceneService,
    pub ui_state: &'a UiStateService,
    pub ui_theme: &'a UiThemeService,
    pub script_event_queue: &'a amigo_scripting_api::ScriptEventQueue,
}

pub fn process_ui_input_with_context(ctx: UiInputSystemContext<'_>) -> amigo_core::AmigoResult<()> {
    // przenieś body z app ui_input.rs od linii 11 dalej,
    // ale zamiast RuntimeContext używaj pól z ctx
}
```

App `ui_input.rs` zostaje cienkim adapterem:

```rust
pub(crate) fn process_ui_input(runtime: &Runtime) -> AmigoResult<()> {
    let viewport = RuntimeContext::new(runtime)
        .required::<super::UiInputViewportState>()?
        .get();

    let ctx = RuntimeContext::new(runtime);
    amigo_ui::process_ui_input_with_context(amigo_ui::UiInputSystemContext {
        viewport,
        ui_input: ctx.required::<UiInputService>()?.as_ref(),
        ui_scene: ctx.required::<UiSceneService>()?.as_ref(),
        ui_state: ctx.required::<UiStateService>()?.as_ref(),
        ui_theme: ctx.required::<UiThemeService>()?.as_ref(),
        script_event_queue: ctx.required::<ScriptEventQueue>()?.as_ref(),
    })
}
```

VERIFY:

```powershell
cargo check -p amigo-ui-core
cargo check -p amigo-app
```

3.6 Audio system.

Target:
`crates/audio/mixer/src/systems.rs`

Przenieś:

```text
crates/apps/app/src/systems/audio.rs
```

Eksport:
`crates/audio/mixer/src/lib.rs`

```rust
mod systems;
pub use systems::*;
```

MODIFY:
`crates/apps/app/src/systems/mod.rs`

Zmień linię 273:

```rust
audio::tick_audio_runtime(runtime, HOST_DELTA_SECONDS)
```

Na:

```rust
amigo_audio_mixer::tick_audio_runtime(runtime, HOST_DELTA_SECONDS)
```

Usuń:

```rust
pub(crate) mod audio;
```

VERIFY:

```powershell
cargo check -p amigo-audio-mixer
cargo check -p amigo-app
```

FINAL GREP SYSTEMS:

```powershell
git grep -n "pub(crate) mod audio\|pub(crate) mod behavior\|pub(crate) mod particles_2d\|pub(crate) mod script_components\|pub(crate) mod script_update\|pub(crate) mod ui_bindings" crates/apps/app/src/systems/mod.rs
git grep -n "tick_.*world\|tick_.*runtime\|tick_.*scripts\|tick_.*bindings" crates/apps/app/src/systems
```

Oczekiwane:
app `systems` zawiera tylko host adapters:
`ui_input` jeśli viewport bridge nadal app-owned
`scene_transition` jeśli hydration/session glue nadal app-owned

ROZDZIAŁ 4 — script runtime migration

Cel:
app nie wykonuje non-host script commands.
App zostawia dispatch, dev console output, host-only debug/dev-shell.

Zostają do migracji:
`asset`
`audio`
`render`
`ui`

Host może zostać:
`debug`
`dev_shell`

4.1 Asset script command.

ADD:
`crates/engine/assets/src/script_command.rs`

Zawartość:

```rust
use amigo_assets::{AssetCatalog, AssetKey, AssetLoadPriority};
use amigo_scripting_api::{ScriptCommand, ScriptEvent, ScriptEventQueue};

pub struct AssetScriptCommandContext<'a> {
    pub asset_catalog: &'a AssetCatalog,
    pub script_event_queue: &'a ScriptEventQueue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetScriptCommandOutcome {
    ReloadRequested { asset_key: String },
    Unhandled,
}

pub fn can_handle_asset_script_command(command: &ScriptCommand) -> bool {
    command.namespace == "asset"
}

pub fn handle_asset_script_command(
    ctx: AssetScriptCommandContext<'_>,
    command: ScriptCommand,
) -> AssetScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("reload", [asset_key]) => {
            ctx.asset_catalog
                .request_reload(AssetKey::new(asset_key.clone()), AssetLoadPriority::Immediate);
            ctx.script_event_queue.publish(ScriptEvent::new(
                "asset.reload-requested",
                vec![asset_key.clone()],
            ));
            AssetScriptCommandOutcome::ReloadRequested {
                asset_key: asset_key.clone(),
            }
        }
        _ => AssetScriptCommandOutcome::Unhandled,
    }
}
```

Jeśli `AssetCatalog::request_reload` nie istnieje:
nie przenoś orchestration do app.host.
Zamiast tego dodaj w `crates/engine/assets` minimalne public API analogiczne do `crate::orchestration::request_asset_reload`.

MODIFY:
`crates/engine/assets/src/lib.rs`

Dodaj:

```rust
mod script_command;
pub use script_command::*;
```

MODIFY:
`crates/apps/app/src/script_runtime/handlers/asset.rs`

Zastąp body `handle`, linie 18-38:

```rust
fn handle(&self, ctx: &AppScriptCommandContext<'a>, command: ScriptCommand) {
    let outcome = amigo_assets::handle_asset_script_command(
        amigo_assets::AssetScriptCommandContext {
            asset_catalog: ctx.asset_catalog,
            script_event_queue: ctx.script_event_queue,
        },
        command.clone(),
    );

    match outcome {
        amigo_assets::AssetScriptCommandOutcome::ReloadRequested { asset_key } => {
            ctx.dev_console_state
                .write_line(format!("requested asset reload `{asset_key}`"));
        }
        amigo_assets::AssetScriptCommandOutcome::Unhandled => {
            ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            ));
        }
    }
}
```

VERIFY:

```powershell
cargo check -p amigo-assets
cargo check -p amigo-app
```

4.2 Audio script command.

ADD:
`crates/audio/api/src/script_command.rs`

Zawartość:

```rust
use amigo_assets::AssetKey;
use amigo_audio_api::{
    AudioClipKey, AudioCommand, AudioCommandQueue, AudioPlaybackMode, AudioSceneService,
    AudioSourceId,
};
use amigo_scripting_api::ScriptCommand;

pub struct AudioScriptCommandContext<'a> {
    pub audio_command_queue: &'a AudioCommandQueue,
    pub audio_scene_service: &'a AudioSceneService,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioScriptCommandOutcome {
    Preloaded { asset_key: AssetKey, mode: AudioPlaybackMode },
    PlayOnce { asset_key: AssetKey },
    CueQueued { cue_name: String, clip: AudioClipKey },
    CueMissing { cue_name: String },
    CueNotReady { cue_name: String },
    SourceStarted { source: String, asset_key: AssetKey },
    SourceStopped { source: String },
    ParamSet { source: String, param: String, value: f32 },
    VolumeSet { bus: String, value: f32 },
    ParseError { message: String },
    Unhandled,
}

pub fn can_handle_audio_script_command(command: &ScriptCommand) -> bool {
    command.namespace == "audio"
}

pub fn handle_audio_script_command(
    ctx: AudioScriptCommandContext<'_>,
    command: ScriptCommand,
    resolve_asset: impl Fn(&str) -> AssetKey,
) -> AudioScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("preload", [clip_name]) => {
            let asset_key = resolve_asset(clip_name);
            AudioScriptCommandOutcome::Preloaded {
                asset_key,
                mode: AudioPlaybackMode::OneShot,
            }
        }
        ("play", [clip_name]) => {
            let asset_key = resolve_asset(clip_name);
            ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                clip: AudioClipKey::new(asset_key.as_str().to_owned()),
            });
            AudioScriptCommandOutcome::PlayOnce { asset_key }
        }
        ("play-asset", [asset_key]) => {
            let asset_key = AssetKey::new(asset_key.clone());
            ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                clip: AudioClipKey::new(asset_key.as_str().to_owned()),
            });
            AudioScriptCommandOutcome::PlayOnce { asset_key }
        }
        ("cue", [cue_name]) => {
            let Some(cue) = ctx.audio_scene_service.cue(cue_name) else {
                return AudioScriptCommandOutcome::CueMissing {
                    cue_name: cue_name.clone(),
                };
            };
            if !ctx.audio_scene_service.mark_cue_played_if_ready(&cue) {
                return AudioScriptCommandOutcome::CueNotReady {
                    cue_name: cue_name.clone(),
                };
            }
            ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                clip: cue.clip.clone(),
            });
            AudioScriptCommandOutcome::CueQueued {
                cue_name: cue.name,
                clip: cue.clip,
            }
        }
        ("start-realtime", [source]) => {
            let asset_key = resolve_asset(source);
            ctx.audio_command_queue.push(AudioCommand::StartSource {
                source: AudioSourceId::new(source.clone()),
                clip: AudioClipKey::new(asset_key.as_str().to_owned()),
            });
            AudioScriptCommandOutcome::SourceStarted {
                source: source.clone(),
                asset_key,
            }
        }
        ("stop", [source]) => {
            ctx.audio_command_queue.push(AudioCommand::StopSource {
                source: AudioSourceId::new(source.clone()),
            });
            AudioScriptCommandOutcome::SourceStopped {
                source: source.clone(),
            }
        }
        ("set-param", [source, param, value]) => match value.parse::<f32>() {
            Ok(value) => {
                ctx.audio_command_queue.push(AudioCommand::SetParam {
                    source: AudioSourceId::new(source.clone()),
                    param: param.clone(),
                    value,
                });
                AudioScriptCommandOutcome::ParamSet {
                    source: source.clone(),
                    param: param.clone(),
                    value,
                }
            }
            Err(error) => AudioScriptCommandOutcome::ParseError {
                message: format!("failed to parse audio param value `{value}` as f32: {error}"),
            },
        },
        ("set-volume", [bus, value]) => match value.parse::<f32>() {
            Ok(value) if bus == "master" => {
                ctx.audio_command_queue
                    .push(AudioCommand::SetMasterVolume { value });
                AudioScriptCommandOutcome::VolumeSet {
                    bus: bus.clone(),
                    value,
                }
            }
            Ok(value) => {
                ctx.audio_command_queue.push(AudioCommand::SetVolume {
                    bus: bus.clone(),
                    value,
                });
                AudioScriptCommandOutcome::VolumeSet {
                    bus: bus.clone(),
                    value,
                }
            }
            Err(error) => AudioScriptCommandOutcome::ParseError {
                message: format!("failed to parse audio volume `{value}` as f32: {error}"),
            },
        },
        _ => AudioScriptCommandOutcome::Unhandled,
    }
}
```

MODIFY:
`crates/audio/api/src/lib.rs`

Dodaj:

```rust
mod script_command;
pub use script_command::*;
```

MODIFY:
`crates/apps/app/src/script_runtime/handlers/audio.rs`

Zastąp body `handle`, linie 18-150, adapterem:

```rust
fn handle(&self, ctx: &AppScriptCommandContext<'a>, command: ScriptCommand) {
    let outcome = amigo_audio_api::handle_audio_script_command(
        amigo_audio_api::AudioScriptCommandContext {
            audio_command_queue: ctx.audio_command_queue,
            audio_scene_service: ctx.audio_scene_service,
        },
        command.clone(),
        |name| crate::app_helpers::resolve_mod_audio_asset_key(ctx.launch_selection, name),
    );

    match outcome {
        amigo_audio_api::AudioScriptCommandOutcome::Preloaded { asset_key, mode } => {
            crate::app_helpers::register_audio_clip_reference(
                ctx.asset_catalog,
                ctx.audio_scene_service,
                &asset_key,
                mode,
            );
            ctx.dev_console_state
                .write_line(format!("preloaded audio clip `{}`", asset_key.as_str()));
        }
        amigo_audio_api::AudioScriptCommandOutcome::PlayOnce { asset_key } => {
            crate::app_helpers::register_audio_clip_reference(
                ctx.asset_catalog,
                ctx.audio_scene_service,
                &asset_key,
                AudioPlaybackMode::OneShot,
            );
            ctx.dev_console_state
                .write_line(format!("queued audio one-shot `{}`", asset_key.as_str()));
        }
        amigo_audio_api::AudioScriptCommandOutcome::CueQueued { cue_name, clip } => {
            ctx.dev_console_state.write_line(format!(
                "queued audio cue `{}` as one-shot `{}`",
                cue_name,
                clip.as_str()
            ));
        }
        amigo_audio_api::AudioScriptCommandOutcome::CueMissing { cue_name } => {
            ctx.dev_console_state
                .write_line(format!("unknown audio cue `{cue_name}`"));
        }
        amigo_audio_api::AudioScriptCommandOutcome::CueNotReady { .. } => {}
        amigo_audio_api::AudioScriptCommandOutcome::SourceStarted { source, asset_key } => {
            crate::app_helpers::register_audio_clip_reference(
                ctx.asset_catalog,
                ctx.audio_scene_service,
                &asset_key,
                AudioPlaybackMode::Looping,
            );
            ctx.dev_console_state.write_line(format!(
                "queued realtime audio source `{}` using `{}`",
                source,
                asset_key.as_str()
            ));
        }
        amigo_audio_api::AudioScriptCommandOutcome::SourceStopped { source } => {
            ctx.dev_console_state
                .write_line(format!("queued stop for audio source `{source}`"));
        }
        amigo_audio_api::AudioScriptCommandOutcome::ParamSet { .. } => {}
        amigo_audio_api::AudioScriptCommandOutcome::VolumeSet { bus, value } => {
            ctx.dev_console_state.write_line(format!(
                "queued audio bus volume `{bus}` = {}",
                value.clamp(0.0, 1.0)
            ));
        }
        amigo_audio_api::AudioScriptCommandOutcome::ParseError { message } => {
            ctx.dev_console_state.write_line(message);
        }
        amigo_audio_api::AudioScriptCommandOutcome::Unhandled => {
            ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            ));
        }
    }
}
```

VERIFY:

```powershell
cargo check -p amigo-audio-api
cargo check -p amigo-app
```

4.3 UI script command.

ADD:
`crates/ui/core/src/script_command.rs`

Przenieś prawie całe body z:
`crates/apps/app/src/script_runtime/handlers/ui.rs`

Docelowy plik:

```rust
use amigo_math::ColorRgba;
use amigo_scripting_api::ScriptCommand;

use crate::UiStateService;

pub struct UiScriptCommandContext<'a> {
    pub ui_state_service: &'a UiStateService,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiScriptCommandOutcome {
    Updated(String),
    ParseError(String),
    Unhandled,
}

pub fn can_handle_ui_script_command(command: &ScriptCommand) -> bool {
    command.namespace == "ui"
}

pub fn handle_ui_script_command(
    ctx: UiScriptCommandContext<'_>,
    command: ScriptCommand,
) -> UiScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("set-text", [path, value]) => {
            if ctx.ui_state_service.set_text(path.clone(), value.clone()) {
                UiScriptCommandOutcome::Updated(format!("updated ui text override `{path}`"))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui text override `{path}` unchanged"))
            }
        }
        ("set-value", [path, value]) => match value.parse::<f32>() {
            Ok(value) => {
                if ctx.ui_state_service.set_value(path.clone(), value) {
                    UiScriptCommandOutcome::Updated(format!(
                        "updated ui value override `{path}` to {}",
                        value.clamp(0.0, 1.0)
                    ))
                } else {
                    UiScriptCommandOutcome::Updated(format!("ui value override `{path}` unchanged"))
                }
            }
            Err(error) => UiScriptCommandOutcome::ParseError(format!(
                "failed to parse ui value `{value}` as f32: {error}"
            )),
        },
        ("set_selected", [path, value]) | ("set-selected", [path, value]) => {
            if ctx.ui_state_service.set_selected(path.clone(), value.clone()) {
                UiScriptCommandOutcome::Updated(format!(
                    "updated ui selected override `{path}` to `{value}`"
                ))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui selected override `{path}` unchanged"))
            }
        }
        ("set-options", [path, options @ ..]) | ("set_options", [path, options @ ..]) => {
            let options = options
                .iter()
                .filter(|option| !option.is_empty())
                .cloned()
                .collect::<Vec<_>>();
            if ctx.ui_state_service.set_options(path.clone(), options.clone()) {
                UiScriptCommandOutcome::Updated(format!(
                    "updated ui options override `{path}` with {} options",
                    options.len()
                ))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui options override `{path}` unchanged"))
            }
        }
        ("set-color", [path, value]) => match parse_color_rgba_hex(value) {
            Some(color) => {
                if ctx.ui_state_service.set_color(path.clone(), color) {
                    UiScriptCommandOutcome::Updated(format!("updated ui color override `{path}`"))
                } else {
                    UiScriptCommandOutcome::Updated(format!("ui color override `{path}` unchanged"))
                }
            }
            None => UiScriptCommandOutcome::ParseError(format!("failed to parse ui color `{value}`")),
        },
        ("set-background", [path, value]) | ("set_background", [path, value]) => {
            match parse_color_rgba_hex(value) {
                Some(color) => {
                    if ctx.ui_state_service.set_background(path.clone(), color) {
                        UiScriptCommandOutcome::Updated(format!(
                            "updated ui background override `{path}`"
                        ))
                    } else {
                        UiScriptCommandOutcome::Updated(format!(
                            "ui background override `{path}` unchanged"
                        ))
                    }
                }
                None => UiScriptCommandOutcome::ParseError(format!(
                    "failed to parse ui background `{value}`"
                )),
            }
        }
        ("show", [path]) => {
            if ctx.ui_state_service.show(path.clone()) {
                UiScriptCommandOutcome::Updated(format!("showed ui path `{path}`"))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui path `{path}` already visible"))
            }
        }
        ("hide", [path]) => {
            if ctx.ui_state_service.hide(path.clone()) {
                UiScriptCommandOutcome::Updated(format!("hid ui path `{path}`"))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui path `{path}` already hidden"))
            }
        }
        ("enable", [path]) => {
            if ctx.ui_state_service.enable(path.clone()) {
                UiScriptCommandOutcome::Updated(format!("enabled ui path `{path}`"))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui path `{path}` already enabled"))
            }
        }
        ("disable", [path]) => {
            if ctx.ui_state_service.disable(path.clone()) {
                UiScriptCommandOutcome::Updated(format!("disabled ui path `{path}`"))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui path `{path}` already disabled"))
            }
        }
        _ => UiScriptCommandOutcome::Unhandled,
    }
}

fn parse_color_rgba_hex(value: &str) -> Option<ColorRgba> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_hex_channel(&hex[0..2])?,
            parse_hex_channel(&hex[2..4])?,
            parse_hex_channel(&hex[4..6])?,
            255,
        ),
        8 => (
            parse_hex_channel(&hex[0..2])?,
            parse_hex_channel(&hex[2..4])?,
            parse_hex_channel(&hex[4..6])?,
            parse_hex_channel(&hex[6..8])?,
        ),
        _ => return None,
    };
    Some(ColorRgba::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

fn parse_hex_channel(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}
```

MODIFY:
`crates/ui/core/src/lib.rs`

Dodaj:

```rust
mod script_command;
pub use script_command::*;
```

MODIFY:
`crates/apps/app/src/script_runtime/handlers/ui.rs`

Zastąp body `handle`, linie 19-120:

```rust
fn handle(&self, ctx: &AppScriptCommandContext<'a>, command: ScriptCommand) {
    let outcome = amigo_ui::handle_ui_script_command(
        amigo_ui::UiScriptCommandContext {
            ui_state_service: ctx.ui_state_service,
        },
        command.clone(),
    );

    match outcome {
        amigo_ui::UiScriptCommandOutcome::Updated(message)
        | amigo_ui::UiScriptCommandOutcome::ParseError(message) => {
            ctx.dev_console_state.write_line(message);
        }
        amigo_ui::UiScriptCommandOutcome::Unhandled => {
            ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            ));
        }
    }
}
```

Usuń z app `ui.rs` helpery:

```rust
parse_color_rgba_hex
parse_hex_channel
```

Usuń import:

```rust
use amigo_math::ColorRgba;
```

VERIFY:

```powershell
cargo check -p amigo-ui-core
cargo check -p amigo-app
```

4.4 Render script command.

Nie rób jednego wielkiego domain pliku.
Rozbij wg namespace.

ADD:
`crates/2d/layered-image/src/script_command.rs`
`crates/2d/lighting/src/script_command.rs`
`crates/2d/composition/src/script_command.rs`
`crates/3d/mesh/src/script_command.rs`
`crates/3d/material/src/script_command.rs`
`crates/3d/text/src/script_command.rs`

Minimalny wzór dla domeny:

```rust
use amigo_scripting_api::ScriptCommand;

pub enum DomainScriptCommandOutcome {
    Handled(String),
    ParseError(String),
    Unhandled,
}

pub fn can_handle_domain_script_command(command: &ScriptCommand) -> bool {
    matches!(command.namespace.as_str(), "...")
}

pub fn handle_domain_script_command(
    ctx: DomainScriptCommandContext<'_>,
    command: ScriptCommand,
) -> DomainScriptCommandOutcome {
    match (...) {
        // przenieś odpowiadające branche z app render.rs
        _ => DomainScriptCommandOutcome::Unhandled,
    }
}
```

Najpierw przenieś mutacje service:
`2d.layered_image set_base_opacity/set_opacity/set_enabled/set_blend`
do `crates/2d/layered-image/src/script_command.rs`

`2d.light set_intensity/set_color`
`2d.light_group set_intensity/set_color`
do `crates/2d/lighting/src/script_command.rs`

`2d.render_layer set_opacity/set_visible`
do `crates/2d/composition/src/script_command.rs`

Spawn commands mogą zostać chwilowo w app adapterze jako scene-command submission, ale docelowo zrób helpers:
`amigo_2d_sprite::build_sprite2d_spawn_script_scene_command`
`amigo_2d_text::build_text2d_spawn_script_scene_command`
`amigo_3d_mesh::build_mesh3d_spawn_script_scene_command`
`amigo_3d_material::build_material3d_bind_script_scene_command`
`amigo_3d_text::build_text3d_spawn_script_scene_command`

MODIFY:
`crates/apps/app/src/script_runtime/handlers/render.rs`

Po migracji każdej gałęzi zastępuj branch w `match` delegacją do domeny.
Na końcu app handler ma tylko:

* wybrać właściwą domenę po namespace
* zbudować context
* wywołać domain handler
* wypisać outcome do dev console
* submitować SceneCommand tylko wtedy, gdy domain helper zwróci gotowy `SceneCommand`

VERIFY:

```powershell
cargo check -p amigo-2d-layered-image
cargo check -p amigo-2d-lighting
cargo check -p amigo-2d-composition
cargo check -p amigo-3d-mesh
cargo check -p amigo-3d-material
cargo check -p amigo-3d-text
cargo check -p amigo-app
```

FINAL GREP SCRIPT:

```powershell
git grep -n "set_base_opacity\|set_layer_opacity\|set_intensity\|set_color\|set_opacity\|set_visible" crates/apps/app/src/script_runtime/handlers
git grep -n "SceneCommand::QueueSprite2d\|SceneCommand::QueueText2d\|SceneCommand::QueueMesh3d\|SceneCommand::QueueMaterial3d\|SceneCommand::QueueText3d" crates/apps/app/src/script_runtime/handlers
```

Oczekiwane:
app script handler nie wykonuje domenowych mutacji.
Może tylko route/adapter.

ROZDZIAŁ 5 — dev-console ownership

Cel:
domain dev-console execution w domenach.
App zostawia shell, parser, overlay, output.

Aktualnie:
app już nie rejestruje kilku domain commands, ale pliki nadal są w app:

```text
crates/apps/app/src/dev_console/commands/composition.rs
crates/apps/app/src/dev_console/commands/layered.rs
crates/apps/app/src/dev_console/commands/lighting.rs
crates/apps/app/src/dev_console/commands/particles.rs
crates/apps/app/src/dev_console/commands/postfx.rs
```

5.1 Dodaj domain dev console API.

ADD:
`crates/2d/composition/src/dev_console.rs`
`crates/2d/layered-image/src/dev_console.rs`
`crates/2d/lighting/src/dev_console.rs`
`crates/2d/particles/src/dev_console.rs`
`crates/2d/post-fx/src/dev_console.rs`

Wzór:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainDevConsoleCommandOutcome {
    Handled(String),
    Error(String),
    Unhandled,
}

pub fn can_handle_domain_dev_console_command(name: &str) -> bool {
    matches!(name, "...")
}
```

Przenieś body z app pliku 1:1 do domeny, ale:

* nie używaj `ConsoleCommandContext`
* context domeny ma mieć konkretne service’y
* wynik zwracaj jako `Outcome`, nie zapisuj bezpośrednio do console

Przykład dla post-fx:

ADD:
`crates/2d/post-fx/src/dev_console.rs`

```rust
use crate::PostFx2dService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostFxDevConsoleCommandOutcome {
    Handled(String),
    Error(String),
    Unhandled,
}

pub struct PostFxDevConsoleCommandContext<'a> {
    pub post_fx_service: &'a PostFx2dService,
}

pub fn can_handle_post_fx_dev_console_command(name: &str) -> bool {
    matches!(name, "postfx" | "post-fx" | "fx")
}

pub fn handle_post_fx_dev_console_command(
    _ctx: PostFxDevConsoleCommandContext<'_>,
    name: &str,
    _args: &[String],
) -> PostFxDevConsoleCommandOutcome {
    if !can_handle_post_fx_dev_console_command(name) {
        return PostFxDevConsoleCommandOutcome::Unhandled;
    }

    PostFxDevConsoleCommandOutcome::Handled(
        "post-fx domain console command handled".to_owned(),
    )
}
```

Potem przenieś realne branch’e z app `postfx.rs`.

MODIFY:
każdy domain `lib.rs`:

Dodaj:

```rust
mod dev_console;
pub use dev_console::*;
```

5.2 Zamień app command files na adaptery.

MODIFY:
`crates/apps/app/src/dev_console/commands/postfx.rs`

Zostaw `impl ConsoleCommandHandler`, ale body `handle` ma tylko:

```rust
fn handle(
    &self,
    ctx: &ConsoleCommandContext<'_>,
    command: ParsedConsoleCommand,
) -> ConsoleCommandResult {
    let service = match ctx.required::<amigo_2d_post_fx::PostFx2dService>() {
        Ok(service) => service,
        Err(error) => return ConsoleCommandResult::error(error.to_string()),
    };

    match amigo_2d_post_fx::handle_post_fx_dev_console_command(
        amigo_2d_post_fx::PostFxDevConsoleCommandContext {
            post_fx_service: service.as_ref(),
        },
        &command.name,
        &command.args,
    ) {
        amigo_2d_post_fx::PostFxDevConsoleCommandOutcome::Handled(message) => {
            ConsoleCommandResult::ok(message)
        }
        amigo_2d_post_fx::PostFxDevConsoleCommandOutcome::Error(message) => {
            ConsoleCommandResult::error(message)
        }
        amigo_2d_post_fx::PostFxDevConsoleCommandOutcome::Unhandled => {
            ConsoleCommandResult::unknown(command.raw)
        }
    }
}
```

Powtórz adapter pattern dla:

```text
composition.rs
layered.rs
lighting.rs
particles.rs
```

5.3 Przywróć rejestrację adapterów, ale ownership descriptor idzie do domeny.

MODIFY:
`crates/apps/app/src/dev_console/commands/mod.rs`

Dodaj moduły z powrotem:

```rust
mod composition;
mod layered;
mod lighting;
mod particles;
mod postfx;
```

Dodaj rejestrację:

```rust
registry.register(composition::Composition2dConsoleCommandHandler);
registry.register(layered::LayeredImageConsoleCommandHandler);
registry.register(lighting::Lighting2dConsoleCommandHandler);
registry.register(particles::ParticlesConsoleCommandHandler);
registry.register(postfx::PostFxConsoleCommandHandler);
```

5.4 Dodaj capability descriptors w domenach.

MODIFY:
`crates/2d/post-fx/src/runtime_capabilities.rs`

Dodaj descriptor:

```rust
RuntimeCapabilityDescriptor {
    domain_id: RuntimeDomainId::new("2d.post-fx"),
    kind: RuntimeCapabilityKind::DevConsoleCommand,
    id: "2d.post-fx.dev-console".to_owned(),
    label: "2D post-fx dev-console commands".to_owned(),
    description: "Domain-owned 2D post-fx dev-console command execution".to_owned(),
    capabilities: vec!["dev-console".to_owned(), "post-fx".to_owned()],
    tags: vec!["2d".to_owned(), "post-fx".to_owned()],
    migration_seam: false,
}
```

Analogicznie:

```text
2d.composition.dev-console
2d.layered-image.dev-console
2d.lighting.dev-console
2d.particles.dev-console
```

VERIFY:

```powershell
cargo check -p amigo-2d-composition
cargo check -p amigo-2d-layered-image
cargo check -p amigo-2d-lighting
cargo check -p amigo-2d-particles
cargo check -p amigo-2d-post-fx
cargo check -p amigo-app
```

GREP:

```powershell
git grep -n "ConsoleCommandHandler for .*Composition\\|ConsoleCommandHandler for .*Lighting\\|ConsoleCommandHandler for .*Particles\\|ConsoleCommandHandler for .*PostFx" crates/apps/app/src/dev_console/commands
git grep -n "RuntimeCapabilityKind::DevConsoleCommand" crates/2d crates/apps/app/src
```

Oczekiwane:
app ma adaptery.
domeny mają execution i descriptors.
app descriptors tylko shell/host.

ROZDZIAŁ 6 — diagnostics / metadata ownership

Cel:
app diagnostics tylko host overview.
Domeny mają swoje descriptors.

6.1 Sprawdź app descriptors.

Plik:
`crates/apps/app/src/diagnostics.rs`

Aktualny stan jest prawidłowo host-only:

```rust
runtime.diagnostics.overview
runtime.metadata.overview
domain_id: app.host
```

Nie rozszerzaj tego pliku o domeny.

6.2 Dodaj domain diagnostics descriptors tam, gdzie domeny mają runtime state.

Dodawaj w:

```text
crates/2d/particles/src/runtime_capabilities.rs
crates/2d/post-fx/src/runtime_capabilities.rs
crates/2d/lighting/src/runtime_capabilities.rs
crates/2d/composition/src/runtime_capabilities.rs
crates/audio/mixer/src/runtime_capabilities.rs
crates/ui/core/src/runtime_capabilities.rs
crates/scripting/rhai/src/runtime_capabilities.rs
```

Wzór:

```rust
RuntimeCapabilityDescriptor {
    domain_id: RuntimeDomainId::new("2d.particles"),
    kind: RuntimeCapabilityKind::DiagnosticsProvider,
    id: "2d.particles.diagnostics".to_owned(),
    label: "2D particles diagnostics".to_owned(),
    description: "Particle emitter/runtime diagnostics owned by the 2D particles domain".to_owned(),
    capabilities: vec!["diagnostics".to_owned(), "particles".to_owned()],
    tags: vec!["2d".to_owned(), "particles".to_owned()],
    migration_seam: false,
}
```

Metadata wzór:

```rust
RuntimeCapabilityDescriptor {
    domain_id: RuntimeDomainId::new("2d.particles"),
    kind: RuntimeCapabilityKind::MetadataProvider,
    id: "2d.particles.metadata".to_owned(),
    label: "2D particles metadata".to_owned(),
    description: "Particle runtime metadata owned by the 2D particles domain".to_owned(),
    capabilities: vec!["metadata".to_owned(), "particles".to_owned()],
    tags: vec!["2d".to_owned(), "particles".to_owned()],
    migration_seam: false,
}
```

6.3 Nie dodawaj nowych app.host descriptors.

GREP:

```powershell
git grep -n "RuntimeCapabilityDescriptor" crates/apps/app/src
git grep -n "APP_HOST_DOMAIN_ID\\|RuntimeDomainId::new(\"app.host\")" crates/apps/app/src
```

Oczekiwane:
app descriptors tylko:

* host render overlay extractors
* debug/dev-shell script handlers
* diagnostics overview
* metadata overview
* real host system/console shell, jeśli nadal istnieje

VERIFY:

```powershell
cargo check -p amigo-session
cargo check -p amigo-app
```

ROZDZIAŁ 7 — final cleanup + docs + global greps

7.1 MODIFY:
`crates/apps/app/README.md`

Dodaj albo zastąp boundary sekcję:

```text
## Thin App Host boundary

`amigo-app` is the host, not the owner of domain runtime logic.

It owns:
- window/event loop
- WGPU surface and frame presentation
- host input bridge
- startup UX
- dev-console shell
- debug overlay presentation
- RuntimeSession wiring
- temporary adapters where shared registries still require host context

It must not own:
- domain scene command execution
- domain render extraction
- domain systems
- domain script command execution
- domain dev-console command execution
- domain diagnostics or metadata ownership

Runtime Capabilities describe valid installed capabilities only.
A capability is either domain-owned or `app.host`.
Domain logic still physically in app is a migration blocker, not an `app.host` capability.
```

7.2 MODIFY:
`crates/engine/session/README.md`

Dodaj:

```text
## Runtime Capabilities and typed registries

Runtime Capabilities describe what is installed.
Typed registries execute runtime behavior.

The catalog owns:
- descriptors
- ownership
- summaries
- duplicate ID diagnostics

Typed registries own:
- scene command handlers
- script command handlers
- system phase handlers
- render extractors
- dev-console command handlers

Domain crates own domain execution.
The app host may assemble registries and provide adapters, but must not own domain behavior.
```

7.3 MODIFY:
`AGENTS.md`

Dodaj regułę:

```text
For Amigo refactors, do not solve domain-owned runtime logic by labeling it `app.host`.
Move the behavior to the owning domain or leave an explicit blocker.
`app.host` is reserved for true host responsibilities only.
```

7.4 Końcowe grepy:

```powershell
git grep -n "domain_contributions\\|RuntimeContribution\\|RuntimeDomainContribution\\|APP_LEGACY_DOMAIN_ID\\|app\\.legacy\\|LegacyApp\\|AppLegacy\\|register_legacy\\|PendingDomainMigration\\|PENDING_DOMAIN_MIGRATION"
git grep -n "queue_.*_scene_command" crates/apps/app/src
git grep -n "SceneCommand::Queue" crates/apps/app/src
git grep -n "SceneEvent::.*Queued\\|SceneEvent::EntitySpawned" crates/apps/app/src
git grep -n "context\\..*_scene_service\\.commands()\\|draw_commands()\\|scene_stack()" crates/apps/app/src/render_runtime/extractors.rs
git grep -n "trait SceneCommandHandler\\|trait ScriptCommandHandler\\|SceneCommandHandlerAdapter\\|ScriptCommandHandlerAdapter" crates/apps/app/src
git grep -n "RuntimeCapabilityDescriptor" crates/apps/app/src
git grep -n "APP_HOST_DOMAIN_ID\\|RuntimeDomainId::new(\"app.host\")" crates/apps/app/src
```

Oczekiwane:

* stare nazwy: zero
* scene execution w app: zero
* direct domain render extraction w app: zero poza host overlay
* local handler traits/adapters w app: zero
* app RuntimeCapabilityDescriptor tylko host

7.5 Targeted verify:

```powershell
cargo check -p amigo-session
cargo check -p amigo-scene
cargo check -p amigo-render-api
cargo check -p amigo-2d-composition
cargo check -p amigo-2d-lighting
cargo check -p amigo-2d-particles
cargo check -p amigo-2d-post-fx
cargo check -p amigo-3d-mesh
cargo check -p amigo-3d-material
cargo check -p amigo-3d-text
cargo check -p amigo-audio-api
cargo check -p amigo-audio-mixer
cargo check -p amigo-ui-core
cargo check -p amigo-rhai
cargo check -p amigo-app
```

Nie robić jeszcze:

```powershell
cargo check --workspace
cargo test --workspace
```

Raport końcowy:

```text
Changed files:
...

Moved ownership:
- render extraction: ...
- systems: ...
- script commands: ...
- dev-console commands: ...
- diagnostics/metadata: ...

Left in app:
- window/event loop
- WGPU surface/presentation
- host input bridge
- startup UX
- dev-console shell
- debug overlay
- RuntimeSession wiring
- explicit temporary adapters

Verify:
...

Remaining blockers:
...
```
