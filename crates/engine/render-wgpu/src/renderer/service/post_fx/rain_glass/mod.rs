mod pipelines;
mod resources;
mod shaders;
mod simulation;
mod types;

use std::time::Instant;

use amigo_composite_plugin::{RainGlass2d, RainGlassRaindropCompose};
use amigo_core::AmigoResult;
use wgpu::util::DeviceExt;

use crate::WgpuOffscreenTarget;
use crate::renderer::service::{WgpuFrameRenderRequest, WgpuSceneRenderer};

use self::pipelines::RainGlassPipelines;
use self::resources::{RainGlassRenderTarget, RainGlassResources};
use self::simulation::RainGlassSimulation;
use self::types::{RainGlassInstance, RainGlassUniform, bytes_of};

pub(crate) struct RainGlassRenderRuntime {
    simulation: RainGlassSimulation,
    resources: RainGlassResources,
    pipelines: RainGlassPipelines,
    last_frame: Option<Instant>,
    last_key: Option<RainGlassStateKey>,
    uniform_buffer: wgpu::Buffer,
    blur_h_buffer: wgpu::Buffer,
    blur_v_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
    live_instance_count: u32,
    trail_instance_count: u32,
    persistent_instance_count: u32,
    sampler: wgpu::Sampler,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct RainGlassStateKey {
    width: u32,
    height: u32,
    quality_scale_milli: u32,
    seed: u32,
    format: wgpu::TextureFormat,
}

impl RainGlassRenderRuntime {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let resources = RainGlassResources::new(device, 1, 1, format, 1.0);
        let pipelines = RainGlassPipelines::new(device, format);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("amigo-rain-glass-uniform-buffer"),
            contents: bytes_of(&RainGlassUniform::new(
                RainGlass2d::default(),
                1,
                1,
                1.0 / 60.0,
            )),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let blur_h_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("amigo-rain-glass-blur-h-buffer"),
            contents: bytes_of(&[1.0f32, 0.0, 0.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let blur_v_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("amigo-rain-glass-blur-v-buffer"),
            contents: bytes_of(&[0.0f32, 1.0, 0.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let instance_capacity = 1;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("amigo-rain-glass-instance-buffer"),
            size: std::mem::size_of::<RainGlassInstance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("amigo-rain-glass-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        Self {
            simulation: RainGlassSimulation::new(1, 1.0, 1.0),
            resources,
            pipelines,
            last_frame: None,
            last_key: None,
            uniform_buffer,
            blur_h_buffer,
            blur_v_buffer,
            instance_buffer,
            instance_capacity,
            live_instance_count: 0,
            trail_instance_count: 0,
            persistent_instance_count: 0,
            sampler,
        }
    }

    pub(crate) fn execute(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        cfg: RainGlass2d,
        input_view: &wgpu::TextureView,
        normal_view: Option<&wgpu::TextureView>,
        wetness_view: Option<&wgpu::TextureView>,
        highlight_view: Option<&wgpu::TextureView>,
        emissive_view: Option<&wgpu::TextureView>,
        output_view: &wgpu::TextureView,
    ) -> AmigoResult<()> {
        let width = width.max(1);
        let height = height.max(1);
        self.ensure_state(device, cfg, width, height, scene_format);
        let dt = self.tick_dt();
        self.simulation.update(cfg, dt, width as f32, height as f32);
        self.write_uniforms(queue, cfg, width, height, dt);
        let (trail_start, persistent_start) = self.write_instances(device, queue, cfg);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("amigo-rain-glass-encoder"),
        });
        self.pass_stamp_live_raindrops(device, &mut encoder, cfg);
        self.pass_fade_map(device, &mut encoder, MapKind::Droplet);
        self.pass_fade_map(device, &mut encoder, MapKind::Streak);
        self.pass_fade_map(device, &mut encoder, MapKind::Mist);
        self.pass_erase_map(device, &mut encoder, MapKind::Droplet);
        self.pass_erase_map(device, &mut encoder, MapKind::Streak);
        self.pass_erase_map(device, &mut encoder, MapKind::Mist);
        self.pass_stamp_live_trails(device, &mut encoder, trail_start, cfg);
        self.pass_stamp_persistent_droplets(device, &mut encoder, persistent_start, cfg);
        self.pass_mist_accumulate(device, &mut encoder);
        self.pass_blur_scene(device, &mut encoder, input_view, cfg);
        self.pass_compose(
            device,
            &mut encoder,
            input_view,
            normal_view,
            wetness_view,
            highlight_view,
            emissive_view,
            output_view,
        );
        queue.submit(Some(encoder.finish()));
        Ok(())
    }

    fn ensure_state(
        &mut self,
        device: &wgpu::Device,
        cfg: RainGlass2d,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) {
        let key = RainGlassStateKey {
            width,
            height,
            quality_scale_milli: (cfg.quality_scale.clamp(0.35, 1.0) * 1000.0).round() as u32,
            seed: cfg.seed,
            format,
        };
        self.resources
            .ensure(device, width, height, format, cfg.quality_scale);
        if self.last_key != Some(key) {
            self.simulation = RainGlassSimulation::new(cfg.seed, width as f32, height as f32);
            self.last_frame = None;
            self.last_key = Some(key);
        }
    }

    fn tick_dt(&mut self) -> f32 {
        let now = Instant::now();
        let dt = self
            .last_frame
            .map(|last| now.duration_since(last).as_secs_f32().clamp(0.001, 0.040))
            .unwrap_or(1.0 / 30.0);
        self.last_frame = Some(now);
        dt
    }

    fn write_uniforms(
        &self,
        queue: &wgpu::Queue,
        cfg: RainGlass2d,
        width: u32,
        height: u32,
        dt: f32,
    ) {
        let uniform = RainGlassUniform::new(cfg, width, height, dt);
        queue.write_buffer(&self.uniform_buffer, 0, bytes_of(&uniform));
    }

    fn write_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cfg: RainGlass2d,
    ) -> (u32, u32) {
        let live = self.simulation.live_instances(cfg);
        let trails = self.simulation.trail_instances(cfg);
        let persistent = self.simulation.persistent_instances(cfg);
        let total = live.len() + trails.len() + persistent.len();
        if total > self.instance_capacity {
            self.instance_capacity = total.next_power_of_two().max(1);
            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("amigo-rain-glass-instance-buffer"),
                size: (self.instance_capacity * std::mem::size_of::<RainGlassInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        let mut all = live;
        let trail_start = all.len() as u32;
        all.extend(trails);
        let persistent_start = all.len() as u32;
        all.extend(persistent);
        self.live_instance_count = trail_start;
        self.trail_instance_count = persistent_start - trail_start;
        self.persistent_instance_count = all.len() as u32 - persistent_start;
        if !all.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, RainGlassInstance::bytes(&all));
        }
        (trail_start, persistent_start)
    }

