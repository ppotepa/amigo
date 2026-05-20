use crate::{Renderable2dPayloadKind, WgpuRenderable2dAdapter};

pub struct Vector2dRenderableAdapter;

impl WgpuRenderable2dAdapter for Vector2dRenderableAdapter {
    fn kind(&self) -> Renderable2dPayloadKind {
        Renderable2dPayloadKind::new("vector_2d")
    }
}
