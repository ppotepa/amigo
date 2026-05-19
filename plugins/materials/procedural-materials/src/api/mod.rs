#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProceduralMaterialTarget2d {
    SceneHighlight,
    SceneEmissive,
    RelightMask,
    RefractiveMask,
}

impl ProceduralMaterialTarget2d {
    pub fn target_id(self) -> &'static str {
        match self {
            ProceduralMaterialTarget2d::SceneHighlight => "SceneHighlight",
            ProceduralMaterialTarget2d::SceneEmissive => "SceneEmissive",
            ProceduralMaterialTarget2d::RelightMask => "RelightMask",
            ProceduralMaterialTarget2d::RefractiveMask => "RefractiveMask",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProceduralMaterial2d {
    pub id: String,
    pub generator: String,
    pub target: ProceduralMaterialTarget2d,
    pub seed: u64,
}

impl ProceduralMaterial2d {
    pub fn new(
        id: impl Into<String>,
        generator: impl Into<String>,
        target: ProceduralMaterialTarget2d,
        seed: u64,
    ) -> Self {
        Self {
            id: id.into(),
            generator: generator.into(),
            target,
            seed,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.id.trim().is_empty() && !self.generator.trim().is_empty()
    }
}
