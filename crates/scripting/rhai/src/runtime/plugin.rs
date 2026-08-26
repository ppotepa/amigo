use crate::handles::EntityRef;
use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_editor_api::{InspectRequest, InspectRequestService, InspectSource, InspectSubject};
use amigo_runtime_control::{ControlValue, RuntimeControlService};

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
        amigo_scene::register_scene_reset_handler(registry, RhaiScriptingSceneResetHandler)?;

        if !registry.has::<InspectRequestService>() {
            registry.register(InspectRequestService::default())?;
        }

        let runtime = RhaiScriptRuntime::from_services(RhaiRuntimeServices::resolve(registry));

        registry.register(RhaiFrameClock::new(
            runtime.time_state.clone(),
            runtime.timer_service.clone(),
        ))?;
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
        let plugin_scene_handlers =
            registry.required::<amigo_scene::ScenePluginCommandHandlerRegistry>()?;
        plugin_scene_handlers.register(
            amigo_scene::SCRIPT_COMPONENT_PLUGIN_SCENE_COMMAND_TYPE,
            Arc::new(crate::scene_command::RhaiSceneCommandHandler),
        );
        registry.required::<amigo_runtime::SystemRegistry>()?.register_fn(
            amigo_runtime::SystemPhase::Update,
            "script_components",
            move |runtime| {
                let dt = amigo_session::simulation_delta_seconds(runtime);
                crate::tick_script_components(runtime, dt)
            },
        );
        registry.required::<amigo_runtime::SystemRegistry>()?.register_fn(
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
    runtime_control: Option<Arc<RuntimeControlService>>,
    binding_namespaces: Vec<String>,
) -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    engine.set_max_expr_depths(256, 512);
    register_world_api(&mut engine, &binding_namespaces);

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

    let inspect_entity_world = world.clone();
    engine.register_fn("inspect", move |entity: EntityRef| -> bool {
        inspect_entity_world.request_inspect(InspectRequest {
            source: InspectSource::Rhai,
            subject: InspectSubject::Entity {
                name: entity.inspect_entity_name(),
            },
            expression: None,
        })
    });
    let inspect_postfx_world = world.clone();
    engine.register_fn(
        "inspect",
        move |fx: crate::bindings::PostFxItemRef| -> bool {
            inspect_postfx_world.request_inspect(InspectRequest {
                source: InspectSource::Rhai,
                subject: InspectSubject::PostFxFrameItem {
                    index: fx.inspect_index(),
                    label: fx.inspect_label(),
                },
                expression: None,
            })
        },
    );
    let inspect_layer_world = world.clone();
    engine.register_fn(
        "inspect",
        move |layer: crate::bindings::RenderLayer2dHandle| -> bool {
            inspect_layer_world.request_inspect(InspectRequest {
                source: InspectSource::Rhai,
                subject: InspectSubject::RenderLayer {
                    id: layer.inspect_layer_id(),
                },
                expression: None,
            })
        },
    );

    let control_for_set = runtime_control.clone();
    engine.register_fn(
        "__amigo_control_set",
        move |path: &str, value: rhai::Dynamic| -> Result<(), Box<rhai::EvalAltResult>> {
            let Some(control) = control_for_set.clone() else {
                return Err("runtime control service unavailable".into());
            };
            control
                .set(path, dynamic_to_control_value(value))
                .map_err(|error| error.to_string().into())
        },
    );
    let control_for_get = runtime_control.clone();
    engine.register_fn(
        "__amigo_control_get",
        move |path: &str| -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
            let Some(control) = control_for_get.clone() else {
                return Err("runtime control service unavailable".into());
            };
            control
                .get(path)
                .map(control_value_to_dynamic)
                .map_err(|error| error.to_string().into())
        },
    );
    let control_for_info = runtime_control.clone();
    engine.register_fn(
        "__amigo_control_info",
        move |path: &str| -> Result<String, Box<rhai::EvalAltResult>> {
            let Some(control) = control_for_info.clone() else {
                return Err("runtime control service unavailable".into());
            };
            control.info(path).map_err(|error| error.to_string().into())
        },
    );
    let control_for_reset = runtime_control.clone();
    engine.register_fn(
        "__amigo_control_reset",
        move |path: &str| -> Result<(), Box<rhai::EvalAltResult>> {
            let Some(control) = control_for_reset.clone() else {
                return Err("runtime control service unavailable".into());
            };
            control.reset(path).map_err(|error| error.to_string().into())
        },
    );
    engine.register_fn(
        "__amigo_control_commit",
        move |path: &str| -> Result<(), Box<rhai::EvalAltResult>> {
            let Some(control) = runtime_control.clone() else {
                return Err("runtime control service unavailable".into());
            };
            control.commit(path).map_err(|error| error.to_string().into())
        },
    );

    engine.set_module_resolver(
        PackageModuleResolver::default_with_context(source_context).with_world(world),
    );
    engine
}

fn dynamic_to_control_value(value: rhai::Dynamic) -> ControlValue {
    if value.is_unit() {
        ControlValue::Null
    } else if value.is_bool() {
        ControlValue::Bool(value.cast::<bool>())
    } else if value.is_int() {
        ControlValue::I64(value.cast::<i64>())
    } else if value.is_float() {
        ControlValue::F64(value.cast::<rhai::FLOAT>() as f64)
    } else if value.is_string() {
        ControlValue::String(value.into_string().unwrap_or_default())
    } else {
        ControlValue::String(format!("{value:?}"))
    }
}

fn control_value_to_dynamic(value: ControlValue) -> rhai::Dynamic {
    match value {
        ControlValue::Bool(value) => value.into(),
        ControlValue::I64(value) => value.into(),
        ControlValue::U64(value) => (value as i64).into(),
        ControlValue::F64(value) => (value as rhai::FLOAT).into(),
        ControlValue::String(value) | ControlValue::AssetRef(value) => value.into(),
        ControlValue::Null => ().into(),
    }
}
