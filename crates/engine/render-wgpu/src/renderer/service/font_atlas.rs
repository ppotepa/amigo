use std::collections::BTreeMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use amigo_assets::{AssetCatalog, AssetKey};
use amigo_font::{Font2dAsset, Font2dFormat, Font2dSource, font2d_asset_from_catalog};
use amigo_math::{ColorRgba, Vec2};
use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};

use crate::renderer::text::layout_ui_text_lines;
use crate::renderer::*;

/// @codemap(P1): render-wgpu-font-atlas
/// Runtime TTF/OTF atlas cache for screen-space UI text.
/// This is used by generated overlays, runtime UI documents, and the dev console.
/// Keep this separate from the emergency 5x7 glyph fallback.
pub(crate) struct CachedFontAtlas {
    pub(crate) texture: CachedTextureResource,
    pub(crate) glyphs: BTreeMap<char, CachedFontGlyph>,
    pub(crate) line_height: f32,
    pub(crate) missing_glyph: char,
}

#[derive(Clone, Copy)]
pub(crate) struct CachedFontGlyph {
    pub(crate) uv: Option<TextureUvRect>,
    pub(crate) size: Vec2,
    pub(crate) offset: Vec2,
    pub(crate) advance: f32,
}

impl CachedFontAtlas {
    fn glyph(&self, ch: char) -> Option<&CachedFontGlyph> {
        self.glyphs
            .get(&ch)
            .or_else(|| self.glyphs.get(&self.missing_glyph))
            .or_else(|| self.glyphs.get(&'?'))
    }
}

impl WgpuSceneRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn append_ui_ttf_font_texture_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        font: &AssetKey,
        content: &str,
        rect: crate::ui_overlay::UiRect,
        font_size: f32,
        color: ColorRgba,
        anchor: crate::ui_overlay::UiTextAnchor,
        word_wrap: bool,
        fit_to_width: bool,
    ) -> bool {
        let Some(asset) = font2d_asset_from_catalog(assets, font) else {
            return false;
        };
        if !asset.format.is_vector_font() {
            return false;
        }

        let (effective_font_size, lines) =
            layout_ui_text_lines(content, rect.width, font_size, word_wrap, fit_to_width);

        let Some(atlas) = self.ensure_ttf_font_atlas(device, queue, &asset, effective_font_size)
        else {
            return false;
        };

        let mut vertices = Vec::new();
        append_ttf_font_screen_space_vertices(
            &mut vertices,
            viewport,
            &lines,
            rect,
            color,
            anchor,
            atlas,
        );

        if vertices.is_empty() {
            return false;
        }

        batches.push(TextureBatch {
            blend_mode: TextureBlendMode::Alpha,
            bind_group: atlas.texture.bind_group.clone(),
            vertices,
        });
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn append_text2d_ttf_font_texture_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera: Transform2,
        font: &AssetKey,
        content: &str,
        transform: Transform2,
        bounds: Vec2,
        color: ColorRgba,
    ) -> bool {
        let Some(asset) = font2d_asset_from_catalog(assets, font) else {
            return false;
        };
        if !asset.format.is_vector_font() || bounds.x <= 0.0 || bounds.y <= 0.0 {
            return false;
        }

        let target_line_height = bounds.y.max(1.0);
        let requested_size = (target_line_height * asset.metrics.default_size
            / asset
                .metrics
                .line_height
                .unwrap_or(asset.metrics.default_size * 1.25)
                .max(1.0))
        .max(1.0);
        let lines = content.split('\n').map(str::to_owned).collect::<Vec<_>>();
        let Some(atlas) = self.ensure_ttf_font_atlas(device, queue, &asset, requested_size) else {
            return false;
        };

        let mut vertices = Vec::new();
        append_ttf_font_world2d_vertices(
            &mut vertices,
            viewport,
            camera,
            &lines,
            transform,
            color,
            atlas,
        );
        if vertices.is_empty() {
            return false;
        }

        batches.push(TextureBatch {
            blend_mode: TextureBlendMode::Alpha,
            bind_group: atlas.texture.bind_group.clone(),
            vertices,
        });
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn append_text3d_ttf_font_texture_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera: Transform3,
        font: &AssetKey,
        content: &str,
        transform: Transform3,
        size: f32,
        color: ColorRgba,
    ) -> bool {
        let Some(asset) = font2d_asset_from_catalog(assets, font) else {
            return false;
        };
        if !asset.format.is_vector_font() {
            return false;
        }

        let target_line_height = (size * 1.26).max(0.35);
        let requested_size = (target_line_height * asset.metrics.default_size
            / asset
                .metrics
                .line_height
                .unwrap_or(asset.metrics.default_size * 1.25)
                .max(1.0))
        .max(1.0);
        let lines = content.split('\n').map(str::to_owned).collect::<Vec<_>>();
        let Some(atlas) = self.ensure_ttf_font_atlas(device, queue, &asset, requested_size) else {
            return false;
        };

        let mut vertices = Vec::new();
        append_ttf_font_world3d_vertices(
            &mut vertices,
            viewport,
            camera,
            &lines,
            transform,
            color,
            atlas,
        );
        if vertices.is_empty() {
            return false;
        }

        batches.push(TextureBatch {
            blend_mode: TextureBlendMode::Alpha,
            bind_group: atlas.texture.bind_group.clone(),
            vertices,
        });
        true
    }

    fn ensure_ttf_font_atlas(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        asset: &Font2dAsset,
        font_size: f32,
    ) -> Option<&CachedFontAtlas> {
        if !matches!(
            asset.format,
            Font2dFormat::TrueType | Font2dFormat::OpenType
        ) {
            return None;
        }

        let source_path = match &asset.source {
            Font2dSource::File { resolved_path, .. } => resolved_path,
            Font2dSource::AssetRef { .. }
            | Font2dSource::Embedded { .. }
            | Font2dSource::Missing => {
                return None;
            }
        };

        let modified_at = fs::metadata(source_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());

        let cache_key = font_atlas_cache_key(asset, source_path, modified_at, font_size);
        let should_reload = self
            .font_atlas_cache
            .get(&cache_key)
            .map(|cached| cached.texture.image_path != *source_path)
            .unwrap_or(true);

        if should_reload {
            let atlas = build_ttf_font_atlas(
                device,
                queue,
                self,
                asset,
                source_path.clone(),
                modified_at,
                font_size,
            )?;
            self.font_atlas_cache.insert(cache_key.clone(), atlas);
        }

        self.font_atlas_cache.get(&cache_key)
    }
}

