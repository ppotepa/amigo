use serde::{Deserialize, Serialize};
use serde_yaml::Mapping;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneComponentEnvelope {
    #[serde(rename = "type")]
    pub component_type: String,
    #[serde(flatten)]
    pub payload: Mapping,
}
