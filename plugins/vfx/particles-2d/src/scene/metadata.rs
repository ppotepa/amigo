use amigo_scene::*;

macro_rules! property {
    ($path:literal, $label:literal, $kind:expr, $editor:expr, $trait_kind:expr, $group:literal) => {
        EditorPropertyDescriptor {
            path: $path,
            label: $label,
            value_kind: $kind,
            access: EditorPropertyAccess::Editable,
            editor: $editor,
            asset_domain: None,
            trait_kind: Some($trait_kind),
            group: $group,
            patch_op: None,
            number_constraints: None,
            options: &[],
            visibility: EditorPropertyVisibility::Advanced,
            order: 0,
            tags: &["Unsupported"],
            readonly_reason: Some("No live runtime binding yet"),
            binding_template: None,
        }
    };
    (live $path:literal, $label:literal, $kind:expr, $editor:expr, $trait_kind:expr, $group:literal, $binding:expr) => {
        EditorPropertyDescriptor {
            path: $path,
            label: $label,
            value_kind: $kind,
            access: EditorPropertyAccess::Editable,
            editor: $editor,
            asset_domain: None,
            trait_kind: Some($trait_kind),
            group: $group,
            patch_op: None,
            number_constraints: None,
            options: &[],
            visibility: EditorPropertyVisibility::Advanced,
            order: 0,
            tags: &["Live"],
            readonly_reason: None,
            binding_template: Some($binding),
        }
    };
    (live num $path:literal, $label:literal, $kind:expr, $editor:expr, $trait_kind:expr, $group:literal, $constraints:expr, $binding:expr) => {
        EditorPropertyDescriptor {
            path: $path,
            label: $label,
            value_kind: $kind,
            access: EditorPropertyAccess::Editable,
            editor: $editor,
            asset_domain: None,
            trait_kind: Some($trait_kind),
            group: $group,
            patch_op: None,
            number_constraints: Some($constraints),
            options: &[],
            visibility: EditorPropertyVisibility::Primary,
            order: 0,
            tags: &["Live"],
            readonly_reason: None,
            binding_template: Some($binding),
        }
    };
    (ro $path:literal, $label:literal, $trait_kind:expr, $group:literal) => {
        EditorPropertyDescriptor {
            path: $path,
            label: $label,
            value_kind: EditorPropertyValueKind::String,
            access: EditorPropertyAccess::ReadOnly,
            editor: EditorPropertyEditorKind::ReadOnly,
            asset_domain: None,
            trait_kind: Some($trait_kind),
            group: $group,
            patch_op: None,
            number_constraints: None,
            options: &[],
            visibility: EditorPropertyVisibility::Advanced,
            order: 0,
            tags: &["Readonly"],
            readonly_reason: Some("Descriptor metadata"),
            binding_template: None,
        }
    };
}