fn font_atlas_cache_key(
    asset: &Font2dAsset,
    source_path: &std::path::Path,
    modified_at: Option<SystemTime>,
    font_size: f32,
) -> String {
    let modified_key = modified_at
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!(
        "ttf:{}:{}:{:.2}:{}:{}",
        asset.key.as_str(),
        source_path.display(),
        font_size.max(1.0),
        modified_key,
        asset.glyphs.cache_key()
    )
}

fn build_ttf_font_atlas(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &WgpuSceneRenderer,
    asset: &Font2dAsset,
    source_path: std::path::PathBuf,
    modified_at: Option<SystemTime>,
    font_size: f32,
) -> Option<CachedFontAtlas> {
    let font_bytes = fs::read(&source_path).ok()?;
    let font = Font::from_bytes(font_bytes, FontSettings::default()).ok()?;

    let font_size = font_size.max(1.0);
    let line_metrics = font.horizontal_line_metrics(font_size);
    let fallback_line_height = line_metrics
        .as_ref()
        .map(|metrics| metrics.new_line_size)
        .unwrap_or(font_size * 1.2)
        .max(font_size);
    let line_height = asset
        .metrics
        .line_height_for(font_size, fallback_line_height);
    let ascent = line_metrics
        .as_ref()
        .map(|metrics| metrics.ascent)
        .unwrap_or(font_size * 0.85);

    let chars = asset.glyphs.characters(asset.fallback.missing_glyph);
    let mut rasterized = Vec::new();

    for ch in chars {
        if ch == '\t' {
            continue;
        }
        let (metrics, bitmap) = font.rasterize(ch, font_size);
        rasterized.push(RasterizedGlyph {
            ch,
            width: metrics.width as u32,
            height: metrics.height as u32,
            xmin: metrics.xmin as f32,
            ymin: metrics.ymin as f32,
            advance: metrics.advance_width.max(1.0),
            bitmap,
        });
    }

    let space_advance = font
        .rasterize(' ', font_size)
        .0
        .advance_width
        .max(font_size * 0.35);

    let atlas_width = 1024_u32;
    let padding = 2_u32;
    let mut placements = BTreeMap::new();
    let mut pen_x = padding;
    let mut pen_y = padding;
    let mut row_height = 0_u32;

    for glyph in &rasterized {
        if glyph.width == 0 || glyph.height == 0 {
            continue;
        }

        if pen_x + glyph.width + padding > atlas_width {
            pen_x = padding;
            pen_y = pen_y.saturating_add(row_height).saturating_add(padding);
            row_height = 0;
        }

        placements.insert(glyph.ch, (pen_x, pen_y));
        pen_x = pen_x.saturating_add(glyph.width).saturating_add(padding);
        row_height = row_height.max(glyph.height);
    }

    let atlas_height = next_power_of_two_u32(
        pen_y
            .saturating_add(row_height)
            .saturating_add(padding)
            .max(8),
    )
    .min(2048);

    if atlas_height == 0 || atlas_height > 2048 {
        return None;
    }

    let mut image = RgbaImage::from_pixel(atlas_width, atlas_height, Rgba([255, 255, 255, 0]));
    let mut glyphs = BTreeMap::new();

    for glyph in rasterized {
        if glyph.width == 0 || glyph.height == 0 {
            glyphs.insert(
                glyph.ch,
                CachedFontGlyph {
                    uv: None,
                    size: Vec2::new(0.0, 0.0),
                    offset: Vec2::new(glyph.xmin, 0.0),
                    advance: glyph.advance,
                },
            );
            continue;
        }

        let Some((x, y)) = placements.get(&glyph.ch).copied() else {
            continue;
        };

        for row in 0..glyph.height {
            for column in 0..glyph.width {
                let source_index = (row * glyph.width + column) as usize;
                let alpha = glyph.bitmap.get(source_index).copied().unwrap_or(0);
                image.put_pixel(x + column, y + row, Rgba([255, 255, 255, alpha]));
            }
        }

        let u0 = x as f32 / atlas_width as f32;
        let v0 = y as f32 / atlas_height as f32;
        let u1 = (x + glyph.width) as f32 / atlas_width as f32;
        let v1 = (y + glyph.height) as f32 / atlas_height as f32;
        let offset_y = ascent - glyph.ymin - glyph.height as f32;

        glyphs.insert(
            glyph.ch,
            CachedFontGlyph {
                uv: Some(TextureUvRect { u0, v0, u1, v1 }),
                size: Vec2::new(glyph.width as f32, glyph.height as f32),
                offset: Vec2::new(glyph.xmin, offset_y),
                advance: glyph.advance + asset.metrics.letter_spacing,
            },
        );
    }

    glyphs.entry(' ').or_insert(CachedFontGlyph {
        uv: None,
        size: Vec2::new(0.0, 0.0),
        offset: Vec2::new(0.0, 0.0),
        advance: space_advance + asset.metrics.letter_spacing,
    });

    glyphs.entry('\t').or_insert(CachedFontGlyph {
        uv: None,
        size: Vec2::new(0.0, 0.0),
        offset: Vec2::new(0.0, 0.0),
        advance: (space_advance * asset.metrics.tab_width as f32) + asset.metrics.letter_spacing,
    });

    let texture = renderer.create_cached_texture_resource(
        device,
        queue,
        source_path,
        modified_at,
        image,
        true,
    );

    Some(CachedFontAtlas {
        texture,
        glyphs,
        line_height,
        missing_glyph: asset.fallback.missing_glyph,
    })
}

