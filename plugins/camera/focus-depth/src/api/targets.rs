use amigo_plugin_api::{scene_depth, TargetId};

pub fn scene_depth_target_id() -> TargetId {
    scene_depth()
}

pub fn focus_field_target_id() -> TargetId {
    TargetId("FocusField".to_owned())
}
