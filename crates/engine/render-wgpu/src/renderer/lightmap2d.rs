use crate::renderer::*;
use amigo_render_api::{
    LayeredImageAsset, LayeredImageBlendMode2d, LayeredImageViewportFit2d,
    RenderLightMap2dSource, RenderLightMap2dSourceKind, RenderPrimitive2d,
};
#[cfg(test)]
use amigo_render_api::{
    LightEmitterKind2d, LightReceiver2dBindingPrimitive, LightReceiverDarkPolicy2dPrimitive,
    LightSampleStrategy2dPrimitive, LightSource2dCommon, Particle2dPrimitive,
    ParticleMaterialLightingMode2dPrimitive,
};

impl WgpuSceneRenderer {
    pub(crate) fn lightmap_2d_samplers(
        &mut self,
        assets: &AssetCatalog,
        viewport: &Viewport,
        renderables: &[Renderable2dItem],
        sources: &[RenderLightMap2dSource],
    ) -> Vec<LightMap2dSampler> {
        sources
            .iter()
            .filter_map(|source| {
                self.lightmap_2d_sampler_from_source(
                    assets,
                    viewport,
                    renderables,
                    source,
                )
            })
            .collect()
    }

    fn lightmap_2d_sampler_from_source(
        &mut self,
        assets: &AssetCatalog,
        viewport: &Viewport,
        renderables: &[Renderable2dItem],
        source: &RenderLightMap2dSource,
    ) -> Option<LightMap2dSampler> {
        match source.source.kind {
            RenderLightMap2dSourceKind::LayeredImage2d => self.lightmap_2d_sampler_from_layered_image(
                assets,
                viewport,
                renderables,
                source,
            ),
        }
    }

    fn lightmap_2d_sampler_from_layered_image(
        &mut self,
        assets: &AssetCatalog,
        viewport: &Viewport,
        renderables: &[Renderable2dItem],
        source: &RenderLightMap2dSource,
    ) -> Option<LightMap2dSampler> {
        let item = renderables.iter().find(|item| {
            item.owner_entity() == source.source.entity_name
                && matches!(item.primitive, RenderPrimitive2d::LayeredTexturedQuads(_))
        })?;
        let RenderPrimitive2d::LayeredTexturedQuads(command) = &item.primitive else {
            return None;
        };
        let prepared = assets.prepared_asset(&command.asset)?;
        let base_dir = prepared.resolved_path.parent()?.to_path_buf();
        let mut asset = assets.layered_image_asset(&command.asset)?;
        Self::apply_primitive_layer_overrides(&mut asset, &command.layer_overrides);
        let transform = command.transform;

        let size = lightmap_2d_render_size(
            viewport,
            command.size,
            asset.canvas_size,
            Self::primitive_layered_image_viewport_fit(command.viewport_fit),
        );
        let mut layer_lookup = BTreeMap::new();

        for layer in asset.layers {
            if !layer.enabled || layer.opacity <= 0.0 {
                continue;
            }
            let image_path = base_dir.join(&layer.image);
            let Some(image) = self.ensure_lightmap_2d_image_from_path(image_path) else {
                continue;
            };
            layer_lookup.insert(
                layer.id,
                LightMap2dLayer {
                    image,
                    opacity: layer.opacity.clamp(0.0, 4.0),
                },
            );
        }

        let channels = source
            .channels
            .iter()
            .filter_map(|channel| {
                let layers = channel
                    .layers
                    .iter()
                    .filter_map(|layer_id| layer_lookup.get(layer_id).cloned())
                    .collect::<Vec<_>>();
                (!layers.is_empty()).then_some((channel.id.clone(), layers))
            })
            .collect::<BTreeMap<_, _>>();

        (!channels.is_empty()).then_some(LightMap2dSampler {
            id: source.source_id.clone(),
            transform,
            size,
            channels,
        })
    }

