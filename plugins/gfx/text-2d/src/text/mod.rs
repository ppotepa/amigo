//! 2D text scene service for labels and HUD-style content.
//! It queues world-space text state that the renderer consumes for captions and lightweight UI.

use std::sync::Mutex;

use amigo_composite_plugin::PostFxHost2dId;
use amigo_assets::AssetKey;
use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
use amigo_camera_optics_plugin::api::CameraOpticalResponse2d;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_material_2d_plugin::{
    Material2d, Material2dLighting, Material2dOptical, Material2dOpticalMode,
};
use amigo_render_api::{
    render_contribution_roles, RenderContributionSet,
};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{
    Material2dOpticalModeSceneCommand, Material2dSceneCommand, SceneEntityId, SceneService,
    Text2dAlignSceneCommand, Text2dBlendModeSceneCommand, Text2dSceneCommand,
    Text2dStyleSceneCommand,
};
mod editor_capability;
mod render_extraction;
mod reset;
mod runtime_capabilities;
mod scene_command;
mod script_command;
#[cfg(test)]
mod tests;
pub use editor_capability::*;
pub use render_extraction::*;
pub use reset::*;
pub use runtime_capabilities::*;
pub use scene_command::*;
pub use script_command::*;

#[derive(Debug, Clone)]
pub struct Text2d {
    pub content: String,
    pub font: AssetKey,
    pub bounds: Vec2,
    pub transform: Transform2,
    pub style: Text2dStyle,
    pub post_fx_host_id: Option<PostFxHost2dId>,
}

#[derive(Debug, Clone, Copy)]
pub struct Text2dStyle {
    pub color: ColorRgba,
    pub opacity: f32,
    pub font_size: Option<f32>,
    pub align: Text2dAlign,
    pub blend: Text2dBlendMode,
    pub shadow: Option<Text2dShadow>,
    pub outline: Option<Text2dOutline>,
    pub glow: Option<Text2dGlow>,
}

impl Default for Text2dStyle {
    fn default() -> Self {
        Self {
            color: ColorRgba::new(1.0, 0.96, 0.82, 1.0),
            opacity: 1.0,
            font_size: None,
            align: Text2dAlign::Left,
            blend: Text2dBlendMode::Alpha,
            shadow: None,
            outline: None,
            glow: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Text2dAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum Text2dBlendMode {
    Alpha,
    Additive,
    Multiply,
    Screen,
}

#[derive(Debug, Clone, Copy)]
pub struct Text2dShadow {
    pub color: ColorRgba,
    pub offset: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct Text2dOutline {
    pub color: ColorRgba,
    pub width: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Text2dGlow {
    pub color: ColorRgba,
    pub radius: f32,
    pub intensity: f32,
    pub passes: u8,
}

#[derive(Debug, Clone)]
pub struct Text2dDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub render_layer: String,
    pub text: Text2d,
    pub z_index: f32,
    pub material: Option<Material2d>,
    pub render_contributions: RenderContributionSet,
}

#[derive(Debug, Default)]
pub struct Text2dSceneService {
    commands: Mutex<Vec<Text2dDrawCommand>>,
}

impl Text2dSceneService {
    pub fn queue(&self, command: Text2dDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("text2d scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("text2d scene service mutex should not be poisoned");
        commands.clear();
    }

    pub fn commands(&self) -> Vec<Text2dDrawCommand> {
        let commands = self
            .commands
            .lock()
            .expect("text2d scene service mutex should not be poisoned");
        commands.clone()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Text2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Text2dPlugin;

impl RuntimePlugin for Text2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-text-2d-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Text2dSceneService::default())?;
        amigo_scene::register_scene_reset_handler(registry, Text2dSceneResetHandler)?;
        if let Some(metadata) = registry.resolve::<amigo_scene::ComponentMetadataProviderRegistry>()
        {
            metadata.register(crate::scene::Text2dComponentMetadataProvider);
        }
        registry.register(Text2dDomainInfo {
            crate_name: "amigo-text-2d-plugin",
            capability: "text_2d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-text-2d-plugin",
            &["text_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            self::scene_command::Text2dSceneCommandHandler,
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            self::script_command::Text2dScriptCommandHandler,
        );
        Ok(())
    }
}

pub fn queue_text2d_scene_command(
    scene_service: &SceneService,
    text_scene_service: &Text2dSceneService,
    command: &Text2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    text_scene_service.queue(Text2dDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        render_layer: command.render_layer.clone(),
        text: Text2d {
            content: command.content.clone(),
            font: command.font.clone(),
            bounds: command.bounds,
            transform: command.transform,
            style: text2d_style_from_scene_command(command.style),
            post_fx_host_id: command.post_fx_host_id.clone(),
        },
        z_index: command.z_index,
        material: material_from_scene_command(command.material.as_ref()),
        render_contributions: text_render_contributions(command),
    });
    entity
}

fn text_render_contributions(command: &Text2dSceneCommand) -> RenderContributionSet {
    let mut render_contributions =
        RenderContributionSet::from_pairs(command.render_contributions.roles.clone());
    render_contributions.merge_defaults([
        (render_contribution_roles::WORLD_COLOR, true),
        (render_contribution_roles::MATERIAL_MASK, false),
        (render_contribution_roles::OPTICS_REFRACT, false),
        (render_contribution_roles::TRANSMISSION_SOURCE, false),
        (render_contribution_roles::BLOOM_SOURCE, false),
        (render_contribution_roles::CAMERA_FX_SOURCE, false),
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

fn text2d_style_from_scene_command(style: Text2dStyleSceneCommand) -> Text2dStyle {
    Text2dStyle {
        color: style.color,
        opacity: style.opacity,
        font_size: style.font_size,
        align: match style.align {
            Text2dAlignSceneCommand::Left => Text2dAlign::Left,
            Text2dAlignSceneCommand::Center => Text2dAlign::Center,
            Text2dAlignSceneCommand::Right => Text2dAlign::Right,
        },
        blend: match style.blend {
            Text2dBlendModeSceneCommand::Alpha => Text2dBlendMode::Alpha,
            Text2dBlendModeSceneCommand::Additive => Text2dBlendMode::Additive,
            Text2dBlendModeSceneCommand::Multiply => Text2dBlendMode::Multiply,
            Text2dBlendModeSceneCommand::Screen => Text2dBlendMode::Screen,
        },
        shadow: style.shadow.map(|shadow| Text2dShadow {
            color: shadow.color,
            offset: shadow.offset,
        }),
        outline: style.outline.map(|outline| Text2dOutline {
            color: outline.color,
            width: outline.width,
        }),
        glow: style.glow.map(|glow| Text2dGlow {
            color: glow.color,
            radius: glow.radius,
            intensity: glow.intensity,
            passes: glow.passes,
        }),
    }
}
