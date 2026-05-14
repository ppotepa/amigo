use amigo_assets::AssetKey;
use amigo_math::ColorRgba;

use crate::DebugOverlayLayoutMode;

#[derive(Debug, Clone, Copy)]
pub struct DebugOverlayViewportTheme {
    pub fallback_width: f32,
    pub fallback_height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct DebugOverlayLayoutTheme {
    pub margin: f32,
    pub width: f32,
    pub padding: f32,
    pub header_height: f32,
    pub line_height: f32,
    pub section_gap: f32,
    pub graph_height: f32,
    pub border_width: f32,
    pub border_radius: f32,
    pub header_font_size: f32,
    pub body_font_size: f32,
}

#[derive(Debug, Clone)]
pub struct DebugOverlayTheme {
    pub viewport: DebugOverlayViewportTheme,
    pub compact: DebugOverlayLayoutTheme,
    pub full: DebugOverlayLayoutTheme,
    pub font: Option<AssetKey>,
    pub panel_background: ColorRgba,
    pub panel_border: ColorRgba,
    pub text: ColorRgba,
    pub muted: ColorRgba,
    pub good: ColorRgba,
    pub warning: ColorRgba,
    pub danger: ColorRgba,
}

impl DebugOverlayTheme {
    pub fn layout(&self, mode: DebugOverlayLayoutMode) -> DebugOverlayLayoutTheme {
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
