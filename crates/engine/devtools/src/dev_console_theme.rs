use amigo_assets::AssetKey;
use amigo_math::ColorRgba;
use amigo_scripting_api::DevConsoleOutputLevel;

/// @codemap(P1): dev-console-theme
/// Central visual configuration for the runtime-owned dev console overlay.
/// Keep console colors, font sizes, spacing, and responsive layout values here,
/// not in overlay.rs.
#[derive(Debug, Clone)]
pub struct DevConsoleTheme {
    pub viewport: DevConsoleViewportTheme,
    pub layout: DevConsoleLayoutTheme,
    pub font: DevConsoleFontTheme,
    pub colors: DevConsoleColorTheme,
    pub levels: DevConsoleLevelTheme,
}

#[derive(Debug, Clone, Copy)]
pub struct DevConsoleViewportTheme {
    pub fallback_width: f32,
    pub fallback_height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct DevConsoleLayoutTheme {
    pub margin: f32,
    pub min_panel_width: f32,
    pub max_panel_width: f32,
    pub min_panel_height: f32,
    pub max_panel_height: f32,
    pub panel_height_fraction: f32,
    pub panel_padding: f32,
    pub header_height: f32,
    pub input_height: f32,
    pub output_gap: f32,
    pub line_height: f32,
    pub scrollbar_width: f32,
    pub scrollbar_gap: f32,
    pub scrollbar_min_thumb_height: f32,
    pub scrollbar_min_visible_ratio: f32,
    pub border_width: f32,
    pub border_radius: f32,
}

#[derive(Debug, Clone)]
pub struct DevConsoleFontTheme {
    pub asset: Option<AssetKey>,
    pub header_size: f32,
    pub output_size: f32,
    pub input_size: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct DevConsoleColorTheme {
    pub backdrop: ColorRgba,
    pub panel_background: ColorRgba,
    pub panel_border: ColorRgba,
    pub header_text: ColorRgba,
    pub input_text: ColorRgba,
    pub scrollbar_track: ColorRgba,
    pub scrollbar_thumb: ColorRgba,
}

#[derive(Debug, Clone, Copy)]
pub struct DevConsoleLevelTheme {
    pub info: ColorRgba,
    pub success: ColorRgba,
    pub warning: ColorRgba,
    pub error: ColorRgba,
    pub command: ColorRgba,
}

impl DevConsoleTheme {
    pub fn level_color(&self, level: DevConsoleOutputLevel) -> ColorRgba {
        match level {
            DevConsoleOutputLevel::Info => self.levels.info,
            DevConsoleOutputLevel::Success => self.levels.success,
            DevConsoleOutputLevel::Warning => self.levels.warning,
            DevConsoleOutputLevel::Error => self.levels.error,
            DevConsoleOutputLevel::Command => self.levels.command,
        }
    }

    pub fn text_font(&self) -> Option<AssetKey> {
        self.font.asset.clone()
    }
}

impl Default for DevConsoleTheme {
    fn default() -> Self {
        Self {
            viewport: DevConsoleViewportTheme {
                fallback_width: 1280.0,
                fallback_height: 720.0,
            },
            layout: DevConsoleLayoutTheme {
                margin: 16.0,
                min_panel_width: 320.0,
                max_panel_width: 1180.0,
                min_panel_height: 240.0,
                max_panel_height: 440.0,
                panel_height_fraction: 0.44,
                panel_padding: 10.0,
                header_height: 16.0,
                input_height: 20.0,
                output_gap: 12.0,
                line_height: 12.5,
                scrollbar_width: 4.0,
                scrollbar_gap: 8.0,
                scrollbar_min_thumb_height: 18.0,
                scrollbar_min_visible_ratio: 0.08,
                border_width: 1.0,
                border_radius: 6.0,
            },
            font: DevConsoleFontTheme {
                asset: Some(AssetKey::new("core/fonts/console-mono")),
                header_size: 10.0,
                output_size: 10.0,
                input_size: 11.0,
            },
            colors: DevConsoleColorTheme {
                backdrop: ColorRgba::new(0.0, 0.0, 0.0, 0.20),
                panel_background: ColorRgba::new(0.015, 0.020, 0.030, 0.92),
                panel_border: ColorRgba::new(0.25, 0.75, 1.0, 0.45),
                header_text: ColorRgba::new(0.48, 0.58, 0.68, 1.0),
                input_text: ColorRgba::new(0.88, 1.0, 0.90, 1.0),
                scrollbar_track: ColorRgba::new(0.20, 0.27, 0.34, 0.35),
                scrollbar_thumb: ColorRgba::new(0.30, 0.78, 1.0, 0.75),
            },
            levels: DevConsoleLevelTheme {
                info: ColorRgba::new(0.68, 0.72, 0.78, 1.0),
                success: ColorRgba::new(0.42, 0.90, 0.55, 1.0),
                warning: ColorRgba::new(0.95, 0.78, 0.38, 1.0),
                error: ColorRgba::new(1.00, 0.35, 0.35, 1.0),
                command: ColorRgba::new(0.58, 0.78, 1.0, 1.0),
            },
        }
    }
}
