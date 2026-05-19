use std::collections::BTreeMap;

use amigo_camera_core_plugin::{
    CameraFocusTarget2d, CameraFocusTarget2dKind, CameraFocusTarget2dService,
    CameraFocusTargetDepth2d,
};
use amigo_core::AmigoResult;
use amigo_math::Vec2;
use amigo_runtime::{Runtime, RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};

pub struct FocusTargets2dRuntimePlugin;

impl RuntimePlugin for FocusTargets2dRuntimePlugin {
    fn name(&self) -> &'static str {
        "amigo-focus-targets-2d"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::PreUpdate,
            "focus_targets_2d_refresh",
            refresh_focus_targets_2d_system,
        );
        Ok(())
    }
}

pub fn refresh_focus_targets_2d_system(runtime: &Runtime) -> AmigoResult<()> {
    let Some(targets) = runtime.resolve::<CameraFocusTarget2dService>() else {
        return Ok(());
    };
    let Some(scene) = runtime.resolve::<amigo_scene::SceneService>() else {
        targets.clear();
        return Ok(());
    };
    let Some(render_layers) = runtime.resolve::<amigo_2d_composition::RenderLayer2dSceneService>()
    else {
        targets.clear();
        return Ok(());
    };

    let depth_space = render_layers.depth_space();
    let layers = render_layers.commands();
    let layer_lookup = layers
        .iter()
        .map(|layer| (layer.id.clone(), layer.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut output = Vec::new();

    for layer in &layers {
        push_layer_focus_target(&mut output, layer, depth_space);
    }

    if let Some(text2d) = runtime.resolve::<amigo_2d_text::Text2dSceneService>() {
        for command in text2d.commands() {
            push_entity_focus_target(
                &mut output,
                &scene,
                &layer_lookup,
                depth_space,
                &command.entity_name,
                &command.render_layer,
                "Text2D",
            );
        }
    }
    if let Some(sprites) = runtime.resolve::<amigo_2d_sprite::SpriteSceneService>() {
        for command in sprites.commands() {
            push_entity_focus_target(
                &mut output,
                &scene,
                &layer_lookup,
                depth_space,
                &command.entity_name,
                &command.render_layer,
                "Sprite2D",
            );
        }
    }
    if let Some(layered_images) =
        runtime.resolve::<amigo_2d_layered_image::LayeredImageSceneService>()
    {
        for command in layered_images.commands() {
            push_entity_focus_target(
                &mut output,
                &scene,
                &layer_lookup,
                depth_space,
                &command.entity_name,
                &command.render_layer,
                "LayeredImage2D",
            );
        }
    }
    if let Some(vectors) = runtime.resolve::<amigo_2d_vector::VectorSceneService>() {
        for command in vectors.commands() {
            push_entity_focus_target(
                &mut output,
                &scene,
                &layer_lookup,
                depth_space,
                &command.entity_name,
                &command.render_layer,
                "Vector2D",
            );
        }
    }
    if let Some(beacons) = runtime.resolve::<amigo_2d_lighting_beacon::BeaconLight2dSceneService>()
    {
        for command in beacons.commands() {
            push_entity_focus_target(
                &mut output,
                &scene,
                &layer_lookup,
                depth_space,
                &command.entity_name,
                &command.render_layer,
                "BeaconLight2D",
            );
        }
    }
    if let Some(particles) = runtime.resolve::<amigo_2d_particles::Particle2dSceneService>() {
        for command in particles.draw_commands() {
            push_entity_focus_target(
                &mut output,
                &scene,
                &layer_lookup,
                depth_space,
                &command.emitter_entity_name,
                &command.render_layer,
                "ParticleEmitter2D",
            );
        }
    }

    targets.replace_all(output);
    Ok(())
}

fn push_layer_focus_target(
    output: &mut Vec<CameraFocusTarget2d>,
    layer: &amigo_2d_composition::RenderLayer2dCommand,
    depth_space: amigo_2d_spatial::DepthSpace2d,
) {
    if layer.depth.is_overlay() {
        return;
    }
    output.push(CameraFocusTarget2d {
        id: format!("layer:{}", layer.id),
        aliases: [
            layer.id.clone(),
            format!("render_layer:{}", layer.id),
            format!("layer:{}", layer.id),
        ]
        .into_iter()
        .collect(),
        kind: CameraFocusTarget2dKind::RenderLayer,
        entity_name: None,
        render_layer: Some(layer.id.clone()),
        source_component: Some("RenderLayer2D".to_owned()),
        world_position: None,
        depth: focus_depth_for_layer(layer, depth_space),
        visible: layer.visible,
    });
}

fn push_entity_focus_target(
    output: &mut Vec<CameraFocusTarget2d>,
    scene: &amigo_scene::SceneService,
    layers: &BTreeMap<String, amigo_2d_composition::RenderLayer2dCommand>,
    depth_space: amigo_2d_spatial::DepthSpace2d,
    entity_name: &str,
    render_layer: &str,
    source_component: &str,
) {
    let Some(layer) = layers.get(render_layer) else {
        return;
    };
    if layer.depth.is_overlay() {
        return;
    }
    let world_position = scene.transform_of(entity_name).map(|transform| {
        Vec2::new(transform.translation.x, transform.translation.y)
    });
    output.push(CameraFocusTarget2d {
        id: format!("entity:{entity_name}"),
        aliases: [entity_name.to_owned(), format!("entity:{entity_name}")]
            .into_iter()
            .collect(),
        kind: CameraFocusTarget2dKind::SceneObject,
        entity_name: Some(entity_name.to_owned()),
        render_layer: Some(render_layer.to_owned()),
        source_component: Some(source_component.to_owned()),
        world_position,
        depth: focus_depth_for_layer(layer, depth_space),
        visible: scene.is_visible(entity_name) && layer.visible,
    });
}

fn focus_depth_for_layer(
    layer: &amigo_2d_composition::RenderLayer2dCommand,
    depth_space: amigo_2d_spatial::DepthSpace2d,
) -> CameraFocusTargetDepth2d {
    // Focus targets expose authored/base layer depth. Camera rig motion applies camera_z_m
    // later when resolving effective focus distance for DOF/capture.
    match layer.depth.mode {
        amigo_2d_composition::RenderDepthMode2d::Distance => {
            let meters = layer.depth.distance_m.unwrap_or(1.0).max(0.0);
            CameraFocusTargetDepth2d::Distance {
                meters,
                z_depth: amigo_2d_spatial::distance_to_z_depth(meters, depth_space),
            }
        }
        amigo_2d_composition::RenderDepthMode2d::ZDepth
        | amigo_2d_composition::RenderDepthMode2d::DepthMap
        | amigo_2d_composition::RenderDepthMode2d::Infinity => CameraFocusTargetDepth2d::Depth {
            z_depth: layer.depth.z_depth.clamp(0.0, 1.0),
        },
        amigo_2d_composition::RenderDepthMode2d::Overlay => CameraFocusTargetDepth2d::Depth {
            z_depth: 1.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_2d_text::{Text2d, Text2dDrawCommand, Text2dStyle};
    use amigo_assets::AssetKey;
    use amigo_math::Transform2;
    use amigo_scene::SceneEntityId;

    #[test]
    fn refresh_focus_targets_registers_text2d_entity_with_layer_distance() {
        let runtime = amigo_runtime::RuntimeBuilder::default()
            .with_service(amigo_scene::SceneService::default())
            .unwrap()
            .with_service(amigo_2d_composition::RenderLayer2dSceneService::default())
            .unwrap()
            .with_service(amigo_2d_text::Text2dSceneService::default())
            .unwrap()
            .with_service(CameraFocusTarget2dService::default())
            .unwrap()
            .build();

        let scene = runtime.required::<amigo_scene::SceneService>().unwrap();
        scene.spawn("title");
        let layers = runtime
            .required::<amigo_2d_composition::RenderLayer2dSceneService>()
            .unwrap();
        layers.queue(amigo_2d_composition::RenderLayer2dCommand {
            source_mod: "test".to_owned(),
            id: "title.depth2d".to_owned(),
            label: None,
            order: 20.0,
            visible: true,
            opacity: 1.0,
            depth: amigo_2d_composition::RenderDepth2d {
                mode: amigo_2d_composition::RenderDepthMode2d::Distance,
                distance_m: Some(1.0),
                z_depth: 0.0,
                blur_scale: 1.0,
            },
            optical_role: amigo_2d_spatial::OpticalLayerRole2d::ForegroundMedium,
        });
        let text = runtime.required::<amigo_2d_text::Text2dSceneService>().unwrap();
        text.queue(Text2dDrawCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "title".to_owned(),
            render_layer: "title.depth2d".to_owned(),
            text: Text2d {
                content: "ROTTEN CLUB".to_owned(),
                font: AssetKey::new("test/font"),
                bounds: Vec2::new(200.0, 40.0),
                transform: Transform2::default(),
                style: Text2dStyle::default(),
                post_fx_host_id: None,
            },
            z_index: 0.0,
            material: None,
            render_contributions: amigo_render_api::RenderContributionSet::default(),
        });

        refresh_focus_targets_2d_system(&runtime).unwrap();

        let targets = runtime.required::<CameraFocusTarget2dService>().unwrap();
        let resolved = targets.resolve("title").expect("title target should resolve");
        assert!(matches!(
            resolved.target.depth,
            CameraFocusTargetDepth2d::Distance { meters, .. } if (meters - 1.0).abs() < 0.001
        ));
        assert!(targets.resolve("layer:title.depth2d").is_some());
    }
}
