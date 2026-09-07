//! Neutral NPR commands shared by extractors and render backends.

use amigo_render_npr::{NprDebugView, NprRenderPacket};

#[derive(Debug, Clone, PartialEq)]
pub struct NprDrawCommand {
    pub packet: NprRenderPacket,
    pub preset: &'static str,
}

impl NprDrawCommand {
    pub fn new(packet: NprRenderPacket) -> Self {
        Self {
            packet,
            preset: "comic-ink",
        }
    }
    pub fn debug_view(&self) -> NprDebugView {
        self.packet.debug_view
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprBackgroundCommand {
    pub color: [f32; 4],
}

pub trait NprRenderOutput {
    fn push_npr_draw_command(&mut self, command: NprDrawCommand);
    fn set_npr_background(&mut self, background: NprBackgroundCommand);
}