    fn uniform_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-rain-glass-uniform-bind-group"),
            layout: &self.pipelines.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.as_entire_binding(),
            }],
        })
    }

    fn map_bind_group(&self, device: &wgpu::Device, source: &wgpu::TextureView) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-rain-glass-map-bind-group"),
            layout: &self.pipelines.map_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    fn erase_bind_group(
        &self,
        device: &wgpu::Device,
        source: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-rain-glass-erase-bind-group"),
            layout: &self.pipelines.erase_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.resources.raindrop_map.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    fn stamp_pipeline(&self, cfg: RainGlass2d) -> &wgpu::RenderPipeline {
        match cfg.raindrop_compose {
            RainGlassRaindropCompose::Smoother => &self.pipelines.stamp_smoother_pipeline,
            RainGlassRaindropCompose::Harder => &self.pipelines.stamp_harder_pipeline,
        }
    }

    fn pass_stamp_live_raindrops(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        cfg: RainGlass2d,
    ) {
        let uniform = self.uniform_bind_group(device);
        let mut pass = self.render_pass(
            encoder,
            &self.resources.raindrop_map,
            "amigo-rain-glass-stamp-live",
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        );
        pass.set_pipeline(self.stamp_pipeline(cfg));
        pass.set_bind_group(0, &uniform, &[]);
        pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        pass.draw(0..6, 0..self.live_instance_count);
    }

    fn pass_stamp_persistent_droplets(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        persistent_start: u32,
        cfg: RainGlass2d,
    ) {
        if self.persistent_instance_count == 0 {
            return;
        }
        let target = self.resources.droplet_map.front();
        let uniform = self.uniform_bind_group(device);
        let mut pass = self.render_pass(
            encoder,
            target,
            "amigo-rain-glass-stamp-persistent",
            wgpu::LoadOp::Load,
        );
        pass.set_pipeline(self.stamp_pipeline(cfg));
        pass.set_bind_group(0, &uniform, &[]);
        pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        pass.draw(
            0..6,
            persistent_start..persistent_start + self.persistent_instance_count,
        );
    }

    fn pass_stamp_live_trails(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        trail_start: u32,
        cfg: RainGlass2d,
    ) {
        let uniform = self.uniform_bind_group(device);
        let target = self.resources.streak_map.front();
        let mut pass = self.render_pass(
            encoder,
            target,
            "amigo-rain-glass-stamp-live-trails",
            wgpu::LoadOp::Load,
        );
        pass.set_pipeline(self.stamp_pipeline(cfg));
        pass.set_bind_group(0, &uniform, &[]);
        pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        pass.draw(0..6, trail_start..trail_start + self.trail_instance_count);
    }

    fn pass_fade_map(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        kind: MapKind,
    ) {
        let (source_view, target_view) = match kind {
            MapKind::Droplet => (
                &self.resources.droplet_map.front().view,
                &self.resources.droplet_map.back().view,
            ),
            MapKind::Streak => (
                &self.resources.streak_map.front().view,
                &self.resources.streak_map.back().view,
            ),
            MapKind::Mist => (
                &self.resources.mist_map.front().view,
                &self.resources.mist_map.back().view,
            ),
        };
        self.pass_map_to_target(
            device,
            encoder,
            source_view,
            target_view,
            &self.pipelines.fade_pipeline,
            "amigo-rain-glass-fade",
        );
        match kind {
            MapKind::Droplet => self.resources.droplet_map.swap(),
            MapKind::Streak => self.resources.streak_map.swap(),
            MapKind::Mist => self.resources.mist_map.swap(),
        }
    }

    fn pass_mist_accumulate(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let target = &self.resources.mist_map.back().view;
        let maps = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-rain-glass-mist-bind-group"),
            layout: &self.pipelines.mist_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.resources.mist_map.front().view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.resources.raindrop_map.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &self.resources.droplet_map.front().view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &self.resources.streak_map.front().view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let uniform = self.uniform_bind_group(device);
        {
            let mut pass = fullscreen_pass(encoder, target, "amigo-rain-glass-mist");
            pass.set_pipeline(&self.pipelines.mist_pipeline);
            pass.set_bind_group(0, &maps, &[]);
            pass.set_bind_group(1, &uniform, &[]);
            pass.draw(0..6, 0..1);
        }
        self.resources.mist_map.swap();
    }

    fn pass_erase_map(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        kind: MapKind,
    ) {
        let (source_view, target_view) = match kind {
            MapKind::Droplet => (
                &self.resources.droplet_map.front().view,
                &self.resources.droplet_map.back().view,
            ),
            MapKind::Streak => (
                &self.resources.streak_map.front().view,
                &self.resources.streak_map.back().view,
            ),
            MapKind::Mist => (
                &self.resources.mist_map.front().view,
                &self.resources.mist_map.back().view,
            ),
        };
        {
            let map = self.erase_bind_group(device, source_view);
            let uniform = self.uniform_bind_group(device);
            let mut pass = fullscreen_pass(encoder, target_view, "amigo-rain-glass-erase");
            pass.set_pipeline(&self.pipelines.erase_pipeline);
            pass.set_bind_group(0, &map, &[]);
            pass.set_bind_group(1, &uniform, &[]);
            pass.draw(0..6, 0..1);
        }
        match kind {
            MapKind::Droplet => self.resources.droplet_map.swap(),
            MapKind::Streak => self.resources.streak_map.swap(),
            MapKind::Mist => self.resources.mist_map.swap(),
        }
    }

    fn pass_blur_scene(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        cfg: RainGlass2d,
    ) {
        let steps = cfg
            .background_blur_steps
            .max(1)
            .min(self.resources.blur_pyramid.len() as u32 + 1);
        self.pass_blur(
            device,
            encoder,
            input_view,
            &self.resources.blurred_scene_a.view,
            &self.blur_h_buffer,
        );
        self.pass_blur(
            device,
            encoder,
            &self.resources.blurred_scene_a.view,
            &self.resources.blurred_scene_b.view,
            &self.blur_v_buffer,
        );

        if steps <= 1 {
            return;
        }

        let down_levels = (steps - 1) as usize;
        let mut source = &self.resources.blurred_scene_b.view;
        for level in self.resources.blur_pyramid.iter().take(down_levels) {
            self.pass_blur(device, encoder, source, &level.a.view, &self.blur_h_buffer);
            self.pass_blur(
                device,
                encoder,
                &level.a.view,
                &level.b.view,
                &self.blur_v_buffer,
            );
            source = &level.b.view;
        }

        // Upsample the smallest blurred level back into the full-size target used by compose.
        self.pass_blur(
            device,
            encoder,
            source,
            &self.resources.blurred_scene_a.view,
            &self.blur_h_buffer,
        );
        self.pass_blur(
            device,
            encoder,
            &self.resources.blurred_scene_a.view,
            &self.resources.blurred_scene_b.view,
            &self.blur_v_buffer,
        );
    }

    fn pass_blur(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        direction: &wgpu::Buffer,
    ) {
        let map = self.map_bind_group(device, source);
        let uniform = self.uniform_bind_group(device);
        let direction_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-rain-glass-blur-direction-bind-group"),
            layout: &self.pipelines.blur_direction_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: direction.as_entire_binding(),
            }],
        });
        let mut pass = fullscreen_pass(encoder, target, "amigo-rain-glass-blur");
        pass.set_pipeline(&self.pipelines.blur_pipeline);
        pass.set_bind_group(0, &map, &[]);
        pass.set_bind_group(1, &uniform, &[]);
        pass.set_bind_group(2, &direction_group, &[]);
        pass.draw(0..6, 0..1);
    }

    fn pass_compose(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        normal_view: Option<&wgpu::TextureView>,
        wetness_view: Option<&wgpu::TextureView>,
        highlight_view: Option<&wgpu::TextureView>,
        emissive_view: Option<&wgpu::TextureView>,
        output_view: &wgpu::TextureView,
    ) {
        let textures = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-rain-glass-compose-bind-group"),
            layout: &self.pipelines.compose_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self.resources.blurred_scene_b.view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.resources.raindrop_map.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &self.resources.droplet_map.front().view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        &self.resources.streak_map.front().view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(
                        &self.resources.mist_map.front().view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(normal_view.unwrap_or(input_view)),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(
                        wetness_view.unwrap_or(input_view),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(
                        highlight_view.unwrap_or(input_view),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(
                        emissive_view.unwrap_or(input_view),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let uniform = self.uniform_bind_group(device);
        let mut pass = fullscreen_pass(encoder, output_view, "amigo-rain-glass-compose");
        pass.set_pipeline(&self.pipelines.compose_pipeline);
        pass.set_bind_group(0, &textures, &[]);
        pass.set_bind_group(1, &uniform, &[]);
        pass.draw(0..6, 0..1);
    }

    fn pass_map_to_target(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        pipeline: &wgpu::RenderPipeline,
        label: &str,
    ) {
        let map = self.map_bind_group(device, source);
        let uniform = self.uniform_bind_group(device);
        let mut pass = fullscreen_pass(encoder, target, label);
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &map, &[]);
        pass.set_bind_group(1, &uniform, &[]);
        pass.draw(0..6, 0..1);
    }

    fn render_pass<'a>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        target: &'a RainGlassRenderTarget,
        label: &'a str,
        load: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        })
    }
}

