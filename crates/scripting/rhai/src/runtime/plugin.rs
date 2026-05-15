use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};

pub struct RhaiScriptingPlugin;

pub const RHAI_SCRIPTING_CAPABILITY: &str = "scripting_rhai";

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

        if !registry.has::<RunLogService>() {
            registry.register(RunLogService::default_for_process()?)?;
        }

        let run_log = registry.resolve::<RunLogService>();
        if let (Some(console), Some(run_log)) =
            (registry.resolve::<DevConsoleState>(), run_log.clone())
        {
            console.attach_run_log(run_log.clone());
            run_log.write_runtime(format!(
                "registered scripting runtime console_log={} runtime_log={}",
                run_log.console_log_path().display(),
                run_log.runtime_log_path().display()
            ));
        }

        if !registry.has::<ScriptLifecycleState>() {
            registry.register(ScriptLifecycleState::default())?;
        }

        if !registry.has::<ScriptComponentService>() {
            registry.register(ScriptComponentService::default())?;
        }

        if !registry.has::<ScriptTraceService>() {
            registry.register(ScriptTraceService::default())?;
        }

        let scene = registry.resolve::<SceneService>();
        let sprite_scene = registry.resolve::<SpriteSceneService>();
        let vector_scene = registry.resolve::<VectorSceneService>();
        let motion_scene = registry.resolve::<Motion2dSceneService>();
        let particle_scene = registry.resolve::<Particle2dSceneService>();
        let particle_preset_scene = registry.resolve::<ParticlePreset2dService>();
        let physics_scene = registry.resolve::<Physics2dSceneService>();
        let post_fx = registry.resolve::<PostFx2dService>();
        let pool_scene = registry.resolve::<EntityPoolSceneService>();
        let lifetime_scene = registry.resolve::<LifetimeSceneService>();
        let state_service = registry.resolve::<SceneStateService>();
        let session_service = registry.resolve::<SessionStateService>();
        let timer_service = registry.resolve::<SceneTimerService>();
        let ui_theme_service = registry.resolve::<UiThemeService>();
        let asset_catalog = registry.resolve::<AssetCatalog>();
        let input_state = registry.resolve::<InputState>();
        let input_actions = registry.resolve::<InputActionService>();
        let launch_selection = registry.resolve::<LaunchSelection>();
        let mod_catalog = registry.resolve::<ModCatalog>();
        let diagnostics = registry.resolve::<RuntimeDiagnostics>();
        let command_queue = registry.resolve::<ScriptCommandQueue>();
        let event_queue = registry.resolve::<ScriptEventQueue>();
        let console_queue = registry.resolve::<DevConsoleQueue>();
        let trace_service = registry.resolve::<ScriptTraceService>();
        let runtime =
            RhaiScriptRuntime::new_with_services_and_ui_theme_and_particle_presets_with_post_fx(
                scene,
                sprite_scene,
                vector_scene,
                motion_scene,
                particle_scene,
                particle_preset_scene,
                physics_scene,
                post_fx,
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
                input_actions,
                trace_service,
            );

        registry.register(ScriptRuntimeInfo {
            backend_name: runtime.backend_name(),
            file_extension: runtime.file_extension(),
        })?;
        registry.register(ScriptRuntimeService::new(runtime))?;

        register_domain_plugin(
            registry,
            "amigo-scripting-rhai",
            &[RHAI_SCRIPTING_CAPABILITY],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::RhaiSceneCommandHandler,
        );
        registry
            .required::<amigo_runtime::SystemRegistry>()?
            .register_fn(
                amigo_runtime::SystemPhase::Update,
                "script_components",
                move |runtime| {
                    let dt = amigo_session::simulation_delta_seconds(runtime);
                    crate::tick_script_components(runtime, dt)
                },
            );
        registry
            .required::<amigo_runtime::SystemRegistry>()?
            .register_fn(
                amigo_runtime::SystemPhase::Update,
                "script_update",
                move |runtime| {
                    let dt = amigo_session::simulation_delta_seconds(runtime);
                    crate::tick_active_scripts(runtime, dt)
                },
            );
        Ok(())
    }
}

fn build_engine(
    world: WorldApi,
    source_context: Arc<Mutex<Option<ScriptSourceContext>>>,
) -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    engine.set_max_expr_depths(256, 512);
    register_world_api(&mut engine);

    let get_entity_world = world.clone();
    engine.register_fn("get_entity", move |entity_name: &str| {
        let mut world = get_entity_world.clone();
        let mut entities = world.entities();
        entities.named(entity_name)
    });

    let entity_world = world.clone();
    engine.register_fn("entity", move |entity_name: &str| {
        let mut world = entity_world.clone();
        let mut entities = world.entities();
        entities.named(entity_name)
    });

    let list_entities_world = world.clone();
    engine.register_fn("list_entities", move || {
        let mut world = list_entities_world.clone();
        let mut entities = world.entities();
        entities.names()
    });

    let list_postfx_world = world.clone();
    engine.register_fn("list_postfx_items", move || {
        let mut world = list_postfx_world.clone();
        let mut postfx = world.postfx();
        postfx.list()
    });

    engine.set_module_resolver(
        PackageModuleResolver::default_with_context(source_context).with_world(world),
    );
    engine
}
