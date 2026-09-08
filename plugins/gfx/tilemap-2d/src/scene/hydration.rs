use amigo_assets::AssetKey;
use amigo_math::Vec2;
use amigo_scene::{
    PluginComponentHydrationContext, PluginComponentHydrator, SceneDocumentResult,
    SceneVec2Document, TileMap2dSceneCommand,
};

use super::Tilemap2dDocument;

#[derive(Default)]
pub struct TileMap2dPluginComponentHydrator;

impl PluginComponentHydrator for TileMap2dPluginComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.tilemap-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.gfx.tilemap-2d.TileMap2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx.payload.as_any().downcast_ref::<Tilemap2dDocument>() else {
            return Err(amigo_scene::SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "TileMap2D plugin hydrator received wrong payload".to_owned(),
            });
        };

        push_tilemap_command(ctx.source_mod, ctx.entity_name, ctx.commands, document)
    }
}

fn vec2_from_document(value: SceneVec2Document) -> Vec2 {
    Vec2::new(value.x, value.y)
}

fn push_tilemap_command(
    source_mod: &str,
    entity_name: &str,
    commands: &mut Vec<amigo_scene::SceneCommand>,
    document: &Tilemap2dDocument,
) -> SceneDocumentResult<()> {
    commands.push(amigo_scene::SceneCommand::Plugin {
        command: amigo_scene::tilemap_2d_plugin_scene_command(TileMap2dSceneCommand {
            source_mod: source_mod.to_owned(),
            entity_name: entity_name.to_owned(),
            render_layer: document.render_layer.clone(),
            tileset: AssetKey::new(document.tileset.clone()),
            ruleset: document.ruleset.clone().map(AssetKey::new),
            tile_size: vec2_from_document(document.tile_size),
            grid: document.grid.clone(),
            depth_fill_rows: document.depth_fill_rows,
            z_index: document.z_index,
        }),
    });

    Ok(())
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
    fn tilemap_hydrator_carries_tileset_ruleset_and_grid() {
        let hydrator = TileMap2dPluginComponentHydrator;
        let payload = Tilemap2dDocument {
            entity_name: String::new(),
            render_layer: "world.tiles".to_owned(),
            tileset: "playground/tiles/base".to_owned(),
            ruleset: Some("playground/rules/platform".to_owned()),
            tile_size: SceneVec2Document { x: 16.0, y: 24.0 },
            editor: None,
            grid: vec![".P..".to_owned(), "####".to_owned()],
            depth_fill_rows: 2,
            z_index: -3.0,
        };
        let entity = test_entity("tilemap");
        let document = test_document(entity.clone());
        let mut commands = Vec::new();

        hydrator
            .hydrate_plugin_payload(PluginComponentHydrationContext {
                source_mod: "playground",
                document: &document,
                entity: &entity,
                entity_name: "tilemap",
                component_index: 0,
                component_type: "amigo.gfx.tilemap-2d.TileMap2D",
                payload: &payload,
                commands: &mut commands,
            })
            .expect("tilemap hydrator should accept plugin payload");

        let command = plugin_payload::<TileMap2dSceneCommand>(&commands);
        assert_eq!(command.source_mod, "playground");
        assert_eq!(command.entity_name, "tilemap");
        assert_eq!(command.render_layer, "world.tiles");
        assert_eq!(command.tileset, AssetKey::new("playground/tiles/base"));
        assert_eq!(
            command.ruleset,
            Some(AssetKey::new("playground/rules/platform"))
        );
        assert_eq!(command.tile_size, Vec2::new(16.0, 24.0));
        assert_eq!(command.grid, vec![".P..".to_owned(), "####".to_owned()]);
        assert_eq!(command.depth_fill_rows, 2);
        assert_eq!(command.z_index, -3.0);
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
