use std::sync::Arc;

use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use super::context::WgpuRenderExtractorRegistry;

pub fn register_world_2d_render_extractors(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuTileMap2dRenderExtractorBridge);
    registry.register(WgpuSprite2dRenderExtractorBridge);
    registry.register(WgpuLayeredImage2dRenderExtractorBridge);
    registry.register(WgpuDepthMap2dRenderExtractorBridge);
    registry.register(WgpuVector2dRenderExtractorBridge);
    registry.register(WgpuBeacon2dRenderExtractorBridge);
    registry.register(WgpuText2dRenderExtractorBridge);
    registry.register(WgpuComposition2dRenderExtractorBridge);
    registry.register(WgpuLighting2dRenderExtractorBridge);
    registry.register(WgpuParticle2dRenderExtractorBridge);
    registry.register(WgpuPostFx2dRenderExtractorBridge);
}

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> Arc<T> {
    runtime
        .required::<T>()
        .expect("render extractor required service should be registered")
}

pub struct WgpuTileMap2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuTileMap2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_tilemap::TileMap2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let tilemap_scene_service = required::<amigo_2d_tilemap::TileMap2dSceneService>(runtime);
        amigo_2d_tilemap::TileMap2dRenderExtractor.extract(
            amigo_2d_tilemap::TileMap2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                tilemap_scene_service: tilemap_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuSprite2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuSprite2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_sprite::Sprite2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let sprite_scene_service = required::<amigo_2d_sprite::SpriteSceneService>(runtime);
        amigo_2d_sprite::Sprite2dRenderExtractor.extract(
            amigo_2d_sprite::Sprite2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                sprite_scene_service: sprite_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuLayeredImage2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket>
    for WgpuLayeredImage2dRenderExtractorBridge
{
    fn name(&self) -> &'static str {
        amigo_2d_layered_image::LayeredImage2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let layered_image_scene_service =
            required::<amigo_2d_layered_image::LayeredImageSceneService>(runtime);
        amigo_2d_layered_image::LayeredImage2dRenderExtractor.extract(
            amigo_2d_layered_image::LayeredImage2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                layered_image_scene_service: layered_image_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuDepthMap2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuDepthMap2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_depth_map::DepthMap2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let depth_map_scene_service =
            required::<amigo_2d_depth_map::DepthMap2dSceneService>(runtime);
        amigo_2d_depth_map::DepthMap2dRenderExtractor.extract(
            amigo_2d_depth_map::DepthMap2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                depth_map_scene_service: depth_map_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuVector2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuVector2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_vector::Vector2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let vector_scene_service = required::<amigo_2d_vector::VectorSceneService>(runtime);
        amigo_2d_vector::Vector2dRenderExtractor.extract(
            amigo_2d_vector::Vector2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                vector_scene_service: vector_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuText2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuText2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_text::Text2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let text_scene_service = required::<amigo_2d_text::Text2dSceneService>(runtime);
        amigo_2d_text::Text2dRenderExtractor.extract(
            amigo_2d_text::Text2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                text_scene_service: text_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuBeacon2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuBeacon2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_lighting_beacon::Beacon2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(beacon_scene_service) =
            runtime.resolve::<amigo_2d_lighting_beacon::BeaconLight2dSceneService>()
        else {
            return;
        };
        amigo_2d_lighting_beacon::Beacon2dRenderExtractor.extract(
            amigo_2d_lighting_beacon::Beacon2dRenderExtractionContext {
                beacon_scene_service: beacon_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuComposition2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket>
    for WgpuComposition2dRenderExtractorBridge
{
    fn name(&self) -> &'static str {
        amigo_2d_composition::Composition2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let render_layer2d_scene_service =
            required::<amigo_2d_composition::RenderLayer2dSceneService>(runtime);
        let light_route2d_scene_service =
            required::<amigo_2d_composition::LightRoute2dSceneService>(runtime);
        amigo_2d_composition::Composition2dRenderExtractor.extract(
            amigo_2d_composition::Composition2dRenderExtractionContext {
                render_layer2d_scene_service: render_layer2d_scene_service.as_ref(),
                light_route2d_scene_service: light_route2d_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuLighting2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuLighting2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_lighting::Lighting2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let global_light2d_scene_service =
            required::<amigo_2d_lighting::GlobalLight2dSceneService>(runtime);
        let lightmap2d_scene_service =
            required::<amigo_2d_lighting::LightMap2dSceneService>(runtime);
        let light_group2d_scene_service =
            required::<amigo_2d_lighting::LightGroup2dSceneService>(runtime);
        amigo_2d_lighting::Lighting2dRenderExtractor.extract(
            amigo_2d_lighting::Lighting2dRenderExtractionContext {
                global_light2d_scene_service: global_light2d_scene_service.as_ref(),
                lightmap2d_scene_service: lightmap2d_scene_service.as_ref(),
                light_group2d_scene_service: light_group2d_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuParticle2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuParticle2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_particles::Particle2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let particle2d_scene_service =
            required::<amigo_2d_particles::Particle2dSceneService>(runtime);
        amigo_2d_particles::Particle2dRenderExtractor.extract(
            amigo_2d_particles::Particle2dRenderExtractionContext {
                particle2d_scene_service: particle2d_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuPostFx2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuPostFx2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_post_fx::PostFx2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let post_fx_service = required::<amigo_2d_post_fx::PostFx2dService>(runtime);
        let viewport = runtime
            .resolve::<amigo_ui::UiInputViewportState>()
            .and_then(|state| state.get())
            .unwrap_or_else(|| amigo_render_wgpu::UiViewportSize::new(1280.0, 720.0));
        amigo_2d_post_fx::PostFx2dRenderExtractor.extract(
            amigo_2d_post_fx::PostFx2dRenderExtractionContext {
                post_fx_service: post_fx_service.as_ref(),
                viewport_width: viewport.width,
                viewport_height: viewport.height,
            },
            packet,
        );

        if let Some(camera_service) = runtime.resolve::<amigo_camera::CameraService>() {
            let depth_space = runtime
                .resolve::<amigo_2d_composition::RenderLayer2dSceneService>()
                .map(|service| service.depth_space())
                .unwrap_or_default();

            let quality_settings = if let Some(camera) = camera_service.main_camera2d() {
                let settings = camera_service.quality_profile_2d(&camera.id).settings();
                packet.set_active_camera_2d_entity(Some(camera.entity_name));
                packet.set_camera_debug_view_2d(camera_service.debug_view_2d(&camera.id));
                settings
            } else {
                amigo_camera::CameraQualityProfile2d::default().settings()
            };

            let assets = runtime.resolve::<amigo_assets::AssetCatalog>();
            let camera_stacks =
                camera_service.frame_post_fx_stacks_for_depth_space(assets.as_deref(), depth_space);
            if !camera_stacks.is_empty() {
                let mut stacks = camera_stacks;
                stacks.extend(packet.post_fx_stacks().iter().cloned());
                packet.set_post_fx_stacks(stacks);
            }
            packet.set_camera_capture_input_2d(build_camera_capture_input(packet, depth_space));
            packet.set_visual_source_flags_2d(build_visual_source_flags_2d(packet, quality_settings));
        }
    }
}

fn build_visual_source_flags_2d(
    packet: &WgpuRenderFramePacket,
    quality_settings: amigo_camera::CameraQualitySettings2d,
) -> amigo_render_wgpu::WgpuVisualSourceFlags2d {
    let capture = packet.camera_capture_input_2d();
    let generate_visual = quality_settings.debug_buffers
        || quality_settings.visual_source_buffer_quality.should_generate()
        || quality_settings.generate_visual_source_buffers;
    let generate_motion = quality_settings.debug_buffers
        || quality_settings.motion_source_quality.should_generate()
        || quality_settings.generate_motion_debug_source;
    let generate_layer_mask = quality_settings.debug_buffers
        || quality_settings.layer_mask_quality.should_generate()
        || quality_settings.generate_layer_mask_debug_source;
    amigo_render_wgpu::WgpuVisualSourceFlags2d {
        layer_mask_generated: generate_layer_mask && !packet.world_2d_render_layers().is_empty(),
        layer_roles_generated: generate_layer_mask && !packet.world_2d_render_layers().is_empty(),
        scene_normal_generated: is_produced(
            capture,
            amigo_render_api::VisualSourceKind2d::SceneNormal,
        ) && generate_visual,
        scene_wetness_generated: is_produced(
            capture,
            amigo_render_api::VisualSourceKind2d::SceneWetness,
        ) && generate_visual,
        scene_highlight_generated: is_produced(
            capture,
            amigo_render_api::VisualSourceKind2d::SceneHighlight,
        ) && generate_visual,
        scene_emissive_generated: is_produced(
            capture,
            amigo_render_api::VisualSourceKind2d::SceneEmissive,
        ) && generate_visual,
        scene_motion_generated: is_produced(
            capture,
            amigo_render_api::VisualSourceKind2d::SceneMotion,
        ) && generate_motion,
    }
}

fn is_produced(
    input: Option<&amigo_render_api::CameraCaptureInput2d>,
    kind: amigo_render_api::VisualSourceKind2d,
) -> bool {
    input
        .and_then(|input| input.source(kind))
        .is_some_and(|source| {
            source.availability == amigo_render_api::VisualSourceAvailability2d::Produced
        })
}

fn build_camera_capture_input(
    packet: &WgpuRenderFramePacket,
    depth_space: amigo_2d_spatial::DepthSpace2d,
) -> amigo_render_api::CameraCaptureInput2d {
    let layers = packet
        .world_2d_render_layers()
        .iter()
        .map(|layer| {
            let z_depth = layer.depth.z_depth.clamp(0.0, 1.0);
            amigo_render_api::ResolvedLayerOptics2d {
                layer_id: layer.id.clone(),
                role: layer.optical_role,
                depth_mode: depth_mode_label(layer.depth.mode).to_owned(),
                distance_m: layer.depth.distance_m,
                z_depth,
                blur_scale: layer.depth.blur_scale,
                camera_motion_scale: amigo_2d_spatial::z_depth_to_camera_motion_scale(z_depth),
            }
        })
        .collect();
    let mut builder = amigo_render_api::CameraCaptureInput2dBuilder::new(depth_space, layers)
        .with_depth("world.depth");
    if !packet.world_2d_render_layers().is_empty() {
        builder = builder.with_layer_mask("world.layer_mask");
    }
    if should_produce_scene_highlight(packet) {
        // V1 limitation: produced by authored visual maps and procedural/light extraction.
        // Final target: dedicated material/light pass writes this buffer before camera post-fx.
        builder = builder.with_highlight_produced("world.highlight");
    }
    if should_produce_scene_emissive(packet) {
        // V1 limitation: produced by authored visual maps and procedural/light extraction.
        // Final target: dedicated material/light pass writes this buffer before camera post-fx.
        builder = builder.with_emissive_produced("world.emissive");
    }
    if should_produce_scene_normal(packet) {
        // V1 limitation: produced by authored visual maps and wet-reflection asset fallback.
        // Final target: dedicated material pass writes this buffer before camera post-fx.
        builder = builder.with_normal_produced("world.normal");
    } else if let Some(normal) = wetness_normal_source(packet.post_fx_stacks()) {
        builder = builder.with_normal_asset(normal);
    }
    if should_produce_scene_wetness(packet) {
        // V1 limitation: produced by authored visual maps and wet-reflection asset fallback.
        // Final target: dedicated material pass writes this buffer before camera post-fx.
        builder = builder.with_wetness_produced("world.wetness");
    } else if let Some(mask) = wetness_mask_source(packet.post_fx_stacks()) {
        builder = builder.with_wetness_asset(mask);
    }
    if motion_source(packet.post_fx_stacks()).is_some() {
        // V1 limitation: produced from previous per-draw transform positions and shutter active state.
        // Final target: typed motion-vector source from motion/runtime systems.
        builder = builder.with_motion_produced("world.motion");
    }
    builder.build()
}

fn should_produce_scene_highlight(packet: &WgpuRenderFramePacket) -> bool {
    has_visual_map(packet, amigo_render_api::VisualSourceKind2d::SceneHighlight)
        || !packet.world_2d_lightmaps().is_empty()
        || !packet.world_2d_light_groups().is_empty()
        || !packet.world_2d_beacons().is_empty()
}

fn should_produce_scene_emissive(packet: &WgpuRenderFramePacket) -> bool {
    has_visual_map(packet, amigo_render_api::VisualSourceKind2d::SceneEmissive)
        || !packet.world_2d_beacons().is_empty()
        || !packet.world_2d_global_lights().is_empty()
}

fn should_produce_scene_normal(packet: &WgpuRenderFramePacket) -> bool {
    has_visual_map(packet, amigo_render_api::VisualSourceKind2d::SceneNormal)
        || wetness_normal_source(packet.post_fx_stacks()).is_some()
}

fn should_produce_scene_wetness(packet: &WgpuRenderFramePacket) -> bool {
    has_visual_map(packet, amigo_render_api::VisualSourceKind2d::SceneWetness)
        || wetness_mask_source(packet.post_fx_stacks()).is_some()
}

fn has_visual_map(
    packet: &WgpuRenderFramePacket,
    kind: amigo_render_api::VisualSourceKind2d,
) -> bool {
    first_visual_map(packet, kind).is_some()
}

fn first_visual_map(
    packet: &WgpuRenderFramePacket,
    kind: amigo_render_api::VisualSourceKind2d,
) -> Option<&amigo_assets::AssetKey> {
    packet
        .world_2d_sprites()
        .iter()
        .filter_map(|command| visual_map_for_kind(command.sprite.visual_maps.as_ref(), kind))
        .chain(packet.world_2d_layered_images().iter().filter_map(|command| {
            visual_map_for_kind(command.image.visual_maps.as_ref(), kind).or_else(|| {
                command
                    .image
                    .layer_overrides
                    .iter()
                    .filter_map(|override_| visual_map_for_kind(override_.visual_maps.as_ref(), kind))
                    .next()
            })
        }))
        .next()
}

fn visual_map_for_kind(
    maps: Option<&amigo_scene::VisualMaps2dSceneCommand>,
    kind: amigo_render_api::VisualSourceKind2d,
) -> Option<&amigo_assets::AssetKey> {
    let maps = maps?;
    match kind {
        amigo_render_api::VisualSourceKind2d::SceneNormal => maps.normal.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneWetness => maps.wetness.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneEmissive => maps.emissive.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneHighlight => maps.highlight.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneColor
        | amigo_render_api::VisualSourceKind2d::SceneDepth
        | amigo_render_api::VisualSourceKind2d::LayerMask
        | amigo_render_api::VisualSourceKind2d::SceneMotion
        | amigo_render_api::VisualSourceKind2d::Debug => None,
    }
}

fn wetness_normal_source(stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack]) -> Option<&str> {
    stacks
        .iter()
        .flat_map(|stack| stack.effects.iter())
        .find_map(|effect| match &effect.effect {
            amigo_2d_post_fx::PostFx2d::WetReflections(wet)
                if wet.is_active()
                    && wet
                        .noise_normal
                        .as_deref()
                        .is_some_and(|path| !path.trim().is_empty()) =>
            {
                wet.noise_normal.as_deref()
            }
            _ => None,
        })
}

fn wetness_mask_source(stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack]) -> Option<&str> {
    stacks
        .iter()
        .flat_map(|stack| stack.effects.iter())
        .find_map(|effect| match &effect.effect {
            amigo_2d_post_fx::PostFx2d::WetReflections(wet)
                if wet.is_active() && !wet.reflection_mask.trim().is_empty() =>
            {
                Some(wet.reflection_mask.as_str())
            }
            _ => None,
        })
}

fn motion_source(stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack]) -> Option<&'static str> {
    stacks
        .iter()
        .flat_map(|stack| stack.effects.iter())
        .find_map(|effect| match &effect.effect {
            amigo_2d_post_fx::PostFx2d::ShutterBlur(shutter) if shutter.is_active() => {
                Some("camera.shutter_history.motion")
            }
            _ => None,
        })
}

fn depth_mode_label(mode: amigo_2d_composition::RenderDepthMode2d) -> &'static str {
    match mode {
        amigo_2d_composition::RenderDepthMode2d::DepthMap => "depth_map",
        amigo_2d_composition::RenderDepthMode2d::Distance => "distance",
        amigo_2d_composition::RenderDepthMode2d::ZDepth => "z_depth",
        amigo_2d_composition::RenderDepthMode2d::Infinity => "infinity",
        amigo_2d_composition::RenderDepthMode2d::Overlay => "overlay",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_capture_input_includes_scene_color_depth_and_layers() {
        let mut packet = WgpuRenderFramePacket::default();
        packet.push_world_2d_render_layer(amigo_2d_composition::RenderLayer2dCommand {
            source_mod: "rotten-club".to_owned(),
            id: "background.city".to_owned(),
            label: None,
            order: 0.0,
            visible: true,
            opacity: 1.0,
            depth: amigo_2d_composition::RenderDepth2d::default(),
            optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
        });

        let input = build_camera_capture_input(&packet, amigo_2d_spatial::DepthSpace2d::default());

        assert_eq!(
            input.color.kind,
            amigo_render_api::VisualSourceKind2d::SceneColor
        );
        assert_eq!(
            input.depth.as_ref().map(|source| source.kind),
            Some(amigo_render_api::VisualSourceKind2d::SceneDepth)
        );
        assert_eq!(
            input.layer_mask.as_ref().map(|source| source.kind),
            Some(amigo_render_api::VisualSourceKind2d::LayerMask)
        );
        assert_eq!(input.layers.len(), 1);
        assert_eq!(input.layers[0].layer_id, "background.city");
        assert_eq!(
            input.layers[0].role,
            amigo_2d_spatial::OpticalLayerRole2d::WorldSurface
        );
        assert!(
            input
                .missing_source_kinds()
                .contains(&amigo_render_api::VisualSourceKind2d::SceneNormal)
        );
    }

    #[test]
    fn camera_capture_input_sets_highlight_when_lightmaps_exist() {
        let mut packet = WgpuRenderFramePacket::default();
        packet.push_world_2d_lightmap(amigo_2d_lighting::LightMap2dSourceCommand {
            source_mod: "rotten-club".to_owned(),
            entity_name: "bar.lightmap".to_owned(),
            id: "bar.lightmap".to_owned(),
            source: amigo_2d_lighting::LightMap2dSourceRef {
                kind: amigo_2d_lighting::LightMap2dSourceKind::LayeredImage2d,
                entity_name: "bar.lightmap".to_owned(),
            },
            channels: Vec::new(),
        });

        let input = build_camera_capture_input(&packet, amigo_2d_spatial::DepthSpace2d::default());

        assert_eq!(
            input.highlight.as_ref().map(|source| source.kind),
            Some(amigo_render_api::VisualSourceKind2d::SceneHighlight)
        );
        assert!(input.emissive.is_none());
    }

    #[test]
    fn camera_capture_input_sets_wetness_from_active_wet_reflections_mask() {
        let mut packet = WgpuRenderFramePacket::default();
        packet.set_post_fx_stacks(vec![amigo_2d_post_fx::ScopedPostFx2dStack::new(
            "scene.weather",
            amigo_2d_post_fx::PostFxScope2d::Frame,
            vec![amigo_2d_post_fx::PostFx2dInstance::new(
                "wetness",
                amigo_2d_post_fx::PostFx2d::WetReflections(
                    amigo_2d_post_fx::PostFxWetReflections2d {
                        enabled: true,
                        reflection_mask:
                            "rotten-club/layered-images/neon-alley/reflection_mask.png".to_owned(),
                        ..Default::default()
                    },
                ),
            )],
        )]);

        let input = build_camera_capture_input(&packet, amigo_2d_spatial::DepthSpace2d::default());

        assert_eq!(
            input.wetness.as_ref().map(|source| source.kind),
            Some(amigo_render_api::VisualSourceKind2d::SceneWetness)
        );
        assert_eq!(
            input.wetness.as_ref().map(|source| source.id.0.as_str()),
            Some("world.wetness")
        );
        assert_eq!(
            input.wetness.as_ref().map(|source| source.availability),
            Some(amigo_render_api::VisualSourceAvailability2d::Produced)
        );
    }

    #[test]
    fn camera_capture_input_sets_normal_from_wet_reflections_noise_normal() {
        let mut packet = WgpuRenderFramePacket::default();
        packet.set_post_fx_stacks(vec![amigo_2d_post_fx::ScopedPostFx2dStack::new(
            "scene.weather",
            amigo_2d_post_fx::PostFxScope2d::Frame,
            vec![amigo_2d_post_fx::PostFx2dInstance::new(
                "wetness",
                amigo_2d_post_fx::PostFx2d::WetReflections(
                    amigo_2d_post_fx::PostFxWetReflections2d {
                        enabled: true,
                        reflection_mask:
                            "rotten-club/layered-images/neon-alley/reflection_mask.png".to_owned(),
                        noise_normal: Some(
                            "rotten-club/layered-images/neon-alley/rain-normal.png".to_owned(),
                        ),
                        ..Default::default()
                    },
                ),
            )],
        )]);

        let input = build_camera_capture_input(&packet, amigo_2d_spatial::DepthSpace2d::default());

        assert_eq!(
            input.normal.as_ref().map(|source| source.kind),
            Some(amigo_render_api::VisualSourceKind2d::SceneNormal)
        );
        assert_eq!(
            input.normal.as_ref().map(|source| source.id.0.as_str()),
            Some("world.normal")
        );
        assert_eq!(
            input.normal.as_ref().map(|source| source.availability),
            Some(amigo_render_api::VisualSourceAvailability2d::Produced)
        );
    }

    #[test]
    fn camera_capture_input_sets_motion_from_active_shutter_blur() {
        let mut packet = WgpuRenderFramePacket::default();
        packet.set_post_fx_stacks(vec![amigo_2d_post_fx::ScopedPostFx2dStack::new(
            "camera.main",
            amigo_2d_post_fx::PostFxScope2d::Frame,
            vec![amigo_2d_post_fx::PostFx2dInstance::new(
                "motion",
                amigo_2d_post_fx::PostFx2d::ShutterBlur(amigo_2d_post_fx::ShutterBlur2d {
                    opacity: 0.8,
                    shutter_angle: 180.0,
                    ..Default::default()
                }),
            )],
        )]);

        let input = build_camera_capture_input(&packet, amigo_2d_spatial::DepthSpace2d::default());

        assert_eq!(
            input.motion.as_ref().map(|source| source.kind),
            Some(amigo_render_api::VisualSourceKind2d::SceneMotion)
        );
        assert_eq!(
            input.motion.as_ref().map(|source| source.id.0.as_str()),
            Some("world.motion")
        );
    }
}
