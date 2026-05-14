use std::path::PathBuf;

use amigo_assets::AssetKey;

use crate::FontGlyphSet;

/// @codemap(P1): font-domain-model
/// Engine-level Font2d model shared by runtime UI, editor metadata,
/// and render backends. This crate does not own WGPU resources.
#[derive(Debug, Clone, PartialEq)]
pub struct Font2dAsset {
    pub key: AssetKey,
    pub label: Option<String>,
    pub format: Font2dFormat,
    pub source: Font2dSource,
    pub glyphs: FontGlyphSet,
    pub metrics: Font2dMetrics,
    pub fallback: FontFallbackPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font2dFormat {
    DebugPlaceholder,
    BitmapSpritesheet,
    TrueType,
    OpenType,
    Unknown,
}

impl Font2dFormat {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "debug-placeholder" | "placeholder" => Self::DebugPlaceholder,
            "bitmap-spritesheet" | "bitmap" | "sprite-font" | "sprite_font" => {
                Self::BitmapSpritesheet
            }
            "truetype" | "true-type" | "ttf" => Self::TrueType,
            "opentype" | "open-type" | "otf" => Self::OpenType,
            _ => Self::Unknown,
        }
    }

    pub fn is_vector_font(self) -> bool {
        matches!(self, Self::TrueType | Self::OpenType)
    }

    pub fn is_bitmap_font(self) -> bool {
        matches!(self, Self::BitmapSpritesheet)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Font2dSource {
    File {
        relative_path: String,
        resolved_path: PathBuf,
    },
    AssetRef {
        key: AssetKey,
    },
    Embedded {
        id: String,
    },
    Missing,
}

impl Font2dSource {
    pub fn resolved_path(&self) -> Option<&PathBuf> {
        match self {
            Self::File { resolved_path, .. } => Some(resolved_path),
            Self::AssetRef { .. } | Self::Embedded { .. } | Self::Missing => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Font2dMetrics {
    pub default_size: f32,
    pub line_height: Option<f32>,
    pub letter_spacing: f32,
    pub tab_width: usize,
}

impl Default for Font2dMetrics {
    fn default() -> Self {
        Self {
            default_size: 12.0,
            line_height: None,
            letter_spacing: 0.0,
            tab_width: 4,
        }
    }
}

impl Font2dMetrics {
    pub fn line_height_for(&self, requested_size: f32, fallback_line_height: f32) -> f32 {
        let requested_size = requested_size.max(1.0);
        match self.line_height {
            Some(line_height) if self.default_size > 0.0 => {
                line_height * (requested_size / self.default_size)
            }
            Some(line_height) => line_height,
            None => fallback_line_height,
        }
        .max(requested_size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFallbackPolicy {
    pub missing_glyph: char,
}

impl Default for FontFallbackPolicy {
    fn default() -> Self {
        Self { missing_glyph: '?' }
    }
}