    fn primitive_layered_image_viewport_fit(
        fit: amigo_render_api::LayeredImageViewportFit2dPrimitive,
    ) -> LayeredImageViewportFit2d {
        match fit {
            amigo_render_api::LayeredImageViewportFit2dPrimitive::Fixed => LayeredImageViewportFit2d::Fixed,
            amigo_render_api::LayeredImageViewportFit2dPrimitive::Stretch => LayeredImageViewportFit2d::Stretch,
            amigo_render_api::LayeredImageViewportFit2dPrimitive::Contain => LayeredImageViewportFit2d::Contain,
            amigo_render_api::LayeredImageViewportFit2dPrimitive::Cover => LayeredImageViewportFit2d::Cover,
        }
    }

    fn apply_primitive_layer_overrides(
        asset: &mut LayeredImageAsset,
        overrides: &[amigo_render_api::LayeredImageLayerOverride2dPrimitive],
    ) {
        for override_entry in overrides {
            let Some(layer) = asset.layers.iter_mut().find(|layer| layer.id == override_entry.id)
            else {
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

    fn ensure_lightmap_2d_image_from_path(
        &mut self,
        image_path: PathBuf,
    ) -> Option<LightMap2dImageData> {
        let key = format!("lightmap:{}", image_path.display());
        let modified_at = fs::metadata(&image_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
        let should_reload = self
            .lightmap_2d_image_cache
            .get(&key)
            .map(|cached| cached.image_path != image_path || cached.modified_at != modified_at)
            .unwrap_or(true);

        if should_reload {
            let image = image::open(&image_path).ok()?.to_rgba8();
            let (width, height) = image.dimensions();
            if width == 0 || height == 0 {
                return None;
            }
            let pixels = image
                .pixels()
                .map(|pixel| {
                    let [r, g, b, a] = pixel.0;
                    [
                        r as f32 / 255.0,
                        g as f32 / 255.0,
                        b as f32 / 255.0,
                        a as f32 / 255.0,
                    ]
                })
                .collect::<Vec<_>>();
            let data = LightMap2dImageData {
                width,
                height,
                pixels: Arc::new(pixels),
            };
            self.lightmap_2d_image_cache.insert(
                key.clone(),
                CachedLightMap2dImage {
                    image_path,
                    modified_at,
                    data,
                },
            );
        }

        self.lightmap_2d_image_cache
            .get(&key)
            .map(|cached| cached.data.clone())
    }
}

#[cfg(test)]
pub(crate) fn lit_particle_color(
    particle: &Particle2dPrimitive,
    lights: &[ParticleRenderLight],
    lightmaps: &[LightMap2dSampler],
    light_sources: &[LightSource2dCommon],
    light_routes: &[LightRoute2dCommand],
) -> ColorRgba {
    if !particle.material.receives_light {
        return particle.color;
    }

    match particle.material.lighting_mode {
        ParticleMaterialLightingMode2dPrimitive::Unlit => particle.color,
        ParticleMaterialLightingMode2dPrimitive::DynamicLights => {
            dynamic_lit_particle_color(particle, lights)
        }
        ParticleMaterialLightingMode2dPrimitive::LightMapSampled => particle
            .material
            .light_receiver
            .as_ref()
            .map(|binding| lightmapped_particle_color(particle, binding, lightmaps, light_sources))
            .unwrap_or(particle.color),
        ParticleMaterialLightingMode2dPrimitive::LightGroupSampled => particle
            .material
            .light_receiver
            .as_ref()
            .map(|binding| {
                light_group_particle_color(
                    particle,
                    binding,
                    lightmaps,
                    light_sources,
                    light_routes,
                )
            })
            .unwrap_or(particle.color),
    }
}

#[cfg(test)]
fn dynamic_lit_particle_color(
    particle: &Particle2dPrimitive,
    lights: &[ParticleRenderLight],
) -> ColorRgba {
    if particle.material.light_response <= 0.0 {
        return particle.color;
    }
    let mut r = particle.color.r;
    let mut g = particle.color.g;
    let mut b = particle.color.b;
    for light in lights {
        let dx = particle.position.x - light.position.x;
        let dy = particle.position.y - light.position.y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance >= light.radius {
            continue;
        }
        let falloff = 1.0 - distance / light.radius;
        let amount = falloff.powf(3.0) * light.intensity * particle.material.light_response;
        r += light.color.r * amount;
        g += light.color.g * amount;
        b += light.color.b * amount;
    }

    ColorRgba::new(
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        particle.color.a,
    )
}

#[cfg(test)]
fn light_group_particle_color(
    particle: &Particle2dPrimitive,
    binding: &LightReceiver2dBindingPrimitive,
    lightmaps: &[LightMap2dSampler],
    light_sources: &[LightSource2dCommon],
    light_routes: &[LightRoute2dCommand],
) -> ColorRgba {
    let allowed_groups =
        permitted_receiver_groups(&binding.groups, &particle.render_layer, light_routes);
    let mut r: f32 = 0.0;
    let mut g: f32 = 0.0;
    let mut b: f32 = 0.0;
    let mut sampled_any_position = false;

    for group_id in allowed_groups {
        for source in light_sources.iter().filter(|source| {
            source.emitter_kind == LightEmitterKind2d::LightGroup && source.owner == group_id
        }) {
            let tint = source
                .color_rgba
                .map(|[r, g, b, a]| ColorRgba::new(r, g, b, a))
                .unwrap_or(ColorRgba::WHITE);
            match parse_light_group_source_ref(group_id, source.emitter_id.as_deref()) {
                Some(LightGroupSourceRef::LightMapChannel { source: lightmap_id, channel }) => {
                    let source_scale = source.effective_intensity.unwrap_or_default().max(0.0);
                    if source_scale <= 0.0 {
                        continue;
                    }
                    let any = sample_lightmap_channel_into(
                        particle,
                        binding,
                        lightmaps,
                        lightmap_id,
                        channel,
                        source_scale,
                        tint,
                        &mut r,
                        &mut g,
                        &mut b,
                    );
                    sampled_any_position = sampled_any_position || any;
                }
                Some(LightGroupSourceRef::GlobalLight { .. }) => {
                    let scale = source.effective_intensity.unwrap_or_default().max(0.0);
                    if scale <= 0.0 {
                        continue;
                    }
                    let [sr, sg, sb, _] = source.color_rgba.unwrap_or([0.0, 0.0, 0.0, 0.0]);
                    r = r.max((sr * scale).clamp(0.0, 1.0));
                    g = g.max((sg * scale).clamp(0.0, 1.0));
                    b = b.max((sb * scale).clamp(0.0, 1.0));
                }
                None => {
                    let scale = source.effective_intensity.unwrap_or_default().max(0.0);
                    if scale <= 0.0 {
                        continue;
                    }
                    let [sr, sg, sb, _] = source.color_rgba.unwrap_or([0.0, 0.0, 0.0, 0.0]);
                    r = r.max((sr * scale).clamp(0.0, 1.0));
                    g = g.max((sg * scale).clamp(0.0, 1.0));
                    b = b.max((sb * scale).clamp(0.0, 1.0));
                }
            }
        }
    }

    finish_lightmapped_particle_color(particle, binding, sampled_any_position, r, g, b)
}

#[cfg(test)]
enum LightGroupSourceRef<'a> {
    GlobalLight {
        id: &'a str,
    },
    LightMapChannel {
        source: &'a str,
        channel: &'a str,
    },
}

#[cfg(test)]
fn parse_light_group_source_ref<'a>(
    group_id: &str,
    emitter_id: Option<&'a str>,
) -> Option<LightGroupSourceRef<'a>> {
    let emitter_id = emitter_id?;
    let global_prefix = format!("{group_id}:global:");
    if let Some(id) = emitter_id.strip_prefix(&global_prefix) {
        return Some(LightGroupSourceRef::GlobalLight { id });
    }

    let lightmap_prefix = format!("{group_id}:lightmap:");
    let remainder = emitter_id.strip_prefix(&lightmap_prefix)?;
    let (source, channel) = remainder.split_once(':')?;
    Some(LightGroupSourceRef::LightMapChannel { source, channel })
}

#[cfg(test)]
fn permitted_receiver_groups<'a>(
    receiver_groups: &'a [String],
    receiver_layer: &str,
    light_routes: &[LightRoute2dCommand],
) -> Vec<&'a str> {
    let Some(route) = light_routes
        .iter()
        .find(|route| route.receiver_layer == receiver_layer)
    else {
        return receiver_groups.iter().map(String::as_str).collect();
    };

    receiver_groups
        .iter()
        .filter(|group| route.groups.iter().any(|allowed| allowed == *group))
        .map(String::as_str)
        .collect()
}

