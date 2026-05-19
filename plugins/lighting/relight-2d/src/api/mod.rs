use amigo_plugin_api::{scene_color, scene_lighting, scene_normal, TargetId};

#[derive(Clone, Debug, PartialEq)]
pub struct Relight2dContribution {
    pub source_id: String,
    pub intensity: f32,
    pub color_rgba: [f32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Relight2dCandidate {
    pub contributions: Vec<Relight2dContribution>,
    pub reads: Vec<TargetId>,
    pub writes: Vec<TargetId>,
}

impl Relight2dCandidate {
    pub fn from_contributions(contributions: Vec<Relight2dContribution>) -> Self {
        Self {
            contributions,
            reads: vec![scene_color(), scene_normal()],
            writes: vec![scene_lighting()],
        }
    }
}
