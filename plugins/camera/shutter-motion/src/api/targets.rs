use amigo_plugin_api::{final_composite, scene_velocity, TargetId};

pub fn scene_velocity_target_id() -> TargetId {
    scene_velocity()
}

pub fn temporal_exposure_target_id() -> TargetId {
    TargetId("TemporalExposure".to_owned())
}

pub fn final_composite_target_id() -> TargetId {
    final_composite()
}
