use std::fs;
use std::path::Path;
use std::sync::Arc;

use amigo_2d_composition::Composition2dPlugin;
use amigo_2d_physics::Physics2dPlugin;
use amigo_beacon_light_2d_plugin::Beacon2dPlugin;
use amigo_composite_plugin::PostFx2dPlugin;
use amigo_core::{AmigoError, AmigoResult};
use amigo_focus_depth_plugin::{DepthMap2dPlugin, FocusTargets2dRuntimePlugin};
use amigo_layered_image_2d_plugin::LayeredImagePlugin;
use amigo_light_2d_plugin::Lighting2dPlugin;
use amigo_particles_2d_plugin::{
    Particle2dPlugin, Particle2dSourceVelocityProvider, ParticleEmitter2d, ParticlePreset2d,
};
use amigo_runtime::{PluginBundle, Runtime, RuntimeBuilder, RuntimePlugin, ServiceRegistry};
use amigo_scene::{
    build_scene_hydration_plan, SceneCommand, SceneComponentDocument, SceneDocument,
    SceneEntityDocument, SceneMetadataDocument,
};
use amigo_session::RuntimeSession;
use amigo_shutter_motion_plugin::{Motion2dSceneService, MOTION_2D_PLUGIN};
use amigo_sprite_2d_plugin::SpritePlugin;
use amigo_text_2d_plugin::Text2dPlugin;
use amigo_tilemap_2d_plugin::TileMap2dPlugin;
use amigo_ui::UiPlugin;
use amigo_vector_2d_plugin::Vector2dPlugin;

use crate::{LoadedAssetDomainPreparer, LoadedAssetDomainPreparerRegistry};
use crate::render_extractor_bridges;
use crate::render_extractor_registry::WgpuRenderExtractorBridgeRegistry;

pub fn load_particle_preset_file(source_mod: &str, path: &Path) -> AmigoResult<ParticlePreset2d> {
    let raw = fs::read_to_string(path)?;
    let document = serde_yaml::from_str::<serde_yaml::Value>(&raw).map_err(|error| {
        AmigoError::Message(format!(
            "failed to parse particle preset `{}`: {error}",
            path.display()
        ))
    })?;
    if string_field(&document, "kind") != Some("particle-preset-2d") {
        return Err(AmigoError::Message(format!(
            "particle preset `{}` must declare kind: particle-preset-2d",
            path.display()
        )));
    }

    let id = string_field(&document, "id")
        .filter(|id| !id.is_empty())
        .ok_or_else(|| {
            AmigoError::Message(format!(
                "particle preset `{}` must declare non-empty id",
                path.display()
            ))
        })?
        .to_owned();
    let label = string_field(&document, "label")
        .unwrap_or(id.as_str())
        .to_owned();
    let category = string_field(&document, "category")
        .unwrap_or_default()
        .to_owned();
    let tags = string_sequence_field(&document, "tags");
    let emitter_value = mapping_value(&document, "emitter").ok_or_else(|| {
        AmigoError::Message(format!(
            "particle preset `{}` must declare emitter",
            path.display()
        ))
    })?;
    let emitter_component = serde_yaml::from_value::<SceneComponentDocument>(emitter_value.clone())
        .map_err(|error| {
            AmigoError::Message(format!(
                "failed to parse emitter in particle preset `{}`: {error}",
                path.display()
            ))
        })?;
    if !emitter_component.is_particle_emitter_2d() {
        return Err(AmigoError::Message(format!(
            "particle preset `{}` emitter must be type: ParticleEmitter2D",
            path.display()
        )));
    }

    let scene_document = SceneDocument {
        version: 1,
        scene: SceneMetadataDocument {
            id: format!("particle-preset-{id}"),
            label: label.clone(),
            description: None,
        },
        transitions: Vec::new(),
        collision_events: Vec::new(),
        audio_cues: Vec::new(),
        activation_sets: Vec::new(),
        visual2d: Default::default(),
        state: Default::default(),
        entities: vec![SceneEntityDocument {
            id: id.clone(),
            name: format!("particle-preset-{id}"),
            tags: Vec::new(),
            groups: Vec::new(),
            visible: false,
            simulation_enabled: false,
            collision_enabled: false,
            properties: Default::default(),
            transform2: None,
            transform3: None,
            post_fx: Vec::new(),
            prefab: None,
            prefab_overrides: Vec::new(),
            components: vec![emitter_component],
        }],
    };
    let plan = build_scene_hydration_plan(source_mod, &scene_document).map_err(|error| {
        AmigoError::Message(format!(
            "failed to hydrate particle preset `{}`: {error}",
            path.display()
        ))
    })?;
    let emitter = plan
        .commands
        .iter()
        .find_map(|command| match command {
            SceneCommand::Plugin { command } => command
                .payload_as::<amigo_scene::ParticleEmitter2dSceneCommand>()
                .map(ParticleEmitter2d::from_scene_command),
            _ => None,
        })
        .ok_or_else(|| {
            AmigoError::Message(format!(
                "particle preset `{}` did not produce ParticleEmitter2D command",
                path.display()
            ))
        })?;

    Ok(ParticlePreset2d {
        source_mod: source_mod.to_owned(),
        id,
        label,
        category,
        tags,
        emitter,
    })
}

