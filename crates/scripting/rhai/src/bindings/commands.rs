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

pub fn queue_scene_activate_set(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    set_id: &str,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "scene",
        "activate-set",
        vec![set_id.to_owned()],
    )
}

pub fn queue_asset_reload(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    asset_key: &str,
) -> bool {
    queue_placeholder_command(command_queue, "asset", "reload", vec![asset_key.to_owned()])
}

pub fn queue_audio_play(command_queue: Option<&Arc<ScriptCommandQueue>>, clip_name: &str) -> bool {
    queue_placeholder_command(command_queue, "audio", "play", vec![clip_name.to_owned()])
}

pub fn queue_audio_preload(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    clip_name: &str,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "audio",
        "preload",
        vec![clip_name.to_owned()],
    )
}

pub fn queue_audio_play_asset(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    asset_key: &str,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "audio",
        "play-asset",
        vec![asset_key.to_owned()],
    )
}

pub fn queue_audio_cue(command_queue: Option<&Arc<ScriptCommandQueue>>, cue_name: &str) -> bool {
    queue_placeholder_command(command_queue, "audio", "cue", vec![cue_name.to_owned()])
}

pub fn queue_audio_start_realtime(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    source: &str,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "audio",
        "start-realtime",
        vec![source.to_owned()],
    )
}

pub fn queue_audio_stop(command_queue: Option<&Arc<ScriptCommandQueue>>, source: &str) -> bool {
    queue_placeholder_command(command_queue, "audio", "stop", vec![source.to_owned()])
}

pub fn queue_audio_set_param(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    source: &str,
    param: &str,
    value: f32,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "audio",
        "set-param",
        vec![source.to_owned(), param.to_owned(), value.to_string()],
    )
}

pub fn queue_audio_set_volume(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    bus: &str,
    value: f32,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "audio",
        "set-volume",
        vec![bus.to_owned(), value.to_string()],
    )
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

pub fn queue_ui_set_color(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    path: &str,
    value: &str,
) -> bool {
    let Some(command_queue) = command_queue else {
        return false;
    };
    command_queue.submit(ScriptCommand::ui_set_color(path, value));
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

pub fn queue_layered_image_set_base_opacity(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    entity_name: &str,
    opacity: f32,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.layered_image",
        "set_base_opacity",
        vec![entity_name.to_owned(), opacity.to_string()],
    )
}

pub fn queue_layered_image_set_opacity(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    entity_name: &str,
    layer_id: &str,
    opacity: f32,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.layered_image",
        "set_opacity",
        vec![
            entity_name.to_owned(),
            layer_id.to_owned(),
            opacity.to_string(),
        ],
    )
}

pub fn queue_layered_image_set_enabled(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    entity_name: &str,
    layer_id: &str,
    enabled: bool,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.layered_image",
        "set_enabled",
        vec![
            entity_name.to_owned(),
            layer_id.to_owned(),
            enabled.to_string(),
        ],
    )
}

pub fn queue_layered_image_set_blend(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    entity_name: &str,
    layer_id: &str,
    blend: &str,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.layered_image",
        "set_blend",
        vec![
            entity_name.to_owned(),
            layer_id.to_owned(),
            blend.to_owned(),
        ],
    )
}

pub fn queue_beacon2d_set_base_intensity(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    target: &str,
    value: f32,
) -> bool {
    queue_beacon2d_value(command_queue, "set_base_intensity", target, value)
}

pub fn queue_beacon2d_set_frequency_hz(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    target: &str,
    value: f32,
) -> bool {
    queue_beacon2d_value(command_queue, "set_frequency_hz", target, value)
}

pub fn queue_beacon2d_set_duty_cycle(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    target: &str,
    value: f32,
) -> bool {
    queue_beacon2d_value(command_queue, "set_duty_cycle", target, value)
}

pub fn queue_beacon2d_set_halo_radius_px(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    target: &str,
    value: f32,
) -> bool {
    queue_beacon2d_value(command_queue, "set_halo_radius_px", target, value)
}

pub fn queue_beacon2d_set_aberration_px(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    target: &str,
    value: f32,
) -> bool {
    queue_beacon2d_value(command_queue, "set_aberration_px", target, value)
}

pub fn queue_beacon2d_set_flare_strength(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    target: &str,
    value: f32,
) -> bool {
    queue_beacon2d_value(command_queue, "set_flare_strength", target, value)
}

fn queue_beacon2d_value(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    name: &str,
    target: &str,
    value: f32,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.beacon",
        name,
        vec![target.to_owned(), value.to_string()],
    )
}

pub fn queue_light2d_set_intensity(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    id: &str,
    intensity: f32,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.light",
        "set_intensity",
        vec![id.to_owned(), intensity.to_string()],
    )
}

pub fn queue_light2d_set_color(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    id: &str,
    color: &str,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.light",
        "set_color",
        vec![id.to_owned(), color.to_owned()],
    )
}

pub fn queue_light_group2d_set_intensity(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    id: &str,
    intensity: f32,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.light_group",
        "set_intensity",
        vec![id.to_owned(), intensity.to_string()],
    )
}

pub fn queue_light_group2d_set_color(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    id: &str,
    color: &str,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.light_group",
        "set_color",
        vec![id.to_owned(), color.to_owned()],
    )
}

pub fn queue_render_layer2d_set_opacity(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    id: &str,
    opacity: f32,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.render_layer",
        "set_opacity",
        vec![id.to_owned(), opacity.to_string()],
    )
}

pub fn queue_render_layer2d_set_visible(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    id: &str,
    visible: bool,
) -> bool {
    queue_placeholder_command(
        command_queue,
        "2d.render_layer",
        "set_visible",
        vec![id.to_owned(), visible.to_string()],
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
