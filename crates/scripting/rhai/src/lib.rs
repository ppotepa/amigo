mod bindings;
mod handles;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use amigo_2d_motion::Motion2dSceneService;
use amigo_2d_particles::{Particle2dSceneService, ParticlePreset2dService};
use amigo_2d_physics::Physics2dSceneService;
use amigo_2d_sprite::SpriteSceneService;
use amigo_2d_vector::VectorSceneService;
use amigo_assets::AssetCatalog;
use amigo_core::{AmigoError, AmigoResult, LaunchSelection, RuntimeDiagnostics};
use amigo_input_api::InputState;
use amigo_modding::ModCatalog;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{EntityPoolSceneService, LifetimeSceneService, SceneService};
use amigo_scripting_api::{
    DevConsoleQueue, DevConsoleState, ScriptCommandQueue, ScriptEventQueue, ScriptLifecycleState,
    ScriptRuntime, ScriptRuntimeInfo, ScriptRuntimeService,
};
use amigo_state::{SceneStateService, SceneTimerService, SessionStateService};
use amigo_ui::UiThemeService;
use bindings::{ScriptTimeState, WorldApi, register_world_api};
use rhai::CallFnOptions;

struct StoredScript {
    ast: rhai::AST,
    scope: rhai::Scope<'static>,
}

pub struct RhaiScriptRuntime {
    engine: rhai::Engine,
    scripts: Mutex<BTreeMap<String, StoredScript>>,
    time_state: Arc<ScriptTimeState>,
    timer_service: Arc<SceneTimerService>,
    world: WorldApi,
}

impl RhaiScriptRuntime {
    pub fn new(
        scene: Option<Arc<SceneService>>,
        sprite_scene: Option<Arc<SpriteSceneService>>,
        asset_catalog: Option<Arc<AssetCatalog>>,
        input_state: Option<Arc<InputState>>,
        launch_selection: Option<Arc<LaunchSelection>>,
        mod_catalog: Option<Arc<ModCatalog>>,
        diagnostics: Option<Arc<RuntimeDiagnostics>>,
        command_queue: Option<Arc<ScriptCommandQueue>>,
        event_queue: Option<Arc<ScriptEventQueue>>,
        console_queue: Option<Arc<DevConsoleQueue>>,
    ) -> Self {
        Self::new_with_motion(
            scene,
            sprite_scene,
            None,
            None,
            asset_catalog,
            input_state,
            launch_selection,
            mod_catalog,
            diagnostics,
            command_queue,
            event_queue,
            console_queue,
        )
    }

