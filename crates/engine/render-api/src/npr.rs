//! Neutral NPR commands shared by extractors and render backends.

use amigo_render_npr::{NprDebugView, NprRenderPacket, NprStyleLayers};

use crate::MeshDrawCommand;

/// A camera-independent NPR mesh contribution. Geometry remains in model space
/// so an animated runtime can submit current camera and instance transforms
/// every frame without rebuilding a screen-space drawing packet.
#[derive(Debug, Clone)]
pub struct NprMeshDrawCommand {
    pub mesh: MeshDrawCommand,
    pub style: NprMeshStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprMeshStyle {
    pub background: [f32; 4],
    pub fill: [f32; 4],
    pub shadow: [f32; 4],
    pub ink: [f32; 4],
    pub light_direction: [f32; 3],
    /// Stable redraw epoch. It advances at the authored stroke cadence rather
    /// than at host FPS, so line boiling holds between artistic frames.
    pub artistic_frame: u64,
    /// Whether this contribution participates in the global artistic redraw
    /// cadence. Runtime NPR sets this for every mesh so stationary geometry
    /// also receives fresh hand-drawn realizations.
    pub temporal: bool,
    pub draw_silhouettes: bool,
    pub draw_creases: bool,
    pub draw_material_seams: bool,
    /// Emits restrained interior form lines for smooth/suggestive presets.
    /// This is an explicit domain contribution, not a renderer guess.
    pub draw_form_lines: bool,
    /// Emits sparse contact/value boundaries where adjacent visible faces
    /// cross a meaningful lighting transition.
    pub draw_contact_lines: bool,
    pub outline_width_pixels: f32,
    pub boundary_width_pixels: f32,
    pub crease_width_pixels: f32,
    pub form_line_width_pixels: f32,
    pub contact_line_width_pixels: f32,
    pub crease_angle_radians: f32,
    pub wobble_pixels: f32,
    /// Number of correlated samples used to turn a straight projected edge
    /// into a hand gesture. This is deliberately an authored quality knob.
    pub stroke_segments: u8,
    /// Enables topology-chain synthesis for an authored character/object.
    /// Large architectural meshes intentionally keep the cheaper edge path.
    pub join_strokes: bool,
    /// Local pressure variation, separate from the average tool pressure.
    pub pressure_variation: f32,
    pub taper: f32,
    pub overstroke: f32,
    pub pressure: f32,
    pub hardness: f32,
    pub dryness: f32,
    /// Graphite grain amount for pencil-like strokes. Zero keeps the ink path.
    pub pencil_grain: f32,
    pub redraw_strength: f32,
    pub hatching_enabled: bool,
    pub hatching_angle_degrees: f32,
    pub hatching_spacing_pixels: f32,
    pub hatching_cross: f32,
    pub hatching_density: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprDrawCommand {
    pub object_id: String,
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
            object_id: String::new(),
            packet,
            preset,
            layers,
            material_base_color: None,
        }
    }
    pub fn with_object_id(mut self, object_id: impl Into<String>) -> Self {
        self.object_id = object_id.into();
        self
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
