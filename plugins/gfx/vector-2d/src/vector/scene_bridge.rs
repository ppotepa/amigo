use super::model::{
    VectorShape2d, VectorShape2dDrawCommand, VectorShapeKind2d, VectorStyle2d, VectorViewportFit2d,
};
use super::service::VectorSceneService;
use amigo_camera_optics_plugin::api::CameraOpticalResponse2d;
use amigo_material_2d_plugin::{
    Material2d, Material2dLighting, Material2dOptical, Material2dOpticalMode,
};
use amigo_render_api::{
    RenderContributionSet, render_contribution_roles as roles,
};
use amigo_scene::{
    Material2dOpticalModeSceneCommand, Material2dSceneCommand, SceneService,
    VectorShape2dSceneCommand, VectorShapeKind2dSceneCommand, VectorStyle2dSceneCommand,
};

pub fn queue_vector_shape_scene_command(
    scene_service: &SceneService,
    vector_scene_service: &VectorSceneService,
    command: &VectorShape2dSceneCommand,
) -> amigo_scene::SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    vector_scene_service.queue(VectorShape2dDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        render_layer: command.render_layer.clone(),
        shape: VectorShape2d {
            kind: map_shape_kind(&command.kind),
            style: map_style(&command.style),
        },
        z_index: command.z_index,
        transform: command.transform,
        viewport_fit: VectorViewportFit2d::Fixed,
        viewport_canvas_size: None,
        material: material_from_scene_command(command.material.as_ref()),
        render_contributions: vector_render_contributions(command),
    });
    entity
}

fn vector_render_contributions(command: &VectorShape2dSceneCommand) -> RenderContributionSet {
    let mut render_contributions =
        RenderContributionSet::from_pairs(command.render_contributions.roles.clone());
    render_contributions.merge_defaults([
        (roles::WORLD_COLOR, true),
        (roles::MATERIAL_MASK, false),
        (roles::OPTICS_REFRACT, false),
        (roles::TRANSMISSION_SOURCE, false),
        (roles::BLOOM_SOURCE, false),
        (roles::CAMERA_FX_SOURCE, false),
    ]);
    render_contributions
}

fn material_from_scene_command(material: Option<&Material2dSceneCommand>) -> Option<Material2d> {
    material.map(|material| {
        Material2d {
            optical: Material2dOptical {
                mode: match material.optical.mode {
                    Material2dOpticalModeSceneCommand::Opaque => Material2dOpticalMode::Opaque,
                    Material2dOpticalModeSceneCommand::Transmissive => {
                        Material2dOpticalMode::Transmissive
                    }
                    Material2dOpticalModeSceneCommand::Refractive => {
                        Material2dOpticalMode::Refractive
                    }
                    Material2dOpticalModeSceneCommand::Emissive => Material2dOpticalMode::Emissive,
                },
                transmission: material.optical.transmission,
                refraction_px: material.optical.refraction_px,
                distortion: material.optical.distortion,
                dispersion: material.optical.dispersion,
                roughness: material.optical.roughness,
                edge_boost: material.optical.edge_boost,
            },
            lighting: Material2dLighting {
                receives_light: material.lighting.receives_light,
                response: material.lighting.response,
            },
            camera_response: CameraOpticalResponse2d {
                enabled: material.camera_response.enabled,
                intensity: material.camera_response.intensity,
                bloom: material.camera_response.bloom,
                glare: material.camera_response.glare,
                ghosting: material.camera_response.ghosting,
                streaks: material.camera_response.streaks,
                chromatic_smear: material.camera_response.chromatic_smear,
                dirt_response: material.camera_response.dirt_response,
                halation: material.camera_response.halation,
                threshold: material.camera_response.threshold,
            },
        }
        .normalized()
    })
}

fn map_shape_kind(kind: &VectorShapeKind2dSceneCommand) -> VectorShapeKind2d {
    match kind {
        VectorShapeKind2dSceneCommand::Polyline { points, closed } => VectorShapeKind2d::Polyline {
            points: points.clone(),
            closed: *closed,
        },
        VectorShapeKind2dSceneCommand::Polygon { points } => VectorShapeKind2d::Polygon {
            points: points.clone(),
        },
        VectorShapeKind2dSceneCommand::Circle { radius, segments } => VectorShapeKind2d::Circle {
            radius: *radius,
            segments: (*segments).max(3),
        },
    }
}

fn map_style(style: &VectorStyle2dSceneCommand) -> VectorStyle2d {
    VectorStyle2d {
        stroke_color: style.stroke_color,
        stroke_width: style.stroke_width.max(0.0),
        fill_color: style.fill_color,
    }
}
