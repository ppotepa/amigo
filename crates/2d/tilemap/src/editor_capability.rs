use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.2d.tilemap.editor";
const COMPONENT_TYPE: &str = "amigo.2d.tilemap";

#[derive(Debug, Clone, Copy)]
pub struct TileMap2dEditorCapability;

impl EditorCapability for TileMap2dEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema::placeholder(self.component_type(), "TileMap2D")
            .with_field(PropertyDescriptor::asset("tileset", "Tileset", "tileset"))
            .with_field(PropertyDescriptor::asset("map", "Map", "tilemap"))
            .with_field(PropertyDescriptor::vec2("tile_size", "Tile Size"))
            .with_field(PropertyDescriptor::text("render_layer", "Render Layer"))
            .with_field(PropertyDescriptor::number("z_index", "Z Index"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TileMap2dEditorCapabilityProvider;

impl EditorCapabilityProvider for TileMap2dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.2d.tilemap.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(TileMap2dEditorCapability);
        Ok(())
    }
}

pub fn register_tile_map2d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    TileMap2dEditorCapabilityProvider.register(registry)
}