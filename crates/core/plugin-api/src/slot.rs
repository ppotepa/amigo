use crate::capability::CapabilityRef;
use crate::ids::{PluginId, SlotId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotRequirement {
    Required,
    Optional,
    OptionalWithNoop,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotContract {
    pub id: SlotId,
    pub version: u32,
    pub requirement: SlotRequirement,
    pub required_capabilities: Vec<CapabilityRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotBinding {
    pub slot: SlotId,
    pub provider: PluginId,
}
