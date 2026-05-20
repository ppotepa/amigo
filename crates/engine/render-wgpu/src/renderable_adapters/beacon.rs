use crate::{Renderable2dPayloadKind, WgpuRenderable2dAdapter};

pub struct Beacon2dRenderableAdapter;

impl WgpuRenderable2dAdapter for Beacon2dRenderableAdapter {
    fn kind(&self) -> Renderable2dPayloadKind {
        Renderable2dPayloadKind::new("beacon_light_2d")
    }
}
