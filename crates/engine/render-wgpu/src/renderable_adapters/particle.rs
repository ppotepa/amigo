use crate::{Renderable2dPayloadKind, WgpuRenderable2dAdapter};

pub struct Particle2dRenderableAdapter;

impl WgpuRenderable2dAdapter for Particle2dRenderableAdapter {
    fn kind(&self) -> Renderable2dPayloadKind {
        Renderable2dPayloadKind::new("particle_2d")
    }
}
