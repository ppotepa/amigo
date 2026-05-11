use crate::renderer::*;

impl WgpuSceneRenderer {
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
            request.world_3d.meshes,
            request.world_3d.materials,
            request.world_3d.text3d,
            request.world_2d.render_layers,
            request.world_2d.light_routes,
            request.world_2d.light_groups,
            request.world_2d.particles,
            &[],
        )
    }

    pub(crate) fn execute_post_fx_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        feature_id: amigo_render_api::RenderFeatureId,
        effect_index: usize,
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
            &feature_id,
            effect_index,
            &source,
            target,
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
            WgpuFrameRenderTarget::Surface(surface) => {
                self.render_texture_to_surface(surface, &source)
            }
            WgpuFrameRenderTarget::Offscreen(target) => {
                self.copy_offscreen_to_offscreen(target, &source)?;
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

    fn render_texture_to_surface(
        &mut self,
        surface: &mut WgpuSurfaceState,
        source_view: &wgpu::TextureView,
    ) -> AmigoResult<()> {
        let mut world_batch = self.create_fullscreen_texture_batch(
            &surface.device,
            source_view,
            TextureBlendMode::Alpha,
        );
        append_fullscreen_texture_vertices(&mut world_batch.vertices);

        self.render_surface_batches(
            surface,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            &[world_batch],
            &[],
            &[],
        )
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
        meshes: &[MeshDrawCommand],
        materials: &[MaterialDrawCommand],
        text3d: Option<&[Text3dDrawCommand]>,
        render_layers: &[RenderLayer2dCommand],
        light_routes: &[LightRoute2dCommand],
        light_groups: &[LightGroup2dCommand],
        particles: &[Particle2dDrawCommand],
        ui_primitives: &[UiDrawPrimitive],
    ) -> AmigoResult<()> {
        let viewport = Viewport::from_offscreen(target);
        let mut color_batches = Vec::new();
        let mut texture_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();
        let camera2d = resolve_camera2d_transform(scene);
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
                    let transform =
                        resolve_transform2(scene, &command.entity_name, command.transform);
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
            let transform = resolve_transform2(scene, &command.entity_name, command.text.transform);
            if self.append_text2d_ttf_font_texture_batch(
                &mut ui_texture_batches,
                &target.device,
                &target.queue,
                assets,
                &viewport,
                camera2d,
                &command.text.font,
                &command.text.content,
                transform,
                command.text.bounds,
                ColorRgba::new(1.0, 0.96, 0.82, 1.0),
            ) {
                continue;
            }

            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            append_text_2d_vertices(
                vertices,
                &viewport,
                camera2d,
                &command.text.content,
                transform,
                command.text.bounds,
                ColorRgba::new(1.0, 0.96, 0.82, 1.0),
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
        meshes: &[MeshDrawCommand],
        materials: &[MaterialDrawCommand],
        text3d: Option<&[Text3dDrawCommand]>,
        render_layers: &[RenderLayer2dCommand],
        light_routes: &[LightRoute2dCommand],
        light_groups: &[LightGroup2dCommand],
        particles: &[Particle2dDrawCommand],
        ui_primitives: &[UiDrawPrimitive],
    ) -> AmigoResult<()> {
        let viewport = Viewport::from_surface(surface);
        let mut color_batches = Vec::new();
        let mut texture_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();
        let camera2d = resolve_camera2d_transform(scene);
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
                    let transform =
                        resolve_transform2(scene, &command.entity_name, command.transform);
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
            let transform = resolve_transform2(scene, &command.entity_name, command.text.transform);
            if self.append_text2d_ttf_font_texture_batch(
                &mut ui_texture_batches,
                &surface.device,
                &surface.queue,
                assets,
                &viewport,
                camera2d,
                &command.text.font,
                &command.text.content,
                transform,
                command.text.bounds,
                ColorRgba::new(1.0, 0.96, 0.82, 1.0),
            ) {
                continue;
            }

            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            append_text_2d_vertices(
                vertices,
                &viewport,
                camera2d,
                &command.text.content,
                transform,
                command.text.bounds,
                ColorRgba::new(1.0, 0.96, 0.82, 1.0),
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
