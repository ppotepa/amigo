use crate::renderer::*;
use amigo_2d_lighting_beacon::BeaconLight2dDrawCommand;
use amigo_2d_text::Text2dDrawCommand;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
enum WorldLayerFilter<'a> {
    All,
    Include {
        layers: &'a BTreeSet<String>,
        include_layerless: bool,
    },
    Exclude(&'a BTreeSet<String>),
}

impl WorldLayerFilter<'_> {
    fn allows(self, render_layer: &str) -> bool {
        match self {
            Self::All => true,
            Self::Include { layers, .. } => layers.contains(render_layer),
            Self::Exclude(layers) => !layers.contains(render_layer),
        }
    }

    fn allows_layerless(self) -> bool {
        match self {
            Self::All | Self::Exclude(_) => true,
            Self::Include {
                include_layerless, ..
            } => include_layerless,
        }
    }
}

#[derive(Clone, Copy)]
enum WorldPassLoad {
    Clear,
    ClearTransparent,
    Load,
}

#[derive(Debug, Clone, PartialEq)]
struct FocusBlurPlaneLayer {
    layer_id: String,
    value: f32,
    blur_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct FocusBlurLayerPlan {
    depth_map_layers: BTreeSet<String>,
    plane_layers: Vec<FocusBlurPlaneLayer>,
    overlay_layers: BTreeSet<String>,
    legacy_affected_layers: Option<BTreeSet<String>>,
    has_explicit_render_depth: bool,
}

impl WgpuSceneRenderer {
    #[allow(clippy::too_many_arguments)]
    fn append_text2d_draw_command(
        &mut self,
        ui_texture_batches: &mut Vec<TextureBatch>,
        color_batches: &mut Vec<ColorBatch>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera2d: Transform2,
        scene: &SceneService,
        command: &Text2dDrawCommand,
    ) {
        let transform = resolve_transform2(scene, &command.entity_name, command.text.transform);
        let style = command.text.style;

        if let Some(glow) = style.glow {
            let passes = glow.passes.max(1);
            let step = glow.radius.max(0.0) / passes as f32;
            for pass in 1..=passes {
                let radius = pass as f32 * step;
                let alpha = glow.intensity.max(0.0) / pass as f32;
                for (dx, dy) in text2d_effect_offsets(radius) {
                    let glow_transform = translated_transform2(transform, Vec2::new(dx, dy));
                    let color = color_with_alpha_mul(glow.color, alpha);
                    let _ = self.append_text2d_ttf_font_texture_batch(
                        ui_texture_batches,
                        device,
                        queue,
                        assets,
                        viewport,
                        camera2d,
                        &command.text.font,
                        &command.text.content,
                        glow_transform,
                        command.text.bounds,
                        style.font_size,
                        color,
                    );
                }
            }
        }

        if let Some(outline) = style.outline {
            let width = outline.width.max(0.0);
            if width > 0.0 {
                for (dx, dy) in text2d_effect_offsets(width) {
                    let outline_transform = translated_transform2(transform, Vec2::new(dx, dy));
                    let _ = self.append_text2d_ttf_font_texture_batch(
                        ui_texture_batches,
                        device,
                        queue,
                        assets,
                        viewport,
                        camera2d,
                        &command.text.font,
                        &command.text.content,
                        outline_transform,
                        command.text.bounds,
                        style.font_size,
                        outline.color,
                    );
                }
            }
        }

        if let Some(shadow) = style.shadow {
            let shadow_transform = translated_transform2(transform, shadow.offset);
            let _ = self.append_text2d_ttf_font_texture_batch(
                ui_texture_batches,
                device,
                queue,
                assets,
                viewport,
                camera2d,
                &command.text.font,
                &command.text.content,
                shadow_transform,
                command.text.bounds,
                style.font_size,
                shadow.color,
            );
        }

        if self.append_text2d_ttf_font_texture_batch(
            ui_texture_batches,
            device,
            queue,
            assets,
            viewport,
            camera2d,
            &command.text.font,
            &command.text.content,
            transform,
            command.text.bounds,
            style.font_size,
            style.color,
        ) {
            return;
        }

        let vertices = color_batch_vertices(color_batches, ParticleBlendMode2d::Alpha);
        append_text_2d_vertices(
            vertices,
            viewport,
            camera2d,
            &command.text.content,
            transform,
            command.text.bounds,
            style.color,
        );
    }

    pub fn render_frame_request(&mut self, request: WgpuFrameRenderRequest<'_>) -> AmigoResult<()> {
        let mut executor = std::mem::take(&mut self.frame_graph_executor);
        let result = executor.execute(self, request);
        self.frame_graph_executor = executor;
        result
    }

    fn first_read(
        &self,
        node: &amigo_render_api::FrameGraphNode,
        name: &str,
    ) -> AmigoResult<amigo_render_api::FrameResourceId> {
        node.reads.first().copied().ok_or_else(|| {
            amigo_core::AmigoError::Message(format!("{name} graph node is missing a read target"))
        })
    }

    fn first_write(
        &self,
        node: &amigo_render_api::FrameGraphNode,
        name: &str,
    ) -> AmigoResult<amigo_render_api::FrameResourceId> {
        node.writes.first().copied().ok_or_else(|| {
            amigo_core::AmigoError::Message(format!("{name} graph node is missing a write target"))
        })
    }

    pub(crate) fn execute_world_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
    ) -> AmigoResult<()> {
        let write_id = self.first_write(node, "world")?;

        let target = resources.target_mut(write_id).ok_or_else(|| {
            amigo_core::AmigoError::Message("world node missing render target".into())
        })?;

        let plan = focus_blur_layer_plan(request.post_fx_stacks, request.world_2d.render_layers);
        let layer_filter = if let Some(plan) = plan.as_ref() {
            if plan.has_explicit_render_depth {
                WorldLayerFilter::Include {
                    layers: &plan.depth_map_layers,
                    include_layerless: true,
                }
            } else if let Some(layers) = plan.legacy_affected_layers.as_ref() {
                WorldLayerFilter::Include {
                    layers,
                    include_layerless: false,
                }
            } else {
                WorldLayerFilter::All
            }
        } else {
            WorldLayerFilter::All
        };

        self.execute_world_to_offscreen(
            target,
            request.scene,
            request.assets,
            request.world_2d.tilemaps,
            request.world_2d.sprites,
            request.world_2d.layered_images,
            request.world_2d.global_lights,
            request.world_2d.lightmaps,
            request.world_2d.text2d,
            request.world_2d.vectors,
            request.world_2d.beacons,
            request.world_3d.meshes,
            request.world_3d.materials,
            request.world_3d.text3d,
            request.world_2d.render_layers,
            request.world_2d.light_routes,
            request.world_2d.light_groups,
            request.world_2d.particles,
            &[],
            request.active_camera_2d_entity,
            layer_filter,
            WorldPassLoad::Clear,
        )
    }

