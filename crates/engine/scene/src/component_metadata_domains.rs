use crate::{
    ComponentMetadataProvider, ComponentRegistry, ComponentTypeDescriptor,
    aabb_collider_2d_descriptor, behavior_descriptor, bounds_2d_descriptor, camera_3d_descriptor,
    camera_follow_2d_descriptor, circle_collider_2d_descriptor, entity_pool_descriptor,
    event_pipeline_descriptor, freeflight_motion_2d_descriptor, input_action_map_descriptor,
    kinematic_body_2d_descriptor, lifetime_descriptor, light_3d_descriptor,
    lightmap_2d_source_descriptor, material_3d_descriptor, mesh_3d_descriptor,
    motion_controller_2d_descriptor, parallax_2d_descriptor, projectile_emitter_2d_descriptor,
    script_component_descriptor, static_collider_2d_descriptor, text_3d_descriptor,
    tile_map_marker_2d_descriptor, trigger_2d_descriptor, ui_document_descriptor,
    ui_model_bindings_descriptor, ui_theme_set_descriptor, velocity_2d_descriptor,
};

pub struct EngineRender3dMetadataProvider;
pub struct EngineRender2dMetadataProvider;
pub struct EnginePhysics2dMetadataProvider;
pub struct EngineGameplayMetadataProvider;
pub struct EngineUiInputMetadataProvider;
pub struct EngineMotionCameraMetadataProvider;

impl ComponentMetadataProvider for EngineRender3dMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.engine.metadata.render3d"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry.extend([
            camera_3d_descriptor(),
            light_3d_descriptor(),
            text_3d_descriptor(),
            mesh_3d_descriptor(),
            material_3d_descriptor(),
        ]);
    }
}

impl ComponentMetadataProvider for EngineRender2dMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.engine.metadata.render2d"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry.extend([
            lightmap_2d_source_descriptor(),
            tile_map_marker_2d_descriptor(),
            parallax_2d_descriptor(),
            bounds_2d_descriptor(),
        ]);
    }
}

impl ComponentMetadataProvider for EnginePhysics2dMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.engine.metadata.physics2d"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry.extend([
            trigger_2d_descriptor(),
            aabb_collider_2d_descriptor(),
            static_collider_2d_descriptor(),
            circle_collider_2d_descriptor(),
        ]);
    }
}

impl ComponentMetadataProvider for EngineGameplayMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.engine.metadata.gameplay"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry.extend([
            script_component_descriptor(),
            behavior_descriptor(),
            event_pipeline_descriptor(),
            lifetime_descriptor(),
            projectile_emitter_2d_descriptor(),
            entity_pool_descriptor(),
        ]);
    }
}

impl ComponentMetadataProvider for EngineUiInputMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.engine.metadata.ui-input"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry.extend([
            input_action_map_descriptor(),
            ui_document_descriptor(),
            ui_model_bindings_descriptor(),
            ui_theme_set_descriptor(),
        ]);
    }
}

impl ComponentMetadataProvider for EngineMotionCameraMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.engine.metadata.motion-camera"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry.extend([
            velocity_2d_descriptor(),
            camera_follow_2d_descriptor(),
            freeflight_motion_2d_descriptor(),
            kinematic_body_2d_descriptor(),
            motion_controller_2d_descriptor(),
        ]);
    }
}

/// Builds the engine-owned compatibility metadata catalog through the same
/// provider contract used by domain plugins. New descriptors should move to
/// their owning crate and disappear from these compatibility providers.
pub fn engine_owned_component_registry() -> ComponentRegistry {
    let mut registry = ComponentRegistry::new(std::iter::empty::<ComponentTypeDescriptor>());
    EngineRender3dMetadataProvider.register_component_metadata(&mut registry);
    EngineRender2dMetadataProvider.register_component_metadata(&mut registry);
    EnginePhysics2dMetadataProvider.register_component_metadata(&mut registry);
    EngineGameplayMetadataProvider.register_component_metadata(&mut registry);
    EngineUiInputMetadataProvider.register_component_metadata(&mut registry);
    EngineMotionCameraMetadataProvider.register_component_metadata(&mut registry);
    registry
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::default_component_registry;

    #[test]
    fn provider_owned_engine_catalog_matches_compatibility_catalog() {
        let expected = default_component_registry()
            .iter()
            .map(|descriptor| descriptor.kind_id)
            .collect::<BTreeSet<_>>();
        let actual = engine_owned_component_registry()
            .iter()
            .map(|descriptor| descriptor.kind_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
    }
}
