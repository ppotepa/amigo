use std::sync::Arc;

use amigo_core::LaunchSelection;
use amigo_scripting_api::{
    DevConsoleCommand, DevConsoleQueue, ScriptCommand, ScriptCommandQueue, ScriptEvent,
    ScriptEventQueue,
};

pub fn queue_placeholder_command(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    namespace: &str,
    name: &str,
    arguments: Vec<String>,
) -> bool {
    let Some(command_queue) = command_queue else {
        return false;
    };
    command_queue.submit(ScriptCommand::new(namespace, name, arguments));
    true
}

pub fn queue_scene_select(command_queue: Option<&Arc<ScriptCommandQueue>>, scene_id: &str) -> bool {
    queue_placeholder_command(command_queue, "scene", "select", vec![scene_id.to_owned()])
}

pub fn queue_scene_reload(command_queue: Option<&Arc<ScriptCommandQueue>>) {
    let _ = queue_placeholder_command(command_queue, "scene", "reload", Vec::<String>::new());
}

pub fn queue_asset_reload(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    asset_key: &str,
) -> bool {
    queue_placeholder_command(command_queue, "asset", "reload", vec![asset_key.to_owned()])
}

pub fn queue_ui_set_text(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    path: &str,
    value: &str,
) -> bool {
    let Some(command_queue) = command_queue else {
        return false;
    };
    command_queue.submit(ScriptCommand::ui_set_text(path, value));
    true
}

pub fn queue_ui_set_value(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    path: &str,
    value: f32,
) -> bool {
    let Some(command_queue) = command_queue else {
        return false;
    };
    command_queue.submit(ScriptCommand::ui_set_value(path, value));
    true
}

pub fn queue_ui_show(command_queue: Option<&Arc<ScriptCommandQueue>>, path: &str) -> bool {
    let Some(command_queue) = command_queue else {
        return false;
    };
    command_queue.submit(ScriptCommand::ui_show(path));
    true
}

pub fn queue_ui_hide(command_queue: Option<&Arc<ScriptCommandQueue>>, path: &str) -> bool {
    let Some(command_queue) = command_queue else {
        return false;
    };
    command_queue.submit(ScriptCommand::ui_hide(path));
    true
}

pub fn queue_ui_enable(command_queue: Option<&Arc<ScriptCommandQueue>>, path: &str) -> bool {
    let Some(command_queue) = command_queue else {
        return false;
    };
    command_queue.submit(ScriptCommand::ui_enable(path));
    true
}

pub fn queue_ui_disable(command_queue: Option<&Arc<ScriptCommandQueue>>, path: &str) -> bool {
    let Some(command_queue) = command_queue else {
        return false;
    };
    command_queue.submit(ScriptCommand::ui_disable(path));
    true
}

pub fn queue_sprite_spawn(
    launch_selection: Option<&Arc<LaunchSelection>>,
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    entity_name: &str,
    texture_key: &str,
    width: rhai::INT,
    height: rhai::INT,
) -> bool {
    let Some(root_mod) = launch_selection.map(|selection| selection.selected_mod().to_owned())
    else {
        return false;
    };

    queue_placeholder_command(
        command_queue,
        "2d.sprite",
        "spawn",
        vec![
            root_mod,
            entity_name.to_owned(),
            texture_key.to_owned(),
            width.to_string(),
            height.to_string(),
        ],
    )
}

pub fn queue_text2d_spawn(
    launch_selection: Option<&Arc<LaunchSelection>>,
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    entity_name: &str,
    content: &str,
    font_key: &str,
    width: rhai::INT,
    height: rhai::INT,
) -> bool {
    let Some(root_mod) = launch_selection.map(|selection| selection.selected_mod().to_owned())
    else {
        return false;
    };

    queue_placeholder_command(
        command_queue,
        "2d.text",
        "spawn",
        vec![
            root_mod,
            entity_name.to_owned(),
            content.to_owned(),
            font_key.to_owned(),
            width.to_string(),
            height.to_string(),
        ],
    )
}

pub fn queue_mesh3d_spawn(
    launch_selection: Option<&Arc<LaunchSelection>>,
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    entity_name: &str,
    mesh_key: &str,
) -> bool {
    let Some(root_mod) = launch_selection.map(|selection| selection.selected_mod().to_owned())
    else {
        return false;
    };

    queue_placeholder_command(
        command_queue,
        "3d.mesh",
        "spawn",
        vec![root_mod, entity_name.to_owned(), mesh_key.to_owned()],
    )
}

pub fn queue_material3d_bind(
    launch_selection: Option<&Arc<LaunchSelection>>,
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    entity_name: &str,
    label: &str,
    material_key: &str,
) -> bool {
    let Some(root_mod) = launch_selection.map(|selection| selection.selected_mod().to_owned())
    else {
        return false;
    };

    queue_placeholder_command(
        command_queue,
        "3d.material",
        "bind",
        vec![
            root_mod,
            entity_name.to_owned(),
            label.to_owned(),
            material_key.to_owned(),
        ],
    )
}

pub fn queue_text3d_spawn(
    launch_selection: Option<&Arc<LaunchSelection>>,
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    entity_name: &str,
    content: &str,
    font_key: &str,
    size: f32,
) -> bool {
    let Some(root_mod) = launch_selection.map(|selection| selection.selected_mod().to_owned())
    else {
        return false;
    };

    queue_placeholder_command(
        command_queue,
        "3d.text",
        "spawn",
        vec![
            root_mod,
            entity_name.to_owned(),
            content.to_owned(),
            font_key.to_owned(),
            size.to_string(),
        ],
    )
}

pub fn emit_script_event(
    event_queue: Option<&Arc<ScriptEventQueue>>,
    topic: &str,
    payload: Option<&str>,
) {
    if let Some(event_queue) = event_queue {
        let payload = payload
            .map(|payload| vec![payload.to_owned()])
            .unwrap_or_default();
        event_queue.publish(ScriptEvent::new(topic, payload));
    }
}

pub fn queue_console_command(console_queue: Option<&Arc<DevConsoleQueue>>, line: &str) {
    if let Some(console_queue) = console_queue {
        console_queue.submit(DevConsoleCommand::new(line));
    }
}

pub fn queue_debug_message(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    level: &str,
    line: &str,
) {
    let _ = queue_placeholder_command(command_queue, "debug", level, vec![line.to_owned()]);
}