    pub(crate) fn execute_post_fx_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        host_id: &amigo_2d_post_fx::PostFxHost2dId,
        effect_id: &amigo_2d_post_fx::PostFx2dId,
        scope: &amigo_2d_post_fx::PostFxScope2d,
        pipeline: amigo_2d_post_fx::PostFxPipelineKind,
        feature_id: amigo_render_api::RenderFeatureId,
        resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
    ) -> AmigoResult<()> {
        let read = self.first_read(node, "post-fx")?;
        let write = self.first_write(node, "post-fx")?;

        let source = resources
            .target(read)
            .ok_or_else(|| {
                amigo_core::AmigoError::Message("post-fx read target unavailable".into())
            })?
            .view
            .clone();
        let target = resources.target_mut(write).ok_or_else(|| {
            amigo_core::AmigoError::Message("post-fx write target unavailable".into())
        })?;

        crate::renderer::service::post_fx::execute_screen_space_post_fx(
            self,
            request,
            host_id,
            effect_id,
            scope,
            pipeline,
            &feature_id,
            &source,
            target,
        )?;

        let Some(plan) = focus_blur_layer_plan_for_effect(
            request.post_fx_stacks,
            request.world_2d.render_layers,
            host_id,
            effect_id,
        ) else {
            return Ok(());
        };

        if plan.has_explicit_render_depth {
            for plane in &plan.plane_layers {
                let plane_layers = BTreeSet::from([plane.layer_id.clone()]);
                let mut plane_source =
                    compatible_offscreen_target(target, "amigo-focus-blur-plane-source");
                let mut plane_blurred =
                    compatible_offscreen_target(target, "amigo-focus-blur-plane-blurred");
                self.execute_world_to_offscreen(
                    &mut plane_source,
                    request.scene,
                    request.assets,
                    request.world_2d.tilemaps,
                    request.world_2d.sprites,
                    request.world_2d.layered_images,
                    request.world_2d.global_lights,
                    request.world_2d.lightmaps,
                    request.world_2d.text2d,
                    request.world_2d.vectors,
                    request.world_2d.beacons,
                    request.world_3d.meshes,
                    request.world_3d.materials,
                    request.world_3d.text3d,
                    request.world_2d.render_layers,
                    request.world_2d.light_routes,
                    request.world_2d.light_groups,
                    request.world_2d.particles,
                    &[],
                    request.active_camera_2d_entity,
                    WorldLayerFilter::Include {
                        layers: &plane_layers,
                        include_layerless: false,
                    },
                    WorldPassLoad::ClearTransparent,
                )?;
                crate::renderer::service::post_fx::focus_blur::execute_focus_blur_with_depth_source(
                    self,
                    request,
                    focus_blur_effect_for(request.post_fx_stacks, host_id, effect_id)
                        .expect("focus blur effect should exist for explicit layer plan"),
                    &plane_source.view,
                    &mut plane_blurred,
                    crate::renderer::service::post_fx::focus_blur::FocusBlurDepthSource::Plane {
                        value: plane.value,
                        blur_scale: plane.blur_scale,
                    },
                )?;
                self.composite_offscreen_over_offscreen(target, &plane_blurred.view)?;
            }

            if !plan.overlay_layers.is_empty() {
                return self.execute_world_to_offscreen(
                    target,
                    request.scene,
                    request.assets,
                    request.world_2d.tilemaps,
                    request.world_2d.sprites,
                    request.world_2d.layered_images,
                    request.world_2d.global_lights,
                    request.world_2d.lightmaps,
                    request.world_2d.text2d,
                    request.world_2d.vectors,
                    request.world_2d.beacons,
                    request.world_3d.meshes,
                    request.world_3d.materials,
                    request.world_3d.text3d,
                    request.world_2d.render_layers,
                    request.world_2d.light_routes,
                    request.world_2d.light_groups,
                    request.world_2d.particles,
                    &[],
                    request.active_camera_2d_entity,
                    WorldLayerFilter::Include {
                        layers: &plan.overlay_layers,
                        include_layerless: false,
                    },
                    WorldPassLoad::Load,
                );
            }

            return Ok(());
        }

        let Some(affected_layers) = plan.legacy_affected_layers.as_ref() else {
            return Ok(());
        };

        self.execute_world_to_offscreen(
            target,
            request.scene,
            request.assets,
            request.world_2d.tilemaps,
            request.world_2d.sprites,
            request.world_2d.layered_images,
            request.world_2d.global_lights,
            request.world_2d.lightmaps,
            request.world_2d.text2d,
            request.world_2d.vectors,
            request.world_2d.beacons,
            request.world_3d.meshes,
            request.world_3d.materials,
            request.world_3d.text3d,
            request.world_2d.render_layers,
            request.world_2d.light_routes,
            request.world_2d.light_groups,
            request.world_2d.particles,
            &[],
            request.active_camera_2d_entity,
            WorldLayerFilter::Exclude(&affected_layers),
            WorldPassLoad::Load,
        )
    }

    pub(crate) fn execute_game_ui_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
    ) -> AmigoResult<()> {
        let read = self.first_read(node, "game-ui")?;
        let write = self.first_write(node, "game-ui")?;

        if read != write {
            let source = resources
                .target(read)
                .ok_or_else(|| {
                    amigo_core::AmigoError::Message("game-ui read target unavailable".into())
                })?
                .view
                .clone();
            let target = resources.target_mut(write).ok_or_else(|| {
                amigo_core::AmigoError::Message("game-ui write target unavailable".into())
            })?;
            self.copy_offscreen_to_offscreen(target, &source)?;
        }

        let target = resources.target_mut(write).ok_or_else(|| {
            amigo_core::AmigoError::Message("game-ui write target unavailable".into())
        })?;
        self.render_ui_documents_to_offscreen(
            target,
            request.assets,
            request.game_ui,
            wgpu::LoadOp::Load,
        )
    }

    pub(crate) fn execute_debug_overlay_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
    ) -> AmigoResult<()> {
        let read = self.first_read(node, "debug-overlay")?;
        let write = self.first_write(node, "debug-overlay")?;

        if read != write {
            let source = resources
                .target(read)
                .ok_or_else(|| {
                    amigo_core::AmigoError::Message("debug-overlay read target unavailable".into())
                })?
                .view
                .clone();
            let target = resources.target_mut(write).ok_or_else(|| {
                amigo_core::AmigoError::Message("debug-overlay write target unavailable".into())
            })?;
            self.copy_offscreen_to_offscreen(target, &source)?;
        }

        let target = resources.target_mut(write).ok_or_else(|| {
            amigo_core::AmigoError::Message("debug-overlay write target unavailable".into())
        })?;
        self.render_ui_documents_to_offscreen(
            target,
            request.assets,
            request.debug_ui,
            wgpu::LoadOp::Load,
        )
    }

    pub(crate) fn execute_present_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
    ) -> AmigoResult<()> {
        let read = node.reads.first().copied().ok_or_else(|| {
            amigo_core::AmigoError::Message("present node has no read target".into())
        })?;
        let source = resources
            .target(read)
            .ok_or_else(|| {
                amigo_core::AmigoError::Message("present read target unavailable".into())
            })?
            .view
            .clone();

        match &mut request.target {
            WgpuFrameRenderTarget::Surface(surface) => self.render_texture_to_surface(
                surface,
                &source,
                request.assets,
                request.debug_ui,
                request.game_viewport,
                &emergency_overlay_lines(request.emergency_overlay, &self.emergency_overlay_lines),
            ),
            WgpuFrameRenderTarget::Offscreen(target) => {
                self.copy_offscreen_to_offscreen(target, &source)?;
                self.render_emergency_overlay_to_offscreen(
                    target,
                    &emergency_overlay_lines(
                        request.emergency_overlay,
                        &self.emergency_overlay_lines,
                    ),
                )?;
                Ok(())
            }
        }
    }

    pub(crate) fn copy_offscreen_to_offscreen(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        source_view: &wgpu::TextureView,
    ) -> AmigoResult<()> {
        let mut world_batch = self.create_fullscreen_texture_batch(
            &target.device,
            source_view,
            TextureBlendMode::Alpha,
        );
        append_fullscreen_texture_vertices(&mut world_batch.vertices);
        let vertex_buffer = target
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("amigo-offscreen-copy-vertex-buffer"),
                contents: texture_vertices_as_bytes(&world_batch.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-copy-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-offscreen-copy-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            pass.set_pipeline(self.texture_pipeline_for(world_batch.blend_mode));
            pass.set_bind_group(0, &world_batch.bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..world_batch.vertices.len() as u32, 0..1);
        }
        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    pub(crate) fn composite_offscreen_over_offscreen(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        source_view: &wgpu::TextureView,
    ) -> AmigoResult<()> {
        let mut batch = self.create_fullscreen_texture_batch(
            &target.device,
            source_view,
            TextureBlendMode::Alpha,
        );
        append_fullscreen_texture_vertices(&mut batch.vertices);
        let vertex_buffer = target
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("amigo-offscreen-composite-vertex-buffer"),
                contents: texture_vertices_as_bytes(&batch.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-composite-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-offscreen-composite-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
            pass.set_bind_group(0, &batch.bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..batch.vertices.len() as u32, 0..1);
        }
        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    fn render_texture_to_surface(
        &mut self,
        surface: &mut WgpuSurfaceState,
        source_view: &wgpu::TextureView,
        assets: &AssetCatalog,
        surface_overlay_ui: &[UiOverlayDocument],
        game_viewport: Option<WgpuGameViewportPlacement>,
        emergency_overlay: &[WgpuEmergencyOverlayLine],
    ) -> AmigoResult<()> {
        let mut world_batch = self.create_fullscreen_texture_batch(
            &surface.device,
            source_view,
            TextureBlendMode::Alpha,
        );
        if let Some(placement) = game_viewport {
            append_surface_texture_rect_vertices(
                &mut world_batch.vertices,
                surface.config.width as f32,
                surface.config.height as f32,
                placement,
            );
        } else {
            append_fullscreen_texture_vertices(&mut world_batch.vertices);
        }

        let mut color_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();

        if !surface_overlay_ui.is_empty() {
            let (mut ui_color_batches, mut ui_textures) =
                self.surface_ui_batches(surface, assets, surface_overlay_ui);
            color_batches.append(&mut ui_color_batches);
            ui_texture_batches.append(&mut ui_textures);
        }

        let mut emergency_batch =
            self.emergency_overlay_color_batch_for_surface(surface, emergency_overlay);
        color_batches.append(&mut emergency_batch);

        self.render_surface_batches(
            surface,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            &[world_batch],
            color_batches.as_slice(),
            ui_texture_batches.as_slice(),
        )
    }

    pub fn present_cached_frame_to_surface(
        &mut self,
        surface: &mut WgpuSurfaceState,
        source: &WgpuOffscreenTarget,
        assets: &AssetCatalog,
        live_overlay_ui: &[UiOverlayDocument],
        game_viewport: Option<WgpuGameViewportPlacement>,
        emergency_overlay: &[WgpuEmergencyOverlayLine],
    ) -> AmigoResult<()> {
        self.render_texture_to_surface(
            surface,
            &source.view,
            assets,
            live_overlay_ui,
            game_viewport,
            emergency_overlay,
        )
    }

    fn surface_ui_batches(
        &mut self,
        surface: &WgpuSurfaceState,
        assets: &AssetCatalog,
        ui_documents: &[UiOverlayDocument],
    ) -> (Vec<ColorBatch>, Vec<TextureBatch>) {
        if ui_documents.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let viewport = Viewport::from_surface(surface);
        let ui_primitives = build_ui_overlay_primitives(
            UiViewportSize::new(surface.config.width as f32, surface.config.height as f32),
            ui_documents,
        );
        let mut color_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();
        let mut ui_color_primitives = Vec::with_capacity(ui_primitives.len());

        for primitive in &ui_primitives {
            if let UiDrawPrimitive::Text {
                rect,
                content,
                color,
                font_size,
                font: Some(font),
                anchor,
                word_wrap,
                fit_to_width,
            } = primitive
            {
                if self.append_ui_ttf_font_texture_batch(
                    &mut ui_texture_batches,
                    &surface.device,
                    &surface.queue,
                    assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }

                if self.append_ui_bitmap_font_texture_batch(
                    &mut ui_texture_batches,
                    &surface.device,
                    &surface.queue,
                    assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }
            }
            ui_color_primitives.push(primitive.clone());
        }

        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);

        color_batches.retain(|batch| !batch.vertices.is_empty());
        ui_texture_batches.retain(|batch| !batch.vertices.is_empty());
        (color_batches, ui_texture_batches)
    }

    pub(crate) fn record_emergency_error(&mut self, message: impl Into<String>) {
        push_emergency_overlay_line(
            &mut self.emergency_overlay_lines,
            WgpuEmergencyOverlayLevel::Error,
            message.into(),
        );
    }

    fn emergency_overlay_color_batch_for_surface(
        &self,
        surface: &WgpuSurfaceState,
        lines: &[WgpuEmergencyOverlayLine],
    ) -> Vec<ColorBatch> {
        let viewport = Viewport::from_surface(surface);
        emergency_overlay_color_batches(&viewport, surface.config.width, lines)
    }

    fn render_emergency_overlay_to_offscreen(
        &self,
        target: &mut WgpuOffscreenTarget,
        lines: &[WgpuEmergencyOverlayLine],
    ) -> AmigoResult<()> {
        let viewport = Viewport::from_offscreen(target);
        let color_batches = emergency_overlay_color_batches(&viewport, target.width, lines);
        if color_batches.iter().all(|batch| batch.vertices.is_empty()) {
            return Ok(());
        }

        let color_vertex_buffers = color_batches
            .iter()
            .map(|batch| {
                target
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-emergency-overlay-vertices"),
                        contents: vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-emergency-overlay-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-emergency-overlay-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for (index, batch) in color_batches.iter().enumerate() {
                pass.set_pipeline(self.color_pipeline_for(batch.blend_mode));
                pass.set_vertex_buffer(0, color_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    fn render_ui_documents_to_offscreen(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        assets: &AssetCatalog,
        ui_documents: &[UiOverlayDocument],
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> AmigoResult<()> {
        if ui_documents.is_empty() {
            return Ok(());
        }

        let viewport = Viewport::from_offscreen(target);
        let ui_primitives = build_ui_overlay_primitives(
            UiViewportSize::new(target.width as f32, target.height as f32),
            ui_documents,
        );
        let mut color_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();
        let mut ui_color_primitives = Vec::with_capacity(ui_primitives.len());

        for primitive in &ui_primitives {
            if let UiDrawPrimitive::Text {
                rect,
                content,
                color,
                font_size,
                font: Some(font),
                anchor,
                word_wrap,
                fit_to_width,
            } = primitive
            {
                if self.append_ui_ttf_font_texture_batch(
                    &mut ui_texture_batches,
                    &target.device,
                    &target.queue,
                    assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }

                if self.append_ui_bitmap_font_texture_batch(
                    &mut ui_texture_batches,
                    &target.device,
                    &target.queue,
                    assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }
            }
            ui_color_primitives.push(primitive.clone());
        }

        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);

        color_batches.retain(|batch| !batch.vertices.is_empty());
        ui_texture_batches.retain(|batch| !batch.vertices.is_empty());

        let color_vertex_buffers = color_batches
            .iter()
            .map(|batch| {
                target
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-offscreen-ui-color-vertices"),
                        contents: vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();
        let ui_texture_vertex_buffers = ui_texture_batches
            .iter()
            .map(|batch| {
                target
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-offscreen-ui-texture-vertices"),
                        contents: texture_vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-ui-render-encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-offscreen-ui-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for (index, batch) in color_batches.iter().enumerate() {
                pass.set_pipeline(self.color_pipeline_for(batch.blend_mode));
                pass.set_vertex_buffer(0, color_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in ui_texture_batches.iter().enumerate() {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, ui_texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    fn execute_world_to_offscreen(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        layered_images: &amigo_2d_layered_image::LayeredImageSceneService,
        global_lights: &GlobalLight2dSceneService,
        lightmaps: &LightMap2dSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        beacons: &[BeaconLight2dDrawCommand],
        meshes: &[MeshDrawCommand],
        materials: &[MaterialDrawCommand],
        text3d: Option<&[Text3dDrawCommand]>,
        render_layers: &[RenderLayer2dCommand],
        light_routes: &[LightRoute2dCommand],
        light_groups: &[LightGroup2dCommand],
        particles: &[Particle2dDrawCommand],
        ui_primitives: &[UiDrawPrimitive],
        active_camera_2d_entity: Option<&str>,
        layer_filter: WorldLayerFilter<'_>,
        pass_load: WorldPassLoad,
    ) -> AmigoResult<()> {
        let viewport = Viewport::from_offscreen(target);
        let mut color_batches = Vec::new();
        let mut texture_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();
        let camera2d = resolve_camera2d_transform(scene, active_camera_2d_entity);
        let particle_lights = particle_render_lights(particles);
        let layered_image_commands = layered_images.commands();
        let global_light_commands = global_lights.commands();
        let lightmap_sources = lightmaps.commands();
        let render_layer_lookup = render_layer_lookup(render_layers);
        let lightmap_samplers = self.lightmap_2d_samplers(
            assets,
            scene,
            &viewport,
            &layered_image_commands,
            &lightmap_sources,
        );
        let mut world2d_items = tilemaps
            .commands()
            .into_iter()
            .map(World2dItem::TileMap)
            .chain(
                layered_image_commands
                    .into_iter()
                    .map(World2dItem::LayeredImage),
            )
            .chain(vectors.commands().into_iter().map(World2dItem::Vector))
            .chain(beacons.iter().cloned().map(World2dItem::Beacon))
            .chain(sprites.commands().into_iter().map(World2dItem::Sprite))
            .chain(particles.iter().cloned().map(World2dItem::Particle))
            .collect::<Vec<_>>();
        world2d_items.retain(|item| layer_filter.allows(item.render_layer()));
        world2d_items.sort_by_key(|item| world2d_sort_key(item, &render_layer_lookup));

        for item in world2d_items {
            match item {
                World2dItem::TileMap(command) => {
                    let transform =
                        resolve_transform2(scene, &command.entity_name, Transform2::default());
                    if !self.append_tilemap_texture_batch(
                        &mut texture_batches,
                        &target.device,
                        &target.queue,
                        assets,
                        &viewport,
                        camera2d,
                        transform,
                        &command.tilemap,
                    ) {
                        let vertices =
                            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                        append_tilemap_fallback_vertices(
                            vertices,
                            &viewport,
                            camera2d,
                            transform,
                            &command.tilemap,
                        );
                    }
                }
                World2dItem::Sprite(command) => {
                    let transform =
                        resolve_transform2(scene, &command.entity_name, command.transform);
                    if !self.append_sprite_texture_batch(
                        &mut texture_batches,
                        &target.device,
                        &target.queue,
                        assets,
                        &viewport,
                        camera2d,
                        transform,
                        &command.sprite,
                    ) {
                        let vertices =
                            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                        append_sprite_vertices(
                            vertices,
                            &viewport,
                            camera2d,
                            transform,
                            &command.sprite,
                            sprite_color(command.sprite.texture.as_str()),
                        );
                    }
                }
                World2dItem::LayeredImage(command) => {
                    let transform =
                        resolve_transform2(scene, &command.entity_name, command.transform);
                    self.append_layered_image_texture_batches(
                        &mut texture_batches,
                        &target.device,
                        &target.queue,
                        assets,
                        &viewport,
                        camera2d,
                        transform,
                        &command,
                    );
                }
                World2dItem::Vector(command) => {
                    let transform = vector_viewport_fit_transform(
                        &viewport,
                        resolve_transform2(scene, &command.entity_name, command.transform),
                        command.viewport_fit,
                        command.viewport_canvas_size,
                    );
                    let vertices =
                        color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                    append_vector_shape_vertices(
                        vertices,
                        &viewport,
                        camera2d,
                        transform,
                        &command.shape,
                    );
                }
                World2dItem::Beacon(command) => {
                    let vertices =
                        color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Additive);
                    append_beacon_vfx_vertices(vertices, &viewport, camera2d, &command);
                }
                World2dItem::Particle(command) => {
                    if command.light.is_some_and(|light| {
                        light.glow && light.mode == ParticleLightMode2d::Particle
                    }) {
                        let vertices =
                            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Additive);
                        append_particle_light_vertices(vertices, &viewport, camera2d, &command);
                    }
                    let vertices = color_batch_vertices(&mut color_batches, command.blend_mode);
                    append_particle_vertices(
                        vertices,
                        &viewport,
                        camera2d,
                        &command,
                        &particle_lights,
                        &lightmap_samplers,
                        &global_light_commands,
                        light_groups,
                        light_routes,
                    );
                }
            }
        }

        for command in text2d.commands() {
            if !layer_filter.allows(&command.render_layer) {
                continue;
            }
            self.append_text2d_draw_command(
                &mut ui_texture_batches,
                &mut color_batches,
                &target.device,
                &target.queue,
                assets,
                &viewport,
                camera2d,
                scene,
                &command,
            );
        }

        let camera = resolve_camera_transform(scene);
        let material_lookup = material_lookup_from_commands(materials);
        let mut projected_triangles = Vec::new();

        if layer_filter.allows_layerless() {
            for command in meshes {
                let transform =
                    resolve_transform3(scene, &command.entity_name, command.mesh.transform);
                let color = material_lookup
                    .get(&command.entity_name)
                    .copied()
                    .unwrap_or_else(|| mesh_color(command.mesh.mesh_asset.as_str()));
                append_mesh_triangles(
                    &mut projected_triangles,
                    &viewport,
                    camera,
                    transform,
                    color,
                );
            }
        }

        projected_triangles.sort_by(|left, right| {
            right
                .depth
                .partial_cmp(&left.depth)
                .unwrap_or(Ordering::Equal)
        });

        for triangle in projected_triangles {
            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            push_triangle(vertices, triangle.points, triangle.color);
        }

        if let Some(text3d) = text3d.filter(|_| layer_filter.allows_layerless()) {
            for command in text3d {
                let transform =
                    resolve_transform3(scene, &command.entity_name, command.text.transform);
                if self.append_text3d_ttf_font_texture_batch(
                    &mut ui_texture_batches,
                    &target.device,
                    &target.queue,
                    assets,
                    &viewport,
                    camera,
                    &command.text.font,
                    &command.text.content,
                    transform,
                    command.text.size,
                    ColorRgba::new(0.94, 0.98, 1.0, 1.0),
                ) {
                    continue;
                }

                let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                append_text_3d_vertices(
                    vertices,
                    &viewport,
                    camera,
                    &command.text.content,
                    transform,
                    command.text.size,
                    ColorRgba::new(0.94, 0.98, 1.0, 1.0),
                );
            }
        }

        let mut ui_color_primitives = Vec::with_capacity(ui_primitives.len());
        if layer_filter.allows_layerless() {
            for primitive in ui_primitives {
                if let UiDrawPrimitive::Text {
                    rect,
                    content,
                    color,
                    font_size,
                    font: Some(font),
                    anchor,
                    word_wrap,
                    fit_to_width,
                } = primitive
                {
                    if self.append_ui_ttf_font_texture_batch(
                        &mut ui_texture_batches,
                        &target.device,
                        &target.queue,
                        assets,
                        &viewport,
                        font,
                        content,
                        *rect,
                        *font_size,
                        *color,
                        *anchor,
                        *word_wrap,
                        *fit_to_width,
                    ) {
                        continue;
                    }

                    if self.append_ui_bitmap_font_texture_batch(
                        &mut ui_texture_batches,
                        &target.device,
                        &target.queue,
                        assets,
                        &viewport,
                        font,
                        content,
                        *rect,
                        *font_size,
                        *color,
                        *anchor,
                        *word_wrap,
                        *fit_to_width,
                    ) {
                        continue;
                    }
                }
                ui_color_primitives.push(primitive.clone());
            }
        }

        {
            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);
        }

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-scene-render-encoder"),
            });

        color_batches.retain(|batch| !batch.vertices.is_empty());
        ui_texture_batches.retain(|batch| !batch.vertices.is_empty());
        let color_vertex_buffers = color_batches
            .iter()
            .map(|batch| {
                target
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-offscreen-scene-color-vertices"),
                        contents: vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();
        let texture_vertex_buffers = texture_batches
            .iter()
            .map(|batch| {
                target
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-offscreen-scene-texture-vertices"),
                        contents: texture_vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();
        let ui_texture_vertex_buffers = ui_texture_batches
            .iter()
            .map(|batch| {
                target
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-offscreen-scene-ui-texture-vertices"),
                        contents: texture_vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-offscreen-scene-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: match pass_load {
                            WorldPassLoad::Clear => wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            WorldPassLoad::ClearTransparent => wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            WorldPassLoad::Load => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for (index, batch) in texture_batches.iter().enumerate() {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in color_batches.iter().enumerate() {
                pass.set_pipeline(self.color_pipeline_for(batch.blend_mode));
                pass.set_vertex_buffer(0, color_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in ui_texture_batches.iter().enumerate() {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, ui_texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    fn create_fullscreen_texture_batch(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        blend_mode: TextureBlendMode,
    ) -> TextureBatch {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("amigo-post-fx-fullscreen-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-post-fx-fullscreen-bind-group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        TextureBatch {
            blend_mode,
            bind_group,
            _owned_sampler: Some(sampler),
            vertices: Vec::new(),
        }
    }

    fn render_surface_batches(
        &self,
        surface: &mut WgpuSurfaceState,
        load_op: wgpu::LoadOp<wgpu::Color>,
        texture_batches: &[TextureBatch],
        color_batches: &[ColorBatch],
        ui_texture_batches: &[TextureBatch],
    ) -> AmigoResult<()> {
        let texture_batches = texture_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();
        let color_batches = color_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();
        let ui_texture_batches = ui_texture_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();

        let frame = match surface.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(()),
            wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Validation
            | wgpu::CurrentSurfaceTexture::Occluded => {
                surface.surface.configure(&surface.device, &surface.config);
                return Ok(());
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = surface
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-scene-render-encoder"),
            });

        let color_vertex_buffers = color_batches
            .iter()
            .map(|batch| {
                surface
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-scene-color-vertices"),
                        contents: vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();
        let texture_vertex_buffers = texture_batches
            .iter()
            .map(|batch| {
                surface
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-scene-texture-vertices"),
                        contents: texture_vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();
        let ui_texture_vertex_buffers = ui_texture_batches
            .iter()
            .map(|batch| {
                surface
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-scene-ui-texture-vertices"),
                        contents: texture_vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-scene-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for (index, batch) in texture_batches.iter().enumerate() {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in color_batches.iter().enumerate() {
                pass.set_pipeline(self.color_pipeline_for(batch.blend_mode));
                pass.set_vertex_buffer(0, color_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in ui_texture_batches.iter().enumerate() {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, ui_texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        surface.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }

    #[allow(dead_code)]
    fn execute_world_to_surface(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        layered_images: &amigo_2d_layered_image::LayeredImageSceneService,
        global_lights: &GlobalLight2dSceneService,
        lightmaps: &LightMap2dSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        beacons: &[BeaconLight2dDrawCommand],
        meshes: &[MeshDrawCommand],
        materials: &[MaterialDrawCommand],
        text3d: Option<&[Text3dDrawCommand]>,
        render_layers: &[RenderLayer2dCommand],
        light_routes: &[LightRoute2dCommand],
        light_groups: &[LightGroup2dCommand],
        particles: &[Particle2dDrawCommand],
        ui_primitives: &[UiDrawPrimitive],
        active_camera_2d_entity: Option<&str>,
    ) -> AmigoResult<()> {
        let viewport = Viewport::from_surface(surface);
        let mut color_batches = Vec::new();
        let mut texture_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();
        let camera2d = resolve_camera2d_transform(scene, active_camera_2d_entity);
        let particle_lights = particle_render_lights(particles);
        let layered_image_commands = layered_images.commands();
        let global_light_commands = global_lights.commands();
        let lightmap_sources = lightmaps.commands();
        let render_layer_lookup = render_layer_lookup(render_layers);
        let lightmap_samplers = self.lightmap_2d_samplers(
            assets,
            scene,
            &viewport,
            &layered_image_commands,
            &lightmap_sources,
        );
        let mut world2d_items = tilemaps
            .commands()
            .into_iter()
            .map(World2dItem::TileMap)
            .chain(
                layered_image_commands
                    .into_iter()
                    .map(World2dItem::LayeredImage),
            )
            .chain(vectors.commands().into_iter().map(World2dItem::Vector))
            .chain(beacons.iter().cloned().map(World2dItem::Beacon))
            .chain(sprites.commands().into_iter().map(World2dItem::Sprite))
            .chain(particles.iter().cloned().map(World2dItem::Particle))
            .collect::<Vec<_>>();
        world2d_items.sort_by_key(|item| world2d_sort_key(item, &render_layer_lookup));

        for item in world2d_items {
            match item {
                World2dItem::TileMap(command) => {
                    let transform =
                        resolve_transform2(scene, &command.entity_name, Transform2::default());
                    if !self.append_tilemap_texture_batch(
                        &mut texture_batches,
                        &surface.device,
                        &surface.queue,
                        assets,
                        &viewport,
                        camera2d,
                        transform,
                        &command.tilemap,
                    ) {
                        let vertices =
                            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                        append_tilemap_fallback_vertices(
                            vertices,
                            &viewport,
                            camera2d,
                            transform,
                            &command.tilemap,
                        );
                    }
                }
                World2dItem::Sprite(command) => {
                    let transform =
                        resolve_transform2(scene, &command.entity_name, command.transform);
                    if !self.append_sprite_texture_batch(
                        &mut texture_batches,
                        &surface.device,
                        &surface.queue,
                        assets,
                        &viewport,
                        camera2d,
                        transform,
                        &command.sprite,
                    ) {
                        let vertices =
                            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                        append_sprite_vertices(
                            vertices,
                            &viewport,
                            camera2d,
                            transform,
                            &command.sprite,
                            sprite_color(command.sprite.texture.as_str()),
                        );
                    }
                }
                World2dItem::LayeredImage(command) => {
                    let transform =
                        resolve_transform2(scene, &command.entity_name, command.transform);
                    self.append_layered_image_texture_batches(
                        &mut texture_batches,
                        &surface.device,
                        &surface.queue,
                        assets,
                        &viewport,
                        camera2d,
                        transform,
                        &command,
                    );
                }
                World2dItem::Vector(command) => {
                    let transform = vector_viewport_fit_transform(
                        &viewport,
                        resolve_transform2(scene, &command.entity_name, command.transform),
                        command.viewport_fit,
                        command.viewport_canvas_size,
                    );
                    let vertices =
                        color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                    append_vector_shape_vertices(
                        vertices,
                        &viewport,
                        camera2d,
                        transform,
                        &command.shape,
                    );
                }
                World2dItem::Beacon(command) => {
                    let vertices =
                        color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Additive);
                    append_beacon_vfx_vertices(vertices, &viewport, camera2d, &command);
                }
                World2dItem::Particle(command) => {
                    if command.light.is_some_and(|light| {
                        light.glow && light.mode == ParticleLightMode2d::Particle
                    }) {
                        let vertices =
                            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Additive);
                        append_particle_light_vertices(vertices, &viewport, camera2d, &command);
                    }
                    let vertices = color_batch_vertices(&mut color_batches, command.blend_mode);
                    append_particle_vertices(
                        vertices,
                        &viewport,
                        camera2d,
                        &command,
                        &particle_lights,
                        &lightmap_samplers,
                        &global_light_commands,
                        light_groups,
                        light_routes,
                    );
                }
            }
        }

        for command in text2d.commands() {
            self.append_text2d_draw_command(
                &mut ui_texture_batches,
                &mut color_batches,
                &surface.device,
                &surface.queue,
                assets,
                &viewport,
                camera2d,
                scene,
                &command,
            );
        }

        let camera = resolve_camera_transform(scene);
        let material_lookup = material_lookup_from_commands(materials);
        let mut projected_triangles = Vec::new();

        for command in meshes {
            let transform = resolve_transform3(scene, &command.entity_name, command.mesh.transform);
            let color = material_lookup
                .get(&command.entity_name)
                .copied()
                .unwrap_or_else(|| mesh_color(command.mesh.mesh_asset.as_str()));
            append_mesh_triangles(
                &mut projected_triangles,
                &viewport,
                camera,
                transform,
                color,
            );
        }

        projected_triangles.sort_by(|left, right| {
            right
                .depth
                .partial_cmp(&left.depth)
                .unwrap_or(Ordering::Equal)
        });

        for triangle in projected_triangles {
            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            push_triangle(vertices, triangle.points, triangle.color);
        }

        if let Some(text3d) = text3d {
            for command in text3d {
                let transform =
                    resolve_transform3(scene, &command.entity_name, command.text.transform);
                if self.append_text3d_ttf_font_texture_batch(
                    &mut ui_texture_batches,
                    &surface.device,
                    &surface.queue,
                    assets,
                    &viewport,
                    camera,
                    &command.text.font,
                    &command.text.content,
                    transform,
                    command.text.size,
                    ColorRgba::new(0.94, 0.98, 1.0, 1.0),
                ) {
                    continue;
                }

                let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                append_text_3d_vertices(
                    vertices,
                    &viewport,
                    camera,
                    &command.text.content,
                    transform,
                    command.text.size,
                    ColorRgba::new(0.94, 0.98, 1.0, 1.0),
                );
            }
        }

        let mut ui_color_primitives = Vec::with_capacity(ui_primitives.len());
        for primitive in ui_primitives {
            if let UiDrawPrimitive::Text {
                rect,
                content,
                color,
                font_size,
                font: Some(font),
                anchor,
                word_wrap,
                fit_to_width,
            } = primitive
            {
                if self.append_ui_ttf_font_texture_batch(
                    &mut ui_texture_batches,
                    &surface.device,
                    &surface.queue,
                    assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }

                if self.append_ui_bitmap_font_texture_batch(
                    &mut ui_texture_batches,
                    &surface.device,
                    &surface.queue,
                    assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }
            }
            ui_color_primitives.push(primitive.clone());
        }

        {
            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);
        }

        color_batches.retain(|batch| !batch.vertices.is_empty());
        ui_texture_batches.retain(|batch| !batch.vertices.is_empty());
        self.render_surface_batches(
            surface,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            &texture_batches,
            &color_batches,
            &ui_texture_batches,
        )
    }
}

fn focus_blur_effect(
    stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack],
) -> Option<amigo_2d_post_fx::FocusBlur2d> {
    stacks
        .iter()
        .flat_map(|stack| stack.effects.iter())
        .find_map(|instance| match &instance.effect {
            amigo_2d_post_fx::PostFx2d::FocusBlur(effect) => Some(effect.clone()),
            _ => None,
        })
}

fn focus_blur_effect_for(
    stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack],
    host_id: &amigo_2d_post_fx::PostFxHost2dId,
    effect_id: &amigo_2d_post_fx::PostFx2dId,
) -> Option<amigo_2d_post_fx::FocusBlur2d> {
    stacks
        .iter()
        .find(|stack| &stack.host_id == host_id)
        .and_then(|stack| stack.effects.iter().find(|effect| &effect.id == effect_id))
        .and_then(|instance| match &instance.effect {
            amigo_2d_post_fx::PostFx2d::FocusBlur(effect) => Some(effect.clone()),
            _ => None,
        })
}

fn focus_blur_layer_plan(
    stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack],
    render_layers: &[RenderLayer2dCommand],
) -> Option<FocusBlurLayerPlan> {
    let effect = focus_blur_effect(stacks)?;
    Some(build_focus_blur_layer_plan(effect, render_layers))
}

fn focus_blur_layer_plan_for_effect(
    stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack],
    render_layers: &[RenderLayer2dCommand],
    host_id: &amigo_2d_post_fx::PostFxHost2dId,
    effect_id: &amigo_2d_post_fx::PostFx2dId,
) -> Option<FocusBlurLayerPlan> {
    let effect = focus_blur_effect_for(stacks, host_id, effect_id)?;
    Some(build_focus_blur_layer_plan(effect, render_layers))
}

fn build_focus_blur_layer_plan(
    effect: amigo_2d_post_fx::FocusBlur2d,
    render_layers: &[RenderLayer2dCommand],
) -> FocusBlurLayerPlan {
    let has_explicit_render_depth = render_layers
        .iter()
        .any(|layer| !layer.depth.is_depth_map());
    let legacy_affected_layers = (!has_explicit_render_depth && !effect.affected_layers.is_empty())
        .then(|| effect.affected_layers.into_iter().collect::<BTreeSet<_>>());

    let mut depth_map_layers = BTreeSet::new();
    let mut plane_layers = Vec::new();
    let mut overlay_layers = BTreeSet::new();

    for layer in render_layers {
        match layer.depth.mode {
            amigo_2d_composition::RenderDepthMode2d::DepthMap => {
                depth_map_layers.insert(layer.id.clone());
            }
            amigo_2d_composition::RenderDepthMode2d::Plane => {
                plane_layers.push(FocusBlurPlaneLayer {
                    layer_id: layer.id.clone(),
                    value: layer.depth.value,
                    blur_scale: layer.depth.blur_scale,
                });
            }
            amigo_2d_composition::RenderDepthMode2d::Overlay => {
                overlay_layers.insert(layer.id.clone());
            }
        }
    }

    FocusBlurLayerPlan {
        depth_map_layers,
        plane_layers,
        overlay_layers,
        legacy_affected_layers,
        has_explicit_render_depth,
    }
}

fn compatible_offscreen_target(
    template: &WgpuOffscreenTarget,
    label: &'static str,
) -> WgpuOffscreenTarget {
    let texture = template.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: template.width.max(1),
            height: template.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: template.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    WgpuOffscreenTarget {
        report: template.report.clone(),
        device: template.device.clone(),
        queue: template.queue.clone(),
        width: template.width,
        height: template.height,
        format: template.format,
        texture,
        view,
    }
}

fn translated_transform2(transform: Transform2, offset: Vec2) -> Transform2 {
    let mut next = transform;
    next.translation.x += offset.x;
    next.translation.y += offset.y;
    next
}

fn color_with_alpha_mul(color: ColorRgba, alpha: f32) -> ColorRgba {
    ColorRgba::new(color.r, color.g, color.b, color.a * alpha.clamp(0.0, 1.0))
}

fn text2d_effect_offsets(radius: f32) -> [(f32, f32); 8] {
    [
        (-radius, 0.0),
        (radius, 0.0),
        (0.0, -radius),
        (0.0, radius),
        (-radius * 0.7, -radius * 0.7),
        (radius * 0.7, -radius * 0.7),
        (-radius * 0.7, radius * 0.7),
        (radius * 0.7, radius * 0.7),
    ]
}

fn append_fullscreen_texture_vertices(vertices: &mut Vec<TextureVertex>) {
    push_textured_quad(
        vertices,
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, 1.0),
        TextureUvRect {
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
        },
        ColorRgba::new(1.0, 1.0, 1.0, 1.0),
    );
}

fn append_surface_texture_rect_vertices(
    vertices: &mut Vec<TextureVertex>,
    surface_width: f32,
    surface_height: f32,
    placement: WgpuGameViewportPlacement,
) {
    let rect = placement.surface_rect;
    let surface_width = surface_width.max(1.0);
    let surface_height = surface_height.max(1.0);
    let logical_width = placement.logical_width.max(1) as f32;
    let logical_height = placement.logical_height.max(1) as f32;
    let base_scale = (rect.width / logical_width)
        .min(rect.height / logical_height)
        .max(0.0001);
    let zoom = placement.zoom.max(0.01);
    let draw_scale = base_scale * zoom;

    let visible_left = (-placement.pan_x / draw_scale).clamp(0.0, logical_width);
    let visible_top = (-placement.pan_y / draw_scale).clamp(0.0, logical_height);
    let visible_right = ((rect.width - placement.pan_x) / draw_scale).clamp(0.0, logical_width);
    let visible_bottom = ((rect.height - placement.pan_y) / draw_scale).clamp(0.0, logical_height);

    if visible_right <= visible_left || visible_bottom <= visible_top {
        return;
    }

    let x0 = rect.x + placement.pan_x + visible_left * draw_scale;
    let y0 = rect.y + placement.pan_y + visible_top * draw_scale;
    let x1 = rect.x + placement.pan_x + visible_right * draw_scale;
    let y1 = rect.y + placement.pan_y + visible_bottom * draw_scale;

    let left = x0 / surface_width * 2.0 - 1.0;
    let right = x1 / surface_width * 2.0 - 1.0;
    let top = 1.0 - y0 / surface_height * 2.0;
    let bottom = 1.0 - y1 / surface_height * 2.0;

    push_textured_quad(
        vertices,
        Vec2::new(left, bottom),
        Vec2::new(right, bottom),
        Vec2::new(right, top),
        Vec2::new(left, top),
        TextureUvRect {
            u0: visible_left / logical_width,
            v0: visible_top / logical_height,
            u1: visible_right / logical_width,
            v1: visible_bottom / logical_height,
        },
        ColorRgba::new(1.0, 1.0, 1.0, 1.0),
    );
}

fn emergency_overlay_lines(
    request_lines: &[WgpuEmergencyOverlayLine],
    renderer_lines: &[WgpuEmergencyOverlayLine],
) -> Vec<WgpuEmergencyOverlayLine> {
    request_lines
        .iter()
        .chain(renderer_lines.iter())
        .rev()
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn push_emergency_overlay_line(
    lines: &mut Vec<WgpuEmergencyOverlayLine>,
    level: WgpuEmergencyOverlayLevel,
    message: String,
) {
    if lines
        .last()
        .is_some_and(|line| line.level == level && line.message == message)
    {
        return;
    }

    lines.push(WgpuEmergencyOverlayLine { level, message });
    if lines.len() > 5 {
        lines.remove(0);
    }
}

fn emergency_overlay_color_batches(
    viewport: &Viewport,
    target_width: u32,
    lines: &[WgpuEmergencyOverlayLine],
) -> Vec<ColorBatch> {
    if lines.is_empty() {
        return Vec::new();
    }

    let mut vertices = Vec::new();
    let pixel = 2.0;
    let row_height = 18.0;
    let x = 12.0;
    let y = 12.0;
    let width = (target_width as f32 - x * 2.0).max(80.0);
    let height = row_height * lines.len() as f32 + 10.0;
    append_screen_rect(
        &mut vertices,
        viewport,
        x - 6.0,
        y - 6.0,
        width,
        height,
        ColorRgba::new(0.0, 0.0, 0.0, 0.72),
    );

    for (index, line) in lines.iter().enumerate() {
        let line_y = y + index as f32 * row_height;
        let label = match line.level {
            WgpuEmergencyOverlayLevel::Warning => "WARNING",
            WgpuEmergencyOverlayLevel::Error => "ERROR",
        };
        let color = match line.level {
            WgpuEmergencyOverlayLevel::Warning => ColorRgba::new(1.0, 0.84, 0.20, 1.0),
            WgpuEmergencyOverlayLevel::Error => ColorRgba::new(1.0, 0.22, 0.18, 1.0),
        };
        let text = format!("{label}: {}", line.message);
        append_bitmap_text(
            &mut vertices,
            viewport,
            x,
            line_y,
            pixel,
            &text,
            color,
            width - 12.0,
        );
    }

    vec![ColorBatch {
        blend_mode: ParticleBlendMode2d::Alpha,
        vertices,
    }]
}

fn append_screen_rect(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: ColorRgba,
) {
    let points = [
        ndc_from_ui_screen(Vec2::new(x, y), viewport),
        ndc_from_ui_screen(Vec2::new(x + width, y), viewport),
        ndc_from_ui_screen(Vec2::new(x + width, y + height), viewport),
        ndc_from_ui_screen(Vec2::new(x, y + height), viewport),
    ];
    push_quad(vertices, points[0], points[1], points[2], points[3], color);
}

fn append_bitmap_text(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    x: f32,
    y: f32,
    pixel: f32,
    text: &str,
    color: ColorRgba,
    max_width: f32,
) {
    let mut cursor_x = x;
    for ch in text.chars().flat_map(char::to_uppercase) {
        if cursor_x + 6.0 * pixel > x + max_width {
            break;
        }
        append_bitmap_glyph(vertices, viewport, cursor_x, y, pixel, ch, color);
        cursor_x += 6.0 * pixel;
    }
}

fn append_bitmap_glyph(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    x: f32,
    y: f32,
    pixel: f32,
    ch: char,
    color: ColorRgba,
) {
    for (row, bits) in bitmap_glyph(ch).iter().enumerate() {
        for col in 0..5 {
            if bits & (1 << (4 - col)) == 0 {
                continue;
            }
            append_screen_rect(
                vertices,
                viewport,
                x + col as f32 * pixel,
                y + row as f32 * pixel,
                pixel,
                pixel,
                color,
            );
        }
    }
}

fn bitmap_glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x12, 0x12, 0x0C],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1E, 0x01, 0x01, 0x0E, 0x01, 0x01, 0x1E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x10, 0x1E, 0x01, 0x01, 0x1E],
        '6' => [0x0E, 0x10, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x01, 0x0E],
        ':' => [0x00, 0x04, 0x04, 0x00, 0x04, 0x04, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x04, 0x04, 0x08],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F],
        '/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        '\\' => [0x10, 0x10, 0x08, 0x04, 0x02, 0x01, 0x01],
        ' ' => [0x00; 7],
        _ => [0x1F, 0x01, 0x02, 0x04, 0x04, 0x00, 0x04],
    }
}

#[cfg(test)]
mod emergency_overlay_tests {
    use super::*;

    #[test]
    fn emergency_overlay_lines_keep_latest_five() {
        let mut renderer_lines = Vec::new();
        for index in 0..8 {
            push_emergency_overlay_line(
                &mut renderer_lines,
                WgpuEmergencyOverlayLevel::Error,
                format!("error {index}"),
            );
        }

        let lines = emergency_overlay_lines(&[], &renderer_lines);
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0].message, "error 3");
        assert_eq!(lines[4].message, "error 7");
    }

    #[test]
    fn emergency_overlay_deduplicates_adjacent_messages() {
        let mut lines = Vec::new();
        push_emergency_overlay_line(
            &mut lines,
            WgpuEmergencyOverlayLevel::Warning,
            "same warning".to_owned(),
        );
        push_emergency_overlay_line(
            &mut lines,
            WgpuEmergencyOverlayLevel::Warning,
            "same warning".to_owned(),
        );

        assert_eq!(lines.len(), 1);
    }
}

#[cfg(test)]
mod focus_blur_layer_plan_tests {
    use super::*;

    fn layer(id: &str, mode: amigo_2d_composition::RenderDepthMode2d) -> RenderLayer2dCommand {
        RenderLayer2dCommand {
            source_mod: "test-mod".to_owned(),
            id: id.to_owned(),
            label: Some(id.to_owned()),
            order: 0.0,
            visible: true,
            opacity: 1.0,
            depth: amigo_2d_composition::RenderDepth2d {
                mode,
                value: 0.5,
                blur_scale: 1.0,
            },
        }
    }

    #[test]
    fn focus_blur_plan_prefers_explicit_render_depth_over_legacy_affected_layers() {
        let plan = build_focus_blur_layer_plan(
            amigo_2d_post_fx::FocusBlur2d {
                affected_layers: vec!["background.city".to_owned()],
                ..Default::default()
            },
            &[
                layer(
                    "background.city",
                    amigo_2d_composition::RenderDepthMode2d::DepthMap,
                ),
                layer(
                    "weather.rain.front",
                    amigo_2d_composition::RenderDepthMode2d::Overlay,
                ),
            ],
        );

        assert!(plan.has_explicit_render_depth);
        assert!(plan.legacy_affected_layers.is_none());
        assert!(plan.overlay_layers.contains("weather.rain.front"));
    }

    #[test]
    fn focus_blur_plan_splits_depth_map_plane_and_overlay_layers() {
        let plan = build_focus_blur_layer_plan(
            amigo_2d_post_fx::FocusBlur2d::default(),
            &[
                layer(
                    "background.city",
                    amigo_2d_composition::RenderDepthMode2d::DepthMap,
                ),
                RenderLayer2dCommand {
                    depth: amigo_2d_composition::RenderDepth2d {
                        mode: amigo_2d_composition::RenderDepthMode2d::Plane,
                        value: 0.34,
                        blur_scale: 0.12,
                    },
                    ..layer(
                        "weather.rain.near",
                        amigo_2d_composition::RenderDepthMode2d::Plane,
                    )
                },
                layer("ui", amigo_2d_composition::RenderDepthMode2d::Overlay),
            ],
        );

        assert!(plan.depth_map_layers.contains("background.city"));
        assert_eq!(plan.plane_layers.len(), 1);
        assert_eq!(plan.plane_layers[0].layer_id, "weather.rain.near");
        assert_eq!(plan.plane_layers[0].value, 0.34);
        assert_eq!(plan.plane_layers[0].blur_scale, 0.12);
        assert!(plan.overlay_layers.contains("ui"));
    }

    #[test]
    fn focus_blur_plan_preserves_plane_layer_order() {
        let plan = build_focus_blur_layer_plan(
            amigo_2d_post_fx::FocusBlur2d::default(),
            &[
                RenderLayer2dCommand {
                    order: -18.0,
                    depth: amigo_2d_composition::RenderDepth2d {
                        mode: amigo_2d_composition::RenderDepthMode2d::Plane,
                        value: 0.68,
                        blur_scale: 0.45,
                    },
                    ..layer(
                        "weather.rain.far",
                        amigo_2d_composition::RenderDepthMode2d::Plane,
                    )
                },
                RenderLayer2dCommand {
                    order: -16.0,
                    depth: amigo_2d_composition::RenderDepth2d {
                        mode: amigo_2d_composition::RenderDepthMode2d::Plane,
                        value: 0.52,
                        blur_scale: 0.25,
                    },
                    ..layer(
                        "weather.rain.mid",
                        amigo_2d_composition::RenderDepthMode2d::Plane,
                    )
                },
                RenderLayer2dCommand {
                    order: -14.0,
                    depth: amigo_2d_composition::RenderDepth2d {
                        mode: amigo_2d_composition::RenderDepthMode2d::Plane,
                        value: 0.34,
                        blur_scale: 0.12,
                    },
                    ..layer(
                        "weather.rain.near",
                        amigo_2d_composition::RenderDepthMode2d::Plane,
                    )
                },
            ],
        );

        assert_eq!(
            plan.plane_layers
                .iter()
                .map(|layer| layer.layer_id.as_str())
                .collect::<Vec<_>>(),
            vec!["weather.rain.far", "weather.rain.mid", "weather.rain.near"]
        );
    }
}
