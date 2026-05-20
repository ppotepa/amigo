use crate::{Renderable2dPayload, Renderable2dPayloadKind};

pub trait WgpuRenderable2dAdapter: Send + Sync {
    fn kind(&self) -> Renderable2dPayloadKind;

    fn supports(&self, payload: &Renderable2dPayload) -> bool {
        payload.kind_id() == self.kind()
    }
}

#[derive(Default)]
pub struct WgpuRenderable2dAdapterRegistry {
    adapters: Vec<Box<dyn WgpuRenderable2dAdapter>>,
}

impl WgpuRenderable2dAdapterRegistry {
    pub fn register<A>(&mut self, adapter: A)
    where
        A: WgpuRenderable2dAdapter + 'static,
    {
        self.adapters.push(Box::new(adapter));
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}
