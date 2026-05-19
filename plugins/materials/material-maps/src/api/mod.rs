#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialMapKind2d {
    SceneHighlight,
    SceneEmissive,
    RelightMask,
    RefractiveMask,
}

impl MaterialMapKind2d {
    pub fn target_id(self) -> &'static str {
        match self {
            MaterialMapKind2d::SceneHighlight => "SceneHighlight",
            MaterialMapKind2d::SceneEmissive => "SceneEmissive",
            MaterialMapKind2d::RelightMask => "RelightMask",
            MaterialMapKind2d::RefractiveMask => "RefractiveMask",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialMapRef2d {
    pub material_id: String,
    pub kind: MaterialMapKind2d,
    pub asset: String,
}

impl MaterialMapRef2d {
    pub fn new(
        material_id: impl Into<String>,
        kind: MaterialMapKind2d,
        asset: impl Into<String>,
    ) -> Self {
        Self {
            material_id: material_id.into(),
            kind,
            asset: asset.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.material_id.trim().is_empty() && !self.asset.trim().is_empty()
    }
}
