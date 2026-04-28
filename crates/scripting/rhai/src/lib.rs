mod bindings;
mod handles;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use amigo_2d_platformer::PlatformerSceneService;
use amigo_2d_sprite::SpriteSceneService;
use amigo_assets::AssetCatalog;
use amigo_core::{AmigoError, AmigoResult, LaunchSelection, RuntimeDiagnostics};
use amigo_input_api::InputState;
use amigo_modding::ModCatalog;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::SceneService;
use amigo_scripting_api::{
    DevConsoleQueue, DevConsoleState, ScriptCommandQueue, ScriptEventQueue, ScriptLifecycleState,
    ScriptRuntime, ScriptRuntimeInfo, ScriptRuntimeService,
};
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
        Self::new_with_platformer(
            scene,
            sprite_scene,
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

    pub fn new_with_platformer(
        scene: Option<Arc<SceneService>>,
        sprite_scene: Option<Arc<SpriteSceneService>>,
        platformer_scene: Option<Arc<PlatformerSceneService>>,
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
        let world = WorldApi::new(
            scene.clone(),
            sprite_scene.clone(),
            platformer_scene.clone(),
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
            world,
        }
    }

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
        let platformer_scene = registry.resolve::<PlatformerSceneService>();
        let asset_catalog = registry.resolve::<AssetCatalog>();
        let input_state = registry.resolve::<InputState>();
        let launch_selection = registry.resolve::<LaunchSelection>();
        let mod_catalog = registry.resolve::<ModCatalog>();
        let diagnostics = registry.resolve::<RuntimeDiagnostics>();
        let command_queue = registry.resolve::<ScriptCommandQueue>();
        let event_queue = registry.resolve::<ScriptEventQueue>();
        let console_queue = registry.resolve::<DevConsoleQueue>();
        let runtime = RhaiScriptRuntime::new_with_platformer(
            scene,
            sprite_scene,
            platformer_scene,
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
    register_world_api(&mut engine);
    engine
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use amigo_2d_platformer::{
        PlatformerAnimationState, PlatformerController2d, PlatformerController2dCommand,
        PlatformerControllerParams, PlatformerControllerState, PlatformerFacing, PlatformerMotor2d,
        PlatformerSceneService,
    };
    use amigo_2d_sprite::{Sprite, SpriteDrawCommand, SpriteSceneService, SpriteSheet};
    use amigo_assets::{
        AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest,
        AssetSourceKind, PreparedAssetKind, prepare_debug_placeholder_asset,
    };
    use amigo_core::{LaunchSelection, RuntimeDiagnostics};
    use amigo_input_api::{InputState, KeyCode};
    use amigo_math::{Transform2, Vec2};
    use amigo_modding::{DiscoveredMod, ModCatalog, ModManifest, ModSceneManifest};
    use amigo_scene::{SceneEntityId, SceneKey, SceneService};
    use amigo_scripting_api::{
        DevConsoleQueue, ScriptCommand, ScriptCommandQueue, ScriptEventQueue,
    };

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
                    if world.input.keys().len != 2 { throw("wrong pressed key count"); }

                    square.rotate_2d(1.0);
                    if !world.entities.set_position_2d("playground-2d-square", 12.0, 34.0) {
                        throw("failed to set position through world.entities");
                    }
                    if !square.set_position_2d(56.0, 78.0) {
                        throw("failed to set position through entity ref");
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
                "#,
            )
            .expect("script should be able to queue world ui commands");

        assert_eq!(command_queue.pending().len(), 6);
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
                    if !world.audio.play("jump") { throw("play should queue"); }
                    if !world.audio.play_asset("playground-sidescroller/audio/coin") { throw("play_asset should queue"); }
                    if !world.audio.start_realtime("proximity-beep") { throw("start_realtime should queue"); }
                    if !world.audio.set_param("proximity-beep", "distance", 128.0) { throw("set_param should queue"); }
                    if !world.audio.set_volume("master", 0.75) { throw("set_volume should queue"); }
                    if !world.audio.stop("proximity-beep") { throw("stop should queue"); }
                "#,
            )
            .expect("script should be able to queue world audio commands");

        assert_eq!(command_queue.pending().len(), 6);
        assert_eq!(
            command_queue.pending()[0],
            ScriptCommand::audio_play("jump")
        );
        assert_eq!(
            command_queue.pending()[1],
            ScriptCommand::audio_play_asset("playground-sidescroller/audio/coin")
        );
        assert_eq!(
            command_queue.pending()[2],
            ScriptCommand::audio_start_realtime("proximity-beep")
        );
        assert_eq!(
            command_queue.pending()[3],
            ScriptCommand::audio_set_param("proximity-beep", "distance", 128.0)
        );
        assert_eq!(
            command_queue.pending()[4],
            ScriptCommand::audio_set_volume("master", 0.75)
        );
        assert_eq!(
            command_queue.pending()[5],
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
    fn update_function_can_drive_platformer_controller_and_read_state() {
        let platformer_scene = Arc::new(PlatformerSceneService::default());
        platformer_scene.queue(PlatformerController2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "playground-sidescroller-player".to_owned(),
            controller: PlatformerController2d {
                params: PlatformerControllerParams {
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
        assert!(platformer_scene.sync_state(
            "playground-sidescroller-player",
            PlatformerControllerState {
                grounded: true,
                facing: PlatformerFacing::Right,
                animation: PlatformerAnimationState::Run,
                velocity: Vec2::new(12.0, -4.0),
            }
        ));

        let runtime = RhaiScriptRuntime::new_with_platformer(
            None,
            None,
            Some(platformer_scene.clone()),
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
                "world-platformer-test",
                r#"
                    fn update(dt) {
                        let state = world.platformer.state("playground-sidescroller-player");
                        if !state.grounded { throw("state should expose grounded"); }
                        if state.facing != "right" { throw("state should expose facing"); }
                        if state.animation != "run" { throw("state should expose animation"); }
                        if state.velocity_x < 10.0 { throw("state should expose velocity_x"); }
                        if !world.platformer.drive("playground-sidescroller-player", -1.0, true, false, dt) {
                            throw("drive should succeed");
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("world-platformer-test", 1.0 / 60.0)
            .expect("update should succeed");

        assert_eq!(
            platformer_scene.motor("playground-sidescroller-player"),
            Some(PlatformerMotor2d {
                move_x: -1.0,
                jump_pressed: true,
                jump_held: false,
            })
        );
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