#[derive(Clone, Copy)]
enum MapKind {
    Droplet,
    Streak,
    Mist,
}

fn fullscreen_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    target: &'a wgpu::TextureView,
    label: &'a str,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    })
}

pub(crate) fn execute_rain_glass(
    renderer: &mut WgpuSceneRenderer,
    _request: &WgpuFrameRenderRequest<'_>,
    host_id: &amigo_composite_plugin::PostFxHost2dId,
    effect_id: &amigo_composite_plugin::PostFx2dId,
    rain: RainGlass2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let rain = rain.normalized();
    if !rain.is_active() {
        return renderer.copy_offscreen_to_offscreen(output, input_view);
    }

    let key = super::runtime_key::PostFxRuntimeKey::new(host_id, effect_id);
    let runtime = renderer
        .rain_glass_runtimes
        .entry(key)
        .or_insert_with(|| RainGlassRenderRuntime::new(&output.device, output.format));
    let normal_view = renderer
        .visual_source_targets_2d
        .scene_normal
        .as_ref()
        .map(|target| target.view.clone());
    let wetness_view = renderer
        .visual_source_targets_2d
        .scene_wetness
        .as_ref()
        .map(|target| target.view.clone());
    let highlight_view = renderer
        .visual_source_targets_2d
        .scene_highlight
        .as_ref()
        .map(|target| target.view.clone());
    let emissive_view = renderer
        .visual_source_targets_2d
        .scene_emissive
        .as_ref()
        .map(|target| target.view.clone());

    runtime.execute(
        &output.device,
        &output.queue,
        output.format,
        output.width.max(1),
        output.height.max(1),
        rain,
        input_view,
        normal_view.as_ref(),
        wetness_view.as_ref(),
        highlight_view.as_ref(),
        emissive_view.as_ref(),
        &output.view,
    )
}