pub fn load_particle_preset_catalog(runtime: &Runtime) -> AmigoResult<()> {
    let mod_catalog = runtime.required::<amigo_modding::ModCatalog>()?;
    let presets = runtime.required::<amigo_particles_2d_plugin::ParticlePreset2dService>()?;
    presets.clear();

    for discovered_mod in mod_catalog.mods() {
        let preset_dir = discovered_mod.root_path.join("presets");
        if !preset_dir.is_dir() {
            continue;
        }

        for entry in fs::read_dir(&preset_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("yml") {
                continue;
            }
            let preset = load_particle_preset_file(&discovered_mod.manifest.id, &path)?;
            presets.register(preset);
        }
    }

    Ok(())
}

pub fn tick_ui_bindings(runtime: &amigo_runtime::Runtime) -> AmigoResult<()> {
    amigo_ui::tick_ui_bindings(runtime)
}

pub fn collect_scene_ui_font_asset_keys(
    document: &amigo_scene::SceneUiDocument,
) -> Vec<amigo_assets::AssetKey> {
    amigo_ui::collect_scene_ui_font_asset_keys(document)
}

pub fn scene_ui_document_to_runtime_document(
    document: &amigo_scene::SceneUiDocument,
) -> amigo_ui::UiDocument {
    amigo_ui::scene_ui_document_to_runtime_document(document)
}

fn string_field<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a str> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(serde_yaml::Value::String(key.to_owned())))
        .and_then(serde_yaml::Value::as_str)
}

fn mapping_value<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a serde_yaml::Value> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(serde_yaml::Value::String(key.to_owned())))
}

fn string_sequence_field(value: &serde_yaml::Value, key: &str) -> Vec<String> {
    mapping_value(value, key)
        .and_then(serde_yaml::Value::as_sequence)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_yaml::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

pub struct TwoDRuntimeBundle;

struct WgpuTwoDRenderExtractorBridgePlugin;
struct LoadedAssetTwoDMetadataPreparerPlugin;
struct Particle2dMotionVelocityBridgePlugin;

struct Motion2dParticleSourceVelocityProvider {
    motion: std::sync::Arc<Motion2dSceneService>,
}

impl Particle2dSourceVelocityProvider for Motion2dParticleSourceVelocityProvider {
    fn source_velocity(&self, entity_name: &str) -> Option<amigo_math::Vec2> {
        Some(self.motion.current_velocity(entity_name))
    }
}

fn two_d_profile_render_extractor_bridge_installers(
) -> Vec<crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller> {
    render_extractor_bridges::available_world_2d_plugin_bridge_installers()
}

fn register_two_d_profile_render_extractor_bridges(bridges: &WgpuRenderExtractorBridgeRegistry) {
    for installer in two_d_profile_render_extractor_bridge_installers() {
        bridges.register_installer(installer);
    }
}

impl PluginBundle for TwoDRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-2d-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(SpritePlugin)?
            .with_plugin(LayeredImagePlugin)?
            .with_plugin(DepthMap2dPlugin)?
            .with_plugin(Lighting2dPlugin)?
            .with_plugin(Composition2dPlugin)?
            .with_plugin(PostFx2dPlugin)?
            .with_plugin(Text2dPlugin)?
            .with_plugin(Vector2dPlugin)?
            .with_plugin(Beacon2dPlugin)?
            .with_plugin(Particle2dPlugin)?
            .with_plugin(UiPlugin)?
            .with_plugin(Physics2dPlugin)?
            .with_plugin(TileMap2dPlugin)?
            .with_plugin(LoadedAssetTwoDMetadataPreparerPlugin)?
            .with_plugin(MOTION_2D_PLUGIN)?
            .with_plugin(Particle2dMotionVelocityBridgePlugin)?
            .with_plugin(FocusTargets2dRuntimePlugin)?
            .with_plugin(WgpuTwoDRenderExtractorBridgePlugin)
    }
}