pub fn particle_emitter_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind_id: "ParticleEmitter2D",
        type_name: "ParticleEmitter2D",
        label: "Particle Emitter 2D",
        domains: &[ComponentDomain::Particles, ComponentDomain::Render2D],
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: None,
        metadata_traits: &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::RenderLayered2D,
            MetadataTraitKind::LightReceiver2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::Motion2D,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[],
        properties: &[
            property!(live "active", "Active", EditorPropertyValueKind::Bool, EditorPropertyEditorKind::Checkbox, MetadataTraitKind::Renderable2D, "render2d.content", EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!(live num "spawn_rate", "Spawn Rate", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Renderable2D, "render2d.content", EDITOR_NUMBER_PARTICLE_RATE, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!(live num "max_particles", "Max Particles", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Renderable2D, "render2d.content", EDITOR_NUMBER_PARTICLE_COUNT, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!(live num "particle_lifetime", "Particle Lifetime", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Renderable2D, "render2d.content", EDITOR_NUMBER_PARTICLE_SECONDS, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!(live num "initial_speed", "Initial Speed", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Motion2D, "motion2.tuning", EDITOR_NUMBER_PARTICLE_SPEED, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!(live num "initial_size", "Initial Size", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::HasBounds2D, "bounds2.size", EDITOR_NUMBER_PARTICLE_SIZE, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!(live num "final_size", "Final Size", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::HasBounds2D, "bounds2.size", EDITOR_NUMBER_PARTICLE_SIZE, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!("render_layer", "Render Layer", EditorPropertyValueKind::String, EditorPropertyEditorKind::Text, MetadataTraitKind::RenderLayered2D, "render2d.order"),
            property!(live num "z_index", "Z Index", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Renderable2D, "render2d.order", EDITOR_NUMBER_ORDER, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!("color", "Color", EditorPropertyValueKind::Color, EditorPropertyEditorKind::Color, MetadataTraitKind::Renderable2D, "render2d.color"),
            property!(ro "color_ramp", "Color Ramp", MetadataTraitKind::Renderable2D, "render2d.color"),
            property!(ro "alpha_curve", "Alpha Curve", MetadataTraitKind::Renderable2D, "render2d.color"),
            property!(ro "size_curve", "Size Curve", MetadataTraitKind::HasBounds2D, "bounds2.size"),
            property!(ro "speed_curve", "Speed Curve", MetadataTraitKind::Motion2D, "motion2.tuning"),
            property!(ro "emission_rate_curve", "Emission Rate Curve", MetadataTraitKind::Renderable2D, "render2d.content"),
            property!(ro "shape", "Shape", MetadataTraitKind::Renderable2D, "render2d.content"),
            property!(ro "spawn_area", "Spawn Area", MetadataTraitKind::HasBounds2D, "bounds2.size"),
            property!(live num "spread_degrees", "Spread Degrees", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Renderable2D, "render2d.content", EDITOR_NUMBER_PARTICLE_DEGREES, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!(live num "local_direction_degrees", "Local Direction Degrees", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Renderable2D, "render2d.content", EDITOR_NUMBER_PARTICLE_DEGREES, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!(live num "lifetime_jitter", "Lifetime Jitter", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Renderable2D, "render2d.content", EDITOR_NUMBER_PARTICLE_SECONDS, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!(live num "speed_jitter", "Speed Jitter", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Motion2D, "motion2.tuning", EDITOR_NUMBER_PARTICLE_SPEED, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!(ro "forces", "Forces", MetadataTraitKind::Motion2D, "motion2.tuning"),
            property!("material", "Material", EditorPropertyValueKind::String, EditorPropertyEditorKind::Text, MetadataTraitKind::LightReceiver2D, "render2d.lighting"),
            property!("attached_to", "Attached To", EditorPropertyValueKind::String, EditorPropertyEditorKind::Text, MetadataTraitKind::Motion2D, "motion2.attachment"),
            property!("local_offset", "Local Offset", EditorPropertyValueKind::Vec2, EditorPropertyEditorKind::Vec2, MetadataTraitKind::HasBounds2D, "bounds2.offset"),
            property!("blend_mode", "Blend Mode", EditorPropertyValueKind::String, EditorPropertyEditorKind::Text, MetadataTraitKind::Renderable2D, "render2d.content"),
            property!(live num "inherit_parent_velocity", "Inherit Parent Velocity", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Motion2D, "motion2.tuning", EDITOR_NUMBER_PARTICLE_VELOCITY_SCALE, EditorRuntimeBindingTemplate::ParticleEmitterField),
            property!("align", "Align", EditorPropertyValueKind::String, EditorPropertyEditorKind::Text, MetadataTraitKind::Renderable2D, "render2d.content"),
            property!("motion_stretch", "Motion Stretch", EditorPropertyValueKind::Number, EditorPropertyEditorKind::Number, MetadataTraitKind::Motion2D, "motion2.tuning"),
            property!("simulation_space", "Simulation Space", EditorPropertyValueKind::String, EditorPropertyEditorKind::Text, MetadataTraitKind::Renderable2D, "render2d.content"),
            property!("light", "Light", EditorPropertyValueKind::Bool, EditorPropertyEditorKind::Checkbox, MetadataTraitKind::Renderable2D, "render2d.content"),
            property!("line_anchor", "Line Anchor", EditorPropertyValueKind::String, EditorPropertyEditorKind::Text, MetadataTraitKind::Renderable2D, "render2d.content"),
            property!("velocity_mode", "Velocity Mode", EditorPropertyValueKind::String, EditorPropertyEditorKind::Text, MetadataTraitKind::Motion2D, "motion2.tuning"),
        ],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::SpawnArea2D {
            field: "spawn_area",
            size_field: "size",
            default_width: 128,
            default_height: 128,
        },
        editor_controls: &[EditorControlKind::Transform2D, EditorControlKind::Rect2D],
        patch_ops: &[EditorPatchOpKind::SetTransform2],
    }
}

pub struct ParticleEmitter2dComponentMetadataProvider;

impl ComponentMetadataProvider for ParticleEmitter2dComponentMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.vfx.particles-2d.component-metadata"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry.insert(particle_emitter_2d_descriptor());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemRegistry};

    #[test]
    fn provider_registers_particle_emitter_descriptor() {
        let mut registry = ComponentRegistry::new([]);

        ParticleEmitter2dComponentMetadataProvider.register_component_metadata(&mut registry);

        let descriptor = registry
            .descriptor("ParticleEmitter2D")
            .expect("particle metadata descriptor should be registered");
        assert_eq!(descriptor.type_name, "ParticleEmitter2D");
        assert!(descriptor.metadata_traits.contains(&MetadataTraitKind::Renderable2D));
        assert!(descriptor.metadata_traits.contains(&MetadataTraitKind::Motion2D));
    }

    #[test]
    fn plugin_registers_particle_metadata_provider() {
        let mut registry = ServiceRegistry::default();
        registry
            .register(ComponentMetadataProviderRegistry::default())
            .unwrap();
        registry
            .register(RuntimeSceneCommandHandlerRegistry::new())
            .unwrap();
        registry.register(SystemRegistry::default()).unwrap();

        crate::Particle2dPlugin.register(&mut registry).unwrap();

        let providers = registry
            .resolve::<ComponentMetadataProviderRegistry>()
            .expect("metadata provider registry should remain registered");
        assert_eq!(
            providers.provider_ids(),
            vec!["amigo.vfx.particles-2d.component-metadata"]
        );

        let component_registry = component_registry_with_providers(Some(providers.as_ref()));
        assert!(component_registry.descriptor("ParticleEmitter2D").is_some());
    }
}
