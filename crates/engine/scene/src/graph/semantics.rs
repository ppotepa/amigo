use crate::metadata_traits::MetadataTraitKind;

/// Semantic capabilities attached to a scene graph node.
///
/// This is intentionally independent from YAML shape. YAML is only one source
/// format; editor, hydration and diagnostics should reason about capabilities
/// instead of raw document keys.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SceneGraphSemantics {
    pub traits: Vec<MetadataTraitKind>,
    pub role: Option<SceneGraphSemanticRole>,
    pub post_fx_host: Option<SceneGraphPostFxHost>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneGraphSemanticRole {
    Scene2D,
    SceneSettings2D,
    DrawLayer2D,
    SceneObject2D,
    Component2D,
    Renderable2D,
    ImagePart2D,
    LightGroup2D,
    LightRoute2D,
    AssetProxy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneGraphPostFxHost {
    pub host_id: String,
    pub scope_label: String,
}

impl SceneGraphSemantics {
    pub fn new(role: SceneGraphSemanticRole) -> Self {
        Self {
            role: Some(role),
            ..Self::default()
        }
    }

    pub fn with_traits(mut self, traits: impl IntoIterator<Item = MetadataTraitKind>) -> Self {
        for trait_kind in traits {
            self.push_trait(trait_kind);
        }
        self
    }

    pub fn with_post_fx_host(
        mut self,
        host_id: impl Into<String>,
        scope_label: impl Into<String>,
    ) -> Self {
        self.post_fx_host = Some(SceneGraphPostFxHost {
            host_id: host_id.into(),
            scope_label: scope_label.into(),
        });
        self
    }

    pub fn push_trait(&mut self, trait_kind: MetadataTraitKind) {
        if !self.traits.contains(&trait_kind) {
            self.traits.push(trait_kind);
        }
    }

    pub fn has_trait(&self, trait_kind: MetadataTraitKind) -> bool {
        self.traits.contains(&trait_kind)
    }
}
