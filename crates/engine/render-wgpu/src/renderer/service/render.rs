use crate::renderer::*;

impl WgpuSceneRenderer {
    pub fn render_scene(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &MeshSceneService,
        materials: &MaterialSceneService,
        text3d: Option<&Text3dSceneService>,
    ) -> AmigoResult<()> {
        let mesh_commands = meshes.commands();
        let material_commands = materials.commands();
        let text3d_commands = text3d.map(|service: &Text3dSceneService| service.commands());
        self.render_scene_with_ui_primitives_and_3d_commands(
            surface,
            scene,
            assets,
            tilemaps,
            sprites,
            text2d,
            vectors,
            &mesh_commands,
            &material_commands,
            text3d_commands.as_deref(),
            &[],
            &[],
        )
    }

    pub fn render_scene_with_ui_documents(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &MeshSceneService,
        materials: &MaterialSceneService,
        text3d: Option<&Text3dSceneService>,
        ui_documents: &[UiOverlayDocument],
    ) -> AmigoResult<()> {
        let ui_primitives = build_ui_overlay_primitives(
            UiViewportSize::new(surface.config.width as f32, surface.config.height as f32),
            ui_documents,
        );
        let mesh_commands = meshes.commands();
        let material_commands = materials.commands();
        let text3d_commands = text3d.map(|service: &Text3dSceneService| service.commands());
        self.render_scene_with_ui_primitives_and_3d_commands(
            surface,
            scene,
            assets,
            tilemaps,
            sprites,
            text2d,
            vectors,
            &mesh_commands,
            &material_commands,
            text3d_commands.as_deref(),
            &[],
            &ui_primitives,
        )
    }

    pub fn render_scene_with_ui_primitives(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &MeshSceneService,
        materials: &MaterialSceneService,
        text3d: Option<&Text3dSceneService>,
        ui_primitives: &[UiDrawPrimitive],
    ) -> AmigoResult<()> {
        let mesh_commands = meshes.commands();
        let material_commands = materials.commands();
        let text3d_commands = text3d.map(|service: &Text3dSceneService| service.commands());
        self.render_scene_with_ui_primitives_and_3d_commands(
            surface,
            scene,
            assets,
            tilemaps,
            sprites,
            text2d,
            vectors,
            &mesh_commands,
            &material_commands,
            text3d_commands.as_deref(),
            &[],
            &ui_primitives,
        )
    }

    pub fn render_scene_with_ui_documents_and_3d_commands(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &[MeshDrawCommand],
        materials: &[MaterialDrawCommand],
        text3d: Option<&[Text3dDrawCommand]>,
        particles: &[Particle2dDrawCommand],
        ui_documents: &[UiOverlayDocument],
    ) -> AmigoResult<()> {
        let ui_primitives = build_ui_overlay_primitives(
            UiViewportSize::new(surface.config.width as f32, surface.config.height as f32),
            ui_documents,
        );
        self.render_scene_with_ui_primitives_and_3d_commands(
            surface,
            scene,
            assets,
            tilemaps,
            sprites,
            text2d,
            vectors,
            meshes,
            materials,
            text3d,
            particles,
            &ui_primitives,
        )
    }

    pub fn render_scene_with_ui_documents_and_3d_commands_offscreen(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &[MeshDrawCommand],
        materials: &[MaterialDrawCommand],
        text3d: Option<&[Text3dDrawCommand]>,
        particles: &[Particle2dDrawCommand],
        ui_documents: &[UiOverlayDocument],
    ) -> AmigoResult<()> {
        let ui_primitives = build_ui_overlay_primitives(
            UiViewportSize::new(target.width as f32, target.height as f32),
            ui_documents,
        );
        self.render_scene_with_ui_primitives_and_3d_commands_offscreen(
            target,
            scene,
            assets,
            tilemaps,
            sprites,
            text2d,
            vectors,
            meshes,
            materials,
            text3d,
            particles,
            &ui_primitives,
        )
    }

    pub fn render_scene_with_ui_primitives_and_3d_commands_offscreen(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &[MeshDrawCommand],
        materials: &[MaterialDrawCommand],
        text3d: Option<&[Text3dDrawCommand]>,
        particles: &[Particle2dDrawCommand],
        ui_primitives: &[UiDrawPrimitive],
    ) -> AmigoResult<()> {
        let viewport = Viewport::from_offscreen(target);
        let mut color_batches = Vec::new();
        let mut texture_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();
        let camera2d = resolve_camera2d_transform(scene);
        let particle_lights = particle_render_lights(particles);
        let mut world2d_items = tilemaps
            .commands()
            .into_iter()
            .map(World2dItem::TileMap)
            .chain(vectors.commands().into_iter().map(World2dItem::Vector))
            .chain(sprites.commands().into_iter().map(World2dItem::Sprite))
            .chain(particles.iter().cloned().map(World2dItem::Particle))
            .collect::<Vec<_>>();
        world2d_items.sort_by(|left, right| {
            let (left_z, left_priority) = world2d_sort_key(left);
            let (right_z, right_priority) = world2d_sort_key(right);
            let z_ordering = left_z.partial_cmp(&right_z).unwrap_or(Ordering::Equal);
            if z_ordering == Ordering::Equal {
                left_priority.cmp(&right_priority)
            } else {
                z_ordering
            }
        });

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
                    );
                }
            }
        }

        for command in text2d.commands() {
            let transform = resolve_transform2(scene, &command.entity_name, command.text.transform);
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
                            r: 0.08,
                            g: 0.09,
                            b: 0.12,
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
                pass.set_pipeline(&self.texture_pipeline);
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
                pass.set_pipeline(&self.texture_pipeline);
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, ui_texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    pub fn render_scene_with_ui_primitives_and_3d_commands(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &[MeshDrawCommand],
        materials: &[MaterialDrawCommand],
        text3d: Option<&[Text3dDrawCommand]>,
        particles: &[Particle2dDrawCommand],
        ui_primitives: &[UiDrawPrimitive],
    ) -> AmigoResult<()> {
        let viewport = Viewport::from_surface(surface);
        let mut color_batches = Vec::new();
        let mut texture_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();
        let camera2d = resolve_camera2d_transform(scene);
        let particle_lights = particle_render_lights(particles);
        let mut world2d_items = tilemaps
            .commands()
            .into_iter()
            .map(World2dItem::TileMap)
            .chain(vectors.commands().into_iter().map(World2dItem::Vector))
            .chain(sprites.commands().into_iter().map(World2dItem::Sprite))
            .chain(particles.iter().cloned().map(World2dItem::Particle))
            .collect::<Vec<_>>();
        world2d_items.sort_by(|left, right| {
            let (left_z, left_priority) = world2d_sort_key(left);
            let (right_z, right_priority) = world2d_sort_key(right);
            let z_ordering = left_z.partial_cmp(&right_z).unwrap_or(Ordering::Equal);
            if z_ordering == Ordering::Equal {
                left_priority.cmp(&right_priority)
            } else {
                z_ordering
            }
        });

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
                    );
                }
            }
        }

        for command in text2d.commands() {
            let transform = resolve_transform2(scene, &command.entity_name, command.text.transform);
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
                    )
                {
                    continue;
                }
            }
            ui_color_primitives.push(primitive.clone());
        }

        {
            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);
        }

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

        color_batches.retain(|batch| !batch.vertices.is_empty());
        ui_texture_batches.retain(|batch| !batch.vertices.is_empty());
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
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.09,
                            b: 0.12,
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
                pass.set_pipeline(&self.texture_pipeline);
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
                pass.set_pipeline(&self.texture_pipeline);
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, ui_texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        surface.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}
