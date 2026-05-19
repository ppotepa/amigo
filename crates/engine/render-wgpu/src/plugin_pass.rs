use amigo_plugin_api::{PluginId, TargetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WgpuPluginPassDescriptor {
    pub id: String,
    pub owner: PluginId,
    pub reads: Vec<TargetId>,
    pub writes: Vec<TargetId>,
}

impl WgpuPluginPassDescriptor {
    pub fn new(id: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            owner: PluginId(owner.into()),
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }

    pub fn reads(mut self, target: TargetId) -> Self {
        self.reads.push(target);
        self
    }

    pub fn writes(mut self, target: TargetId) -> Self {
        self.writes.push(target);
        self
    }

    pub fn is_valid(&self) -> bool {
        !self.id.trim().is_empty()
            && !self.owner.0.trim().is_empty()
            && !self.writes.is_empty()
    }
}