struct RasterizedGlyph {
    ch: char,
    width: u32,
    height: u32,
    xmin: f32,
    ymin: f32,
    advance: f32,
    bitmap: Vec<u8>,
}

fn append_ttf_font_screen_space_vertices(
    vertices: &mut Vec<TextureVertex>,
    viewport: &Viewport,
    lines: &[String],
    rect: crate::ui_overlay::UiRect,
    color: ColorRgba,
    anchor: crate::ui_overlay::UiTextAnchor,
    atlas: &CachedFontAtlas,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let line_widths = lines
        .iter()
        .map(|line| measure_ttf_line_width(line, atlas))
        .collect::<Vec<_>>();
    let text_width = line_widths.iter().copied().fold(0.0, f32::max);
    let text_height = atlas.line_height * lines.len().max(1) as f32;

    let origin_x = match anchor {
        crate::ui_overlay::UiTextAnchor::TopLeft => rect.x,
        crate::ui_overlay::UiTextAnchor::Center => {
            rect.x + (rect.width - text_width).max(0.0) * 0.5
        }
    };
    let origin_y = match anchor {
        crate::ui_overlay::UiTextAnchor::TopLeft => rect.y,
        crate::ui_overlay::UiTextAnchor::Center => {
            rect.y + (rect.height - text_height).max(0.0) * 0.5
        }
    };

    for (line_index, line) in lines.iter().enumerate() {
        let line_y = origin_y + line_index as f32 * atlas.line_height;
        let line_x = match anchor {
            crate::ui_overlay::UiTextAnchor::TopLeft => origin_x,
            crate::ui_overlay::UiTextAnchor::Center => {
                rect.x + (rect.width - line_widths[line_index]).max(0.0) * 0.5
            }
        };

        let mut cursor_x = 0.0;
        for ch in line.chars() {
            let Some(glyph) = atlas.glyph(ch) else {
                continue;
            };

            if let Some(uv) = glyph.uv {
                let min = Vec2::new(line_x + cursor_x + glyph.offset.x, line_y + glyph.offset.y);
                let max = Vec2::new(min.x + glyph.size.x, min.y + glyph.size.y);

                let bottom_left = ndc_from_ui_screen(Vec2::new(min.x, max.y), viewport);
                let bottom_right = ndc_from_ui_screen(max, viewport);
                let top_right = ndc_from_ui_screen(Vec2::new(max.x, min.y), viewport);
                let top_left = ndc_from_ui_screen(min, viewport);

                push_textured_quad(
                    vertices,
                    bottom_left,
                    bottom_right,
                    top_right,
                    top_left,
                    uv,
                    color,
                );
            }

            cursor_x += glyph.advance;
        }
    }
}

