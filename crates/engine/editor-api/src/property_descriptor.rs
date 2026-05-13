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

impl PropertyDescriptor {
    pub fn text(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor: PropertyEditorKind::Text,
            read_only: false,
        }
    }

    pub fn number(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor: PropertyEditorKind::Number,
            read_only: false,
        }
    }

    pub fn asset(
        id: impl Into<String>,
        label: impl Into<String>,
        asset_kind: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor: PropertyEditorKind::AssetPicker {
                asset_kind: asset_kind.into(),
            },
            read_only: false,
        }
    }

    pub fn bool(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor: PropertyEditorKind::Bool,
            read_only: false,
        }
    }

    pub fn vec2(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor: PropertyEditorKind::Vec2,
            read_only: false,
        }
    }

    pub fn vec3(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor: PropertyEditorKind::Vec3,
            read_only: false,
        }
    }

    pub fn color(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor: PropertyEditorKind::Color,
            read_only: false,
        }
    }

    pub fn read_only_text(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor: PropertyEditorKind::Text,
            read_only: true,
        }
    }
}

