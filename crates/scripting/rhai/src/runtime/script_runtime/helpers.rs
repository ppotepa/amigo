impl RhaiScriptRuntime {
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
                CallFnOptions::new().eval_ast(true),
                &mut script.scope,
                &script.lifecycle_ast,
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

    fn rhai_params(params: &ScriptParams) -> rhai::Map {
        params
            .iter()
            .map(|(key, value)| {
                let value = match value {
                    ScriptValue::Bool(value) => rhai::Dynamic::from_bool(*value),
                    ScriptValue::Int(value) => rhai::Dynamic::from_int(*value as rhai::INT),
                    ScriptValue::Float(value) => rhai::Dynamic::from_float(*value as rhai::FLOAT),
                    ScriptValue::String(value) => rhai::Dynamic::from(value.clone()),
                };
                (key.clone().into(), value)
            })
            .collect()
    }
}
