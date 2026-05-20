use crate::{Renderable2dPayloadKind, WgpuRenderable2dAdapter};

pub struct TileMap2dRenderableAdapter;

impl WgpuRenderable2dAdapter for TileMap2dRenderableAdapter {
    fn kind(&self) -> Renderable2dPayloadKind {
        Renderable2dPayloadKind::new("tilemap_2d")
    }
}
