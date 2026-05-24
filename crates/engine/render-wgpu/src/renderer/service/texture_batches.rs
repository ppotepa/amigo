use crate::renderer::service::post_fx::apply_cached_image_post_fx_rgba;
use crate::renderer::*;
use std::collections::BTreeSet;

use amigo_render_api::{
    LayeredImageAsset, LayeredImageBlendMode2d, LayeredImageViewportFit2d,
};

impl WgpuSceneRenderer {
    pub(crate) fn append_textured_quad_texture_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera: Transform2,
        quad: &amigo_render_api::TexturedQuad2dPrimitive,
    ) -> bool {
        let Some(prepared) = assets.prepared_asset(&quad.texture) else {
            return false;
        };
        let Some(texture) = self.ensure_texture(device, queue, &prepared) else {
            return false;
        };
        let uv = textured_quad_uv_rect(
            texture.dimensions(),
            quad.sheet.as_ref(),
            quad.frame_index,
        );
        let mut vertices = Vec::with_capacity(6);
        append_textured_sprite_vertices(
            &mut vertices,
            viewport,
            camera,
            quad.transform,
            quad.size,
            uv,
        );
        batches.push(TextureBatch {
            blend_mode: TextureBlendMode::Alpha,
            bind_group: texture.bind_group.clone(),
            _owned_sampler: None,
            vertices,
        });
        true
    }

    pub(crate) fn color_pipeline_for(
        &self,
        blend_mode: ParticleBlendMode2d,
    ) -> &wgpu::RenderPipeline {
        match blend_mode {
            ParticleBlendMode2d::Alpha => self.pipeline(CORE_COLOR_ALPHA_PIPELINE),
            ParticleBlendMode2d::Additive => self.pipeline(CORE_COLOR_ADDITIVE_PIPELINE),
            ParticleBlendMode2d::Multiply => self.pipeline(CORE_COLOR_MULTIPLY_PIPELINE),
            ParticleBlendMode2d::Screen => self.pipeline(CORE_COLOR_SCREEN_PIPELINE),
        }
    }

    pub(crate) fn texture_pipeline_for(
        &self,
        blend_mode: TextureBlendMode,
    ) -> &wgpu::RenderPipeline {
        match blend_mode {
            TextureBlendMode::Opaque => self.pipeline(CORE_TEXTURE_OPAQUE_PIPELINE),
            TextureBlendMode::Alpha => self.pipeline(CORE_TEXTURE_ALPHA_PIPELINE),
            TextureBlendMode::Additive => self.pipeline(CORE_TEXTURE_ADDITIVE_PIPELINE),
            TextureBlendMode::Multiply => self.pipeline(CORE_TEXTURE_MULTIPLY_PIPELINE),
            TextureBlendMode::Screen => self.pipeline(CORE_TEXTURE_SCREEN_PIPELINE),
            TextureBlendMode::Lighten => self.pipeline(CORE_TEXTURE_LIGHTEN_PIPELINE),
        }
    }

    pub(crate) fn append_tilemap_primitive_texture_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera: Transform2,
        transform: Transform2,
        tilemap: &amigo_render_api::TileMap2dPrimitive,
    ) -> bool {
        let Some(prepared) = assets.prepared_asset(&tilemap.tileset) else {
            return false;
        };
        let sheet_prepared =
            resolve_tileset_sheet_key(&prepared).and_then(|key| assets.prepared_asset(&key));
        let texture_source = sheet_prepared.as_ref().unwrap_or(&prepared);
        let Some(texture) = self.ensure_texture(device, queue, texture_source) else {
            return false;
        };
        let Some(tileset) =
            infer_tileset_from_asset(&prepared, sheet_prepared.as_ref(), tilemap.tile_size)
        else {
            return false;
        };
        let mut vertices = Vec::new();
        append_textured_tilemap_vertices(
            &mut vertices,
            viewport,
            camera,
            transform,
            tilemap,
            texture.dimensions(),
            &tileset,
        );
        if vertices.is_empty() {
            return false;
        }
        batches.push(TextureBatch {
            blend_mode: TextureBlendMode::Alpha,
            bind_group: texture.bind_group.clone(),
            _owned_sampler: None,
            vertices,
        });
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn append_layered_image_primitive_texture_batches_filtered(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera: Transform2,
        command: &amigo_render_api::LayeredImage2dPrimitive,
        included_parts: Option<&BTreeSet<String>>,
        excluded_parts: Option<&BTreeSet<String>>,
        include_base_image: bool,
    ) -> bool {
        self.append_layered_image_texture_batches_filtered_tinted(
            batches,
            device,
            queue,
            assets,
            viewport,
            camera,
            command,
            included_parts,
            excluded_parts,
            include_base_image,
            ColorRgba::WHITE,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn append_layered_image_texture_batches_filtered_tinted(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera: Transform2,
        command: &amigo_render_api::LayeredImage2dPrimitive,
        included_parts: Option<&BTreeSet<String>>,
        excluded_parts: Option<&BTreeSet<String>>,
        include_base_image: bool,
        tint: ColorRgba,
    ) -> bool {
        let Some(prepared) = assets.prepared_asset(&command.asset) else {
            return false;
        };
        let Some(base_dir) = prepared
            .resolved_path
            .parent()
            .map(|path| path.to_path_buf())
        else {
            return false;
        };
        let Some(mut asset) = assets.layered_image_asset(&command.asset) else {
            return false;
        };
        apply_primitive_layer_overrides(&mut asset, &command.layer_overrides);

        let size = layered_image_render_size(
            viewport,
            command.size,
            asset.canvas_size,
            primitive_layered_image_viewport_fit(command.viewport_fit),
        );
        let mut appended = false;
        if include_base_image {
            appended = self.append_layered_image_file_batch(
                batches,
                device,
                queue,
                viewport,
                camera,
                command.transform,
                size,
                base_dir.join(&asset.base_image),
                TextureBlendMode::Alpha,
                command.base_opacity,
                None,
                tint,
            );
        }

        for layer in &asset.layers {
            if included_parts.is_some_and(|parts| !parts.contains(&layer.id)) {
                continue;
            }
            if excluded_parts.is_some_and(|parts| parts.contains(&layer.id)) {
                continue;
            }
            if !layer.enabled {
                continue;
            }

            if layer.opacity <= 0.0 {
                // Warm hidden layer textures/post-fx so intro light reveals do not decode
                // and blur-process assets on the first visible frame.
                let _ = self.ensure_layered_image_texture_from_path(
                    device,
                    queue,
                    base_dir.join(&layer.image),
                    true,
                    layer.post_fx.as_ref(),
                );
                continue;
            }

            appended |= self.append_layered_image_file_batch(
                batches,
                device,
                queue,
                viewport,
                camera,
                command.transform,
                layered_image_layer_render_size(size, layer.post_fx.as_ref()),
                base_dir.join(&layer.image),
                texture_blend_from_layer(layer.blend_mode),
                layer.opacity,
                layer.post_fx.as_ref(),
                tint,
            );
        }

        appended
    }

    #[allow(clippy::too_many_arguments)]
    fn append_layered_image_file_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: &Viewport,
        camera: Transform2,
        transform: Transform2,
        size: Vec2,
        image_path: PathBuf,
        blend_mode: TextureBlendMode,
        opacity: f32,
        post_fx: Option<&amigo_render_api::PostFx2dStack>,
        tint: ColorRgba,
    ) -> bool {
        let Some(texture) =
            self.ensure_layered_image_texture_from_path(device, queue, image_path, true, post_fx)
        else {
            return false;
        };
        let bind_group = texture.bind_group.clone();
        let opacity = opacity.clamp(0.0, 4.0);
        let mut vertices = Vec::with_capacity(6);

        append_tinted_textured_sprite_vertices(
            &mut vertices,
            viewport,
            camera,
            transform,
            size,
            TextureUvRect {
                u0: 0.0,
                v0: 0.0,
                u1: 1.0,
                v1: 1.0,
            },
            texture_color_for_opacity_tinted(blend_mode, opacity, tint),
        );
        batches.push(TextureBatch {
            blend_mode,
            bind_group,
            _owned_sampler: None,
            vertices,
        });
        true
    }

    fn ensure_layered_image_texture_from_path(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image_path: PathBuf,
        linear_sampling: bool,
        post_fx: Option<&amigo_render_api::PostFx2dStack>,
    ) -> Option<&CachedTextureResource> {
        match post_fx.and_then(post_fx_effect) {
            Some(effect) => {
                let source_id = format!("file:{}", image_path.display());
                let cache_key = match effect.render_descriptor().cached_image_policy {
                    amigo_render_api::PostFxCachedImagePolicy::RasterEffect
                    | amigo_render_api::PostFxCachedImagePolicy::RasterEffectWithBoundsExpansion => {
                        cached_image_post_fx_cache_key(&effect, &source_id)?
                    }
                    amigo_render_api::PostFxCachedImagePolicy::PassthroughCopy
                    | amigo_render_api::PostFxCachedImagePolicy::Unsupported => {
                        return self.ensure_texture_from_path(
                            device, queue, source_id, image_path, true, false,
                        );
                    }
                };
                self.ensure_post_fx_texture_from_path(
                    device,
                    queue,
                    format!(
                        "post-fx:{}:{}:{}:{}:{}",
                        cache_key.effect_kind,
                        cache_key.source_id,
                        cache_key.radius_milli,
                        cache_key.downsample_milli,
                        cache_key.intensity_milli
                    ),
                    image_path,
                    linear_sampling,
                    effect,
                )
            }
            None => {
                let cache_key = format!("file:{}", image_path.display());
                self.ensure_texture_from_path(device, queue, cache_key, image_path, true, false)
            }
        }
    }

    fn ensure_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedAsset,
    ) -> Option<&CachedTextureResource> {
        let image_path = resolve_image_path(prepared)?;
        let linear_sampling = metadata_bool(prepared, "sampling.linear")
            || prepared
                .metadata
                .get("sampling")
                .map(|value| value.eq_ignore_ascii_case("linear"))
                .unwrap_or(false);
        let alpha_from_ink = metadata_bool(prepared, "alpha_from_ink");
        self.ensure_texture_from_path(
            device,
            queue,
            prepared.key.as_str().to_owned(),
            image_path,
            linear_sampling,
            alpha_from_ink,
        )
    }

    pub(crate) fn ensure_texture_from_path(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: String,
        image_path: PathBuf,
        linear_sampling: bool,
        alpha_from_ink: bool,
    ) -> Option<&CachedTextureResource> {
        self.ensure_texture_from_path_with_format(
            device,
            queue,
            key,
            image_path,
            linear_sampling,
            alpha_from_ink,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        )
    }

    pub(crate) fn ensure_data_texture_from_path(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: String,
        image_path: PathBuf,
        linear_sampling: bool,
        alpha_from_ink: bool,
    ) -> Option<&CachedTextureResource> {
        self.ensure_texture_from_path_with_format(
            device,
            queue,
            key,
            image_path,
            linear_sampling,
            alpha_from_ink,
            wgpu::TextureFormat::Rgba8Unorm,
        )
    }

    fn ensure_texture_from_path_with_format(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: String,
        image_path: PathBuf,
        linear_sampling: bool,
        alpha_from_ink: bool,
        format: wgpu::TextureFormat,
    ) -> Option<&CachedTextureResource> {
        let modified_at = fs::metadata(&image_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
        let format_key = format!("{key}::{format:?}");
        let should_reload = self
            .texture_cache
            .get(&format_key)
            .map(|cached| cached.image_path != image_path || cached.modified_at != modified_at)
            .unwrap_or(true);

        if should_reload {
            let image = match image::open(&image_path) {
                Ok(image) => image,
                Err(error) => {
                    self.record_emergency_error(format!(
                        "failed to decode texture `{}`: {error}",
                        image_path.display()
                    ));
                    return None;
                }
            };
            let mut rgba = image.to_rgba8();
            let (width, height) = image.dimensions();
            if width == 0 || height == 0 {
                return None;
            }
            if alpha_from_ink {
                apply_alpha_from_ink(&mut rgba);
            }

            let resource = self.create_cached_texture_resource_with_format(
                device,
                queue,
                image_path,
                modified_at,
                rgba,
                linear_sampling,
                format,
            );
            self.texture_cache.insert(format_key.clone(), resource);
        }

        self.texture_cache.get(&format_key)
    }

    fn ensure_post_fx_texture_from_path(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: String,
        image_path: PathBuf,
        linear_sampling: bool,
        effect: PostFx2d,
    ) -> Option<&CachedTextureResource> {
        let modified_at = fs::metadata(&image_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
        let should_reload = self
            .texture_cache
            .get(&key)
            .map(|cached| cached.image_path != image_path || cached.modified_at != modified_at)
            .unwrap_or(true);

        if should_reload {
            let image = match image::open(&image_path) {
                Ok(image) => image,
                Err(error) => {
                    self.record_emergency_error(format!(
                        "failed to decode texture `{}`: {error}",
                        image_path.display()
                    ));
                    return None;
                }
            };
            let rgba = apply_cached_image_post_fx_rgba(image.to_rgba8(), effect);
            let resource = self.create_cached_texture_resource(
                device,
                queue,
                image_path,
                modified_at,
                rgba,
                linear_sampling,
            );
            self.texture_cache.insert(key.clone(), resource);
        }

        self.texture_cache.get(&key)
    }

    pub(crate) fn create_cached_texture_resource(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image_path: PathBuf,
        modified_at: Option<SystemTime>,
        rgba: RgbaImage,
        linear_sampling: bool,
    ) -> CachedTextureResource {
        self.create_cached_texture_resource_with_format(
            device,
            queue,
            image_path,
            modified_at,
            rgba,
            linear_sampling,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        )
    }

    pub(crate) fn create_cached_texture_resource_with_format(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image_path: PathBuf,
        modified_at: Option<SystemTime>,
        rgba: RgbaImage,
        linear_sampling: bool,
        format: wgpu::TextureFormat,
    ) -> CachedTextureResource {
        let (width, height) = rgba.dimensions();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("amigo-scene-texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let (mag_filter, min_filter, mipmap_filter) = if linear_sampling {
            (
                wgpu::FilterMode::Linear,
                wgpu::FilterMode::Linear,
                wgpu::MipmapFilterMode::Linear,
            )
        } else {
            (
                wgpu::FilterMode::Nearest,
                wgpu::FilterMode::Nearest,
                wgpu::MipmapFilterMode::Nearest,
            )
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("amigo-scene-texture-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter,
            min_filter,
            mipmap_filter,
            ..wgpu::SamplerDescriptor::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-scene-texture-bind-group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        CachedTextureResource {
            _texture: texture,
            _view: view,
            _sampler: sampler,
            bind_group,
            image_path,
            modified_at,
            width,
            height,
        }
    }
}

pub(crate) fn primitive_layered_image_viewport_fit(
    fit: amigo_render_api::LayeredImageViewportFit2dPrimitive,
) -> LayeredImageViewportFit2d {
    match fit {
        amigo_render_api::LayeredImageViewportFit2dPrimitive::Fixed => LayeredImageViewportFit2d::Fixed,
        amigo_render_api::LayeredImageViewportFit2dPrimitive::Stretch => LayeredImageViewportFit2d::Stretch,
        amigo_render_api::LayeredImageViewportFit2dPrimitive::Contain => LayeredImageViewportFit2d::Contain,
        amigo_render_api::LayeredImageViewportFit2dPrimitive::Cover => LayeredImageViewportFit2d::Cover,
    }
}

pub(crate) fn apply_primitive_layer_overrides(
    asset: &mut LayeredImageAsset,
    overrides: &[amigo_render_api::LayeredImageLayerOverride2dPrimitive],
) {
    for override_entry in overrides {
        let Some(layer) = asset.layers.iter_mut().find(|layer| layer.id == override_entry.id) else {
            continue;
        };
        if let Some(opacity) = override_entry.opacity {
            layer.opacity = opacity;
        }
        if let Some(enabled) = override_entry.enabled {
            layer.enabled = enabled;
        }
        if let Some(blend_mode) = override_entry.blend_mode {
            layer.blend_mode = match blend_mode {
                amigo_render_api::LayeredImageBlendMode2dPrimitive::Alpha => LayeredImageBlendMode2d::Alpha,
                amigo_render_api::LayeredImageBlendMode2dPrimitive::Additive => LayeredImageBlendMode2d::Additive,
                amigo_render_api::LayeredImageBlendMode2dPrimitive::Screen => LayeredImageBlendMode2d::Screen,
                amigo_render_api::LayeredImageBlendMode2dPrimitive::Multiply => LayeredImageBlendMode2d::Multiply,
                amigo_render_api::LayeredImageBlendMode2dPrimitive::Lighten => LayeredImageBlendMode2d::Lighten,
            };
        }
    }
}

fn textured_quad_uv_rect(
    texture_size: Vec2,
    sheet: Option<&amigo_render_api::TexturedQuad2dSheet>,
    frame_index: u32,
) -> TextureUvRect {
    let Some(sheet) = sheet else {
        return TextureUvRect {
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
        };
    };

    let columns = sheet.columns.max(1);
    let rows = sheet.rows.max(1);
    let frame_count = sheet.frame_count.max(1).min(columns.saturating_mul(rows));
    let frame = frame_index.min(frame_count.saturating_sub(1));
    let column = frame % columns;
    let row = frame / columns;
    let frame_width = if sheet.frame_size.x > 0.0 {
        sheet.frame_size.x
    } else {
        texture_size.x / columns as f32
    };
    let frame_height = if sheet.frame_size.y > 0.0 {
        sheet.frame_size.y
    } else {
        texture_size.y / rows as f32
    };
    let u0 = (column as f32 * frame_width) / texture_size.x.max(1.0);
    let v0 = (row as f32 * frame_height) / texture_size.y.max(1.0);
    let u1 = ((column as f32 + 1.0) * frame_width) / texture_size.x.max(1.0);
    let v1 = ((row as f32 + 1.0) * frame_height) / texture_size.y.max(1.0);
    TextureUvRect { u0, v0, u1, v1 }
}

fn apply_alpha_from_ink(rgba: &mut image::RgbaImage) {
    for pixel in rgba.pixels_mut() {
        let [r, g, b, a] = pixel.0;
        let is_ink = a > 0 && b > 70 && r < 135 && g < 150 && b > r.saturating_add(28) && b > g;
        if is_ink {
            let darkness = 255_u8.saturating_sub(((r as u16 + g as u16) / 2).min(255) as u8);
            let alpha = (((darkness.max(96) as u16) * (a as u16)) / 255).min(255) as u8;
            *pixel = image::Rgba([255, 255, 255, alpha]);
        } else {
            *pixel = image::Rgba([255, 255, 255, 0]);
        }
    }
}

fn layered_image_layer_render_size(
    size: Vec2,
    post_fx: Option<&amigo_render_api::PostFx2dStack>,
) -> Vec2 {
    let Some(effect) = post_fx.and_then(post_fx_effect) else {
        return size;
    };
    if effect.render_descriptor().cached_image_policy
        != amigo_render_api::PostFxCachedImagePolicy::RasterEffectWithBoundsExpansion
    {
        return size;
    }
    let PostFx2d::Blur(blur) = effect else {
        return size;
    };
    let blur = blur.normalized();
    let spread = (blur.radius * 2.0).clamp(0.0, 512.0);
    Vec2::new(size.x + spread, size.y + spread)
}

fn post_fx_effect(stack: &amigo_render_api::PostFx2dStack) -> Option<PostFx2d> {
    stack.effects.first().cloned()
}

fn cached_image_post_fx_cache_key(effect: &PostFx2d, source_id: &str) -> Option<PostFx2dCacheKey> {
    match effect {
        PostFx2d::Blur(blur) => Some(PostFx2dCacheKey::blur(source_id.to_owned(), blur.clone())),
        PostFx2d::EmbossEdges(emboss) => Some(PostFx2dCacheKey::embossed_edges(
            source_id.to_owned(),
            emboss.clone(),
        )),
        _ => None,
    }
}

fn texture_color_for_opacity(blend_mode: TextureBlendMode, opacity: f32) -> ColorRgba {
    let opacity = opacity.clamp(0.0, 4.0);
    if blend_mode == TextureBlendMode::Lighten {
        ColorRgba::new(opacity, opacity, opacity, 1.0)
    } else {
        ColorRgba::new(1.0, 1.0, 1.0, opacity)
    }
}

fn texture_color_for_opacity_tinted(
    blend_mode: TextureBlendMode,
    opacity: f32,
    tint: ColorRgba,
) -> ColorRgba {
    let base = texture_color_for_opacity(blend_mode, opacity);
    ColorRgba::new(
        base.r * tint.r,
        base.g * tint.g,
        base.b * tint.b,
        base.a * tint.a,
    )
}

impl WgpuSceneRenderer {
    pub(crate) fn append_ui_bitmap_font_texture_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        font: &amigo_assets::AssetKey,
        content: &str,
        rect: crate::ui_overlay::UiRect,
        font_size: f32,
        color: ColorRgba,
        anchor: crate::ui_overlay::UiTextAnchor,
        word_wrap: bool,
        fit_to_width: bool,
    ) -> bool {
        let Some(prepared) = assets.prepared_asset(font) else {
            return false;
        };
        if !is_bitmap_font_asset(&prepared) {
            return false;
        }
        let Some(texture) = self.ensure_texture(device, queue, &prepared) else {
            return false;
        };
        let bind_group = texture.bind_group.clone();
        let texture_size = texture.dimensions();
        let mut vertices = Vec::new();
        append_bitmap_font_screen_space_vertices(
            &mut vertices,
            viewport,
            content,
            rect,
            font_size,
            color,
            anchor,
            word_wrap,
            fit_to_width,
            texture_size,
            &prepared,
        );
        if vertices.is_empty() {
            return false;
        }
        batches.push(TextureBatch {
            blend_mode: TextureBlendMode::Alpha,
            bind_group,
            _owned_sampler: None,
            vertices,
        });
        true
    }
}

fn is_bitmap_font_asset(prepared: &PreparedAsset) -> bool {
    matches!(prepared.kind, PreparedAssetKind::Font2d)
        && (prepared
            .format
            .as_deref()
            .map(|format| format == "bitmap-spritesheet")
            .unwrap_or(false)
            || prepared
                .metadata
                .get("type")
                .map(|value| value == "bitmap_font")
                .unwrap_or(false)
            || prepared
                .metadata
                .get("render_mode")
                .map(|value| value == "sprite_font")
                .unwrap_or(false))
}

fn texture_blend_from_layer(blend: LayeredImageBlendMode2d) -> TextureBlendMode {
    match blend {
        LayeredImageBlendMode2d::Alpha => TextureBlendMode::Alpha,
        LayeredImageBlendMode2d::Additive => TextureBlendMode::Additive,
        LayeredImageBlendMode2d::Screen => TextureBlendMode::Screen,
        LayeredImageBlendMode2d::Multiply => TextureBlendMode::Multiply,
        LayeredImageBlendMode2d::Lighten => TextureBlendMode::Lighten,
    }
}

fn layered_image_render_size(
    viewport: &Viewport,
    fixed_size: Vec2,
    canvas_size: Vec2,
    fit: LayeredImageViewportFit2d,
) -> Vec2 {
    let viewport_size = viewport.size();
    match fit {
        LayeredImageViewportFit2d::Fixed => fixed_size,
        LayeredImageViewportFit2d::Stretch => viewport_size,
        LayeredImageViewportFit2d::Contain => scaled_to_viewport(canvas_size, viewport_size, f32::min),
        LayeredImageViewportFit2d::Cover => scaled_to_viewport(canvas_size, viewport_size, f32::max),
    }
}

fn scaled_to_viewport(
    source_size: Vec2,
    viewport_size: Vec2,
    choose_scale: impl Fn(f32, f32) -> f32,
) -> Vec2 {
    let source_size = if source_size.x > 0.0 && source_size.y > 0.0 {
        source_size
    } else {
        viewport_size
    };
    let scale = choose_scale(
        viewport_size.x / source_size.x,
        viewport_size.y / source_size.y,
    );
    Vec2::new(source_size.x * scale, source_size.y * scale)
}
