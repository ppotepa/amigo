use crate::ids::CapabilityId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityRef {
    pub id: CapabilityId,
    pub version: u32,
}

impl CapabilityRef {
    pub fn new(id: impl Into<String>, version: u32) -> Self {
        Self {
            id: CapabilityId(id.into()),
            version,
        }
    }
}
