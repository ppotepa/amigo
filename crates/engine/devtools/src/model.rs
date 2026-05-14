use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DebugOverlayPanel {
    Fps,
    FpsGraph,
    Stats,
    Particles,
    Render,
    Audio,
    Input,
    Lights,
    Layers,
    Timings,
    Scheduler,
    Memory,
}

impl DebugOverlayPanel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fps => "fps",
            Self::FpsGraph => "fps_graph",
            Self::Stats => "stats",
            Self::Particles => "particles",
            Self::Render => "render",
            Self::Audio => "audio",
            Self::Input => "input",
            Self::Lights => "lights",
            Self::Layers => "layers",
            Self::Timings => "timings",
            Self::Scheduler => "scheduler",
            Self::Memory => "memory",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugOverlayLayoutMode {
    Compact,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugOverlayCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone)]
pub struct DebugOverlaySettings {
    pub enabled: bool,
    pub layout_mode: DebugOverlayLayoutMode,
    pub corner: DebugOverlayCorner,
    pub scale: f32,
    pub panels: BTreeSet<DebugOverlayPanel>,
}

impl Default for DebugOverlaySettings {
    fn default() -> Self {
        let mut panels = BTreeSet::new();
        panels.insert(DebugOverlayPanel::Fps);
        Self {
            enabled: false,
            layout_mode: DebugOverlayLayoutMode::Compact,
            corner: DebugOverlayCorner::TopLeft,
            scale: 1.0,
            panels,
        }
    }
}
