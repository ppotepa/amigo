#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub id: String,
    pub label: String,
    pub editor: PropertyEditorKind,
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub enum PropertyEditorKind {
    Text,
    Number,
    Bool,
    Vec2,
    Vec3,
    Color,
    AssetPicker { asset_kind: String },
    Enum { options: Vec<String> },
}

