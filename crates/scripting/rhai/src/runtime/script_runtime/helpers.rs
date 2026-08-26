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

        match self.engine.call_fn_with_options::<rhai::Dynamic>(
            CallFnOptions::new().eval_ast(true),
            &mut script.scope,
            &script.lifecycle_ast,
            function_name,
            args,
        ) {
            Ok(_) => Ok(()),
            Err(error)
                if matches!(
                    error.as_ref(),
                    rhai::EvalAltResult::ErrorFunctionNotFound(signature, _)
                        if signature.starts_with(function_name)
                ) =>
            {
                Ok(())
            }
            Err(error) => Err(AmigoError::Message(format!(
                "failed to call {function_name} for script `{source_name}`: {error}"
            ))),
        }
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
