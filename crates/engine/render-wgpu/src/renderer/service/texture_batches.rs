use crate::renderer::*;

impl WgpuSceneRenderer {
    pub(crate) fn append_sprite_texture_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera: Transform2,
        transform: Transform2,
        sprite: &Sprite,
    ) -> bool {
        let Some(prepared) = assets.prepared_asset(&sprite.texture) else {
            return false;
        };
        let Some(texture) = self.ensure_texture(device, queue, &prepared) else {
            return false;
        };
        let sheet = sprite
            .sheet
            .or_else(|| infer_sprite_sheet_from_asset(&prepared));
        let uv = sprite_uv_rect(texture.dimensions(), sheet, sprite.frame_index);
        let mut vertices = Vec::with_capacity(6);
        append_textured_sprite_vertices(
            &mut vertices,
            viewport,
            camera,
            transform,
            sprite.size,
            uv,
        );
        batches.push(TextureBatch {
            bind_group: texture.bind_group.clone(),
            vertices,
        });
        true
    }

    pub(crate) fn color_pipeline_for(&self, blend_mode: ParticleBlendMode2d) -> &wgpu::RenderPipeline {
        match blend_mode {
            ParticleBlendMode2d::Alpha => &self.color_alpha_pipeline,
            ParticleBlendMode2d::Additive => &self.color_additive_pipeline,
            ParticleBlendMode2d::Multiply => &self.color_multiply_pipeline,
            ParticleBlendMode2d::Screen => &self.color_screen_pipeline,
        }
    }

    pub(crate) fn append_tilemap_texture_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera: Transform2,
        transform: Transform2,
        tilemap: &TileMap2d,
    ) -> bool {
        let Some(prepared) = assets.prepared_asset(&tilemap.tileset) else {
            return false;
        };
        let Some(texture) = self.ensure_texture(device, queue, &prepared) else {
            return false;
        };
        let Some(tileset) = infer_tileset_from_asset(&prepared, tilemap.tile_size) else {
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
            bind_group: texture.bind_group.clone(),
            vertices,
        });
        true
    }

    fn ensure_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedAsset,
    ) -> Option<&CachedTextureResource> {
        let image_path = resolve_image_path(prepared)?;
        let modified_at = fs::metadata(&image_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
        let key = prepared.key.as_str().to_owned();
        let should_reload = self
            .texture_cache
            .get(&key)
            .map(|cached| cached.image_path != image_path || cached.modified_at != modified_at)
            .unwrap_or(true);

        if should_reload {
            let image = image::open(&image_path).ok()?;
            let mut rgba = image.to_rgba8();
            let (width, height) = image.dimensions();
            if width == 0 || height == 0 {
                return None;
            }
            apply_alpha_from_ink(prepared, &mut rgba);

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
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
            let linear_sampling = metadata_bool(prepared, "sampling.linear")
                || prepared
                    .metadata
                    .get("sampling")
                    .map(|value| value.eq_ignore_ascii_case("linear"))
                    .unwrap_or(false);
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

            self.texture_cache.insert(
                key.clone(),
                CachedTextureResource {
                    _texture: texture,
                    _view: view,
                    _sampler: sampler,
                    bind_group,
                    image_path,
                    modified_at,
                    width,
                    height,
                },
            );
        }

        self.texture_cache.get(&key)
    }
}

fn apply_alpha_from_ink(prepared: &PreparedAsset, rgba: &mut image::RgbaImage) {
    if !metadata_bool(prepared, "alpha_from_ink") {
        return;
    }

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
            bind_group,
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
