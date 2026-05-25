use amigo_assets::AssetKey;
use amigo_math::Vec2;
use amigo_scene::{
    ComponentHydrationContext, ComponentHydrator, PluginComponentHydrationContext,
    PluginComponentHydrator, SceneComponentDocument, SceneDocumentResult, SceneVec2Document,
    TileMap2dSceneCommand,
};
use amigo_scene::SceneComponentDocument as ComponentDocument;

use super::Tilemap2dDocument;

pub struct TileMap2dComponentHydrator;
pub struct TileMap2dPluginComponentHydrator;

impl ComponentHydrator for TileMap2dComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.tilemap-2d"
    }

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool {
        matches!(component, ComponentDocument::TileMap2d { .. })
    }

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()> {
        let document = match ctx.component {
            ComponentDocument::TileMap2d { .. } => {
                let Some(document) = Tilemap2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                document
            }
            _ => return Ok(()),
        };

        push_tilemap_command(ctx.source_mod, ctx.entity_name, ctx.commands, &document)
    }
}

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