fn measure_ttf_line_width(line: &str, atlas: &CachedFontAtlas) -> f32 {
    line.chars()
        .filter_map(|ch| atlas.glyph(ch))
        .map(|glyph| glyph.advance)
        .sum()
}

fn append_ttf_font_world2d_vertices(
    vertices: &mut Vec<TextureVertex>,
    viewport: &Viewport,
    camera: Transform2,
    lines: &[String],
    transform: Transform2,
    color: ColorRgba,
    atlas: &CachedFontAtlas,
) {
    let line_widths = lines
        .iter()
        .map(|line| measure_ttf_line_width(line, atlas))
        .collect::<Vec<_>>();
    let text_height = atlas.line_height * lines.len().max(1) as f32;
    let origin_y = text_height * 0.5;

    for (line_index, line) in lines.iter().enumerate() {
        let line_x = -line_widths[line_index] * 0.5;
        let line_top_y = origin_y - line_index as f32 * atlas.line_height;
        let mut cursor_x = 0.0;

        for ch in line.chars() {
            let Some(glyph) = atlas.glyph(ch) else {
                continue;
            };

            if let Some(uv) = glyph.uv {
                let left = line_x + cursor_x + glyph.offset.x;
                let top = line_top_y - glyph.offset.y;
                let right = left + glyph.size.x;
                let bottom = top - glyph.size.y;
                let quad = [
                    transform_point_2d(Vec2::new(left, bottom), transform),
                    transform_point_2d(Vec2::new(right, bottom), transform),
                    transform_point_2d(Vec2::new(right, top), transform),
                    transform_point_2d(Vec2::new(left, top), transform),
                ];
                push_textured_quad(
                    vertices,
                    ndc_from_world_2d(quad[0], camera, viewport),
                    ndc_from_world_2d(quad[1], camera, viewport),
                    ndc_from_world_2d(quad[2], camera, viewport),
                    ndc_from_world_2d(quad[3], camera, viewport),
                    uv,
                    color,
                );
            }

            cursor_x += glyph.advance;
        }
    }
}

fn append_ttf_font_world3d_vertices(
    vertices: &mut Vec<TextureVertex>,
    viewport: &Viewport,
    camera: Transform3,
    lines: &[String],
    transform: Transform3,
    color: ColorRgba,
    atlas: &CachedFontAtlas,
) {
    let line_widths = lines
        .iter()
        .map(|line| measure_ttf_line_width(line, atlas))
        .collect::<Vec<_>>();
    let text_height = atlas.line_height * lines.len().max(1) as f32;
    let origin_y = text_height * 0.5;

    for (line_index, line) in lines.iter().enumerate() {
        let line_x = -line_widths[line_index] * 0.5;
        let line_top_y = origin_y - line_index as f32 * atlas.line_height;
        let mut cursor_x = 0.0;

        for ch in line.chars() {
            let Some(glyph) = atlas.glyph(ch) else {
                continue;
            };

            if let Some(uv) = glyph.uv {
                let left = line_x + cursor_x + glyph.offset.x;
                let top = line_top_y - glyph.offset.y;
                let right = left + glyph.size.x;
                let bottom = top - glyph.size.y;
                let quad = [
                    transform_point_3d(Vec3::new(left, bottom, 0.0), transform),
                    transform_point_3d(Vec3::new(right, bottom, 0.0), transform),
                    transform_point_3d(Vec3::new(right, top, 0.0), transform),
                    transform_point_3d(Vec3::new(left, top, 0.0), transform),
                ];
                let [Some(a), Some(b), Some(c), Some(d)] = quad.map(|point| {
                    project_point(point, camera, *viewport).map(|projected| projected.position)
                }) else {
                    cursor_x += glyph.advance;
                    continue;
                };
                push_textured_quad(vertices, a, b, c, d, uv, color);
            }

            cursor_x += glyph.advance;
        }
    }
}

fn next_power_of_two_u32(value: u32) -> u32 {
    if value <= 1 {
        return 1;
    }
    value.next_power_of_two()
}