#[cfg(test)]
fn lightmapped_particle_color(
    particle: &Particle2dPrimitive,
    binding: &LightReceiver2dBindingPrimitive,
    lightmaps: &[LightMap2dSampler],
    light_sources: &[LightSource2dCommon],
) -> ColorRgba {
    let mut r: f32 = 0.0;
    let mut g: f32 = 0.0;
    let mut b: f32 = 0.0;
    let sampled_any_position = sample_lightmap_channel_into(
        particle,
        binding,
        lightmaps,
        &binding.source,
        &binding.channel,
        1.0,
        ColorRgba::WHITE,
        &mut r,
        &mut g,
        &mut b,
    );

    for response in &binding.global_lights {
        sample_global_light_into(
            light_sources,
            &response.id,
            response.response.max(0.0),
            ColorRgba::WHITE,
            &mut r,
            &mut g,
            &mut b,
        );
    }

    finish_lightmapped_particle_color(particle, binding, sampled_any_position, r, g, b)
}

#[cfg(test)]
fn sample_lightmap_channel_into(
    particle: &Particle2dPrimitive,
    binding: &LightReceiver2dBindingPrimitive,
    lightmaps: &[LightMap2dSampler],
    source: &str,
    channel: &str,
    response: f32,
    tint: ColorRgba,
    r: &mut f32,
    g: &mut f32,
    b: &mut f32,
) -> bool {
    let Some(sampler) = lightmaps.iter().find(|sampler| sampler.id == source) else {
        return false;
    };
    let Some(layers) = sampler.channels.get(channel) else {
        return false;
    };
    let mut sampled_any_position = false;
    for_each_particle_light_sample_position(particle, binding, |position| {
        let Some(uv) = sampler.uv_for_world_position(position) else {
            return;
        };
        sampled_any_position = true;

        for layer in layers {
            let [sr, sg, sb, sa] = layer.image.sample_soft(uv, binding.radius_px);
            let scale = layer.opacity * sa * response;
            *r = r.max((sr * tint.r * scale).clamp(0.0, 1.0));
            *g = g.max((sg * tint.g * scale).clamp(0.0, 1.0));
            *b = b.max((sb * tint.b * scale).clamp(0.0, 1.0));
        }
    });
    sampled_any_position
}

