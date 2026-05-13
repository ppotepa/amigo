use std::sync::{Arc, Mutex};

use crate::{EditorCapability, GizmoProvider, ValidationProvider};

pub struct EditorCapabilityRegistry {
    capabilities: Mutex<Vec<Arc<dyn EditorCapability>>>,
    validation_providers: Mutex<Vec<Arc<dyn ValidationProvider>>>,
    gizmo_providers: Mutex<Vec<Arc<dyn GizmoProvider>>>,
}

impl Default for EditorCapabilityRegistry {
    fn default() -> Self {
        Self {
            capabilities: Mutex::new(Vec::new()),
            validation_providers: Mutex::new(Vec::new()),
            gizmo_providers: Mutex::new(Vec::new()),
        }
    }
}

impl EditorCapabilityRegistry {
    pub fn register_capability<C>(&self, capability: C)
    where
        C: EditorCapability + 'static,
    {
        self.capabilities
            .lock()
            .expect("editor capability registry mutex should not be poisoned")
            .push(Arc::new(capability));
    }

    pub fn register_validation_provider<P>(&self, provider: P)
    where
        P: ValidationProvider + 'static,
    {
        self.validation_providers
            .lock()
            .expect("editor validation registry mutex should not be poisoned")
            .push(Arc::new(provider));
    }

    pub fn register_gizmo_provider<P>(&self, provider: P)
    where
        P: GizmoProvider + 'static,
    {
        self.gizmo_providers
            .lock()
            .expect("editor gizmo registry mutex should not be poisoned")
            .push(Arc::new(provider));
    }

    pub fn capabilities(&self) -> Vec<Arc<dyn EditorCapability>> {
        self.capabilities
            .lock()
            .expect("editor capability registry mutex should not be poisoned")
            .clone()
    }

    pub fn validation_providers(&self) -> Vec<Arc<dyn ValidationProvider>> {
        self.validation_providers
            .lock()
            .expect("editor validation registry mutex should not be poisoned")
            .clone()
    }

    pub fn gizmo_providers(&self) -> Vec<Arc<dyn GizmoProvider>> {
        self.gizmo_providers
            .lock()
            .expect("editor gizmo registry mutex should not be poisoned")
            .clone()
    }
}

