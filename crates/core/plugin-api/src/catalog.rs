use crate::ids::{CapabilityId, SlotId};

pub mod capabilities {
    use super::CapabilityId;

    pub fn camera_frame_context_2d() -> CapabilityId {
        CapabilityId("camera.frame_context.2d".to_string())
    }

    pub fn camera_optics_2d() -> CapabilityId {
        CapabilityId("camera.optics.2d".to_string())
    }

    pub fn camera_focus_depth_2d() -> CapabilityId {
        CapabilityId("camera.focus_depth.2d".to_string())
    }

    pub fn camera_shutter_motion_2d() -> CapabilityId {
        CapabilityId("camera.shutter_motion.2d".to_string())
    }

    pub fn camera_film_look_2d() -> CapabilityId {
        CapabilityId("camera.film_look.2d".to_string())
    }

    pub fn sprite_2d() -> CapabilityId {
        CapabilityId("gfx.sprite.2d".to_string())
    }

    pub fn text_2d() -> CapabilityId {
        CapabilityId("gfx.text.2d".to_string())
    }

    pub fn vector_2d() -> CapabilityId {
        CapabilityId("gfx.vector.2d".to_string())
    }

    pub fn material_2d() -> CapabilityId {
        CapabilityId("materials.material.2d".to_string())
    }

    pub fn light_2d() -> CapabilityId {
        CapabilityId("lighting.light.2d".to_string())
    }

    pub fn light_group_2d() -> CapabilityId {
        CapabilityId("lighting.light_group.2d".to_string())
    }

    pub fn particle_2d() -> CapabilityId {
        CapabilityId("vfx.particle.2d".to_string())
    }

    pub fn bloom() -> CapabilityId {
        CapabilityId("postfx.bloom".to_string())
    }

    pub fn color_grading() -> CapabilityId {
        CapabilityId("postfx.color_grading".to_string())
    }

    pub fn codemap_index() -> CapabilityId {
        CapabilityId("devtools.codemap.index".to_string())
    }

    pub fn diagnostics_provider() -> CapabilityId {
        CapabilityId("devtools.diagnostics.provider".to_string())
    }
}

pub mod slots {
    use super::SlotId;

    pub fn camera_frame_provider_2d() -> SlotId {
        SlotId("camera.frame_provider.2d".to_string())
    }

    pub fn camera_focus_model_2d() -> SlotId {
        SlotId("camera.focus_model.2d".to_string())
    }

    pub fn camera_optics_consumer_2d() -> SlotId {
        SlotId("camera.optics.consumer.2d".to_string())
    }

    pub fn camera_shutter_model_2d() -> SlotId {
        SlotId("camera.shutter_model.2d".to_string())
    }

    pub fn camera_film_model_2d() -> SlotId {
        SlotId("camera.film_model.2d".to_string())
    }

    pub fn render_backend() -> SlotId {
        SlotId("render.backend".to_string())
    }

    pub fn scene_component_hydrator() -> SlotId {
        SlotId("scene.component_hydrator".to_string())
    }

    pub fn scripting_binding_provider() -> SlotId {
        SlotId("scripting.binding_provider".to_string())
    }

    pub fn diagnostics_provider() -> SlotId {
        SlotId("diagnostics.provider".to_string())
    }

    pub fn editor_panel() -> SlotId {
        SlotId("editor.panel".to_string())
    }

    pub fn codemap_index_provider() -> SlotId {
        SlotId("codemap.index_provider".to_string())
    }
}
