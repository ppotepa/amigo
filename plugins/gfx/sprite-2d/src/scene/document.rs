use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use amigo_scene::{
    Material2dDocument, RenderContributionsDocument, SceneComponentDocument,
    SceneComponentSchemaProvider, SceneDocumentError, SceneDocumentResult,
    SceneSpriteAnimationDocument, SceneSpriteSheetDocument, SceneVec2Document,
    VisualMaps2dDocument,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Sprite2dDocument {
    #[serde(default)]
    pub entity_name: String,
    #[serde(default = "default_render_layer")]
    pub render_layer: String,
    pub texture: String,
    pub size: SceneVec2Document,
    #[serde(default)]
    pub sheet: Option<SceneSpriteSheetDocument>,
    #[serde(default)]
    pub animation: Option<SceneSpriteAnimationDocument>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_maps: Option<VisualMaps2dDocument>,
    #[serde(default)]
    pub render_contributions: RenderContributionsDocument,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<Material2dDocument>,
    #[serde(default)]
    pub z_index: f32,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default = "default_visible")]
    pub visible: bool,
}

impl Sprite2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            SceneComponentDocument::Sprite2d {
                render_layer,
                texture,
                size,
                sheet,
                animation,
                visual_maps,
                render_contributions,
                material,
                z_index,
                post_fx: _,
            } => Some(Self {
                entity_name: String::new(),
                render_layer: render_layer.clone(),
                texture: texture.clone(),
                size: *size,
                sheet: *sheet,
                animation: *animation,
                visual_maps: visual_maps.clone(),
                render_contributions: render_contributions.clone(),
                material: material.clone(),
                z_index: *z_index,
                opacity: 1.0,
                visible: true,
            }),
            _ => None,
        }
    }
}

pub fn parse_sprite_2d_plugin_payload(payload: &Value) -> SceneDocumentResult<Sprite2dDocument> {
    serde_yaml::from_value::<Sprite2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy)]
pub struct Sprite2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for Sprite2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.gfx.sprite-2d.Sprite2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["Sprite2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<Sprite2dDocument>(Value::Mapping(payload))?)
    }
}

fn default_render_layer() -> String {
    "world".to_owned()
}

fn default_opacity() -> f32 {
    1.0
}

fn default_visible() -> bool {
    true
}
