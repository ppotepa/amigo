use crate::ids::CapabilityId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

    pub fn is_empty(&self) -> bool {
        self.id.0.trim().is_empty() || self.version == 0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapabilityManifest {
    pub provides: Vec<CapabilityRef>,
    pub requires: Vec<CapabilityRef>,
}
