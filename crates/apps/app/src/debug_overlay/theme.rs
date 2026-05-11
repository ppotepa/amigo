use amigo_assets::AssetKey;
use amigo_math::ColorRgba;

use super::service::DebugOverlayLayoutMode;

#[derive(Debug, Clone, Copy)]
pub(crate) struct DebugOverlayViewportTheme {
    pub(crate) fallback_width: f32,
    pub(crate) fallback_height: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DebugOverlayLayoutTheme {
    pub(crate) margin: f32,
    pub(crate) width: f32,
    pub(crate) padding: f32,
    pub(crate) header_height: f32,
    pub(crate) line_height: f32,
    pub(crate) section_gap: f32,
    pub(crate) graph_height: f32,
    pub(crate) border_width: f32,
    pub(crate) border_radius: f32,
    pub(crate) header_font_size: f32,
    pub(crate) body_font_size: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct DebugOverlayTheme {
    pub(crate) viewport: DebugOverlayViewportTheme,
    pub(crate) compact: DebugOverlayLayoutTheme,
    pub(crate) full: DebugOverlayLayoutTheme,
    pub(crate) font: Option<AssetKey>,
    pub(crate) panel_background: ColorRgba,
    pub(crate) panel_border: ColorRgba,
    pub(crate) text: ColorRgba,
    pub(crate) muted: ColorRgba,
    pub(crate) good: ColorRgba,
    pub(crate) warning: ColorRgba,
    pub(crate) danger: ColorRgba,
}

impl DebugOverlayTheme {
    pub(crate) fn layout(&self, mode: DebugOverlayLayoutMode) -> DebugOverlayLayoutTheme {
        match mode {
            DebugOverlayLayoutMode::Compact => self.compact,
            DebugOverlayLayoutMode::Full => self.full,
        }
    }
}

impl Default for DebugOverlayTheme {
    fn default() -> Self {
        Self {
            viewport: DebugOverlayViewportTheme {
                fallback_width: 1280.0,
                fallback_height: 720.0,
            },
            compact: DebugOverlayLayoutTheme {
                margin: 12.0,
                width: 320.0,
                padding: 10.0,
                header_height: 16.0,
                line_height: 12.0,
                section_gap: 8.0,
                graph_height: 54.0,
                border_width: 1.0,
                border_radius: 6.0,
                header_font_size: 10.0,
                body_font_size: 10.0,
            },
            full: DebugOverlayLayoutTheme {
                margin: 16.0,
                width: 460.0,
                padding: 12.0,
                header_height: 18.0,
                line_height: 13.0,
                section_gap: 10.0,
                graph_height: 72.0,
                border_width: 1.0,
                border_radius: 6.0,
                header_font_size: 11.0,
                body_font_size: 10.5,
            },
            font: Some(AssetKey::new("core/fonts/console-mono")),
            panel_background: ColorRgba::new(0.02, 0.03, 0.06, 0.74),
            panel_border: ColorRgba::new(0.20, 0.80, 1.00, 0.55),
            text: ColorRgba::new(0.90, 0.97, 1.00, 1.0),
            muted: ColorRgba::new(0.52, 0.60, 0.70, 1.0),
            good: ColorRgba::new(0.36, 1.00, 0.62, 1.0),
            warning: ColorRgba::new(1.00, 0.70, 0.15, 1.0),
            danger: ColorRgba::new(1.00, 0.30, 0.43, 1.0),
        }
    }
}
