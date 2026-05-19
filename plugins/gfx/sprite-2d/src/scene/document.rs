#[derive(Clone, Debug, PartialEq)]
pub struct Sprite2dDocument {
    pub entity_name: String,
    pub render_layer: String,
    pub texture: String,
    pub opacity: f32,
    pub visible: bool,
}
