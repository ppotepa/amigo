#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RhaiSandboxLimits {
    pub max_operations: u64,
    pub max_call_levels: usize,
    pub max_variables: usize,
    pub max_functions: usize,
    pub max_modules: usize,
    pub max_string_size: usize,
    pub max_array_size: usize,
    pub max_map_size: usize,
}

impl Default for RhaiSandboxLimits {
    fn default() -> Self {
        Self {
            max_operations: 2_000_000,
            max_call_levels: 64,
            max_variables: 4_096,
            max_functions: 1_024,
            max_modules: 64,
            max_string_size: 1024 * 1024,
            max_array_size: 100_000,
            max_map_size: 20_000,
        }
    }
}

pub(crate) fn configure_rhai_sandbox(engine: &mut rhai::Engine) {
    configure_rhai_sandbox_with(engine, RhaiSandboxLimits::default());
}

pub(crate) fn configure_rhai_sandbox_with(engine: &mut rhai::Engine, limits: RhaiSandboxLimits) {
    engine
        .set_max_expr_depths(256, 512)
        .set_max_operations(limits.max_operations)
        .set_max_call_levels(limits.max_call_levels)
        .set_max_variables(limits.max_variables)
        .set_max_functions(limits.max_functions)
        .set_max_modules(limits.max_modules)
        .set_max_string_size(limits.max_string_size)
        .set_max_array_size(limits.max_array_size)
        .set_max_map_size(limits.max_map_size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_budget_stops_infinite_loop() {
        let mut engine = rhai::Engine::new();
        configure_rhai_sandbox_with(&mut engine, RhaiSandboxLimits {
            max_operations: 2_000,
            ..RhaiSandboxLimits::default()
        });
        let error = engine.eval::<()>("loop { let x = 1 + 1; }").expect_err("loop must hit operation budget");
        assert!(error.to_string().to_ascii_lowercase().contains("operations"));
    }

    #[test]
    fn collection_budget_rejects_oversized_array() {
        let mut engine = rhai::Engine::new();
        configure_rhai_sandbox_with(&mut engine, RhaiSandboxLimits {
            max_array_size: 8,
            ..RhaiSandboxLimits::default()
        });
        assert!(engine.eval::<rhai::Dynamic>("[1,2,3,4,5,6,7,8,9]").is_err());
    }
}
