use crate::renderer::*;

impl WgpuSceneRenderer {
    pub(crate) fn lightmap_2d_samplers(
        &mut self,
        assets: &AssetCatalog,
        scene: &SceneService,
        viewport: &Viewport,
        layered_image_commands: &[LayeredImageDrawCommand],
        sources: &[LightMap2dSourceCommand],
    ) -> Vec<LightMap2dSampler> {
        sources
            .iter()
            .filter_map(|source| {
                self.lightmap_2d_sampler_from_source(
                    assets,
                    scene,
                    viewport,
                    layered_image_commands,
                    source,
                )
            })
            .collect()
    }

    fn lightmap_2d_sampler_from_source(
        &mut self,
        assets: &AssetCatalog,
        scene: &SceneService,
        viewport: &Viewport,
        layered_image_commands: &[LayeredImageDrawCommand],
        source: &LightMap2dSourceCommand,
    ) -> Option<LightMap2dSampler> {
        match source.source.kind {
            LightMap2dSourceKind::LayeredImage2d => self.lightmap_2d_sampler_from_layered_image(
                assets,
                scene,
                viewport,
                layered_image_commands,
                source,
            ),
        }
    }

    fn lightmap_2d_sampler_from_layered_image(
        &mut self,
        assets: &AssetCatalog,
        scene: &SceneService,
        viewport: &Viewport,
        layered_image_commands: &[LayeredImageDrawCommand],
        source: &LightMap2dSourceCommand,
    ) -> Option<LightMap2dSampler> {
        let command = layered_image_commands
            .iter()
            .find(|command| command.entity_name == source.source.entity_name)?;
        let prepared = assets.prepared_asset(&command.image.asset)?;
        let base_dir = prepared.resolved_path.parent()?.to_path_buf();
        let mut asset = assets.layered_image_asset(&command.image.asset)?;
        apply_layer_overrides(&mut asset, &command.image.layer_overrides);
        let transform = resolve_transform2(scene, &command.entity_name, command.transform);

        let size = lightmap_2d_render_size(
            viewport,
            command.image.size,
            asset.canvas_size,
            command.image.viewport_fit,
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
            id: source.id.clone(),
            transform,
            size,
            channels,
        })
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

pub(crate) fn lit_particle_color(
    particle: &Particle2dDrawCommand,
    lights: &[ParticleRenderLight],
    lightmaps: &[LightMap2dSampler],
    global_lights: &[GlobalLight2dCommand],
    light_groups: &[LightGroup2dCommand],
    light_routes: &[LightRoute2dCommand],
) -> ColorRgba {
    if !particle.material.receives_light {
        return particle.color;
    }

    match particle.material.lighting_mode {
        amigo_light_2d_plugin::Material2dLightingMode::Unlit => particle.color,
        amigo_light_2d_plugin::Material2dLightingMode::DynamicLights => {
            dynamic_lit_particle_color(particle, lights)
        }
        amigo_light_2d_plugin::Material2dLightingMode::LightMapSampled => particle
            .material
            .light_receiver
            .as_ref()
            .map(|binding| lightmapped_particle_color(particle, binding, lightmaps, global_lights))
            .unwrap_or(particle.color),
        amigo_light_2d_plugin::Material2dLightingMode::LightGroupSampled => particle
            .material
            .light_receiver
            .as_ref()
            .map(|binding| {
                light_group_particle_color(
                    particle,
                    binding,
                    lightmaps,
                    global_lights,
                    light_groups,
                    light_routes,
                )
            })
            .unwrap_or(particle.color),
    }
}

fn dynamic_lit_particle_color(
    particle: &Particle2dDrawCommand,
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

fn light_group_particle_color(
    particle: &Particle2dDrawCommand,
    binding: &LightReceiver2dBinding,
    lightmaps: &[LightMap2dSampler],
    global_lights: &[GlobalLight2dCommand],
    light_groups: &[LightGroup2dCommand],
    light_routes: &[LightRoute2dCommand],
) -> ColorRgba {
    let allowed_groups =
        permitted_receiver_groups(&binding.groups, &particle.render_layer, light_routes);
    let mut r: f32 = 0.0;
    let mut g: f32 = 0.0;
    let mut b: f32 = 0.0;
    let mut sampled_any_position = false;

    for group_id in allowed_groups {
        let Some(group) = light_groups.iter().find(|group| group.id == group_id) else {
            continue;
        };
        let group_scale = group.intensity.max(0.0);
        if group_scale <= 0.0 {
            continue;
        }
        for source in &group.sources {
            let source_scale = group_scale * source.response.max(0.0);
            if source_scale <= 0.0 {
                continue;
            }
            match &source.kind {
                amigo_light_2d_plugin::LightGroup2dSourceKind::LightMapChannel { source, channel } => {
                    let any = sample_lightmap_channel_into(
                        particle,
                        binding,
                        lightmaps,
                        source,
                        channel,
                        source_scale,
                        group.color,
                        &mut r,
                        &mut g,
                        &mut b,
                    );
                    sampled_any_position = sampled_any_position || any;
                }
                amigo_light_2d_plugin::LightGroup2dSourceKind::GlobalLight { id } => {
                    sample_global_light_into(
                        global_lights,
                        id,
                        source_scale,
                        group.color,
                        &mut r,
                        &mut g,
                        &mut b,
                    );
                }
            }
        }
    }

    finish_lightmapped_particle_color(particle, binding, sampled_any_position, r, g, b)
}

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

fn lightmapped_particle_color(
    particle: &Particle2dDrawCommand,
    binding: &LightReceiver2dBinding,
    lightmaps: &[LightMap2dSampler],
    global_lights: &[GlobalLight2dCommand],
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
            global_lights,
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

fn sample_lightmap_channel_into(
    particle: &Particle2dDrawCommand,
    binding: &LightReceiver2dBinding,
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

fn sample_global_light_into(
    global_lights: &[GlobalLight2dCommand],
    id: &str,
    response: f32,
    tint: ColorRgba,
    r: &mut f32,
    g: &mut f32,
    b: &mut f32,
) {
    let Some(light) = global_lights.iter().find(|light| light.id == id) else {
        return;
    };
    let scale = light.intensity.max(0.0) * response.max(0.0);
    if scale <= 0.0 {
        return;
    }
    *r = r.max((light.color.r * tint.r * scale).clamp(0.0, 1.0));
    *g = g.max((light.color.g * tint.g * scale).clamp(0.0, 1.0));
    *b = b.max((light.color.b * tint.b * scale).clamp(0.0, 1.0));
}

fn finish_lightmapped_particle_color(
    particle: &Particle2dDrawCommand,
    binding: &LightReceiver2dBinding,
    sampled_any_position: bool,
    r: f32,
    g: f32,
    b: f32,
) -> ColorRgba {
    let intensity = r.max(g).max(b);
    if intensity <= 0.002 {
        return match binding.dark_policy {
            LightReceiverDarkPolicy2d::Transparent => {
                ColorRgba::new(0.0, 0.0, 0.0, particle.color.a)
            }
            LightReceiverDarkPolicy2d::BaseColor => particle.color,
            LightReceiverDarkPolicy2d::ShadowTint => ColorRgba::new(
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
    fit: amigo_layered_image_2d_plugin::LayeredImageViewportFit2d,
) -> Vec2 {
    let viewport_size = viewport.size();
    match fit {
        amigo_layered_image_2d_plugin::LayeredImageViewportFit2d::Fixed => fixed_size,
        amigo_layered_image_2d_plugin::LayeredImageViewportFit2d::Stretch => viewport_size,
        amigo_layered_image_2d_plugin::LayeredImageViewportFit2d::Contain => {
            scaled_lightmap_2d_size(canvas_size, viewport_size, f32::min)
        }
        amigo_layered_image_2d_plugin::LayeredImageViewportFit2d::Cover => {
            scaled_lightmap_2d_size(canvas_size, viewport_size, f32::max)
        }
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

fn for_each_particle_light_sample_position(
    particle: &Particle2dDrawCommand,
    binding: &LightReceiver2dBinding,
    mut f: impl FnMut(Vec2),
) {
    if binding.sample_strategy == LightSampleStrategy2d::Point {
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

fn particle_light_sample_length(particle: &Particle2dDrawCommand) -> f32 {
    let ParticleShape2d::Line { length } = particle.shape else {
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
    use amigo_light_2d_plugin::{
        LightGroup2dCommand, LightGroup2dSourceCommand, LightGroup2dSourceKind,
        LightReceiver2dBinding, LightReceiverDarkPolicy2d, LightReceiverGlobalLight2d,
        LightSampleStrategy2d, Material2dLightingMode,
    };

    fn particle_with_binding(binding: LightReceiver2dBinding) -> Particle2dDrawCommand {
        Particle2dDrawCommand {
            emitter_entity_name: "rain".to_owned(),
            previous_position: Vec2::ZERO,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            size: 1.0,
            color: ColorRgba::new(1.0, 1.0, 1.0, 0.25),
            render_layer: "default".to_owned(),
            z_index: 0.0,
            shape: ParticleShape2d::Line { length: 8.0 },
            line_anchor: ParticleLineAnchor2d::Center,
            blend_mode: ParticleBlendMode2d::Screen,
            motion_stretch: None,
            material: amigo_particles_2d_plugin::ParticleMaterial2d {
                lighting_mode: Material2dLightingMode::LightMapSampled,
                receives_light: true,
                light_response: 1.0,
                light_receiver: Some(binding),
            },
            light: None,
            light_position: None,
            transform: Transform2::default(),
        }
    }

    fn binding(channel: &str) -> LightReceiver2dBinding {
        LightReceiver2dBinding {
            groups: Vec::new(),
            source: "test-lightmap".to_owned(),
            channel: channel.to_owned(),
            sample_strategy: LightSampleStrategy2d::Point,
            sample_points: 1,
            radius_px: 0.0,
            exposure: 1.0,
            dark_policy: LightReceiverDarkPolicy2d::Transparent,
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

    fn light_group(id: &str, channel: &str) -> LightGroup2dCommand {
        LightGroup2dCommand {
            source_mod: "test-mod".to_owned(),
            id: id.to_owned(),
            label: None,
            color: ColorRgba::WHITE,
            intensity: 1.0,
            render_contributions: amigo_render_api::RenderContributionSet::default(),
            camera_response: amigo_camera_optics_plugin::api::CameraOpticalResponse2d::default(),
            sources: vec![LightGroup2dSourceCommand {
                kind: LightGroup2dSourceKind::LightMapChannel {
                    source: "test-lightmap".to_owned(),
                    channel: channel.to_owned(),
                },
                response: 1.0,
            }],
        }
    }

    fn light_group_particle(groups: Vec<String>) -> Particle2dDrawCommand {
        let mut receiver = binding("near");
        receiver.groups = groups;
        let mut particle = particle_with_binding(receiver);
        particle.material.lighting_mode = Material2dLightingMode::LightGroupSampled;
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
            &[],
        );
        let far = lit_particle_color(
            &particle_with_binding(binding("far")),
            &[],
            &[sampler()],
            &[],
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
            &[],
        );

        assert_eq!(color, ColorRgba::new(0.0, 0.0, 0.0, 0.25));
    }

    #[test]
    fn base_color_dark_policy_preserves_particle_color_in_darkness() {
        let mut binding = binding("near");
        binding.dark_policy = LightReceiverDarkPolicy2d::BaseColor;

        let color = lit_particle_color(&particle_with_binding(binding), &[], &[], &[], &[], &[]);

        assert_eq!(color, ColorRgba::new(1.0, 1.0, 1.0, 0.25));
    }

    #[test]
    fn shadow_tint_dark_policy_keeps_dim_particle_color_in_darkness() {
        let mut binding = binding("near");
        binding.dark_policy = LightReceiverDarkPolicy2d::ShadowTint;

        let color = lit_particle_color(&particle_with_binding(binding), &[], &[], &[], &[], &[]);

        assert_eq!(color, ColorRgba::new(0.18, 0.18, 0.18, 0.25));
    }

    #[test]
    fn receives_light_false_returns_base_particle_color_even_with_light_binding() {
        let mut particle = particle_with_binding(binding("near"));
        particle.material.receives_light = false;

        let color = lit_particle_color(&particle, &[], &[], &[], &[], &[]);

        assert_eq!(color, ColorRgba::new(1.0, 1.0, 1.0, 0.25));
    }

    #[test]
    fn global_light_can_light_particle_without_lightmap_sample() {
        let mut binding = binding("near");
        binding.global_lights = vec![LightReceiverGlobalLight2d {
            id: "lightning".to_owned(),
            response: 1.0,
        }];
        let color = lit_particle_color(
            &particle_with_binding(binding),
            &[],
            &[],
            &[GlobalLight2dCommand {
                source_mod: "test-mod".to_owned(),
                entity_name: "storm-controller".to_owned(),
                id: "lightning".to_owned(),
                color: ColorRgba::new(0.5, 0.75, 1.0, 1.0),
                intensity: 1.0,
            }],
            &[],
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
            &[],
            &[light_group("bar", "near")],
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
            &[],
            &[light_group("bar", "near")],
            &[LightRoute2dCommand {
                source_mod: "test-mod".to_owned(),
                receiver_layer: "default".to_owned(),
                groups: vec!["skyline".to_owned()],
            }],
        );

        assert_eq!(color, ColorRgba::new(0.0, 0.0, 0.0, 0.25));
    }
}