#[cfg(test)]
fn sample_global_light_into(
    light_sources: &[LightSource2dCommon],
    id: &str,
    response: f32,
    tint: ColorRgba,
    r: &mut f32,
    g: &mut f32,
    b: &mut f32,
) {
    let Some(light) = light_sources.iter().find(|light| {
        light.emitter_kind == LightEmitterKind2d::GlobalLight
            && light.emitter_id.as_deref() == Some(id)
    }) else {
        return;
    };
    let scale = light.intensity.unwrap_or_default().max(0.0) * response.max(0.0);
    if scale <= 0.0 {
        return;
    }
    let [lr, lg, lb, _] = light.color_rgba.unwrap_or([0.0, 0.0, 0.0, 0.0]);
    *r = r.max((lr * tint.r * scale).clamp(0.0, 1.0));
    *g = g.max((lg * tint.g * scale).clamp(0.0, 1.0));
    *b = b.max((lb * tint.b * scale).clamp(0.0, 1.0));
}

#[cfg(test)]
fn finish_lightmapped_particle_color(
    particle: &Particle2dPrimitive,
    binding: &LightReceiver2dBindingPrimitive,
    sampled_any_position: bool,
    r: f32,
    g: f32,
    b: f32,
) -> ColorRgba {
    let intensity = r.max(g).max(b);
    if intensity <= 0.002 {
        return match binding.dark_policy {
            LightReceiverDarkPolicy2dPrimitive::Transparent => {
                ColorRgba::new(0.0, 0.0, 0.0, particle.color.a)
            }
            LightReceiverDarkPolicy2dPrimitive::BaseColor => particle.color,
            LightReceiverDarkPolicy2dPrimitive::ShadowTint => ColorRgba::new(
                particle.color.r * 0.18,
                particle.color.g * 0.18,
                particle.color.b * 0.18,
                particle.color.a,
            ),
        };
    }

    let sampled_scale = if sampled_any_position { 1.0 } else { 0.85 };
    let energy = (particle.color.a.clamp(0.0, 1.0).sqrt()
        * particle.material.light_response.max(0.0)
        * binding.exposure.max(0.0)
        * sampled_scale)
        .clamp(0.0, 1.0);

    ColorRgba::new(
        (r * energy).clamp(0.0, 1.0),
        (g * energy).clamp(0.0, 1.0),
        (b * energy).clamp(0.0, 1.0),
        particle.color.a,
    )
}

