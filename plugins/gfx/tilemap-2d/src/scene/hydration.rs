use amigo_assets::AssetKey;
use amigo_math::Vec2;
use amigo_scene::{
    ComponentHydrationContext, ComponentHydrator, SceneComponentDocument, SceneDocumentResult,
    SceneVec2Document, TileMap2dSceneCommand,
};

use super::{parse_tilemap_2d_plugin_payload, Tilemap2dDocument};

pub struct TileMap2dComponentHydrator;

impl ComponentHydrator for TileMap2dComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.tilemap-2d"
    }

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool {
        matches!(component, SceneComponentDocument::TileMap2d { .. })
            || matches!(
                component,
                SceneComponentDocument::Plugin { component_type, .. }
                    if component_type == "amigo.gfx.tilemap-2d.TileMap2D"
                        || component_type == "TileMap2D"
            )
    }

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()> {
        let document = match ctx.component {
            SceneComponentDocument::TileMap2d { .. } => {
                let Some(document) = Tilemap2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                document
            }
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.gfx.tilemap-2d.TileMap2D"
                || component_type == "TileMap2D" =>
            {
                parse_tilemap_2d_plugin_payload(payload)?
            }
            _ => return Ok(()),
        };

        ctx.commands.push(amigo_scene::SceneCommand::QueueTileMap2d {
            command: TileMap2dSceneCommand {
                source_mod: ctx.source_mod.to_owned(),
                entity_name: ctx.entity_name.to_owned(),
                render_layer: document.render_layer,
                tileset: AssetKey::new(document.tileset),
                ruleset: document.ruleset.map(AssetKey::new),
                tile_size: vec2_from_document(document.tile_size),
                grid: document.grid,
                depth_fill_rows: document.depth_fill_rows,
                z_index: document.z_index,
            },
        });

        Ok(())
    }
}

fn vec2_from_document(value: SceneVec2Document) -> Vec2 {
    Vec2::new(value.x, value.y)
}
