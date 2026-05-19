use amigo_plugin_api::{scene_alpha, scene_color, TargetId};

pub fn text_render_targets() -> Vec<TargetId> {
    vec![scene_color(), scene_alpha()]
}
