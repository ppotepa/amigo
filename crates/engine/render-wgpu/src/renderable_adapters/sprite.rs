use crate::{Renderable2dPayloadKind, WgpuRenderable2dAdapter};

pub struct Sprite2dRenderableAdapter;

impl WgpuRenderable2dAdapter for Sprite2dRenderableAdapter {
    fn kind(&self) -> Renderable2dPayloadKind {
        Renderable2dPayloadKind::new("sprite_2d")
    }
}
