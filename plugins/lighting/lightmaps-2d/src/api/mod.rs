use amigo_plugin_api::{light_map, TargetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lightmap2dChannel {
    pub id: String,
    pub layers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lightmap2dSource {
    pub id: String,
    pub channels: Vec<Lightmap2dChannel>,
}

pub fn lightmap_target_id() -> TargetId {
    light_map()
}
