impl RhaiScriptRuntime {
    fn eval_console_source(
        &self,
        context: DevConsoleScriptContext,
        source: &str,
    ) -> AmigoResult<DevConsoleEvalResult> {
        let key = console_scope_key(&context);

        let mut scopes = self
            .console_scopes
            .lock()
            .expect("rhai console scope mutex should not be poisoned");

        let scope = scopes
            .entry(key)
            .or_insert_with(|| self.initial_console_scope());

        let value = self
            .engine
            .eval_with_scope::<rhai::Dynamic>(scope, source)
            .map_err(|error| {
                AmigoError::Message(format!(
                    "failed to eval dev console Rhai `{}`: {error}",
                    context.source_name
                ))
            })?;

        if value.is_unit() {
            Ok(DevConsoleEvalResult::Unit)
        } else {
            Ok(DevConsoleEvalResult::Value(format_dynamic_for_console(value)))
        }
    }

    fn initial_console_scope(&self) -> rhai::Scope<'static> {
        let mut world = self.world.clone();
        let mut scope = rhai::Scope::new();

        scope.push_constant("world", world.clone());
        scope.push_constant("scene", world.scene());
        scope.push_constant("entities", world.entities());
        scope.push_constant("postfx", world.postfx());
        scope.push_constant("state", world.state());
        scope.push_constant("session", world.session());
        scope.push_constant("particles", world.particles());
        scope.push_constant("ui", world.ui());
        scope.push_constant("audio", world.audio());
        scope.push_constant("runtime", world.runtime());

        scope
    }
}

fn console_scope_key(context: &DevConsoleScriptContext) -> String {
    match context.scene_id.as_deref() {
        Some(scene_id) if !scene_id.trim().is_empty() => format!("scene:{scene_id}"),
        _ => "scene:<none>".to_owned(),
    }
}

fn format_dynamic_for_console(value: rhai::Dynamic) -> String {
    if value.is_string() {
        return value.into_string().unwrap_or_default();
    }

    format!("{value:?}")
}
