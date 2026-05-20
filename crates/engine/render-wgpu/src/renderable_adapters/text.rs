use crate::{Renderable2dPayloadKind, WgpuRenderable2dAdapter};

pub struct Text2dRenderableAdapter;

impl WgpuRenderable2dAdapter for Text2dRenderableAdapter {
    fn kind(&self) -> Renderable2dPayloadKind {
        Renderable2dPayloadKind::new("text_2d")
    }
}
