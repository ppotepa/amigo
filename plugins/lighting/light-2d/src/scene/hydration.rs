use amigo_math::ColorRgba;
use amigo_scene::{
    GlobalLight2dSceneCommand, PluginComponentHydrationContext, PluginComponentHydrator,
    SceneDocumentError, SceneDocumentResult,
};

use super::GlobalLight2dDocument;

#[derive(Default)]
pub struct GlobalLight2dPluginComponentHydrator;

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

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_scene::{
        PluginComponentHydrationContext, SceneCommand, SceneDocument, SceneEntityDocument,
        SceneMetadataDocument, SceneVisual2dDocument,
    };
    use std::collections::BTreeMap;

    #[test]
    fn global_light_hydrator_carries_color_and_clamped_intensity() {
        let hydrator = GlobalLight2dPluginComponentHydrator;
        let payload = GlobalLight2dDocument {
            id: "ambient".to_owned(),
            color: "#204080FF".to_owned(),
            intensity: -1.0,
        };
        let entity = test_entity("light");
        let document = test_document(entity.clone());
        let mut commands = Vec::new();

        hydrator
            .hydrate_plugin_payload(PluginComponentHydrationContext {
                source_mod: "test-mod",
                document: &document,
                entity: &entity,
                entity_name: "light",
                component_index: 0,
                component_type: "amigo.lighting.light-2d.GlobalLight2D",
                payload: &payload,
                commands: &mut commands,
            })
            .expect("global light hydrator should accept plugin payload");

        let command = plugin_payload::<GlobalLight2dSceneCommand>(&commands);
        assert_eq!(command.source_mod, "test-mod");
        assert_eq!(command.entity_name, "light");
        assert_eq!(command.id, "ambient");
        assert_eq!(
            command.color,
            ColorRgba::new(32.0 / 255.0, 64.0 / 255.0, 128.0 / 255.0, 1.0)
        );
        assert_eq!(command.intensity, 0.0);
    }

    fn plugin_payload<T: 'static>(commands: &[SceneCommand]) -> &T {
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            SceneCommand::Plugin { command } => command
                .payload_as::<T>()
                .expect("plugin scene payload should downcast"),
            other => panic!("expected plugin scene command, got {other:?}"),
        }
    }

    fn test_entity(name: &str) -> SceneEntityDocument {
        SceneEntityDocument {
            id: name.to_owned(),
            name: name.to_owned(),
            tags: Vec::new(),
            groups: Vec::new(),
            visible: true,
            simulation_enabled: true,
            collision_enabled: true,
            properties: BTreeMap::new(),
            transform2: None,
            transform3: None,
            post_fx: Vec::new(),
            prefab: None,
            prefab_overrides: Vec::new(),
            components: Vec::new(),
        }
    }

    fn test_document(entity: SceneEntityDocument) -> SceneDocument {
        SceneDocument {
            panels: Vec::new(),
            version: 1,
            scene: SceneMetadataDocument {
                id: "test-scene".to_owned(),
                label: String::new(),
                description: None,
            },
            transitions: Vec::new(),
            collision_events: Vec::new(),
            audio_cues: Vec::new(),
            activation_sets: Vec::new(),
            visual2d: SceneVisual2dDocument::default(),
            state: BTreeMap::new(),
            entities: vec![entity],
        }
    }
}
