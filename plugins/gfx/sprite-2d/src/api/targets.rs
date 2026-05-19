use amigo_plugin_api::{scene_alpha, scene_color, TargetId};

pub fn sprite_scene_color_target_id() -> TargetId {
    scene_color()
}

pub fn sprite_scene_alpha_target_id() -> TargetId {
    scene_alpha()
}
