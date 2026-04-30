use super::super::*;
use crate::runtime_context::RuntimeContext;

pub(crate) fn tick_ui_bindings(runtime: &Runtime) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let ui_scene = ctx.required::<UiSceneService>()?;
    let ui_state = ctx.required::<UiStateService>()?;
    let scene_state = ctx.required::<amigo_state::SceneStateService>()?;

    for command in ui_scene.commands() {
        let root_segment = command
            .document
            .root
            .id
            .clone()
            .unwrap_or_else(|| "root".to_owned());
        let root_path = format!("{}.{}", command.entity_name, root_segment);
        apply_node_binds(
            &command.document.root,
            &root_path,
            ui_state.as_ref(),
            scene_state.as_ref(),
        );
    }

    Ok(())
}

fn apply_node_binds(
    node: &RuntimeUiNode,
    path: &str,
    ui_state: &UiStateService,
    scene_state: &amigo_state::SceneStateService,
) {
    if let Some(key) = node.binds.text.as_deref() {
        if let Some(value) = scene_state_value_as_text(scene_state, key) {
            let _ = ui_state.set_text(path, value);
        }
    }

    for (index, child) in node.children.iter().enumerate() {
        let segment = child
            .id
            .clone()
            .unwrap_or_else(|| format!("{}-{index}", child.kind.label()));
        let child_path = format!("{path}.{segment}");
        apply_node_binds(child, &child_path, ui_state, scene_state);
    }
}

fn scene_state_value_as_text(
    scene_state: &amigo_state::SceneStateService,
    key: &str,
) -> Option<String> {
    if let Some(value) = scene_state.get_string(key) {
        return Some(value);
    }
    if let Some(value) = scene_state.get_int(key) {
        return Some(value.to_string());
    }
    if let Some(value) = scene_state.get_float(key) {
        return Some(format_float(value));
    }
    scene_state.get_bool(key).map(|value| value.to_string())
}

fn format_float(value: f64) -> String {
    let text = value.to_string();
    text.strip_suffix(".0").unwrap_or(&text).to_owned()
}
