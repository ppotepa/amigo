//! Neutral NPR commands shared by extractors and render backends.

use amigo_render_npr::{NprDebugView, NprRenderPacket, NprStyleLayers};

#[derive(Debug, Clone, PartialEq)]
pub struct NprDrawCommand {
    pub packet: NprRenderPacket,
    pub preset: &'static str,
    /// Authorial layer semantics travel with the neutral packet; WGPU is only
    /// responsible for executing this declared visibility and opacity.
    pub layers: NprStyleLayers,
    /// Declared material base colour for layers using `ModelBaseColor`. `None`
    /// means the extractor has no material contribution; a backend must reject
    /// such a layer instead of guessing from a palette or object name.
    pub material_base_color: Option<[f32; 4]>,
}

impl NprDrawCommand {
    pub fn new(packet: NprRenderPacket) -> Self {
        Self::with_preset(packet, "comic-ink")
    }

    pub fn with_preset(packet: NprRenderPacket, preset: &'static str) -> Self {
        Self::with_preset_and_layers(packet, preset, NprStyleLayers::default())
    }

    pub fn with_preset_and_layers(
        packet: NprRenderPacket,
        preset: &'static str,
        layers: NprStyleLayers,
    ) -> Self {
        Self {
            packet,
            preset,
            layers,
            material_base_color: None,
        }
    }

    pub fn with_material_base_color(mut self, color: [f32; 4]) -> Self {
        self.material_base_color = Some(color);
        self
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
