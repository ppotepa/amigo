pub struct UiDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub fn register_ui_services(registry: &mut amigo_runtime::ServiceRegistry) -> AmigoResult<()> {
    registry.register(UiSceneService::default())?;
    registry.register(UiStateService::default())?;
    registry.register(UiModelBindingService::default())?;
    registry.register(UiThemeService::default())?;
    registry.register(crate::input::UiInputService::default())?;
    registry.register(UiLayoutService)?;
    registry.register(UiDomainInfo {
        crate_name: "amigo-ui",
        capability: "screen_space_ui",
    })?;
    amigo_capabilities::register_domain_plugin(
        registry,
        "amigo-ui",
        &["screen_space_ui"],
        &[],
        amigo_capabilities::DEFAULT_CAPABILITY_VERSION,
    )
}
