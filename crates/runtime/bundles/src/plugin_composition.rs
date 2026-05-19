use amigo_plugin_api::{CapabilityId, PluginId, SlotId};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginComposition {
    pub plugins: Vec<PluginId>,
    pub required_capabilities: Vec<CapabilityId>,
    pub required_slots: Vec<SlotId>,
}

impl RuntimePluginComposition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_plugin(mut self, id: impl Into<String>) -> Self {
        self.plugins.push(PluginId(id.into()));
        self
    }

    pub fn require_capability(mut self, id: impl Into<String>) -> Self {
        self.required_capabilities.push(CapabilityId(id.into()));
        self
    }

    pub fn require_slot(mut self, id: impl Into<String>) -> Self {
        self.required_slots.push(SlotId(id.into()));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
            && self.required_capabilities.is_empty()
            && self.required_slots.is_empty()
    }
}

pub fn default_camera_2d_plugin_composition() -> RuntimePluginComposition {
    RuntimePluginComposition::new()
        .with_plugin("amigo.camera.camera-core")
        .with_plugin("amigo.camera.camera-optics")
        .with_plugin("amigo.camera.focus-depth")
        .with_plugin("amigo.camera.shutter-motion")
        .require_capability("camera.frame_context.2d")
        .require_capability("camera.optics.2d")
        .require_capability("camera.focus_depth.2d")
        .require_slot("camera.frame_provider.2d")
        .require_slot("camera.optics.consumer.2d")
}

