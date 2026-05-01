use amigo_2d_particles::{Particle2dDrawCommand, Particle2dSceneService};
use amigo_2d_sprite::{SpriteDrawCommand, SpriteSceneService};
use amigo_2d_text::{Text2dDrawCommand, Text2dSceneService};
use amigo_2d_tilemap::{TileMap2dDrawCommand, TileMap2dSceneService};
use amigo_2d_vector::{VectorSceneService, VectorShape2dDrawCommand};
use amigo_3d_material::{MaterialDrawCommand, MaterialSceneService};
use amigo_3d_mesh::{MeshDrawCommand, MeshSceneService};
use amigo_3d_text::{Text3dDrawCommand, Text3dSceneService};
use amigo_render_api::{RenderFrameExtractor, RenderFrameExtractorRegistry};
use amigo_render_wgpu::UiOverlayDocument;
use amigo_scene::SceneService;
use amigo_ui::{UiSceneService, UiStateService, UiThemeService};

use crate::ui_runtime;

pub(crate) struct AppRenderExtractContext<'a> {
    pub(crate) scene_service: &'a SceneService,
    pub(crate) tilemap_scene_service: &'a TileMap2dSceneService,
    pub(crate) sprite_scene_service: &'a SpriteSceneService,
    pub(crate) text2d_scene_service: &'a Text2dSceneService,
    pub(crate) vector_scene_service: &'a VectorSceneService,
    pub(crate) particle2d_scene_service: &'a Particle2dSceneService,
    pub(crate) mesh_scene_service: &'a MeshSceneService,
    pub(crate) material_scene_service: &'a MaterialSceneService,
    pub(crate) text3d_scene_service: &'a Text3dSceneService,
    pub(crate) ui_scene_service: &'a UiSceneService,
    pub(crate) ui_state_service: &'a UiStateService,
    pub(crate) ui_theme_service: &'a UiThemeService,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct AppRenderFramePacket {
    world_2d_tilemaps: Vec<TileMap2dDrawCommand>,
    world_2d_sprites: Vec<SpriteDrawCommand>,
    world_2d_text: Vec<Text2dDrawCommand>,
    world_2d_vectors: Vec<VectorShape2dDrawCommand>,
    world_2d_particles: Vec<Particle2dDrawCommand>,
    world_3d_meshes: Vec<MeshDrawCommand>,
    world_3d_materials: Vec<MaterialDrawCommand>,
    world_3d_text: Vec<Text3dDrawCommand>,
    overlay: Vec<UiOverlayDocument>,
}

impl AppRenderFramePacket {
    pub(crate) fn push_world_2d_tilemap(&mut self, command: TileMap2dDrawCommand) {
        self.world_2d_tilemaps.push(command);
    }

    pub(crate) fn push_world_2d_sprite(&mut self, command: SpriteDrawCommand) {
        self.world_2d_sprites.push(command);
    }

    pub(crate) fn push_world_2d_vector(&mut self, command: VectorShape2dDrawCommand) {
        self.world_2d_vectors.push(command);
    }

    pub(crate) fn push_world_2d_text(&mut self, command: Text2dDrawCommand) {
        self.world_2d_text.push(command);
    }

    pub(crate) fn push_world_2d_particle(&mut self, command: Particle2dDrawCommand) {
        self.world_2d_particles.push(command);
    }

    pub(crate) fn push_world_3d_mesh(&mut self, command: MeshDrawCommand) {
        self.world_3d_meshes.push(command);
    }

    pub(crate) fn push_world_3d_material(&mut self, command: MaterialDrawCommand) {
        self.world_3d_materials.push(command);
    }

    pub(crate) fn push_world_3d_text(&mut self, command: Text3dDrawCommand) {
        self.world_3d_text.push(command);
    }

    pub(crate) fn extend_overlay<I>(&mut self, overlay: I)
    where
        I: IntoIterator<Item = UiOverlayDocument>,
    {
        self.overlay.extend(overlay);
    }

    pub(crate) fn world_2d_vectors(&self) -> &[VectorShape2dDrawCommand] {
        &self.world_2d_vectors
    }

    pub(crate) fn world_2d_sprites(&self) -> &[SpriteDrawCommand] {
        &self.world_2d_sprites
    }

    pub(crate) fn world_2d_tilemaps(&self) -> &[TileMap2dDrawCommand] {
        &self.world_2d_tilemaps
    }

    pub(crate) fn world_2d_text(&self) -> &[Text2dDrawCommand] {
        &self.world_2d_text
    }

