struct StoredScript {
    lifecycle_ast: rhai::AST,
    scope: rhai::Scope<'static>,
}

pub struct RhaiScriptRuntime {
    engine: rhai::Engine,
    scripts: Mutex<BTreeMap<String, StoredScript>>,
    time_state: Arc<ScriptTimeState>,
    timer_service: Arc<SceneTimerService>,
    source_context: Arc<Mutex<Option<ScriptSourceContext>>>,
    world: WorldApi,
}

