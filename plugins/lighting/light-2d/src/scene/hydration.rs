use amigo_math::ColorRgba;
use amigo_scene::SceneComponentDocument as ComponentDocument;
use amigo_scene::{
    ComponentHydrationContext, ComponentHydrator, GlobalLight2dSceneCommand,
    PluginComponentHydrationContext, PluginComponentHydrator, SceneComponentDocument,
    SceneDocumentError, SceneDocumentResult,
};

use super::GlobalLight2dDocument;

pub struct GlobalLight2dComponentHydrator;
pub struct GlobalLight2dPluginComponentHydrator;

impl ComponentHydrator for GlobalLight2dComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.lighting.light-2d"
    }

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool {
        matches!(component, ComponentDocument::GlobalLight2d { .. })
    }

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()> {
        let document = match ctx.component {
            ComponentDocument::GlobalLight2d { .. } => {
                let Some(document) = GlobalLight2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                document
            }
            _ => return Ok(()),
        };

        push_global_light_command(
            ctx.source_mod,
            &ctx.document.scene.id,
            &ctx.entity.id,
            ctx.entity_name,
            ctx.commands,
            &document,
        )?;

        Ok(())
    }
}

impl PluginComponentHydrator for GlobalLight2dPluginComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.lighting.light-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.lighting.light-2d.GlobalLight2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx.payload.as_any().downcast_ref::<GlobalLight2dDocument>() else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "GlobalLight2D plugin hydrator received wrong payload".to_owned(),
            });
        };
        push_global_light_command(
            ctx.source_mod,
            &ctx.document.scene.id,
            &ctx.entity.id,
            ctx.entity_name,
            ctx.commands,
            document,
        )
    }
}

fn push_global_light_command(
    source_mod: &str,
    scene_id: &str,
    entity_id: &str,
    entity_name: &str,
    commands: &mut Vec<amigo_scene::SceneCommand>,
    document: &GlobalLight2dDocument,
) -> SceneDocumentResult<()> {
    commands.push(amigo_scene::SceneCommand::Plugin {
        command: amigo_scene::global_light_2d_plugin_scene_command(GlobalLight2dSceneCommand {
            source_mod: source_mod.to_owned(),
            entity_name: entity_name.to_owned(),
            id: document.id.clone(),
            color: parse_color_rgba_hex(&document.color, scene_id, entity_id, "GlobalLight2D")?,
            intensity: document.intensity.max(0.0),
        }),
    });
    Ok(())
}

fn parse_color_rgba_hex(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ColorRgba> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);
    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            255,
        ),
        8 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[6..8], value, scene_id, entity_id, component_kind)?,
        ),
        _ => {
            return Err(SceneDocumentError::Hydration {
                scene_id: scene_id.to_owned(),
                entity_id: entity_id.to_owned(),
                component_kind: component_kind.to_owned(),
                message: format!("expected color `{value}` to use #RRGGBB or #RRGGBBAA syntax"),
            });
        }
    };

    Ok(ColorRgba::new(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    ))
}

fn parse_hex_channel(
    channel: &str,
    original: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<u8> {
    u8::from_str_radix(channel, 16).map_err(|source| SceneDocumentError::Hydration {
        scene_id: scene_id.to_owned(),
        entity_id: entity_id.to_owned(),
        component_kind: component_kind.to_owned(),
        message: format!("invalid color `{original}`: {source}"),
    })
}