    pub fn new_with_motion(
        scene: Option<Arc<SceneService>>,
        sprite_scene: Option<Arc<SpriteSceneService>>,
        motion_scene: Option<Arc<Motion2dSceneService>>,
        physics_scene: Option<Arc<Physics2dSceneService>>,
        asset_catalog: Option<Arc<AssetCatalog>>,
        input_state: Option<Arc<InputState>>,
        launch_selection: Option<Arc<LaunchSelection>>,
        mod_catalog: Option<Arc<ModCatalog>>,
        diagnostics: Option<Arc<RuntimeDiagnostics>>,
        command_queue: Option<Arc<ScriptCommandQueue>>,
        event_queue: Option<Arc<ScriptEventQueue>>,
        console_queue: Option<Arc<DevConsoleQueue>>,
    ) -> Self {
        Self::new_with_motion_and_vector(
            scene,
            sprite_scene,
            None,
            motion_scene,
            physics_scene,
            asset_catalog,
            input_state,
            launch_selection,
            mod_catalog,
            diagnostics,
            command_queue,
            event_queue,
            console_queue,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_motion_and_vector(
        scene: Option<Arc<SceneService>>,
        sprite_scene: Option<Arc<SpriteSceneService>>,
        vector_scene: Option<Arc<VectorSceneService>>,
        motion_scene: Option<Arc<Motion2dSceneService>>,
        physics_scene: Option<Arc<Physics2dSceneService>>,
        asset_catalog: Option<Arc<AssetCatalog>>,
        input_state: Option<Arc<InputState>>,
        launch_selection: Option<Arc<LaunchSelection>>,
        mod_catalog: Option<Arc<ModCatalog>>,
        diagnostics: Option<Arc<RuntimeDiagnostics>>,
        command_queue: Option<Arc<ScriptCommandQueue>>,
        event_queue: Option<Arc<ScriptEventQueue>>,
        console_queue: Option<Arc<DevConsoleQueue>>,
    ) -> Self {
        Self::new_with_services(
            scene,
            sprite_scene,
            vector_scene,
            motion_scene,
            None,
            physics_scene,
            None,
            None,
            None,
            None,
            None,
            asset_catalog,
            input_state,
            launch_selection,
            mod_catalog,
            diagnostics,
            command_queue,
            event_queue,
            console_queue,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_services(
        scene: Option<Arc<SceneService>>,
        sprite_scene: Option<Arc<SpriteSceneService>>,
        vector_scene: Option<Arc<VectorSceneService>>,
        motion_scene: Option<Arc<Motion2dSceneService>>,
        particle_scene: Option<Arc<Particle2dSceneService>>,
        physics_scene: Option<Arc<Physics2dSceneService>>,
        pool_scene: Option<Arc<EntityPoolSceneService>>,
        lifetime_scene: Option<Arc<LifetimeSceneService>>,
        state_service: Option<Arc<SceneStateService>>,
        session_service: Option<Arc<SessionStateService>>,
        timer_service: Option<Arc<SceneTimerService>>,
        asset_catalog: Option<Arc<AssetCatalog>>,
        input_state: Option<Arc<InputState>>,
        launch_selection: Option<Arc<LaunchSelection>>,
        mod_catalog: Option<Arc<ModCatalog>>,
        diagnostics: Option<Arc<RuntimeDiagnostics>>,
        command_queue: Option<Arc<ScriptCommandQueue>>,
        event_queue: Option<Arc<ScriptEventQueue>>,
        console_queue: Option<Arc<DevConsoleQueue>>,
    ) -> Self {
        Self::new_with_services_and_ui_theme(
            scene,
            sprite_scene,
            vector_scene,
            motion_scene,
            particle_scene,
            physics_scene,
            pool_scene,
            lifetime_scene,
            state_service,
            session_service,
            timer_service,
            None,
            asset_catalog,
            input_state,
            launch_selection,
            mod_catalog,
            diagnostics,
            command_queue,
            event_queue,
            console_queue,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_services_and_ui_theme(
        scene: Option<Arc<SceneService>>,
        sprite_scene: Option<Arc<SpriteSceneService>>,
        vector_scene: Option<Arc<VectorSceneService>>,
        motion_scene: Option<Arc<Motion2dSceneService>>,
        particle_scene: Option<Arc<Particle2dSceneService>>,
        physics_scene: Option<Arc<Physics2dSceneService>>,
        pool_scene: Option<Arc<EntityPoolSceneService>>,
        lifetime_scene: Option<Arc<LifetimeSceneService>>,
        state_service: Option<Arc<SceneStateService>>,
        session_service: Option<Arc<SessionStateService>>,
        timer_service: Option<Arc<SceneTimerService>>,
        ui_theme_service: Option<Arc<UiThemeService>>,
        asset_catalog: Option<Arc<AssetCatalog>>,
        input_state: Option<Arc<InputState>>,
        launch_selection: Option<Arc<LaunchSelection>>,
        mod_catalog: Option<Arc<ModCatalog>>,
        diagnostics: Option<Arc<RuntimeDiagnostics>>,
        command_queue: Option<Arc<ScriptCommandQueue>>,
        event_queue: Option<Arc<ScriptEventQueue>>,
        console_queue: Option<Arc<DevConsoleQueue>>,
    ) -> Self {
        Self::new_with_services_and_ui_theme_and_particle_presets(
            scene,
            sprite_scene,
            vector_scene,
            motion_scene,
            particle_scene,
            None,
            physics_scene,
            pool_scene,
            lifetime_scene,
            state_service,
            session_service,
            timer_service,
            ui_theme_service,
            asset_catalog,
            input_state,
            launch_selection,
            mod_catalog,
            diagnostics,
            command_queue,
            event_queue,
            console_queue,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_services_and_ui_theme_and_particle_presets(
        scene: Option<Arc<SceneService>>,
        sprite_scene: Option<Arc<SpriteSceneService>>,
        vector_scene: Option<Arc<VectorSceneService>>,
        motion_scene: Option<Arc<Motion2dSceneService>>,
        particle_scene: Option<Arc<Particle2dSceneService>>,
        particle_preset_scene: Option<Arc<ParticlePreset2dService>>,
        physics_scene: Option<Arc<Physics2dSceneService>>,
        pool_scene: Option<Arc<EntityPoolSceneService>>,
        lifetime_scene: Option<Arc<LifetimeSceneService>>,
        state_service: Option<Arc<SceneStateService>>,
        session_service: Option<Arc<SessionStateService>>,
        timer_service: Option<Arc<SceneTimerService>>,
        ui_theme_service: Option<Arc<UiThemeService>>,
        asset_catalog: Option<Arc<AssetCatalog>>,
        input_state: Option<Arc<InputState>>,
        launch_selection: Option<Arc<LaunchSelection>>,
        mod_catalog: Option<Arc<ModCatalog>>,
        diagnostics: Option<Arc<RuntimeDiagnostics>>,
        command_queue: Option<Arc<ScriptCommandQueue>>,
        event_queue: Option<Arc<ScriptEventQueue>>,
        console_queue: Option<Arc<DevConsoleQueue>>,
    ) -> Self {
        let time_state = Arc::new(ScriptTimeState::default());
        let state_service = state_service.unwrap_or_else(|| Arc::new(SceneStateService::default()));
        let session_service =
            session_service.unwrap_or_else(|| Arc::new(SessionStateService::default()));
        let timer_service = timer_service.unwrap_or_else(|| Arc::new(SceneTimerService::default()));
        let world = WorldApi::new(
            scene.clone(),
            sprite_scene.clone(),
            vector_scene.clone(),
            motion_scene.clone(),
            particle_scene.clone(),
            particle_preset_scene.clone(),
            physics_scene.clone(),
            pool_scene.clone(),
            lifetime_scene.clone(),
            Some(state_service.clone()),
            Some(session_service.clone()),
            Some(timer_service.clone()),
            ui_theme_service,
            asset_catalog.clone(),
            input_state.clone(),
            time_state.clone(),
            launch_selection.clone(),
            mod_catalog.clone(),
            diagnostics.clone(),
            command_queue.clone(),
            event_queue.clone(),
            console_queue.clone(),
        );
        Self {
            engine: build_engine(),
            scripts: Mutex::new(BTreeMap::new()),
            time_state,
            timer_service,
            world,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn call_optional_void<Args>(
        &self,
        source_name: &str,
        function_name: &str,
        args: Args,
    ) -> AmigoResult<()>
    where
        Args: rhai::FuncArgs,
    {
        let mut scripts = self
            .scripts
            .lock()
            .expect("rhai script registry mutex should not be poisoned");
        let Some(script) = scripts.get_mut(source_name) else {
            return Ok(());
        };

        self.engine
            .call_fn_with_options::<rhai::Dynamic>(
                CallFnOptions::new().eval_ast(false),
                &mut script.scope,
                &script.ast,
                function_name,
                args,
            )
            .map(|_| ())
            .map_err(|error| {
                let message = error.to_string();
                if message.contains(&format!("Function not found: {function_name}")) {
                    AmigoError::Message(String::new())
                } else {
                    AmigoError::Message(format!(
                        "failed to call {function_name} for script `{source_name}`: {error}"
                    ))
                }
            })
            .or_else(|error| {
                if error.to_string().is_empty() {
                    Ok(())
                } else {
                    Err(error)
                }
            })
    }
}

impl ScriptRuntime for RhaiScriptRuntime {
    fn backend_name(&self) -> &'static str {
        "rhai"
    }

    fn file_extension(&self) -> &'static str {
        "rhai"
    }

    fn validate(&self, source: &str) -> AmigoResult<()> {
        self.engine
            .compile(source)
            .map(|_| ())
            .map_err(|error| AmigoError::Message(error.to_string()))
    }

    fn execute(&self, source_name: &str, source: &str) -> AmigoResult<()> {
        let ast = self.engine.compile(source).map_err(|error| {
            AmigoError::Message(format!(
                "failed to compile script `{source_name}` for execution: {error}"
            ))
        })?;
        let mut scope = rhai::Scope::new();
        scope.push_constant("world", self.world.clone());
        self.engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(|error| {
                AmigoError::Message(format!("failed to execute script `{source_name}`: {error}"))
            })?;

        self.scripts
            .lock()
            .expect("rhai script registry mutex should not be poisoned")
            .insert(source_name.to_owned(), StoredScript { ast, scope });

        Ok(())
    }

    fn unload(&self, source_name: &str) -> AmigoResult<()> {
        self.scripts
            .lock()
            .expect("rhai script registry mutex should not be poisoned")
            .remove(source_name);
        Ok(())
    }

    fn call_update(&self, source_name: &str, delta_seconds: f32) -> AmigoResult<()> {
        self.time_state.advance_frame(delta_seconds);
        self.timer_service.tick(delta_seconds);
        self.call_optional_void(source_name, "update", (delta_seconds as rhai::FLOAT,))
    }

    fn call_on_enter(&self, source_name: &str) -> AmigoResult<()> {
        self.time_state.set_passive_delta(0.0);
        self.call_optional_void(source_name, "on_enter", ())
    }

    fn call_on_exit(&self, source_name: &str) -> AmigoResult<()> {
        self.time_state.set_passive_delta(0.0);
        self.call_optional_void(source_name, "on_exit", ())
    }

    fn call_on_event(&self, source_name: &str, topic: &str, payload: &[String]) -> AmigoResult<()> {
        self.time_state.set_passive_delta(0.0);
        let payload = payload
            .iter()
            .cloned()
            .map(Into::into)
            .collect::<rhai::Array>();
        self.call_optional_void(source_name, "on_event", (topic.to_owned(), payload))
    }
}

pub struct RhaiScriptingPlugin;

impl RuntimePlugin for RhaiScriptingPlugin {
    fn name(&self) -> &'static str {
        "amigo-scripting-rhai"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        if !registry.has::<ScriptCommandQueue>() {
            registry.register(ScriptCommandQueue::default())?;
        }

        if !registry.has::<ScriptEventQueue>() {
            registry.register(ScriptEventQueue::default())?;
        }

        if !registry.has::<DevConsoleQueue>() {
            registry.register(DevConsoleQueue::default())?;
        }

        if !registry.has::<DevConsoleState>() {
            registry.register(DevConsoleState::default())?;
        }

        if !registry.has::<ScriptLifecycleState>() {
            registry.register(ScriptLifecycleState::default())?;
        }

        let scene = registry.resolve::<SceneService>();
        let sprite_scene = registry.resolve::<SpriteSceneService>();
        let vector_scene = registry.resolve::<VectorSceneService>();
        let motion_scene = registry.resolve::<Motion2dSceneService>();
        let particle_scene = registry.resolve::<Particle2dSceneService>();
        let particle_preset_scene = registry.resolve::<ParticlePreset2dService>();
        let physics_scene = registry.resolve::<Physics2dSceneService>();
        let pool_scene = registry.resolve::<EntityPoolSceneService>();
        let lifetime_scene = registry.resolve::<LifetimeSceneService>();
        let state_service = registry.resolve::<SceneStateService>();
        let session_service = registry.resolve::<SessionStateService>();
        let timer_service = registry.resolve::<SceneTimerService>();
        let ui_theme_service = registry.resolve::<UiThemeService>();
        let asset_catalog = registry.resolve::<AssetCatalog>();
        let input_state = registry.resolve::<InputState>();
        let launch_selection = registry.resolve::<LaunchSelection>();
        let mod_catalog = registry.resolve::<ModCatalog>();
        let diagnostics = registry.resolve::<RuntimeDiagnostics>();
        let command_queue = registry.resolve::<ScriptCommandQueue>();
        let event_queue = registry.resolve::<ScriptEventQueue>();
        let console_queue = registry.resolve::<DevConsoleQueue>();
        let runtime = RhaiScriptRuntime::new_with_services_and_ui_theme_and_particle_presets(
            scene,
            sprite_scene,
            vector_scene,
            motion_scene,
            particle_scene,
            particle_preset_scene,
            physics_scene,
            pool_scene,
            lifetime_scene,
            state_service,
            session_service,
            timer_service,
            ui_theme_service,
            asset_catalog,
            input_state,
            launch_selection,
            mod_catalog,
            diagnostics,
            command_queue,
            event_queue,
            console_queue,
        );

        registry.register(ScriptRuntimeInfo {
            backend_name: runtime.backend_name(),
            file_extension: runtime.file_extension(),
        })?;
        registry.register(ScriptRuntimeService::new(runtime))
    }
}

fn build_engine() -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    engine.set_max_expr_depths(256, 512);
    register_world_api(&mut engine);
    engine
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    use amigo_2d_motion::{
        Facing2d, Motion2dSceneService, MotionAnimationState, MotionController2d,
        MotionController2dCommand, MotionIntent2d, MotionProfile2d, MotionState2d,
        ProjectileEmitter2d, ProjectileEmitter2dCommand,
    };
    use amigo_2d_particles::{
        Particle2dSceneService, ParticleEmitter2d, ParticleEmitter2dCommand, ParticleShape2d,
    };
    use amigo_2d_physics::{CircleCollider2d, CircleCollider2dCommand, Physics2dSceneService};
    use amigo_2d_sprite::{Sprite, SpriteDrawCommand, SpriteSceneService, SpriteSheet};
    use amigo_2d_vector::{
        VectorSceneService, VectorShape2d, VectorShape2dDrawCommand, VectorShapeKind2d,
        VectorStyle2d,
    };
    use amigo_assets::{
        AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest,
        AssetSourceKind, PreparedAssetKind, prepare_debug_placeholder_asset,
    };
    use amigo_core::{LaunchSelection, RuntimeDiagnostics};
    use amigo_input_api::{InputState, KeyCode};
    use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};
    use amigo_modding::{DiscoveredMod, ModCatalog, ModManifest, ModSceneManifest};
    use amigo_scene::{
        EntityPoolSceneCommand, EntityPoolSceneService, SceneEntityId, SceneEntityLifecycle,
        SceneKey, ScenePropertyValue, SceneService,
    };
    use amigo_scripting_api::{
        DevConsoleQueue, ScriptCommand, ScriptCommandQueue, ScriptEventQueue,
    };
    use amigo_state::{SceneStateService, SceneTimerService, SessionStateService};
    use amigo_ui::{UiTheme, UiThemePalette, UiThemeService};

    use crate::RhaiScriptRuntime;
    use amigo_scripting_api::ScriptRuntime;

    #[test]
    fn executes_scene_spawn_script() {
        let scene = Arc::new(SceneService::default());
        let runtime = RhaiScriptRuntime::new(
            Some(scene.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "test-script",
                r#"
                    world.entities.create("camera-2d");
                    world.entities.create("player");
                "#,
            )
            .expect("script execution should succeed");

        assert_eq!(scene.entity_count(), 2);
        assert_eq!(
            scene.entity_names(),
            vec!["camera-2d".to_owned(), "player".to_owned()]
        );
    }

    #[test]
    fn exposes_launch_selection_to_scripts() {
        let launch_selection = Arc::new(LaunchSelection::new(
            Some("core-game".to_owned()),
            Some("dev-shell".to_owned()),
            vec!["core".to_owned(), "core-game".to_owned()],
            true,
        ));
        let catalog = Arc::new(ModCatalog::from_discovered_mods(vec![discovered_mod(
            "core-game",
            &["dev_interface", "console_shell"],
            &["dev-shell", "console"],
        )]));
        let runtime = RhaiScriptRuntime::new(
            None,
            None,
            None,
            None,
            Some(launch_selection),
            Some(catalog),
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "launch-selection-script",
                r#"
                    if world.mod.current_id() != "core-game" { throw("wrong mod"); }
                    if world.mod.scenes().len != 2 { throw("wrong scene count"); }
                    if !world.mod.has_scene("dev-shell") { throw("missing scene"); }
                    if !world.runtime.dev_mode() { throw("dev mode disabled"); }
                "#,
            )
            .expect("script should be able to inspect launch selection");
    }

    #[test]
    fn exposes_world_domains_and_entity_refs_to_scripts() {
        let scene = Arc::new(SceneService::default());
        scene.select_scene(SceneKey::new("hello-world-square"));
        scene.spawn("playground-2d-square");
        scene.configure_entity_metadata(
            "playground-2d-square",
            SceneEntityLifecycle::default(),
            vec!["debug".to_owned(), "actor".to_owned()],
            vec!["preview".to_owned()],
            BTreeMap::from([
                ("score_value".to_owned(), ScenePropertyValue::Int(100)),
                (
                    "label".to_owned(),
                    ScenePropertyValue::String("square".to_owned()),
                ),
            ]),
        );

        let input = Arc::new(InputState::default());
        input.set_key(KeyCode::Left, true);
        input.set_key(KeyCode::Up, true);

        let command_queue = Arc::new(ScriptCommandQueue::default());
        let event_queue = Arc::new(ScriptEventQueue::default());
        let console_queue = Arc::new(DevConsoleQueue::default());
        let launch_selection = Arc::new(LaunchSelection::new(
            Some("playground-2d".to_owned()),
            Some("hello-world-square".to_owned()),
            vec!["core".to_owned(), "playground-2d".to_owned()],
            true,
        ));
        let catalog = Arc::new(ModCatalog::from_discovered_mods(vec![discovered_mod(
            "playground-2d",
            &["rendering_2d", "text_2d"],
            &["hello-world-square", "hello-world-spritesheet"],
        )]));
        let runtime = RhaiScriptRuntime::new(
            Some(scene.clone()),
            None,
            None,
            Some(input),
            Some(launch_selection),
            Some(catalog),
            None,
            Some(command_queue.clone()),
            Some(event_queue.clone()),
            Some(console_queue.clone()),
        );

        runtime
            .execute(
                "world-api-script",
                r#"
                    let square = world.entities.named("playground-2d-square");

                    if !square.exists() { throw("missing entity"); }
                    if square.name() != "playground-2d-square" { throw("wrong entity name"); }
                    if world.entities.count() != 1 { throw("wrong entity count"); }
                    if world.entities.names().len != 1 { throw("wrong entity names"); }
                    if world.scene.current_id() != "hello-world-square" { throw("wrong current scene"); }
                    if !world.scene.has("hello-world-spritesheet") { throw("missing scene"); }
                    if world.scene.available().len != 2 { throw("wrong available scene count"); }
                    if !world.input.down("ArrowLeft") { throw("missing key down"); }
                    if !world.input.pressed("ArrowUp") { throw("missing key press"); }
                    if !world.input.any_down("A, ArrowLeft") { throw("missing any_down csv"); }
                    if world.input.any_down("A,D") { throw("unexpected any_down csv"); }
                    if !world.input.any_down(["A", "ArrowUp"]) { throw("missing any_down array"); }
                    if !world.input.any_pressed("Space, ArrowUp") { throw("missing any_pressed csv"); }
                    if world.input.any_pressed(["A", "Space"]) { throw("unexpected any_pressed array"); }
                    if world.input.axis("ArrowUp", "ArrowDown") != 1 { throw("wrong positive axis"); }
                    if world.input.axis("ArrowRight", "ArrowLeft") != -1 { throw("wrong negative axis"); }
                    if world.input.axis(["ArrowUp"], ["ArrowLeft"]) != 0 { throw("opposed axis should cancel"); }
                    if world.input.keys().len != 2 { throw("wrong pressed key count"); }

                    square.rotate_2d(1.0);
                    if !world.entities.set_position_2d("playground-2d-square", 12.0, 34.0) {
                        throw("failed to set position through world.entities");
                    }
                    if world.entities.hide_many([square.name()]) != 1 {
                        throw("failed to hide_many through world.entities");
                    }
                    if !square.set_position_2d(56.0, 78.0) {
                        throw("failed to set position through entity ref");
                    }
                    if !square.hide() {
                        throw("failed to hide through entity ref");
                    }
                    if square.is_visible() {
                        throw("hide should clear visible flag");
                    }
                    if !square.show() {
                        throw("failed to show through entity ref");
                    }
                    if !world.entities.is_visible(square.name()) {
                        throw("show should set visible flag");
                    }
                    if !square.disable() || square.is_enabled() {
                        throw("disable should clear simulation flag");
                    }
                    if !world.entities.enable(square.name()) || !square.is_enabled() {
                        throw("enable should set simulation flag");
                    }
                    if !square.set_collision_enabled(false) || square.collision_enabled() {
                        throw("collision flag should be mutable");
                    }
                    if !world.entities.set_collision_enabled(square.name(), true) || !world.entities.collision_enabled(square.name()) {
                        throw("world collision flag helper failed");
                    }
                    if !square.has_tag("debug") || !world.entities.has_tag(square.name(), "actor") {
                        throw("missing tag");
                    }
                    if !square.has_group("preview") || !world.entities.has_group(square.name(), "preview") {
                        throw("missing group");
                    }
                    if world.entities.by_tag("debug").len != 1 {
                        throw("wrong by_tag count");
                    }
                    if world.entities.by_group("preview").len != 1 {
                        throw("wrong by_group count");
                    }
                    if world.entities.active_by_tag("debug").len != 1 {
                        throw("wrong active_by_tag count");
                    }
                    if square.property_int("score_value") != 100 {
                        throw("wrong entity ref property int");
                    }
                    if world.entities.property_string(square.name(), "label") != "square" {
                        throw("wrong world property string");
                    }
                    if !world.entities.set_property_int(square.name(), "score_value", 250) {
                        throw("failed to set int property");
                    }
                    if !world.entities.set_property_string(square.name(), "label", "renamed") {
                        throw("failed to set string property");
                    }
                    if square.property_int("score_value") != 250 {
                        throw("updated property int missing");
                    }
                    if world.entities.property_string(square.name(), "label") != "renamed" {
                        throw("updated world property string missing");
                    }
                    world.scene.select("hello-world-spritesheet");
                    world.dev.event("scene.intent", "hello-world-spritesheet");
                    world.dev.command("help");
                    world.dev.log("hello from world");
                    world.dev.warn("careful");
                    world.dev.refresh_diagnostics("playground-2d");
                "#,
            )
            .expect("script should be able to use the world API");

        assert_eq!(
            scene
                .transform_of("playground-2d-square")
                .expect("square should exist")
                .translation
                .x,
            56.0
        );
        assert_eq!(
            scene
                .transform_of("playground-2d-square")
                .expect("square should exist")
                .translation
                .y,
            78.0
        );
        assert!(scene.is_visible("playground-2d-square"));
        assert_eq!(
            scene
                .transform_of("playground-2d-square")
                .expect("square should exist")
                .rotation_euler
                .z,
            1.0
        );
        assert_eq!(command_queue.pending().len(), 4);
        assert_eq!(command_queue.pending()[0].namespace, "scene");
        assert_eq!(command_queue.pending()[1].namespace, "debug");
        assert_eq!(command_queue.pending()[2].namespace, "debug");
        assert_eq!(command_queue.pending()[3].namespace, "dev-shell");
        assert_eq!(event_queue.pending().len(), 1);
        assert_eq!(console_queue.pending().len(), 1);
    }

    #[test]
    fn exposes_world_physics_overlap_queries_to_scripts() {
        let scene = Arc::new(SceneService::default());
        scene.spawn("bullet");
        scene.spawn("asteroid");
        assert!(scene.configure_entity_metadata(
            "asteroid",
            SceneEntityLifecycle::default(),
            vec!["hazard".to_owned()],
            vec!["targets".to_owned()],
            BTreeMap::new(),
        ));
        assert!(scene.set_transform(
            "bullet",
            amigo_math::Transform3 {
                translation: amigo_math::Vec3::new(16.0, 24.0, 0.0),
                ..Default::default()
            },
        ));
        assert!(scene.set_transform(
            "asteroid",
            amigo_math::Transform3 {
                translation: amigo_math::Vec3::new(24.0, 24.0, 0.0),
                ..Default::default()
            },
        ));

        let physics = Arc::new(Physics2dSceneService::default());
        physics.queue_circle_collider(CircleCollider2dCommand {
            entity_id: SceneEntityId::new(0),
            entity_name: "bullet".to_owned(),
            collider: CircleCollider2d {
                radius: 4.0,
                offset: Vec2::ZERO,
            },
        });
        physics.queue_circle_collider(CircleCollider2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "asteroid".to_owned(),
            collider: CircleCollider2d {
                radius: 6.0,
                offset: Vec2::ZERO,
            },
        });

        let runtime = RhaiScriptRuntime::new_with_motion(
            Some(scene),
            None,
            None,
            Some(physics.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "world-physics-test",
                r#"
                    fn update(dt) {
                        if !world.physics.overlaps("bullet", "asteroid") {
                            throw("physics overlap should be true");
                        }

                        if world.physics.overlaps("bullet", "missing") {
                            throw("missing collider should return false");
                        }

                        let hit = world.physics.first_overlap("bullet", ["missing", "asteroid"]);
                        if hit != "asteroid" {
                            throw("first overlap should return asteroid");
                        }

                        let hit_index = world.physics.first_overlap_index("bullet", ["missing", "asteroid"]);
                        if hit_index != 1 {
                            throw("first overlap index should return the candidate index");
                        }

                        let no_hit = world.physics.first_overlap("bullet", ["missing", "ghost"]);
                        if no_hit != "" {
                            throw("first overlap should return empty string when nothing matches");
                        }

                        let no_hit_index = world.physics.first_overlap_index("bullet", ["missing", "ghost"]);
                        if no_hit_index != -1 {
                            throw("first overlap index should return -1 when nothing matches");
                        }

                        if world.physics.first_overlap_by_tag("bullet", "hazard") != "asteroid" {
                            throw("tag selector overlap should return asteroid");
                        }

                        if world.physics.first_overlap_by_group("bullet", "targets") != "asteroid" {
                            throw("group selector overlap should return asteroid");
                        }

                        if world.physics.first_overlap_by_selector("bullet", "tag", "hazard") != "asteroid" {
                            throw("generic selector overlap should return asteroid");
                        }

                        if !world.physics.overlaps_by_tag("bullet", "hazard") {
                            throw("overlaps_by_tag should be true");
                        }

                        if world.physics.selector_candidates("tag", "hazard").len() != 1 {
                            throw("selector candidates should include tagged collider");
                        }

                        if !world.physics.set_circle_radius("asteroid", 16.0) {
                            throw("circle radius setter should succeed");
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("world-physics-test", 1.0 / 60.0)
            .expect("update should succeed");
        assert_eq!(
            physics
                .circle_collider("asteroid")
                .expect("asteroid circle should exist")
                .collider
                .radius,
            16.0
        );
    }

    #[test]
    fn update_function_can_set_vector_polygon_points() {
        let vector_scene = Arc::new(VectorSceneService::default());
        vector_scene.queue(VectorShape2dDrawCommand {
            entity_id: SceneEntityId::new(9),
            entity_name: "playground-2d-asteroids-asteroid-big".to_owned(),
            shape: VectorShape2d {
                kind: VectorShapeKind2d::Polygon {
                    points: vec![
                        Vec2::new(-10.0, -10.0),
                        Vec2::new(0.0, 10.0),
                        Vec2::new(10.0, -10.0),
                    ],
                },
                style: VectorStyle2d::default(),
            },
            z_index: 0.0,
            transform: Transform2::default(),
        });

        let runtime = RhaiScriptRuntime::new_with_motion_and_vector(
            None,
            None,
            Some(vector_scene.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "world-vector-test",
                r#"
                    fn update(dt) {
                        if !world.vector.set_polygon("playground-2d-asteroids-asteroid-big", [[-12.0, -4.0], [0.0, 14.0], [12.0, -6.0], [2.0, -15.0]]) {
                            throw("set_polygon should succeed");
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("world-vector-test", 1.0 / 60.0)
            .expect("update should succeed");

        let commands = vector_scene.commands();
        assert_eq!(commands.len(), 1);
        match &commands[0].shape.kind {
            VectorShapeKind2d::Polygon { points } => {
                assert_eq!(points.len(), 4);
                assert_eq!(points[0], Vec2::new(-12.0, -4.0));
            }
            other => panic!("expected polygon shape, got {other:?}"),
        }
    }

    #[test]
    fn exposes_runtime_catalog_and_diagnostics_to_scripts() {
        let catalog = Arc::new(ModCatalog::from_discovered_mods(vec![
            discovered_mod(
                "core-game",
                &["dev_interface", "console_shell"],
                &["dev-shell", "console"],
            ),
            discovered_mod("playground-2d", &["2d"], &["sprite-lab", "text-lab"]),
        ]));
        let diagnostics = Arc::new(RuntimeDiagnostics::new(
            "winit",
            "winit",
            "wgpu",
            "rhai",
            vec!["core-game".to_owned(), "playground-2d".to_owned()],
            vec!["2d.sprite".to_owned(), "3d.mesh".to_owned()],
            vec![
                "amigo-modding".to_owned(),
                "amigo-scripting-rhai".to_owned(),
            ],
            vec!["amigo_core::RuntimeDiagnostics".to_owned()],
        ));
        let scene = Arc::new(SceneService::default());
        scene.spawn("core-game-shell");
        let assets = Arc::new(AssetCatalog::default());
        let command_queue = Arc::new(ScriptCommandQueue::default());
        let launch_selection = Arc::new(LaunchSelection::new(
            Some("playground-2d".to_owned()),
            Some("sprite-lab".to_owned()),
            vec!["core".to_owned(), "playground-2d".to_owned()],
            true,
        ));
        assets.register_manifest(AssetManifest {
            key: AssetKey::new("playground-2d/textures/sprite-lab"),
            source: AssetSourceKind::Mod("playground-2d".to_owned()),
            tags: vec!["phase3".to_owned(), "2d".to_owned(), "sprite".to_owned()],
        });
        assets.request_load(AssetLoadRequest::new(
            AssetKey::new("playground-2d/textures/sprite-lab"),
            AssetLoadPriority::Immediate,
        ));
        assets.mark_loaded(amigo_assets::LoadedAsset {
            key: AssetKey::new("playground-2d/textures/sprite-lab"),
            source: AssetSourceKind::Mod("playground-2d".to_owned()),
            resolved_path: PathBuf::from("mods/playground-2d/textures/sprite-lab"),
            byte_len: 84,
        });
        let prepared = prepare_debug_placeholder_asset(
            &assets
                .loaded_asset(&AssetKey::new("playground-2d/textures/sprite-lab"))
                .expect("loaded asset should exist"),
            r#"
                kind = "sprite-2d"
                label = "Sprite Lab Placeholder"
                format = "debug-placeholder"
            "#,
        )
        .expect("prepared asset should parse");
        assert_eq!(prepared.kind, PreparedAssetKind::Sprite2d);
        assets.mark_prepared(prepared);
        let runtime = RhaiScriptRuntime::new(
            Some(scene),
            None,
            Some(assets),
            None,
            Some(launch_selection),
            Some(catalog),
            Some(diagnostics),
            Some(command_queue.clone()),
            None,
            None,
        );

        runtime
            .execute(
                "catalog-script",
                r#"
                    let sprite = world.assets.get("playground-2d/textures/sprite-lab");
                    if world.entities.count() != 1 { throw("wrong entity count"); }
                    if !world.assets.has("playground-2d/textures/sprite-lab") { throw("world assets missing key"); }
                    if world.assets.registered().len != 1 { throw("wrong registered asset count"); }
                    if world.assets.by_mod("playground-2d").len != 1 { throw("wrong world mod asset count"); }
                    if world.assets.pending().len != 0 { throw("wrong world pending asset count"); }
                    if world.assets.loaded().len != 1 { throw("wrong world loaded asset count"); }
                    if world.assets.prepared().len != 1 { throw("wrong world prepared asset count"); }
                    if world.assets.failed().len != 0 { throw("wrong world failed asset count"); }
                    if sprite.key() != "playground-2d/textures/sprite-lab" { throw("wrong asset key"); }
                    if !sprite.exists() { throw("asset ref should exist"); }
                    if sprite.state() != "prepared" { throw("wrong asset ref state"); }
                    if sprite.source() != "mod:playground-2d" { throw("wrong asset ref source"); }
                    if sprite.path().len == 0 { throw("missing asset ref path"); }
                    if sprite.kind() != "sprite-2d" { throw("wrong asset ref kind"); }
                    if sprite.label() != "Sprite Lab Placeholder" { throw("wrong asset ref label"); }
                    if sprite.format() != "debug-placeholder" { throw("wrong asset ref format"); }
                    if sprite.tags().len != 3 { throw("wrong asset ref tags"); }
                    if sprite.reason().len != 0 { throw("unexpected asset ref reason"); }
                    if !world.assets.reload("playground-2d/textures/sprite-lab") { throw("failed to queue world asset reload"); }
                    if !sprite.reload() { throw("failed to queue asset ref reload"); }

                    if world.mod.current_id() != "playground-2d" { throw("wrong current mod"); }
                    if world.mod.scenes().len != 2 { throw("wrong world scene count"); }
                    if !world.mod.has_scene("text-lab") { throw("missing world mod scene"); }
                    if world.mod.capabilities().len != 1 { throw("wrong world capability count"); }
                    if world.mod.loaded().len != 2 { throw("wrong world loaded mod count"); }

                    if world.runtime.window_backend() != "winit" { throw("wrong world window backend"); }
                    if world.runtime.input_backend() != "winit" { throw("wrong world input backend"); }
                    if world.runtime.render_backend() != "wgpu" { throw("wrong world render backend"); }
                    if world.runtime.script_backend() != "rhai" { throw("wrong world script backend"); }
                    if world.runtime.capabilities().len != 2 { throw("wrong world runtime capability count"); }
                    if world.runtime.plugins().len != 2 { throw("wrong world runtime plugin count"); }
                    if world.runtime.services().len != 1 { throw("wrong world runtime service count"); }
                    if !world.runtime.dev_mode() { throw("world runtime should be in dev mode"); }
                "#,
            )
            .expect("script should be able to inspect runtime catalog and diagnostics");

        assert_eq!(command_queue.pending().len(), 2);
        assert_eq!(command_queue.pending()[0].namespace, "asset");
        assert_eq!(command_queue.pending()[1].namespace, "asset");
    }

    #[test]
    fn queues_placeholder_script_and_console_messages() {
        let command_queue = Arc::new(ScriptCommandQueue::default());
        let event_queue = Arc::new(ScriptEventQueue::default());
        let console_queue = Arc::new(DevConsoleQueue::default());
        let launch_selection = Arc::new(LaunchSelection::new(
            Some("playground-2d".to_owned()),
            Some("sprite-lab".to_owned()),
            vec!["core".to_owned(), "playground-2d".to_owned()],
            true,
        ));
        let runtime = RhaiScriptRuntime::new(
            None,
            None,
            None,
            None,
            Some(launch_selection),
            None,
            None,
            Some(command_queue.clone()),
            Some(event_queue.clone()),
            Some(console_queue.clone()),
        );

        runtime
            .execute(
                "queue-script",
                r#"
                    world.scene.select("dev-shell");
                    world.scene.reload();
                    world.assets.reload("playground-2d/textures/sprite-lab");
                    world.dev.event("scene.selected", "dev-shell");
                    world.dev.command("help");
                    world.sprite2d.queue("playground-2d-sprite", "playground-2d/textures/sprite-lab", 128, 128);
                    world.text2d.queue("playground-2d-label", "AMIGO 2D", "playground-2d/fonts/debug-ui", 320, 64);
                    world.mesh3d.queue("playground-3d-probe", "playground-3d/meshes/probe");
                    world.material3d.bind("playground-3d-probe", "debug-surface", "playground-3d/materials/debug-surface");
                "#,
            )
            .expect("script should be able to queue placeholder bridge messages");

        assert_eq!(command_queue.pending().len(), 7);
        assert_eq!(event_queue.pending().len(), 1);
        assert_eq!(console_queue.pending().len(), 1);
        assert_eq!(command_queue.pending()[1].namespace, "scene".to_owned());
        assert_eq!(command_queue.pending()[2].namespace, "asset".to_owned());
        assert_eq!(command_queue.pending()[3].namespace, "2d.sprite".to_owned());
        assert_eq!(command_queue.pending()[4].namespace, "2d.text".to_owned());
        assert_eq!(command_queue.pending()[5].namespace, "3d.mesh".to_owned());
        assert_eq!(
            command_queue.pending()[6].namespace,
            "3d.material".to_owned()
        );
    }

    #[test]
    fn queues_world_content_domain_commands() {
        let command_queue = Arc::new(ScriptCommandQueue::default());
        let launch_selection = Arc::new(LaunchSelection::new(
            Some("playground-3d".to_owned()),
            Some("hello-world-cube".to_owned()),
            vec!["core".to_owned(), "playground-3d".to_owned()],
            true,
        ));
        let runtime = RhaiScriptRuntime::new(
            None,
            None,
            None,
            None,
            Some(launch_selection),
            None,
            None,
            Some(command_queue.clone()),
            None,
            None,
        );

        runtime
            .execute(
                "world-content-script",
                r#"
                    world.sprite2d.queue("playground-2d-sprite", "playground-2d/textures/sprite-lab", 128, 128);
                    world.text2d.queue("playground-2d-label", "AMIGO 2D", "playground-2d/fonts/debug-ui", 320, 64);
                    world.mesh3d.queue("playground-3d-probe", "playground-3d/meshes/probe");
                    world.material3d.bind("playground-3d-probe", "debug-surface", "playground-3d/materials/debug-surface");
                    world.text3d.queue("playground-3d-hello", "HELLO WORLD", "playground-3d/fonts/debug-3d", 0.5);
                    world.dev.refresh_diagnostics("playground-3d");
                "#,
            )
            .expect("script should be able to queue world content domain commands");

        assert_eq!(command_queue.pending().len(), 6);
        assert_eq!(command_queue.pending()[0].namespace, "2d.sprite".to_owned());
        assert_eq!(command_queue.pending()[1].namespace, "2d.text".to_owned());
        assert_eq!(command_queue.pending()[2].namespace, "3d.mesh".to_owned());
        assert_eq!(
            command_queue.pending()[3].namespace,
            "3d.material".to_owned()
        );
        assert_eq!(command_queue.pending()[4].namespace, "3d.text".to_owned());
        assert_eq!(command_queue.pending()[5].namespace, "dev-shell".to_owned());
    }

    #[test]
    fn queues_world_ui_commands() {
        let command_queue = Arc::new(ScriptCommandQueue::default());
        let runtime = RhaiScriptRuntime::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(command_queue.clone()),
            None,
            None,
        );

        runtime
            .execute(
                "world-ui-script",
                r#"
                    if !world.ui.set_text("playground-2d-ui-preview.subtitle", "Updated from Rhai") {
                        throw("set_text should queue a command");
                    }
                    if !world.ui.set_value("playground-2d-ui-preview.hp-bar", 0.5) {
                        throw("set_value should queue a command");
                    }
                    if !world.ui.show("playground-2d-ui-preview.root") {
                        throw("show should queue a command");
                    }
                    if !world.ui.hide("playground-2d-ui-preview.root") {
                        throw("hide should queue a command");
                    }
                    if !world.ui.enable("playground-2d-ui-preview.root.control-card.button-row.repair-button") {
                        throw("enable should queue a command");
                    }
                    if !world.ui.disable("playground-2d-ui-preview.root.control-card.button-row.repair-button") {
                        throw("disable should queue a command");
                    }
                    let hud = #{};
                    hud["playground-2d-ui-preview.score"] = "Score: 10";
                    hud["playground-2d-ui-preview.status"] = "Ready";
                    if world.ui.set_many(hud) != 2 {
                        throw("set_many should queue two commands");
                    }
                "#,
            )
            .expect("script should be able to queue world ui commands");

        assert_eq!(command_queue.pending().len(), 8);
        assert_eq!(command_queue.pending()[0].namespace, "ui".to_owned());
        assert_eq!(command_queue.pending()[0].name, "set-text".to_owned());
        assert_eq!(
            command_queue.pending()[0].arguments,
            vec![
                "playground-2d-ui-preview.subtitle".to_owned(),
                "Updated from Rhai".to_owned(),
            ]
        );
        assert_eq!(command_queue.pending()[1].name, "set-value".to_owned());
        assert_eq!(
            command_queue.pending()[1].arguments,
            vec![
                "playground-2d-ui-preview.hp-bar".to_owned(),
                "0.5".to_owned(),
            ]
        );
        assert_eq!(command_queue.pending()[2].name, "show".to_owned());
        assert_eq!(command_queue.pending()[3].name, "hide".to_owned());
        assert_eq!(command_queue.pending()[4].name, "enable".to_owned());
        assert_eq!(command_queue.pending()[5].name, "disable".to_owned());
        assert_eq!(command_queue.pending()[6].name, "set-text".to_owned());
        assert_eq!(
            command_queue.pending()[6].arguments,
            vec![
                "playground-2d-ui-preview.score".to_owned(),
                "Score: 10".to_owned(),
            ]
        );
        assert_eq!(command_queue.pending()[7].name, "set-text".to_owned());
        assert_eq!(
            command_queue.pending()[7].arguments,
            vec![
                "playground-2d-ui-preview.status".to_owned(),
                "Ready".to_owned(),
            ]
        );
    }

    #[test]
    fn queues_world_audio_commands() {
        let command_queue = Arc::new(ScriptCommandQueue::default());
        let runtime = RhaiScriptRuntime::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(command_queue.clone()),
            None,
            None,
        );

        runtime
            .execute(
                "world-audio-script",
                r#"
                    if !world.audio.preload("jump") { throw("preload should queue"); }
                    if !world.audio.play("jump") { throw("play should queue"); }
                    if !world.audio.play_asset("playground-sidescroller/audio/coin") { throw("play_asset should queue"); }
                    if !world.audio.start_realtime("proximity-beep") { throw("start_realtime should queue"); }
                    if !world.audio.set_param("proximity-beep", "distance", 128.0) { throw("set_param should queue"); }
                    if !world.audio.set_volume("master", 0.75) { throw("set_volume should queue"); }
                    if !world.audio.stop("proximity-beep") { throw("stop should queue"); }
                "#,
            )
            .expect("script should be able to queue world audio commands");

        assert_eq!(command_queue.pending().len(), 7);
        assert_eq!(
            command_queue.pending()[0],
            ScriptCommand::audio_preload("jump")
        );
        assert_eq!(
            command_queue.pending()[1],
            ScriptCommand::audio_play("jump")
        );
        assert_eq!(
            command_queue.pending()[2],
            ScriptCommand::audio_play_asset("playground-sidescroller/audio/coin")
        );
        assert_eq!(
            command_queue.pending()[3],
            ScriptCommand::audio_start_realtime("proximity-beep")
        );
        assert_eq!(
            command_queue.pending()[4],
            ScriptCommand::audio_set_param("proximity-beep", "distance", 128.0)
        );
        assert_eq!(
            command_queue.pending()[5],
            ScriptCommand::audio_set_volume("master", 0.75)
        );
        assert_eq!(
            command_queue.pending()[6],
            ScriptCommand::audio_stop("proximity-beep")
        );
    }

    #[test]
    fn rejects_invalid_world_ui_commands() {
        let command_queue = Arc::new(ScriptCommandQueue::default());
        let runtime = RhaiScriptRuntime::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(command_queue.clone()),
            None,
            None,
        );

        runtime
            .execute(
                "world-ui-invalid-script",
                r#"
                    if world.ui.set_text("", "Updated from Rhai") { throw("empty path should fail"); }
                    if world.ui.show("") { throw("empty show path should fail"); }
                    if world.ui.hide("") { throw("empty hide path should fail"); }
                    if world.ui.enable("") { throw("empty enable path should fail"); }
                    if world.ui.disable("") { throw("empty disable path should fail"); }
                    let hud = #{};
                    hud[""] = "empty path";
                    if world.ui.set_many(hud) != 0 { throw("empty set_many path should fail"); }
                "#,
            )
            .expect("invalid ui script should still execute");

        assert!(
            command_queue.pending().is_empty(),
            "invalid ui commands should not enqueue anything"
        );
    }

    #[test]
    fn update_function_can_rotate_scene_entities() {
        let scene = Arc::new(SceneService::default());
        scene.spawn("playground-2d-square");
        scene.spawn("playground-3d-cube");
        let input = Arc::new(InputState::default());
        input.set_key(KeyCode::Left, true);
        let runtime = RhaiScriptRuntime::new(
            Some(scene.clone()),
            None,
            None,
            Some(input),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "rotate-test",
                r#"
                    fn update(dt) {
                        let square = world.entities.named("playground-2d-square");
                        let cube = world.entities.named("playground-3d-cube");

                        if world.input.down("ArrowLeft") {
                            let square_rotated = square.rotate_2d(dt);
                            let cube_rotated = cube.rotate_3d(dt, dt * 2.0, 0.0);
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("rotate-test", 1.0)
            .expect("update function should succeed");

        assert_eq!(
            scene
                .transform_of("playground-2d-square")
                .expect("2d entity should exist")
                .rotation_euler
                .z,
            1.0
        );
        let cube = scene
            .transform_of("playground-3d-cube")
            .expect("3d entity should exist");
        assert_eq!(cube.rotation_euler.x, 1.0);
        assert_eq!(cube.rotation_euler.y, 2.0);
    }

    #[test]
    fn update_function_can_use_world_input_and_entity_refs() {
        let scene = Arc::new(SceneService::default());
        scene.spawn("playground-2d-square");
        let input = Arc::new(InputState::default());
        input.set_key(KeyCode::Right, true);
        let runtime = RhaiScriptRuntime::new(
            Some(scene.clone()),
            None,
            None,
            Some(input),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "world-update-test",
                r#"
                    fn update(dt) {
                        let square = world.entities.named("playground-2d-square");

                        if world.input.down("ArrowRight") {
                            let applied = square.rotate_2d(dt);
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("world-update-test", 0.5)
            .expect("update function should succeed");

        assert_eq!(
            scene
                .transform_of("playground-2d-square")
                .expect("2d entity should exist")
                .rotation_euler
                .z,
            0.5
        );
    }

    #[test]
    fn update_function_can_drive_motion_controller_and_read_state() {
        let motion_scene = Arc::new(Motion2dSceneService::default());
        motion_scene.queue_motion_controller(MotionController2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "playground-sidescroller-player".to_owned(),
            controller: MotionController2d {
                params: MotionProfile2d {
                    max_speed: 180.0,
                    acceleration: 900.0,
                    deceleration: 1200.0,
                    air_acceleration: 500.0,
                    gravity: 900.0,
                    jump_velocity: -360.0,
                    terminal_velocity: 720.0,
                },
            },
        });
        assert!(motion_scene.sync_motion_state(
            "playground-sidescroller-player",
            MotionState2d {
                grounded: true,
                facing: Facing2d::Right,
                animation: MotionAnimationState::Run,
                velocity: Vec2::new(12.0, -4.0),
            }
        ));

        let runtime = RhaiScriptRuntime::new_with_motion(
            None,
            None,
            Some(motion_scene.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "world-motion-controller-test",
                r#"
                    fn update(dt) {
                        let state = world.motion.state("playground-sidescroller-player");
                        if !state.grounded { throw("state should expose grounded"); }
                        if state.facing != "right" { throw("state should expose facing"); }
                        if state.animation != "run" { throw("state should expose animation"); }
                        if state.velocity_x < 10.0 { throw("state should expose velocity_x"); }
                        if !world.motion.drive("playground-sidescroller-player", -1.0, true, false, dt) {
                            throw("drive should succeed");
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("world-motion-controller-test", 1.0 / 60.0)
            .expect("update should succeed");

        assert_eq!(
            motion_scene.motion_intent("playground-sidescroller-player"),
            Some(MotionIntent2d {
                move_x: -1.0,
                jump_pressed: true,
                jump_held: false,
            })
        );
    }

    #[test]
    fn update_function_can_drive_motion_alias_and_read_state() {
        let motion_scene = Arc::new(Motion2dSceneService::default());
        motion_scene.queue_motion_controller(MotionController2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "playground-sidescroller-player".to_owned(),
            controller: MotionController2d {
                params: MotionProfile2d {
                    max_speed: 180.0,
                    acceleration: 900.0,
                    deceleration: 1200.0,
                    air_acceleration: 500.0,
                    gravity: 900.0,
                    jump_velocity: -360.0,
                    terminal_velocity: 720.0,
                },
            },
        });
        assert!(motion_scene.sync_motion_state(
            "playground-sidescroller-player",
            MotionState2d {
                grounded: true,
                facing: Facing2d::Right,
                animation: MotionAnimationState::Run,
                velocity: Vec2::new(12.0, -4.0),
            }
        ));

        let runtime = RhaiScriptRuntime::new_with_motion(
            None,
            None,
            Some(motion_scene.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "world-motion-test",
                r#"
                    fn update(dt) {
                        let state = world.motion.state("playground-sidescroller-player");
                        if !state.grounded { throw("state should expose grounded"); }
                        if state.facing != "right" { throw("state should expose facing"); }
                        if state.animation != "run" { throw("state should expose animation"); }
                        if state.velocity_x < 10.0 { throw("state should expose velocity_x"); }
                        if !world.motion.drive("playground-sidescroller-player", 1.0, false, true, dt) {
                            throw("drive should succeed");
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("world-motion-test", 1.0 / 60.0)
            .expect("update should succeed");

        assert_eq!(
            motion_scene.motion_intent("playground-sidescroller-player"),
            Some(MotionIntent2d {
                move_x: 1.0,
                jump_pressed: false,
                jump_held: true,
            })
        );
    }

    #[test]
    fn projectiles_fire_from_activates_pooled_projectile() {
        let scene = Arc::new(SceneService::default());
        let motion_scene = Arc::new(Motion2dSceneService::default());
        let pool_scene = Arc::new(EntityPoolSceneService::default());

        scene.spawn_with_transform(
            "player",
            Transform3 {
                translation: Vec3::new(10.0, 20.0, 0.0),
                rotation_euler: Vec3::new(0.0, 0.0, 0.0),
                ..Transform3::default()
            },
        );
        scene.spawn_with_transform(
            "bullet-a",
            Transform3 {
                translation: Vec3::new(-100.0, -100.0, 0.0),
                ..Transform3::default()
            },
        );
        let _ = scene.set_visible("bullet-a", false);
        let _ = scene.set_simulation_enabled("bullet-a", false);
        let _ = scene.set_collision_enabled("bullet-a", false);

        pool_scene.queue(EntityPoolSceneCommand::new(
            "test",
            "bullets",
            vec!["bullet-a".to_owned()],
        ));
        motion_scene.queue_projectile_emitter(ProjectileEmitter2dCommand {
            entity_id: SceneEntityId::new(3),
            entity_name: "player-gun".to_owned(),
            emitter: ProjectileEmitter2d {
                pool: "bullets".to_owned(),
                speed: 100.0,
                spawn_offset: Vec2::new(5.0, 0.0),
                inherit_velocity_scale: 0.0,
            },
        });

        let runtime = RhaiScriptRuntime::new_with_services(
            Some(scene.clone()),
            None,
            None,
            Some(motion_scene),
            None,
            None,
            Some(pool_scene.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "projectile-fire-test",
                r#"
                    if !world.projectiles.fire_from("player-gun", "player") {
                        throw("fire_from should activate projectile");
                    }
                    if world.pools.active_members("bullets").len != 1 {
                        throw("pool should report active projectile");
                    }
                    if world.pools.active_count("bullets") != 1 {
                        throw("pool should report active count");
                    }
                "#,
            )
            .expect("script execution should succeed");

        assert!(scene.is_visible("bullet-a"));
        assert!(scene.is_simulation_enabled("bullet-a"));
        assert!(scene.is_collision_enabled("bullet-a"));
        assert_eq!(
            pool_scene.active_members("bullets"),
            vec!["bullet-a".to_owned()]
        );
        let bullet_transform = scene
            .transform_of("bullet-a")
            .expect("projectile should have a transform");
        assert_eq!(bullet_transform.translation, Vec3::new(15.0, 20.0, 0.0));
    }

    #[test]
    fn projectiles_release_returns_pooled_projectile_without_teleporting() {
        let scene = Arc::new(SceneService::default());
        let pool_scene = Arc::new(EntityPoolSceneService::default());
        let projectile_transform = Transform3 {
            translation: Vec3::new(42.0, -7.0, 0.0),
            ..Transform3::default()
        };
        scene.spawn_with_transform("bullet-a", projectile_transform);
        pool_scene.queue(EntityPoolSceneCommand::new(
            "test",
            "bullets",
            vec!["bullet-a".to_owned()],
        ));
        assert_eq!(
            pool_scene.acquire(&scene, "bullets"),
            Some("bullet-a".to_owned())
        );

        let runtime = RhaiScriptRuntime::new_with_services(
            Some(scene.clone()),
            None,
            None,
            None,
            None,
            None,
            Some(pool_scene.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "projectile-release-test",
                r#"
                    if !world.projectiles.release("bullets", "bullet-a") {
                        throw("release should return active projectile");
                    }
                    if world.pools.active_members("bullets").len != 0 {
                        throw("pool should no longer report active projectile");
                    }
                    if world.pools.active_count("bullets") != 0 {
                        throw("pool active count should be zero after release");
                    }
                    if world.pools.acquire("bullets") != "bullet-a" {
                        throw("pool acquire should reuse released projectile");
                    }
                    if world.pools.release_all("bullets") != 1 {
                        throw("release_all should release one projectile");
                    }
                "#,
            )
            .expect("script execution should succeed");

        assert!(!scene.is_visible("bullet-a"));
        assert!(!scene.is_simulation_enabled("bullet-a"));
        assert!(!scene.is_collision_enabled("bullet-a"));
        assert_eq!(scene.transform_of("bullet-a"), Some(projectile_transform));
        assert!(pool_scene.active_members("bullets").is_empty());
    }

    #[test]
    fn call_update_does_not_rerun_top_level_script_body() {
        let scene = Arc::new(SceneService::default());
        let runtime = RhaiScriptRuntime::new(
            Some(scene.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "bootstrap-once",
                r#"
                    world.entities.create("boot-only");

                    fn update(dt) {
                    }
                "#,
            )
            .expect("script execution should succeed");

        assert_eq!(scene.entity_count(), 1);

        runtime
            .call_update("bootstrap-once", 1.0 / 60.0)
            .expect("update function should succeed");

        assert_eq!(
            scene.entity_count(),
            1,
            "top-level script body should not be re-evaluated during update ticks"
        );
    }

    #[test]
    fn unload_removes_script_from_registry() {
        let scene = Arc::new(SceneService::default());
        scene.spawn("playground-2d-square");
        let runtime = RhaiScriptRuntime::new(
            Some(scene.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "unload-test",
                r#"
                    fn update(dt) {
                        let square = world.entities.named("playground-2d-square");
                        let applied = square.rotate_2d(dt);
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("unload-test", 1.0)
            .expect("update should run before unload");

        assert_eq!(
            scene
                .transform_of("playground-2d-square")
                .expect("entity should exist")
                .rotation_euler
                .z,
            1.0
        );

        runtime
            .unload("unload-test")
            .expect("unload should succeed");
        runtime
            .call_update("unload-test", 1.0)
            .expect("update on unloaded source should be a no-op");

        assert_eq!(
            scene
                .transform_of("playground-2d-square")
                .expect("entity should still exist")
                .rotation_euler
                .z,
            1.0,
            "unloaded script should no longer receive updates"
        );
    }

    #[test]
    fn can_reexecute_source_after_unload() {
        let scene = Arc::new(SceneService::default());
        scene.spawn("playground-2d-square");
        let runtime = RhaiScriptRuntime::new(
            Some(scene.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "reloadable-script",
                r#"
                    fn update(dt) {
                        let square = world.entities.named("playground-2d-square");
                        let applied = square.rotate_2d(dt);
                    }
                "#,
            )
            .expect("first script execution should succeed");
        runtime
            .call_update("reloadable-script", 1.0)
            .expect("first update should succeed");
        runtime
            .unload("reloadable-script")
            .expect("unload should succeed");
        runtime
            .execute(
                "reloadable-script",
                r#"
                    fn update(dt) {
                        let square = world.entities.named("playground-2d-square");
                        let applied = square.rotate_2d(dt * 2.0);
                    }
                "#,
            )
            .expect("second script execution should succeed");
        runtime
            .call_update("reloadable-script", 1.0)
            .expect("second update should succeed");

        assert_eq!(
            scene
                .transform_of("playground-2d-square")
                .expect("entity should exist")
                .rotation_euler
                .z,
            3.0,
            "re-executed script should be registered again under the same source name"
        );
    }

    #[test]
    fn update_function_can_advance_sprite_animation_frames() {
        let sprite_scene = Arc::new(SpriteSceneService::default());
        sprite_scene.queue(SpriteDrawCommand {
            entity_id: SceneEntityId::new(17),
            entity_name: "playground-2d-spritesheet".to_owned(),
            sprite: Sprite {
                texture: AssetKey::new("playground-2d/textures/hello-world-spritesheet"),
                size: Vec2::new(256.0, 128.0),
                sheet: Some(SpriteSheet {
                    columns: 4,
                    rows: 2,
                    frame_count: 8,
                    frame_size: Vec2::new(32.0, 32.0),
                    fps: 8.0,
                    looping: true,
                }),
                sheet_is_explicit: true,
                animation_override: None,
                frame_index: 0,
                frame_elapsed: 0.0,
            },
            transform: Transform2::default(),
            z_index: 0.0,
        });

        let runtime = RhaiScriptRuntime::new(
            None,
            Some(sprite_scene.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "sprite-animation-test",
                r#"
                    fn update(dt) {
                        let advanced = world.sprite2d.advance("playground-2d-spritesheet", dt);
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("sprite-animation-test", 0.25)
            .expect("update function should succeed");

        assert_eq!(sprite_scene.frame_of("playground-2d-spritesheet"), Some(2));
    }

    #[test]
    fn lifecycle_hooks_can_use_world_time_and_dev_domains() {
        let console_queue = Arc::new(DevConsoleQueue::default());
        let runtime = RhaiScriptRuntime::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(console_queue.clone()),
        );

        runtime
            .execute(
                "lifecycle-test",
                r#"
                    fn on_enter() {
                        if world.time.frame() != 0 { throw("on_enter should not advance frames"); }
                        if world.time.delta() != 0.0 { throw("on_enter should have zero delta"); }
                        world.dev.command("enter");
                    }

                    fn update(dt) {
                        if world.time.frame() < 1 { throw("update should advance frames"); }
                        if world.time.delta() <= 0.0 { throw("update should expose delta"); }
                        if world.time.frame() == 2 && world.time.elapsed() < 0.75 { throw("elapsed time should accumulate"); }
                        world.dev.command("tick");
                    }

                    fn on_event(topic, payload) {
                        if topic != "demo.event" { throw("unexpected event topic"); }
                        if payload.len != 2 { throw("unexpected payload length"); }
                        if world.time.delta() != 0.0 { throw("on_event should be passive"); }
                        world.dev.command("event");
                    }

                    fn on_exit() {
                        if world.time.delta() != 0.0 { throw("on_exit should be passive"); }
                        world.dev.command("exit");
                    }
                "#,
            )
            .expect("script execution should succeed");

        runtime
            .call_on_enter("lifecycle-test")
            .expect("on_enter should succeed");
        runtime
            .call_update("lifecycle-test", 0.25)
            .expect("first update should succeed");
        runtime
            .call_update("lifecycle-test", 0.50)
            .expect("second update should succeed");
        runtime
            .call_on_event(
                "lifecycle-test",
                "demo.event",
                &["one".to_owned(), "two".to_owned()],
            )
            .expect("on_event should succeed");
        runtime
            .call_on_exit("lifecycle-test")
            .expect("on_exit should succeed");

        assert_eq!(console_queue.pending().len(), 5);
        assert_eq!(console_queue.pending()[0].line, "enter".to_owned());
        assert_eq!(console_queue.pending()[1].line, "tick".to_owned());
        assert_eq!(console_queue.pending()[2].line, "tick".to_owned());
        assert_eq!(console_queue.pending()[3].line, "event".to_owned());
        assert_eq!(console_queue.pending()[4].line, "exit".to_owned());
    }

    #[test]
    fn exposes_scene_state_and_timers_to_scripts() {
        let state = Arc::new(SceneStateService::default());
        let session = Arc::new(SessionStateService::default());
        let timers = Arc::new(SceneTimerService::default());
        let runtime = RhaiScriptRuntime::new_with_services(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(state.clone()),
            Some(session.clone()),
            Some(timers.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "state-and-timers-test",
                r#"
                    if !world.state.set_int("score", 10) { throw("set_int failed"); }
                    if world.state.add_int("score", 5) != 15 { throw("add_int failed"); }
                    if world.state.get_int("score") != 15 { throw("get_int failed"); }

                    if !world.state.set_float("speed", 1.5) { throw("set_float failed"); }
                    if world.state.add_float("speed", 0.25) != 1.75 { throw("add_float failed"); }
                    if world.state.get_float("speed") != 1.75 { throw("get_float failed"); }

                    if !world.state.set_bool("armed", false) { throw("set_bool failed"); }
                    if !world.state.add_bool("armed", true) { throw("add_bool failed"); }
                    if !world.state.get_bool("armed") { throw("get_bool failed"); }

                    if !world.state.set_string("label", "wave") { throw("set_string failed"); }
                    if world.state.add_string("label", " 1") != "wave 1" { throw("add_string failed"); }
                    if world.state.get_string("label") != "wave 1" { throw("get_string failed"); }

                    if !world.session.set_bool("asteroids.low_mode", true) { throw("session set_bool failed"); }
                    if !world.session.get_bool("asteroids.low_mode") { throw("session get_bool failed"); }
                    if !world.session.set_int("asteroids.highscore.1", 10000) { throw("session set_int failed"); }
                    if world.session.add_int("asteroids.highscore.1", 250) != 10250 { throw("session add_int failed"); }

                    if !world.timers.start("cooldown", 0.5) { throw("timer start failed"); }
                    if !world.timers.active("cooldown") { throw("timer should be active"); }
                    if world.timers.ready("cooldown") { throw("timer should not be ready"); }

                    fn update(dt) {
                        if world.time.frame() == 1 && !world.timers.ready("cooldown") {
                            throw("timer should be ready after runtime tick");
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");

        runtime
            .call_update("state-and-timers-test", 0.5)
            .expect("update should tick timers before script update");

        assert_eq!(state.get_int("score"), Some(15));
        assert_eq!(session.get_bool("asteroids.low_mode"), Some(true));
        assert_eq!(session.get_int("asteroids.highscore.1"), Some(10_250));
        assert!(timers.ready("cooldown"));
    }

    #[test]
    fn script_can_control_particle_emitter() {
        let particles = Arc::new(Particle2dSceneService::default());
        particles.queue_emitter(ParticleEmitter2dCommand {
            entity_id: SceneEntityId::new(44),
            entity_name: "thruster".to_owned(),
            emitter: ParticleEmitter2d {
                attached_to: None,
                local_offset: Vec2::ZERO,
                local_direction_radians: 0.0,
                spawn_area: amigo_2d_particles::ParticleSpawnArea2d::Point,
                active: false,
                spawn_rate: 10.0,
                max_particles: 16,
                particle_lifetime: 1.0,
                lifetime_jitter: 0.0,
                initial_speed: 0.0,
                speed_jitter: 0.0,
                spread_radians: 0.0,
                inherit_parent_velocity: 0.0,
                initial_size: 1.0,
                final_size: 1.0,
                color: ColorRgba::WHITE,
                color_ramp: None,
                z_index: 1.0,
                shape: ParticleShape2d::Circle { segments: 8 },
                emission_rate_curve: amigo_math::Curve1d::Constant(1.0),
                size_curve: amigo_math::Curve1d::Constant(1.0),
                alpha_curve: amigo_math::Curve1d::Constant(1.0),
                speed_curve: amigo_math::Curve1d::Constant(1.0),
                forces: Vec::new(),
            },
        });
        let runtime = RhaiScriptRuntime::new_with_services(
            None,
            None,
            None,
            None,
            Some(particles.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "particles-test",
                r#"
                    fn update(dt) {
                        world.particles.copy_config("thruster", "thruster");
                        if !world.particles.set_active("thruster", true) {
                            throw("expected particle emitter to exist");
                        }
                        world.particles.set_intensity("thruster", 0.75);
                        world.particles.set_gravity("thruster", 0.0, -120.0);
                        world.particles.set_drag("thruster", 0.5);
                        world.particles.set_wind("thruster", 20.0, 0.0, 0.25);
                        world.particles.set_max_particles("thruster", 12);
                        world.particles.set_spawn_area_rect("thruster", 20.0, 10.0);
                        world.particles.set_spawn_area_line("thruster", 18.0);
                        world.particles.set_spawn_area_ring("thruster", 4.0, 12.0);
                        world.particles.set_shape_line("thruster", 11.0);
                        world.particles.set_color_ramp4(
                            "thruster",
                            "linear_rgb",
                            0.0, "FFFFFFFF",
                            0.33, "39D7FFFF",
                            0.66, "246DFFFF",
                            1.0, "00000000"
                        );
                        world.particles.burst("thruster", 3);
                        world.particles.burst_at("thruster", 12.0, -8.0, 2);
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("particles-test", 1.0 / 60.0)
            .expect("update should succeed");

        assert!(particles.is_active("thruster"));
        assert_eq!(particles.intensity("thruster"), 0.75);
        assert_eq!(particles.particle_count("thruster"), 0);
        let emitter = particles.emitter("thruster").expect("emitter should exist");
        assert_eq!(emitter.emitter.max_particles, 12);
        assert_eq!(emitter.emitter.forces.len(), 3);
        assert_eq!(
            emitter.emitter.shape,
            ParticleShape2d::Line { length: 11.0 }
        );
        assert!(emitter.emitter.color_ramp.is_some());
    }

    #[test]
    fn script_can_switch_ui_theme() {
        let themes = Arc::new(UiThemeService::default());
        themes.register_theme(UiTheme::from_palette(
            "space_dark",
            UiThemePalette {
                background: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
                surface: ColorRgba::new(0.1, 0.1, 0.15, 1.0),
                surface_alt: ColorRgba::new(0.15, 0.15, 0.2, 1.0),
                text: ColorRgba::WHITE,
                text_muted: ColorRgba::new(0.6, 0.7, 0.8, 1.0),
                border: ColorRgba::new(0.2, 0.4, 0.6, 1.0),
                accent: ColorRgba::new(0.0, 0.8, 1.0, 1.0),
                accent_text: ColorRgba::new(0.0, 0.05, 0.08, 1.0),
                danger: ColorRgba::new(1.0, 0.1, 0.2, 1.0),
                warning: ColorRgba::new(1.0, 0.7, 0.0, 1.0),
                success: ColorRgba::new(0.2, 1.0, 0.5, 1.0),
            },
        ));
        let runtime = RhaiScriptRuntime::new_with_services_and_ui_theme(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(themes.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "ui-theme-test",
                r#"
                    if !world.ui.set_theme("space_dark") {
                        throw("theme should switch");
                    }
                    if world.ui.theme() != "space_dark" {
                        throw("theme should be readable");
                    }
                "#,
            )
            .expect("script execution should succeed");

        assert_eq!(themes.active_theme_id().as_deref(), Some("space_dark"));
    }

    #[test]
    fn timers_after_can_be_driven_by_script_tick_and_reset() {
        let timers = Arc::new(SceneTimerService::default());
        let runtime = RhaiScriptRuntime::new_with_services(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(timers.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "timer-after-test",
                r#"
                    if world.timers.after("spawn", 0.25) { throw("after should start pending"); }
                    world.timers.tick(0.25);
                    if !world.timers.after("spawn", 0.25) { throw("after should fire once"); }
                    if world.timers.active("spawn") { throw("after should consume timer"); }
                    if !world.timers.start("reset-me", 1.0) { throw("start reset timer failed"); }
                    world.timers.reset_scene();
                    if world.timers.active("reset-me") { throw("reset should clear scene timers"); }
                "#,
            )
            .expect("script execution should succeed");

        assert!(!timers.active("spawn"));
        assert!(!timers.active("reset-me"));
    }

    fn discovered_mod(id: &str, capabilities: &[&str], scenes: &[&str]) -> DiscoveredMod {
        DiscoveredMod {
            manifest: ModManifest {
                id: id.to_owned(),
                name: id.to_owned(),
                version: "0.1.0".to_owned(),
                description: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                capabilities: capabilities
                    .iter()
                    .map(|capability| (*capability).to_owned())
                    .collect(),
                scripting: None,
                scenes: scenes
                    .iter()
                    .map(|scene_id| ModSceneManifest {
                        id: (*scene_id).to_owned(),
                        label: scene_id.to_string(),
                        description: None,
                        path: format!("scenes/{scene_id}"),
                        document: None,
                        script: None,
                        launcher_visible: true,
                    })
                    .collect(),
            },
            root_path: PathBuf::from(format!("mods/{id}")),
        }
    }
}