struct SpriteLoadedAssetDomainPreparer {
    sprites: Arc<amigo_sprite_2d_plugin::SpriteSceneService>,
}

impl LoadedAssetDomainPreparer for SpriteLoadedAssetDomainPreparer {
    fn name(&self) -> &'static str {
        "amigo-sprite-2d-loaded-asset-domain-preparer"
    }

    fn prepare(&self, asset_catalog: &amigo_assets::AssetCatalog, asset_key: &amigo_assets::AssetKey) {
        let Some(prepared) = asset_catalog.prepared_asset(asset_key) else {
            return;
        };
        if let Some(sheet) = amigo_sprite_2d_plugin::infer_sprite_sheet_from_prepared_asset(&prepared)
        {
            self.sprites.sync_sheet_for_texture(asset_key, sheet);
        }
    }
}

struct TileMapLoadedAssetDomainPreparer {
    tilemaps: Arc<amigo_tilemap_2d_plugin::TileMap2dSceneService>,
}

impl LoadedAssetDomainPreparer for TileMapLoadedAssetDomainPreparer {
    fn name(&self) -> &'static str {
        "amigo-tilemap-2d-loaded-asset-domain-preparer"
    }

    fn prepare(&self, asset_catalog: &amigo_assets::AssetCatalog, asset_key: &amigo_assets::AssetKey) {
        let Some(prepared) = asset_catalog.prepared_asset(asset_key) else {
            return;
        };
        if let Some(ruleset) =
            amigo_tilemap_2d_plugin::infer_tile_ruleset_from_prepared_asset(&prepared)
        {
            self.tilemaps.sync_ruleset_for_asset(asset_key, &ruleset);
        }
    }
}

impl RuntimePlugin for LoadedAssetTwoDMetadataPreparerPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-loaded-asset-domain-preparers"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        let preparers = registry.required::<LoadedAssetDomainPreparerRegistry>()?;
        let sprites = registry.required::<amigo_sprite_2d_plugin::SpriteSceneService>()?;
        let tilemaps = registry.required::<amigo_tilemap_2d_plugin::TileMap2dSceneService>()?;

        preparers.register(Arc::new(SpriteLoadedAssetDomainPreparer { sprites }));
        preparers.register(Arc::new(TileMapLoadedAssetDomainPreparer { tilemaps }));

        Ok(())
    }
}

impl RuntimePlugin for Particle2dMotionVelocityBridgePlugin {
    fn name(&self) -> &'static str {
        "amigo-particles-2d-motion-velocity-bridge"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        if let (Some(particles), Some(motion)) = (
            registry.resolve::<amigo_particles_2d_plugin::Particle2dSourceVelocityProviderRegistry>(
            ),
            registry.resolve::<Motion2dSceneService>(),
        ) {
            particles.register(Arc::new(Motion2dParticleSourceVelocityProvider { motion }));
        }

        Ok(())
    }
}

impl RuntimePlugin for WgpuTwoDRenderExtractorBridgePlugin {
    fn name(&self) -> &'static str {
        "amigo-wgpu-2d-render-extractor-bridges"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        let bridges = registry.required::<WgpuRenderExtractorBridgeRegistry>()?;
        register_two_d_profile_render_extractor_bridges(bridges.as_ref());

        Ok(())
    }
}

pub fn register_two_d_runtime_capabilities(session: &mut RuntimeSession) {
    amigo_text_2d_plugin::register_text2d_runtime_capabilities(session);
    amigo_sprite_2d_plugin::register_sprite2d_runtime_capabilities(session);
    amigo_tilemap_2d_plugin::register_tilemap2d_runtime_capabilities(session);
    amigo_layered_image_2d_plugin::register_layered_image_runtime_capabilities(session);
    amigo_focus_depth_plugin::register_depth_map_runtime_capabilities(session);
    amigo_2d_composition::register_composition2d_runtime_capabilities(session);
    amigo_light_2d_plugin::register_lighting2d_runtime_capabilities(session);
    amigo_composite_plugin::register_post_fx_runtime_capabilities(session);
    amigo_particles_2d_plugin::register_particles2d_runtime_capabilities(session);
    amigo_shutter_motion_plugin::register_motion2d_runtime_capabilities(session);
    amigo_2d_physics::register_physics2d_runtime_capabilities(session);
    amigo_vector_2d_plugin::register_vector2d_runtime_capabilities(session);
    amigo_beacon_light_2d_plugin::register_beacon2d_runtime_capabilities(session);
    amigo_ui::register_ui_runtime_capabilities(session);
}