fn lightmap_2d_render_size(
    viewport: &Viewport,
    fixed_size: Vec2,
    canvas_size: Vec2,
    fit: LayeredImageViewportFit2d,
) -> Vec2 {
    let viewport_size = viewport.size();
    match fit {
        LayeredImageViewportFit2d::Fixed => fixed_size,
        LayeredImageViewportFit2d::Stretch => viewport_size,
        LayeredImageViewportFit2d::Contain => scaled_lightmap_2d_size(canvas_size, viewport_size, f32::min),
        LayeredImageViewportFit2d::Cover => scaled_lightmap_2d_size(canvas_size, viewport_size, f32::max),
    }
}

fn scaled_lightmap_2d_size(
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

#[cfg(test)]
impl LightMap2dSampler {
    fn uv_for_world_position(&self, position: Vec2) -> Option<Vec2> {
        if self.size.x <= f32::EPSILON || self.size.y <= f32::EPSILON {
            return None;
        }
        let local = inverse_transform_point_2d(position, self.transform);
        let u = local.x / self.size.x + 0.5;
        let v = 0.5 - local.y / self.size.y;
        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return None;
        }
        Some(Vec2::new(u, v))
    }
}

#[cfg(test)]
impl LightMap2dImageData {
    fn sample(&self, uv: Vec2) -> [f32; 4] {
        if self.width == 0 || self.height == 0 || self.pixels.is_empty() {
            return [0.0, 0.0, 0.0, 0.0];
        }
        let x = (uv.x.clamp(0.0, 1.0) * (self.width.saturating_sub(1)) as f32).round() as u32;
        let y = (uv.y.clamp(0.0, 1.0) * (self.height.saturating_sub(1)) as f32).round() as u32;
        self.pixels
            .get((y * self.width + x) as usize)
            .copied()
            .unwrap_or([0.0, 0.0, 0.0, 0.0])
    }

