use crate::{Renderable2dPayloadKind, WgpuRenderable2dAdapter};

pub struct LayeredImage2dRenderableAdapter;

impl WgpuRenderable2dAdapter for LayeredImage2dRenderableAdapter {
    fn kind(&self) -> Renderable2dPayloadKind {
        Renderable2dPayloadKind::new("layered_image_2d")
    }
}
