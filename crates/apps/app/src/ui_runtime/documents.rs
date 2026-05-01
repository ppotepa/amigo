pub(crate) struct ResolvedUiOverlayDocument {
    pub(crate) overlay: UiOverlayDocument,
    pub(crate) click_bindings: BTreeMap<String, UiEventBinding>,
    pub(crate) change_bindings: BTreeMap<String, UiEventBinding>,
}

pub(crate) fn resolve_ui_overlay_documents(
    ui_scene_service: &UiSceneService,
    ui_state_service: &UiStateService,
    ui_theme_service: &UiThemeService,
) -> Vec<ResolvedUiOverlayDocument> {
    let snapshot = ui_state_service.snapshot();
    let active_theme = ui_theme_service.active_theme();
    let mut documents = ui_scene_service
        .commands()
        .into_iter()
        .filter_map(|command| {
            resolve_ui_overlay_document(
                &command.entity_name,
                &command.document,
                &snapshot,
                active_theme.as_ref(),
            )
        })
        .collect::<Vec<_>>();
    documents.sort_by_key(|document| document.overlay.layer);
    documents
}

fn resolve_ui_overlay_document(
    entity_name: &str,
    document: &RuntimeUiDocument,
    snapshot: &UiStateSnapshot,
    active_theme: Option<&UiTheme>,
) -> Option<ResolvedUiOverlayDocument> {
    let root_segment = document
        .root
        .id
        .clone()
        .unwrap_or_else(|| "root".to_owned());
    let root_path = format!("{entity_name}.{root_segment}");
    let mut click_bindings = BTreeMap::new();
    let mut change_bindings = BTreeMap::new();
    let root = resolve_ui_overlay_node(
        &document.root,
        &root_path,
        snapshot,
        active_theme,
        &mut click_bindings,
        &mut change_bindings,
    )?;

    Some(ResolvedUiOverlayDocument {
        overlay: UiOverlayDocument {
            entity_name: entity_name.to_owned(),
            layer: resolve_ui_overlay_layer(&document.target),
            viewport: resolve_ui_overlay_viewport(&document.target),
            root,
        },
        click_bindings,
        change_bindings,
    })
}

fn resolve_ui_overlay_layer(target: &RuntimeUiTarget) -> UiOverlayLayer {
    match target.layer() {
        RuntimeUiLayer::Background => UiOverlayLayer::Background,
        RuntimeUiLayer::Hud => UiOverlayLayer::Hud,
        RuntimeUiLayer::Menu => UiOverlayLayer::Menu,
        RuntimeUiLayer::Debug => UiOverlayLayer::Debug,
    }
}

fn resolve_ui_overlay_viewport(target: &RuntimeUiTarget) -> Option<UiOverlayViewport> {
    match target {
        RuntimeUiTarget::ScreenSpace { viewport, .. } => {
            viewport.map(|viewport| UiOverlayViewport {
                width: viewport.width,
                height: viewport.height,
                scaling: match viewport.scaling {
                    RuntimeUiViewportScaling::Expand => UiOverlayViewportScaling::Expand,
                    RuntimeUiViewportScaling::Fixed => UiOverlayViewportScaling::Fixed,
                    RuntimeUiViewportScaling::Fit => UiOverlayViewportScaling::Fit,
                },
            })
        }
    }
}