    fn sample_soft(&self, uv: Vec2, radius_px: f32) -> [f32; 4] {
        if radius_px <= 0.5 {
            return self.sample(uv);
        }

        let steps = if radius_px > 48.0 { 2 } else { 1 };
        let radius_u = radius_px / self.width.max(1) as f32;
        let radius_v = radius_px / self.height.max(1) as f32;
        let mut premul = [0.0_f32; 3];
        let mut light_sum = 0.0_f32;
        let mut weight_sum = 0.0_f32;

        for y in -steps..=steps {
            for x in -steps..=steps {
                let tx = x as f32 / steps as f32;
                let ty = y as f32 / steps as f32;
                let sample = self.sample(Vec2::new(uv.x + tx * radius_u, uv.y + ty * radius_v));
                let distance = (tx * tx + ty * ty).sqrt();
                if distance > 1.01 {
                    continue;
                }
                let weight = (1.0 - distance * 0.58).max(0.0);
                let light = sample[0].max(sample[1]).max(sample[2]) * sample[3];
                if light <= 0.01 {
                    weight_sum += weight;
                    continue;
                }
                premul[0] += sample[0] * light * weight;
                premul[1] += sample[1] * light * weight;
                premul[2] += sample[2] * light * weight;
                light_sum += light * weight;
                weight_sum += weight;
            }
        }

        if light_sum <= f32::EPSILON || weight_sum <= f32::EPSILON {
            return [0.0, 0.0, 0.0, 0.0];
        }

        [
            (premul[0] / light_sum).clamp(0.0, 1.0),
            (premul[1] / light_sum).clamp(0.0, 1.0),
            (premul[2] / light_sum).clamp(0.0, 1.0),
            (light_sum / weight_sum).clamp(0.0, 1.0),
        ]
    }
}

#[cfg(test)]
fn for_each_particle_light_sample_position(
    particle: &Particle2dPrimitive,
    binding: &LightReceiver2dBindingPrimitive,
    mut f: impl FnMut(Vec2),
) {
    if binding.sample_strategy == LightSampleStrategy2dPrimitive::Point {
        f(particle.position);
        return;
    }

    let length = particle_light_sample_length(particle);
    if length <= f32::EPSILON {
        f(particle.position);
        return;
    }

    let direction = Vec2::new(
        particle.transform.rotation_radians.cos(),
        particle.transform.rotation_radians.sin(),
    );
    let count = binding.sample_points.clamp(1, 9);
    if count == 1 {
        f(particle.position);
        return;
    }

    for index in 0..count {
        let t = index as f32 / (count - 1) as f32 - 0.5;
        f(Vec2::new(
            particle.position.x + direction.x * length * t,
            particle.position.y + direction.y * length * t,
        ));
    }
}

#[cfg(test)]
fn particle_light_sample_length(particle: &Particle2dPrimitive) -> f32 {
    let amigo_render_api::ParticleShape2dPrimitive::Line { length } = particle.shape else {
        return particle.size.max(1.0);
    };
    let Some(stretch) = particle.motion_stretch else {
        return length.max(0.0);
    };
    if !stretch.enabled {
        return length.max(0.0);
    }
    let delta = Vec2::new(
        particle.position.x - particle.previous_position.x,
        particle.position.y - particle.previous_position.y,
    );
    let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
    (length + distance * stretch.velocity_scale).min(stretch.max_length.max(length))
}

