use amigo_core::{AmigoError, AmigoResult};
use amigo_plugin_api::{PluginId, PluginManifest};
use amigo_plugin_index::{plan_plugin_composition, validate_plugin_index, PluginIndex};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

pub struct EmbeddedPluginManifestsPlugin;

#[derive(Clone, Debug)]
pub struct RuntimePluginCompositionPlan {
    pub ordered_plugin_ids: Vec<PluginId>,
}

const MANIFESTS: &[(&str, &str)] = &[
    ("camera/camera-core", include_str!("../../../../plugins/camera/camera-core/plugin.toml")),
    ("camera/camera-optics", include_str!("../../../../plugins/camera/camera-optics/plugin.toml")),
    ("camera/focus-depth", include_str!("../../../../plugins/camera/focus-depth/plugin.toml")),
    ("camera/shutter-motion", include_str!("../../../../plugins/camera/shutter-motion/plugin.toml")),
    ("camera/film-look", include_str!("../../../../plugins/camera/film-look/plugin.toml")),
    ("camera/camera-profiles", include_str!("../../../../plugins/camera/camera-profiles/plugin.toml")),
    ("devtools/codemap", include_str!("../../../../plugins/devtools/codemap/plugin.toml")),
    ("gfx/sprite-2d", include_str!("../../../../plugins/gfx/sprite-2d/plugin.toml")),
    ("gfx/text-2d", include_str!("../../../../plugins/gfx/text-2d/plugin.toml")),
    ("gfx/vector-2d", include_str!("../../../../plugins/gfx/vector-2d/plugin.toml")),
    ("gfx/layered-image-2d", include_str!("../../../../plugins/gfx/layered-image-2d/plugin.toml")),
    ("gfx/tilemap-2d", include_str!("../../../../plugins/gfx/tilemap-2d/plugin.toml")),
    ("lighting/light-2d", include_str!("../../../../plugins/lighting/light-2d/plugin.toml")),
    ("lighting/light-groups-2d", include_str!("../../../../plugins/lighting/light-groups-2d/plugin.toml")),
    ("lighting/lightmaps-2d", include_str!("../../../../plugins/lighting/lightmaps-2d/plugin.toml")),
    ("lighting/beacon-light-2d", include_str!("../../../../plugins/lighting/beacon-light-2d/plugin.toml")),
    ("lighting/relight-2d", include_str!("../../../../plugins/lighting/relight-2d/plugin.toml")),
    ("materials/material-2d", include_str!("../../../../plugins/materials/material-2d/plugin.toml")),
    ("materials/material-maps", include_str!("../../../../plugins/materials/material-maps/plugin.toml")),
    ("materials/procedural-materials", include_str!("../../../../plugins/materials/procedural-materials/plugin.toml")),
    ("vfx/particles-2d", include_str!("../../../../plugins/vfx/particles-2d/plugin.toml")),
    ("vfx/trails-2d", include_str!("../../../../plugins/vfx/trails-2d/plugin.toml")),
    ("postfx/bloom", include_str!("../../../../plugins/postfx/bloom/plugin.toml")),
    ("postfx/color-grading", include_str!("../../../../plugins/postfx/color-grading/plugin.toml")),
    ("postfx/scopes", include_str!("../../../../plugins/postfx/scopes/plugin.toml")),
    ("postfx/composite", include_str!("../../../../plugins/postfx/composite/plugin.toml")),
    ("postfx/debug-views", include_str!("../../../../plugins/postfx/debug-views/plugin.toml")),
    ("gameplay/behavior", include_str!("../../../../plugins/gameplay/behavior/plugin.toml")),
];

fn embedded_manifests() -> AmigoResult<Vec<PluginManifest>> {
    MANIFESTS
        .iter()
        .map(|(path, source)| {
            amigo_plugin_manifest::parse_plugin_manifest_str(source).map_err(|error| {
                AmigoError::Message(format!("invalid embedded plugin manifest `{path}`: {error:?}"))
            })
        })
        .collect()
}

impl RuntimePlugin for EmbeddedPluginManifestsPlugin {
    fn name(&self) -> &'static str { "amigo-embedded-plugin-manifests" }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        let manifests = embedded_manifests()?;
        for manifest in &manifests {
            amigo_capabilities::register_plugin_manifest(registry, manifest)?;
        }
        let index = PluginIndex::from_manifests(manifests);
        validate_plugin_index(&index).map_err(|errors| {
            AmigoError::Message(format!("embedded plugin index is invalid: {errors:?}"))
        })?;
        let plan = plan_plugin_composition(&index).map_err(|error| {
            AmigoError::Message(format!("embedded plugin composition cannot be planned: {error:?}"))
        })?;
        registry.register(RuntimePluginCompositionPlan {
            ordered_plugin_ids: plan.ordered_plugins,
        })?;
        Ok(())
    }
}