    pub(crate) fn world_2d_particles(&self) -> &[Particle2dDrawCommand] {
        &self.world_2d_particles
    }

    pub(crate) fn world_3d_meshes(&self) -> &[MeshDrawCommand] {
        &self.world_3d_meshes
    }

    pub(crate) fn world_3d_materials(&self) -> &[MaterialDrawCommand] {
        &self.world_3d_materials
    }

    pub(crate) fn world_3d_text(&self) -> &[Text3dDrawCommand] {
        &self.world_3d_text
    }

    pub(crate) fn overlay(&self) -> &[UiOverlayDocument] {
        &self.overlay
    }
}

pub(crate) type AppRenderExtractorRegistry<'a> =
    RenderFrameExtractorRegistry<AppRenderExtractContext<'a>, AppRenderFramePacket>;

pub(crate) fn default_app_render_extractor_registry<'a>() -> AppRenderExtractorRegistry<'a> {
    let mut registry = RenderFrameExtractorRegistry::new();
    registry.register(ResolvedTileMap2dExtractor);
    registry.register(ResolvedSprite2dExtractor);
    registry.register(ResolvedText2dExtractor);
    registry.register(ResolvedVector2dExtractor);
    registry.register(ResolvedParticle2dExtractor);
    registry.register(ResolvedMesh3dExtractor);
    registry.register(ResolvedMaterial3dExtractor);
    registry.register(ResolvedText3dExtractor);
    registry.register(ResolvedUiOverlayExtractor);
    registry
}