#[cfg(test)]
fn inverse_transform_point_2d(point: Vec2, transform: Transform2) -> Vec2 {
    let translated = Vec2::new(
        point.x - transform.translation.x,
        point.y - transform.translation.y,
    );
    let sin = (-transform.rotation_radians).sin();
    let cos = (-transform.rotation_radians).cos();
    let rotated = Vec2::new(
        translated.x * cos - translated.y * sin,
        translated.x * sin + translated.y * cos,
    );
    Vec2::new(
        rotated.x / transform.scale.x.max(0.0001),
        rotated.y / transform.scale.y.max(0.0001),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_2d_composition::LightRoute2dCommand;
    use amigo_render_api::{
        LightContributionKind2d, LightReceiver2dBindingPrimitive,
        LightReceiverDarkPolicy2dPrimitive, LightReceiverGlobalLight2dPrimitive,
        LightSampleStrategy2dPrimitive, LightSource2dCommonParams, Particle2dPrimitive,
        ParticleBlendMode2dPrimitive, ParticleLineAnchor2dPrimitive,
        ParticleMaterial2dPrimitive, ParticleMaterialLightingMode2dPrimitive,
        ParticleShape2dPrimitive,
    };

    fn particle_with_binding(binding: LightReceiver2dBindingPrimitive) -> Particle2dPrimitive {
        Particle2dPrimitive {
            emitter_entity_name: "rain".to_owned(),
            render_layer: "default".to_owned(),
            previous_position: Vec2::ZERO,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            size: 1.0,
            color: ColorRgba::new(1.0, 1.0, 1.0, 0.25),
            shape: ParticleShape2dPrimitive::Line { length: 8.0 },
            line_anchor: ParticleLineAnchor2dPrimitive::Center,
            blend_mode: ParticleBlendMode2dPrimitive::Screen,
            motion_stretch: None,
            material: ParticleMaterial2dPrimitive {
                lighting_mode: ParticleMaterialLightingMode2dPrimitive::LightMapSampled,
                receives_light: true,
                light_response: 1.0,
                light_receiver: Some(binding),
            },
            light: None,
            light_position: None,
            transform: Transform2::default(),
        }
    }

    fn binding(channel: &str) -> LightReceiver2dBindingPrimitive {
        LightReceiver2dBindingPrimitive {
            groups: Vec::new(),
            source: "test-lightmap".to_owned(),
            channel: channel.to_owned(),
            sample_strategy: LightSampleStrategy2dPrimitive::Point,
            sample_points: 1,
            radius_px: 0.0,
            exposure: 1.0,
            dark_policy: LightReceiverDarkPolicy2dPrimitive::Transparent,
            global_lights: Vec::new(),
        }
    }

    fn sampler() -> LightMap2dSampler {
        let red = LightMap2dLayer {
            image: LightMap2dImageData {
                width: 1,
                height: 1,
                pixels: Arc::new(vec![[1.0, 0.0, 0.0, 1.0]]),
            },
            opacity: 1.0,
        };
        let blue = LightMap2dLayer {
            image: LightMap2dImageData {
                width: 1,
                height: 1,
                pixels: Arc::new(vec![[0.0, 0.0, 1.0, 1.0]]),
            },
            opacity: 1.0,
        };

        LightMap2dSampler {
            id: "test-lightmap".to_owned(),
            transform: Transform2::default(),
            size: Vec2::new(128.0, 128.0),
            channels: BTreeMap::from([
                ("near".to_owned(), vec![red]),
                ("far".to_owned(), vec![blue]),
            ]),
        }
    }

    fn light_group_sources(id: &str, channel: &str) -> Vec<LightSource2dCommon> {
        vec![LightSource2dCommon::active(LightSource2dCommonParams {
            owner: id.to_owned(),
            component_kind: "LightGroup2D".to_owned(),
            emitter_kind: LightEmitterKind2d::LightGroup,
            emitter_id: Some(format!("{id}:lightmap:test-lightmap:{channel}")),
            render_layer: None,
            color_rgba: Some([1.0, 1.0, 1.0, 1.0]),
            intensity: Some(1.0),
            effective_intensity: Some(1.0),
            response: Some(1.0),
            camera_response: None,
            bloom: None,
            radius_px: None,
            falloff: None,
            distance_m: None,
            z_depth: None,
            contributions: vec![LightContributionKind2d::LightingEmit],
            reason: "test_light_group".to_owned(),
            position_px: None,
        })]
    }

    fn light_group_particle(groups: Vec<String>) -> Particle2dPrimitive {
        let mut receiver = binding("near");
        receiver.groups = groups;
        let mut particle = particle_with_binding(receiver);
        particle.material.lighting_mode = ParticleMaterialLightingMode2dPrimitive::LightGroupSampled;
        particle
    }

    #[test]
    fn lightmap_channel_controls_particle_color() {
        let near = lit_particle_color(
            &particle_with_binding(binding("near")),
            &[],
            &[sampler()],
            &[],
            &[],
        );
        let far = lit_particle_color(
            &particle_with_binding(binding("far")),
            &[],
            &[sampler()],
            &[],
            &[],
        );

        assert!(near.r > near.b);
        assert!(far.b > far.r);
    }

    #[test]
    fn transparent_dark_policy_returns_black_particle_color_in_darkness() {
        let color = lit_particle_color(
            &particle_with_binding(binding("near")),
            &[],
            &[],
            &[],
            &[],
        );

        assert_eq!(color, ColorRgba::new(0.0, 0.0, 0.0, 0.25));
    }

    #[test]
    fn base_color_dark_policy_preserves_particle_color_in_darkness() {
        let mut binding = binding("near");
        binding.dark_policy = LightReceiverDarkPolicy2dPrimitive::BaseColor;

        let color = lit_particle_color(&particle_with_binding(binding), &[], &[], &[], &[]);

        assert_eq!(color, ColorRgba::new(1.0, 1.0, 1.0, 0.25));
    }

    #[test]
    fn shadow_tint_dark_policy_keeps_dim_particle_color_in_darkness() {
        let mut binding = binding("near");
        binding.dark_policy = LightReceiverDarkPolicy2dPrimitive::ShadowTint;

        let color = lit_particle_color(&particle_with_binding(binding), &[], &[], &[], &[]);

        assert_eq!(color, ColorRgba::new(0.18, 0.18, 0.18, 0.25));
    }

    #[test]
    fn receives_light_false_returns_base_particle_color_even_with_light_binding() {
        let mut particle = particle_with_binding(binding("near"));
        particle.material.receives_light = false;

        let color = lit_particle_color(&particle, &[], &[], &[], &[]);

        assert_eq!(color, ColorRgba::new(1.0, 1.0, 1.0, 0.25));
    }

    #[test]
    fn global_light_can_light_particle_without_lightmap_sample() {
        let mut binding = binding("near");
        binding.global_lights = vec![LightReceiverGlobalLight2dPrimitive {
            id: "lightning".to_owned(),
            response: 1.0,
        }];
        let color = lit_particle_color(
            &particle_with_binding(binding),
            &[],
            &[],
            &[LightSource2dCommon::active(LightSource2dCommonParams {
                owner: "storm-controller".to_owned(),
                component_kind: "GlobalLight2D".to_owned(),
                emitter_kind: LightEmitterKind2d::GlobalLight,
                emitter_id: Some("lightning".to_owned()),
                render_layer: None,
                color_rgba: Some([0.5, 0.75, 1.0, 1.0]),
                intensity: Some(1.0),
                effective_intensity: Some(1.0),
                response: Some(1.0),
                camera_response: None,
                bloom: None,
                radius_px: None,
                falloff: None,
                distance_m: None,
                z_depth: None,
                contributions: vec![LightContributionKind2d::LightingEmit],
                reason: "test_global_light".to_owned(),
                position_px: None,
            })],
            &[],
        );

        assert!(color.b > 0.0);
        assert!(color.g > color.r);
    }

    #[test]
    fn light_group_sampled_particle_uses_matching_lightmap_channel() {
        let color = lit_particle_color(
            &light_group_particle(vec!["bar".to_owned()]),
            &[],
            &[sampler()],
            &light_group_sources("bar", "near"),
            &[],
        );

        assert!(color.r > color.b);
    }

    #[test]
    fn light_route_blocks_unlisted_group() {
        let color = lit_particle_color(
            &light_group_particle(vec!["bar".to_owned()]),
            &[],
            &[sampler()],
            &light_group_sources("bar", "near"),
            &[LightRoute2dCommand {
                source_mod: "test-mod".to_owned(),
                receiver_layer: "default".to_owned(),
                groups: vec!["skyline".to_owned()],
            }],
        );

        assert_eq!(color, ColorRgba::new(0.0, 0.0, 0.0, 0.25));
    }
}
