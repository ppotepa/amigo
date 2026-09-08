//! Neutral NPR commands shared by extractors and render backends.

use amigo_render_npr::{NprDebugView, NprRenderPacket};

#[derive(Debug, Clone, PartialEq)]
pub struct NprDrawCommand {
    pub packet: NprRenderPacket,
    pub preset: &'static str,
}

impl NprDrawCommand {
    pub fn new(packet: NprRenderPacket) -> Self {
        Self::with_preset(packet, "comic-ink")
    }

    pub fn with_preset(packet: NprRenderPacket, preset: &'static str) -> Self {
        Self { packet, preset }
    }
    pub fn debug_view(&self) -> NprDebugView {
        self.packet.debug_view
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprBackgroundCommand {
    pub color: [f32; 4],
    pub grain: f32,
    pub tooth: f32,
    pub seed: u64,
}

pub trait NprRenderOutput {
    fn push_npr_draw_command(&mut self, command: NprDrawCommand);
    fn set_npr_background(&mut self, background: NprBackgroundCommand);
}