pub(crate) fn build_sprite_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> SpriteSceneService {
    let service = SpriteSceneService::default();
    for command in packet.world_2d_sprites() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_tilemap_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> TileMap2dSceneService {
    let service = TileMap2dSceneService::default();
    for command in packet.world_2d_tilemaps() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_vector_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> VectorSceneService {
    let service = VectorSceneService::default();
    for command in packet.world_2d_vectors() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_text2d_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> Text2dSceneService {
    let service = Text2dSceneService::default();
    for command in packet.world_2d_text() {
        service.queue(command.clone());
    }
    service
}

#[cfg(test)]
pub(crate) fn build_text3d_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> Text3dSceneService {
    let service = Text3dSceneService::default();
    for command in packet.world_3d_text() {
        service.queue(command.clone());
    }
    service
}

#[cfg(test)]
pub(crate) fn build_mesh_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> MeshSceneService {
    let service = MeshSceneService::default();
    for command in packet.world_3d_meshes() {
        service.queue(command.clone());
    }
    service
}

#[cfg(test)]
pub(crate) fn build_material_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> MaterialSceneService {
    let service = MaterialSceneService::default();
    for command in packet.world_3d_materials() {
        service.queue(command.clone());
    }
    service
}

struct ResolvedTileMap2dExtractor;

struct ResolvedSprite2dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedTileMap2dExtractor
{
    fn name(&self) -> &'static str {
        "resolved_tilemap_2d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.tilemap_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_2d_tilemap(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedSprite2dExtractor
{
    fn name(&self) -> &'static str {
        "resolved_sprite_2d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.sprite_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_2d_sprite(command);
            }
        }
    }
}

struct ResolvedVector2dExtractor;

struct ResolvedParticle2dExtractor;

struct ResolvedText2dExtractor;

struct ResolvedMesh3dExtractor;

struct ResolvedMaterial3dExtractor;

struct ResolvedText3dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedVector2dExtractor
{
    fn name(&self) -> &'static str {
        "resolved_vector_2d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.vector_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_2d_vector(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedText2dExtractor
{
    fn name(&self) -> &'static str {
        "resolved_text_2d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.text2d_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_2d_text(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedMesh3dExtractor
{
    fn name(&self) -> &'static str {
        "resolved_mesh_3d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.mesh_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_3d_mesh(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedMaterial3dExtractor
{
    fn name(&self) -> &'static str {
        "resolved_material_3d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.material_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_3d_material(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedText3dExtractor
{
    fn name(&self) -> &'static str {
        "resolved_text_3d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.text3d_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_3d_text(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedParticle2dExtractor
{
    fn name(&self) -> &'static str {
        "resolved_particle_2d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.particle2d_scene_service.draw_commands() {
            packet.push_world_2d_particle(command);
        }
    }
}

fn is_entity_render_visible(scene_service: &SceneService, entity_name: &str) -> bool {
    scene_service
        .entity_by_name(entity_name)
        .map(|entity| entity.lifecycle.visible)
        .unwrap_or(true)
}

struct ResolvedUiOverlayExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedUiOverlayExtractor
{
    fn name(&self) -> &'static str {
        "resolved_ui_overlay"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        let overlays = ui_runtime::resolve_ui_overlay_documents(
            context.ui_scene_service,
            context.ui_state_service,
            context.ui_theme_service,
        )
        .into_iter()
        .map(|document| document.overlay);
        packet.extend_overlay(overlays);
    }
}

#[cfg(test)]
mod tests {
    use amigo_assets::AssetKey;
    use amigo_math::{ColorRgba, Transform2, Transform3, Vec2};
    use amigo_scene::SceneEntityId;
    use amigo_ui::{
        UiDocument as RuntimeUiDocument, UiDrawCommand, UiLayer as RuntimeUiLayer,
        UiNode as RuntimeUiNode, UiNodeKind as RuntimeUiNodeKind, UiSceneService, UiStateService,
        UiStyle as RuntimeUiStyle, UiTarget as RuntimeUiTarget, UiTheme, UiThemePalette,
    };

    use super::*;
    use amigo_2d_particles::Particle2dSceneService;
    use amigo_2d_sprite::{Sprite, SpriteSheet};
    use amigo_2d_text::Text2d;
    use amigo_2d_tilemap::TileMap2d;
    use amigo_2d_vector::{VectorShape2d, VectorShapeKind2d, VectorStyle2d};
    use amigo_3d_material::Material3d;
    use amigo_3d_mesh::Mesh3d;
    use amigo_3d_text::Text3d;

    fn hud_document(entity_name: &str, text: &str) -> UiDrawCommand {
        UiDrawCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: entity_name.to_owned(),
            document: RuntimeUiDocument {
                target: RuntimeUiTarget::ScreenSpace {
                    layer: RuntimeUiLayer::Hud,
                    viewport: None,
                },
                root: RuntimeUiNode {
                    id: Some("root".to_owned()),
                    kind: RuntimeUiNodeKind::Text {
                        content: text.to_owned(),
                        font: None,
                    },
                    style_class: Some("root".to_owned()),
                    style: RuntimeUiStyle::default(),
                    binds: Default::default(),
                    events: Default::default(),
                    children: Vec::new(),
                },
            },
        }
    }

    #[test]
    fn app_render_extractor_registry_collects_vector_and_ui_data() {
        let tilemaps = TileMap2dSceneService::default();
        tilemaps.queue(TileMap2dDrawCommand {
            entity_id: SceneEntityId::new(2),
            entity_name: "arena".to_owned(),
            tilemap: TileMap2d {
                tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
                ruleset: None,
                tile_size: Vec2::new(16.0, 16.0),
                grid: vec!["....".to_owned(), "####".to_owned()],
                origin_offset: Vec2::new(0.0, 0.0),
                resolved: None,
            },
            z_index: -1.0,
        });
        let sprites = SpriteSceneService::default();
        sprites.queue(SpriteDrawCommand {
            entity_id: SceneEntityId::new(5),
            entity_name: "player".to_owned(),
            sprite: Sprite {
                texture: AssetKey::new("playground-2d/textures/sprite-lab"),
                size: Vec2::new(32.0, 32.0),
                sheet: Some(SpriteSheet {
                    columns: 4,
                    rows: 1,
                    frame_count: 4,
                    frame_size: Vec2::new(32.0, 32.0),
                    fps: 8.0,
                    looping: true,
                }),
                sheet_is_explicit: true,
                animation_override: None,
                frame_index: 2,
                frame_elapsed: 0.1,
            },
            z_index: 1.0,
            transform: Transform2::default(),
        });
        let vectors = VectorSceneService::default();
        vectors.queue(VectorShape2dDrawCommand {
            entity_id: SceneEntityId::new(7),
            entity_name: "ship".to_owned(),
            shape: VectorShape2d {
                kind: VectorShapeKind2d::Polyline {
                    points: vec![Vec2::new(0.0, 12.0), Vec2::new(-8.0, -8.0)],
                    closed: true,
                },
                style: VectorStyle2d {
                    stroke_color: ColorRgba::WHITE,
                    stroke_width: 2.0,
                    fill_color: None,
                },
            },
            z_index: 2.0,
            transform: Transform2::default(),
        });
        let particles = Particle2dSceneService::default();
        particles.queue_emitter(amigo_2d_particles::ParticleEmitter2dCommand {
            entity_id: SceneEntityId::new(14),
            entity_name: "spark".to_owned(),
            emitter: amigo_2d_particles::ParticleEmitter2d {
                attached_to: None,
                local_offset: Vec2::ZERO,
                local_direction_radians: 0.0,
                spawn_area: amigo_2d_particles::ParticleSpawnArea2d::Point,
                active: true,
                spawn_rate: 1.0,
                max_particles: 4,
                particle_lifetime: 1.0,
                lifetime_jitter: 0.0,
                initial_speed: 0.0,
                speed_jitter: 0.0,
                spread_radians: 0.0,
                inherit_parent_velocity: 0.0,
                initial_size: 2.0,
                final_size: 2.0,
                color: ColorRgba::WHITE,
                color_ramp: None,
                z_index: 3.5,
                shape: amigo_2d_particles::ParticleShape2d::Circle { segments: 8 },
                shape_choices: Vec::new(),
                shape_over_lifetime: Vec::new(),
                align: amigo_2d_particles::ParticleAlignMode2d::Velocity,
                blend_mode: amigo_2d_particles::ParticleBlendMode2d::Alpha,
                emission_rate_curve: amigo_math::Curve1d::Constant(1.0),
                size_curve: amigo_math::Curve1d::Constant(1.0),
                alpha_curve: amigo_math::Curve1d::Constant(1.0),
                speed_curve: amigo_math::Curve1d::Constant(1.0),
                forces: Vec::new(),
            },
        });
        particles.tick(
            &[amigo_2d_particles::Particle2dEmitterRuntimeInput {
                emitter_entity_name: "spark".to_owned(),
                source_entity_name: "spark".to_owned(),
                source_transform: Transform2::default(),
                source_velocity: Vec2::ZERO,
                source_visible: true,
                source_simulation_enabled: true,
            }],
            1.0,
        );
        vectors.queue(VectorShape2dDrawCommand {
            entity_id: SceneEntityId::new(13),
            entity_name: "hidden-dot".to_owned(),
            shape: VectorShape2d {
                kind: VectorShapeKind2d::Circle {
                    radius: 1.0,
                    segments: 8,
                },
                style: VectorStyle2d {
                    stroke_color: ColorRgba::WHITE,
                    stroke_width: 1.0,
                    fill_color: Some(ColorRgba::WHITE),
                },
            },
            z_index: 3.0,
            transform: Transform2::default(),
        });
        let text2d = Text2dSceneService::default();
        text2d.queue(Text2dDrawCommand {
            entity_id: SceneEntityId::new(8),
            entity_name: "label".to_owned(),
            text: Text2d {
                content: "AMIGO".to_owned(),
                font: AssetKey::new("playground-2d/fonts/debug-ui"),
                bounds: Vec2::new(320.0, 64.0),
                transform: Transform2::default(),
            },
        });
        let text3d = Text3dSceneService::default();
        text3d.queue(Text3dDrawCommand {
            entity_id: SceneEntityId::new(10),
            entity_name: "hello-3d".to_owned(),
            text: Text3d {
                content: "HELLO".to_owned(),
                font: AssetKey::new("playground-3d/fonts/debug-3d"),
                size: 0.5,
                transform: Transform3::default(),
            },
        });
        let meshes = MeshSceneService::default();
        meshes.queue(MeshDrawCommand {
            entity_id: SceneEntityId::new(11),
            entity_name: "probe-mesh".to_owned(),
            mesh: Mesh3d {
                mesh_asset: AssetKey::new("playground-3d/meshes/probe"),
                transform: Transform3::default(),
            },
        });
        let materials = MaterialSceneService::default();
        materials.queue(MaterialDrawCommand {
            entity_id: SceneEntityId::new(12),
            entity_name: "probe-material".to_owned(),
            material: Material3d {
                label: "debug-surface".to_owned(),
                albedo: ColorRgba::WHITE,
                source: Some(AssetKey::new("playground-3d/materials/debug-surface")),
            },
        });

        let ui_scene = UiSceneService::default();
        let ui_state = UiStateService::default();
        let ui_theme = UiThemeService::default();
        ui_theme.register_theme(UiTheme::from_palette(
            "space_dark",
            UiThemePalette {
                background: ColorRgba::new(0.02, 0.03, 0.07, 1.0),
                surface: ColorRgba::new(0.08, 0.1, 0.15, 1.0),
                surface_alt: ColorRgba::new(0.1, 0.12, 0.18, 1.0),
                text: ColorRgba::WHITE,
                text_muted: ColorRgba::new(0.6, 0.7, 0.8, 1.0),
                border: ColorRgba::new(0.2, 0.4, 0.6, 1.0),
                accent: ColorRgba::new(0.0, 0.8, 1.0, 1.0),
                accent_text: ColorRgba::new(0.0, 0.05, 0.08, 1.0),
                danger: ColorRgba::new(1.0, 0.1, 0.2, 1.0),
                warning: ColorRgba::new(1.0, 0.7, 0.0, 1.0),
                success: ColorRgba::new(0.2, 1.0, 0.5, 1.0),
            },
        ));
        ui_theme.set_active_theme("space_dark");
        ui_scene.queue(hud_document("hud", "Hello"));
        let scene = SceneService::default();
        scene.spawn("hidden-dot");
        scene.set_visible("hidden-dot", false);

        let context = AppRenderExtractContext {
            scene_service: &scene,
            tilemap_scene_service: &tilemaps,
            sprite_scene_service: &sprites,
            text2d_scene_service: &text2d,
            vector_scene_service: &vectors,
            particle2d_scene_service: &particles,
            mesh_scene_service: &meshes,
            material_scene_service: &materials,
            text3d_scene_service: &text3d,
            ui_scene_service: &ui_scene,
            ui_state_service: &ui_state,
            ui_theme_service: &ui_theme,
        };

        let packet = default_app_render_extractor_registry().extract_all(&context);

        assert_eq!(packet.world_2d_tilemaps().len(), 1);
        assert_eq!(packet.world_2d_tilemaps()[0].entity_name, "arena");
        assert_eq!(packet.world_2d_sprites().len(), 1);
        assert_eq!(packet.world_2d_sprites()[0].entity_name, "player");
        assert_eq!(packet.world_2d_text().len(), 1);
        assert_eq!(packet.world_2d_text()[0].entity_name, "label");
        assert_eq!(packet.world_2d_vectors().len(), 1);
        assert_eq!(packet.world_2d_vectors()[0].entity_name, "ship");
        assert_eq!(packet.world_2d_particles().len(), 1);
        assert_eq!(packet.world_3d_meshes().len(), 1);
        assert_eq!(packet.world_3d_meshes()[0].entity_name, "probe-mesh");
        assert_eq!(packet.world_3d_materials().len(), 1);
        assert_eq!(packet.world_3d_materials()[0].entity_name, "probe-material");
        assert_eq!(packet.world_3d_text().len(), 1);
        assert_eq!(packet.world_3d_text()[0].entity_name, "hello-3d");
        assert_eq!(packet.overlay().len(), 1);
        assert_eq!(packet.overlay()[0].entity_name, "hud");
        assert_eq!(
            packet.overlay()[0].root.style.background,
            Some(ColorRgba::new(0.02, 0.03, 0.07, 1.0))
        );
    }

    #[test]
    fn rebuilds_vector_scene_service_from_packet() {
        let mut packet = AppRenderFramePacket::default();
        packet.push_world_2d_vector(VectorShape2dDrawCommand {
            entity_id: SceneEntityId::new(9),
            entity_name: "asteroid".to_owned(),
            shape: VectorShape2d {
                kind: VectorShapeKind2d::Polygon {
                    points: vec![
                        Vec2::new(-8.0, 0.0),
                        Vec2::new(0.0, 8.0),
                        Vec2::new(8.0, 0.0),
                    ],
                },
                style: VectorStyle2d::default(),
            },
            z_index: 1.0,
            transform: Transform2::default(),
        });

        let rebuilt = build_vector_scene_service_from_packet(&packet);

        assert_eq!(rebuilt.commands().len(), 1);
        assert_eq!(rebuilt.commands()[0].entity_name, "asteroid");
    }

    #[test]
    fn rebuilds_sprite_scene_service_from_packet() {
        let mut packet = AppRenderFramePacket::default();
        packet.push_world_2d_sprite(SpriteDrawCommand {
            entity_id: SceneEntityId::new(3),
            entity_name: "coin".to_owned(),
            sprite: Sprite {
                texture: AssetKey::new("playground-sidescroller/textures/coin"),
                size: Vec2::new(16.0, 16.0),
                sheet: Some(SpriteSheet {
                    columns: 4,
                    rows: 1,
                    frame_count: 4,
                    frame_size: Vec2::new(16.0, 16.0),
                    fps: 8.0,
                    looping: true,
                }),
                sheet_is_explicit: false,
                animation_override: None,
                frame_index: 1,
                frame_elapsed: 0.0,
            },
            z_index: 0.0,
            transform: Transform2::default(),
        });

        let rebuilt = build_sprite_scene_service_from_packet(&packet);

        assert_eq!(rebuilt.commands().len(), 1);
        assert_eq!(rebuilt.commands()[0].entity_name, "coin");
        assert_eq!(rebuilt.commands()[0].sprite.frame_index, 1);
    }

    #[test]
    fn rebuilds_text2d_scene_service_from_packet() {
        let mut packet = AppRenderFramePacket::default();
        packet.push_world_2d_text(Text2dDrawCommand {
            entity_id: SceneEntityId::new(4),
            entity_name: "caption".to_owned(),
            text: Text2d {
                content: "Vector Demo".to_owned(),
                font: AssetKey::new("playground-2d/fonts/debug-ui"),
                bounds: Vec2::new(240.0, 48.0),
                transform: Transform2::default(),
            },
        });

        let rebuilt = build_text2d_scene_service_from_packet(&packet);

        assert_eq!(rebuilt.commands().len(), 1);
        assert_eq!(rebuilt.commands()[0].entity_name, "caption");
        assert_eq!(rebuilt.commands()[0].text.content, "Vector Demo");
    }

    #[test]
    fn rebuilds_tilemap_scene_service_from_packet() {
        let mut packet = AppRenderFramePacket::default();
        packet.push_world_2d_tilemap(TileMap2dDrawCommand {
            entity_id: SceneEntityId::new(12),
            entity_name: "tilemap".to_owned(),
            tilemap: TileMap2d {
                tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
                ruleset: None,
                tile_size: Vec2::new(16.0, 16.0),
                grid: vec!["....".to_owned(), ".##.".to_owned()],
                origin_offset: Vec2::new(0.0, 0.0),
                resolved: None,
            },
            z_index: 0.0,
        });

        let rebuilt = build_tilemap_scene_service_from_packet(&packet);

        assert_eq!(rebuilt.commands().len(), 1);
        assert_eq!(rebuilt.commands()[0].entity_name, "tilemap");
        assert_eq!(rebuilt.commands()[0].tilemap.grid.len(), 2);
    }

    #[test]
    fn rebuilds_text3d_scene_service_from_packet() {
        let mut packet = AppRenderFramePacket::default();
        packet.push_world_3d_text(Text3dDrawCommand {
            entity_id: SceneEntityId::new(16),
            entity_name: "caption-3d".to_owned(),
            text: Text3d {
                content: "AMIGO 3D".to_owned(),
                font: AssetKey::new("playground-3d/fonts/debug-3d"),
                size: 0.75,
                transform: Transform3::default(),
            },
        });

        let rebuilt = build_text3d_scene_service_from_packet(&packet);

        assert_eq!(rebuilt.commands().len(), 1);
        assert_eq!(rebuilt.commands()[0].entity_name, "caption-3d");
        assert_eq!(rebuilt.commands()[0].text.content, "AMIGO 3D");
    }

    #[test]
    fn rebuilds_mesh_scene_service_from_packet() {
        let mut packet = AppRenderFramePacket::default();
        packet.push_world_3d_mesh(MeshDrawCommand {
            entity_id: SceneEntityId::new(18),
            entity_name: "probe-mesh".to_owned(),
            mesh: Mesh3d {
                mesh_asset: AssetKey::new("playground-3d/meshes/probe"),
                transform: Transform3::default(),
            },
        });

        let rebuilt = build_mesh_scene_service_from_packet(&packet);

        assert_eq!(rebuilt.commands().len(), 1);
        assert_eq!(rebuilt.commands()[0].entity_name, "probe-mesh");
    }

    #[test]
    fn rebuilds_material_scene_service_from_packet() {
        let mut packet = AppRenderFramePacket::default();
        packet.push_world_3d_material(MaterialDrawCommand {
            entity_id: SceneEntityId::new(19),
            entity_name: "probe-material".to_owned(),
            material: Material3d {
                label: "debug-surface".to_owned(),
                albedo: ColorRgba::WHITE,
                source: Some(AssetKey::new("playground-3d/materials/debug-surface")),
            },
        });

        let rebuilt = build_material_scene_service_from_packet(&packet);

        assert_eq!(rebuilt.commands().len(), 1);
        assert_eq!(rebuilt.commands()[0].entity_name, "probe-material");
        assert_eq!(rebuilt.commands()[0].material.label, "debug-surface");
    }
}
