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

    fn set_source_context(&self, context: ScriptSourceContext) -> AmigoResult<()> {
        *self
            .source_context
            .lock()
            .expect("rhai source context mutex should not be poisoned") = Some(context);
        Ok(())
    }

    fn execute(&self, source_name: &str, source: &str) -> AmigoResult<()> {
        let mut initial_scope = rhai::Scope::new();
        initial_scope.push_constant("world", self.world.clone());
        let ast = self
            .engine
            .compile_into_self_contained(&initial_scope, source)
            .map_err(|error| {
                AmigoError::Message(format!(
                    "failed to compile script `{source_name}` for execution: {error}"
                ))
            })?;
        let lifecycle_source = lifecycle_source_from_script(source);
        let lifecycle_ast = self
            .engine
            .compile_into_self_contained(&initial_scope, &lifecycle_source)
            .map_err(|error| {
                AmigoError::Message(format!(
                    "failed to compile lifecycle callbacks for script `{source_name}`: {error}"
                ))
            })?;
        let mut scope = initial_scope;
        self.engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(|error| {
                AmigoError::Message(format!("failed to execute script `{source_name}`: {error}"))
            })?;

        self.scripts
            .lock()
            .expect("rhai script registry mutex should not be poisoned")
            .insert(
                source_name.to_owned(),
                StoredScript {
                    lifecycle_ast,
                    scope,
                },
            );

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

    fn call_event_function(
        &self,
        source_name: &str,
        function_name: &str,
        topic: &str,
        payload: &[String],
    ) -> AmigoResult<()> {
        self.time_state.set_passive_delta(0.0);
        let payload = payload
            .iter()
            .cloned()
            .map(Into::into)
            .collect::<rhai::Array>();
        self.call_optional_void(source_name, function_name, (topic.to_owned(), payload))
    }

    fn call_component_on_attach(
        &self,
        source_name: &str,
        entity_name: &str,
        params: &ScriptParams,
    ) -> AmigoResult<()> {
        self.time_state.set_passive_delta(0.0);
        self.call_optional_void(
            source_name,
            "on_attach",
            (entity_name.to_owned(), Self::rhai_params(params)),
        )
    }

    fn call_component_update(
        &self,
        source_name: &str,
        entity_name: &str,
        params: &ScriptParams,
        delta_seconds: f32,
    ) -> AmigoResult<()> {
        self.time_state.set_passive_delta(delta_seconds);
        self.call_optional_void(
            source_name,
            "update",
            (
                entity_name.to_owned(),
                Self::rhai_params(params),
                delta_seconds as rhai::FLOAT,
            ),
        )
    }

    fn call_component_on_detach(
        &self,
        source_name: &str,
        entity_name: &str,
        params: &ScriptParams,
    ) -> AmigoResult<()> {
        self.time_state.set_passive_delta(0.0);
        self.call_optional_void(
            source_name,
            "on_detach",
            (entity_name.to_owned(), Self::rhai_params(params)),
        )
    }
}

