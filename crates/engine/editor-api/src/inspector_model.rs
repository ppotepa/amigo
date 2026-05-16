#[derive(Debug, Clone, PartialEq)]
pub struct AuthoringPropertyPanel {
    pub title: String,
    pub groups: Vec<AuthoringPropertyGroup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthoringPropertyGroup {
    pub id: String,
    pub title: String,
    pub properties: Vec<AuthoringProperty>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthoringProperty {
    pub id: String,
    pub label: String,
    pub value: AuthoringPropertyValue,
    pub editor: AuthoringPropertyEditor,
    pub hints: AuthoringPropertyHints,
    pub read_only: bool,
    pub source_file: String,
    pub yaml_pointer: String,
    pub group: String,
    pub trait_kind: Option<String>,
    pub binding: Option<AuthoringRuntimeBinding>,
    pub display: AuthoringPropertyDisplay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthoringPropertyVisibility {
    Primary,
    Advanced,
    Debug,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthoringPropertyApplyMode {
    Live,
    Mock,
    ReadOnly,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringPropertyDisplay {
    pub icon: Option<String>,
    pub tags: Vec<String>,
    pub visibility: AuthoringPropertyVisibility,
    pub apply_mode: AuthoringPropertyApplyMode,
    pub order: i32,
}

impl Default for AuthoringPropertyDisplay {
    fn default() -> Self {
        Self {
            icon: None,
            tags: Vec::new(),
            visibility: AuthoringPropertyVisibility::Primary,
            apply_mode: AuthoringPropertyApplyMode::Unsupported,
            order: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthoringPropertyValue {
    Text(String),
    Number(f32),
    Bool(bool),
    AssetRef(String),
    Enum(String),
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Color(String),
    Empty,
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthoringPropertyEditor {
    ReadOnly,
    Text,
    Number,
    Slider { min: f32, max: f32, step: f32 },
    Toggle,
    AssetPicker { domain: String },
    Enum { options: Vec<String> },
    Color,
    Vec2,
    Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthoringPropertyHints {
    pub number: Option<AuthoringNumberConstraints>,
    pub options: Vec<AuthoringOption>,
}

impl Default for AuthoringPropertyHints {
    fn default() -> Self {
        Self {
            number: None,
            options: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthoringNumberConstraints {
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub clamp: bool,
    pub unit: Option<String>,
    pub display_scale: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringOption {
    pub id: String,
    pub label: String,
}

// Editor terminology:
// - RenderLayer* bindings target Draw Layer runtime state.
// - LayeredImageLayer* bindings target Image Part runtime state.
// Keep variant names stable until the runtime binding API is migrated in one pass.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthoringRuntimeBinding {
    RenderLayerOpacity {
        layer_id: String,
    },
    RenderLayerVisible {
        layer_id: String,
    },
    RenderLayerOrder {
        layer_id: String,
    },
    LayeredImageBaseOpacity {
        entity_name: String,
    },
    LayeredImageLayerOpacity {
        entity_name: String,
        layer_id: String,
    },
    LayeredImageLayerEnabled {
        entity_name: String,
        layer_id: String,
    },
    ParticleEmitterProperty {
        entity_name: String,
        field: String,
    },
    PostFxFrameEnabled {
        index: usize,
    },
    PostFxFrameField {
        index: usize,
        field: String,
    },
    PostFxMock {
        effect_id: String,
        field: String,
    },
    Mock {
        namespace: String,
        subject: String,
        field: String,
    },
}
