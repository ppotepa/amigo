use crate::renderer::*;

impl WgpuSceneRenderer {
    pub fn render_frame_request(
        &mut self,
        request: WgpuFrameRenderRequest<'_>,
    ) -> AmigoResult<()> {
        self.frame_graph_executor
            .prepare_transient_resources(request.frame_graph, &request);

        if request.execution_mode == WgpuFrameGraphExecutionMode::SplitPassExperimental {
            return self.render_frame_request_split_pass_experimental(request);
        }

        let mut ui_documents = Vec::with_capacity(request.game_ui.len() + request.debug_ui.len());
        ui_documents.extend_from_slice(request.game_ui);
        ui_documents.extend_from_slice(request.debug_ui);

        self.render_scene_with_ui_documents_and_3d_commands_and_post_fx(
            request.surface,
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
            &ui_documents,
            request.post_fx_stack,
        )
    }

    fn render_frame_request_split_pass_experimental(
        &mut self,
        request: WgpuFrameRenderRequest<'_>,
    ) -> AmigoResult<()> {
        let mut world_target = create_surface_offscreen_target(request.surface);
        self.render_scene_with_ui_documents_and_3d_commands_offscreen(
            &mut world_target,
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
        )?;

        let mut ui_documents = Vec::with_capacity(request.game_ui.len() + request.debug_ui.len());
        ui_documents.extend_from_slice(request.game_ui);
        ui_documents.extend_from_slice(request.debug_ui);
        let lens = request.post_fx_stack.and_then(first_active_lens_droplets);

        self.render_world_texture_with_ui_documents_to_surface(
            request.surface,
            request.assets,
            &world_target.view,
            &ui_documents,
            lens,
        )
    }

    fn render_world_texture_with_ui_documents_to_surface(
        &mut self,
        surface: &mut WgpuSurfaceState,
        assets: &AssetCatalog,
        world_view: &wgpu::TextureView,
        ui_documents: &[UiOverlayDocument],
        lens: Option<amigo_2d_post_fx::PostFxLensDroplets2d>,
    ) -> AmigoResult<()> {
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

        {
            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);
        }

        let mut world_batch =
            self.create_fullscreen_texture_batch(&surface.device, world_view, TextureBlendMode::Alpha);
        append_fullscreen_texture_vertices(&mut world_batch.vertices);
        if let Some(lens) = lens {
            append_lens_droplets_overlay(&mut color_batches, &viewport, lens);
        }

        self.render_surface_batches(
            surface,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            &[world_batch],
            &color_batches,
            &ui_texture_batches,
        )
    }

    pub fn render_scene(
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
            layered_images,
            global_lights,
            lightmaps,
            text2d,
            vectors,
            &mesh_commands,
            &material_commands,
            text3d_commands.as_deref(),
            &[],
            &[],
            &[],
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
        layered_images: &amigo_2d_layered_image::LayeredImageSceneService,
        global_lights: &GlobalLight2dSceneService,
        lightmaps: &LightMap2dSceneService,
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
            layered_images,
            global_lights,
            lightmaps,
            text2d,
            vectors,
            &mesh_commands,
            &material_commands,
            text3d_commands.as_deref(),
            &[],
            &[],
            &[],
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
        layered_images: &amigo_2d_layered_image::LayeredImageSceneService,
        global_lights: &GlobalLight2dSceneService,
        lightmaps: &LightMap2dSceneService,
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
            layered_images,
            global_lights,
            lightmaps,
            text2d,
            vectors,
            &mesh_commands,
            &material_commands,
            text3d_commands.as_deref(),
            &[],
            &[],
            &[],
            &[],
            &ui_primitives,
        )
    }

    pub fn render_scene_with_ui_documents_and_3d_commands_and_post_fx(
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
        ui_documents: &[UiOverlayDocument],
        post_fx_stack: Option<&amigo_2d_post_fx::PostFx2dStack>,
    ) -> AmigoResult<()> {
        let _ = post_fx_stack.and_then(first_active_lens_droplets);

        self.render_scene_with_ui_documents_and_3d_commands(
            surface,
            scene,
            assets,
            tilemaps,
            sprites,
            layered_images,
            global_lights,
            lightmaps,
            text2d,
            vectors,
            meshes,
            materials,
            text3d,
            render_layers,
            light_routes,
            light_groups,
            particles,
            ui_documents,
        )
    }

    pub fn render_scene_with_ui_documents_and_3d_commands(
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
            layered_images,
            global_lights,
            lightmaps,
            text2d,
            vectors,
            meshes,
            materials,
            text3d,
            render_layers,
            light_routes,
            light_groups,
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
            layered_images,
            global_lights,
            lightmaps,
            text2d,
            vectors,
            meshes,
            materials,
            text3d,
            render_layers,
            light_routes,
            light_groups,
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

    #[allow(dead_code)]
    fn render_scene_with_lens_droplets_overlay(
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
        ui_documents: &[UiOverlayDocument],
        lens: amigo_2d_post_fx::PostFxLensDroplets2d,
    ) -> AmigoResult<()> {
        let mut target = create_surface_offscreen_target(surface);
        self.render_scene_with_ui_documents_and_3d_commands_offscreen(
            &mut target,
            scene,
            assets,
            tilemaps,
            sprites,
            layered_images,
            global_lights,
            lightmaps,
            text2d,
            vectors,
            meshes,
            materials,
            text3d,
            render_layers,
            light_routes,
            light_groups,
            particles,
            &[],
        )?;

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

        {
            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);
        }

        let mut world_batch =
            self.create_fullscreen_texture_batch(&surface.device, &target.view, TextureBlendMode::Alpha);
        append_fullscreen_texture_vertices(&mut world_batch.vertices);
        append_lens_droplets_overlay(&mut color_batches, &viewport, lens);

        self.render_surface_batches(
            surface,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            &[world_batch],
            &color_batches,
            &ui_texture_batches,
        )
    }

    #[allow(dead_code)]
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

    pub fn render_scene_with_ui_primitives_and_3d_commands(
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

fn first_active_lens_droplets(
    stack: &amigo_2d_post_fx::PostFx2dStack,
) -> Option<amigo_2d_post_fx::PostFxLensDroplets2d> {
    stack.effects.iter().find_map(|effect| match effect {
        amigo_2d_post_fx::PostFx2d::LensDroplets(lens) if lens.is_active() => Some(*lens),
        _ => None,
    })
}

#[allow(dead_code)]
fn create_surface_offscreen_target(surface: &WgpuSurfaceState) -> WgpuOffscreenTarget {
    let texture = surface.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("amigo-surface-offscreen-render-target"),
        size: wgpu::Extent3d {
            width: surface.config.width.max(1),
            height: surface.config.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: surface.config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    WgpuOffscreenTarget {
        report: surface.report.clone(),
        device: surface.device.clone(),
        queue: surface.queue.clone(),
        width: surface.config.width.max(1),
        height: surface.config.height.max(1),
        format: surface.config.format,
        texture,
        view,
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn append_lens_droplets_overlay(
    color_batches: &mut Vec<ColorBatch>,
    viewport: &Viewport,
    lens: amigo_2d_post_fx::PostFxLensDroplets2d,
) {
    if lens.darken > 0.0 {
        let vertices = color_batch_vertices(color_batches, ParticleBlendMode2d::Multiply);
        push_quad(
            vertices,
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
            ColorRgba::new(0.0, 0.0, 0.0, lens.darken.clamp(0.0, 1.0)),
        );
    }

    let droplet_count = lens.max_droplets.min(24) as usize;
    for index in 0..droplet_count {
        let radius_px = lerp_f32(
            lens.min_radius_px,
            lens.max_radius_px,
            hash_unit(index as u32 * 17 + 11),
        );
        let opacity = lerp_f32(
            lens.min_opacity,
            lens.max_opacity,
            hash_unit(index as u32 * 29 + 7),
        );
        let center = Vec2::new(
            hash_unit(index as u32 * 37 + 3) * viewport.size().x,
            hash_unit(index as u32 * 53 + 19) * viewport.size().y,
        );

        let dark_vertices = color_batch_vertices(color_batches, ParticleBlendMode2d::Multiply);
        append_soft_circle(
            dark_vertices,
            viewport,
            center,
            radius_px,
            18,
            ColorRgba::new(0.0, 0.0, 0.0, (opacity * lens.dirt_opacity).clamp(0.0, 1.0)),
        );

        let highlight_vertices = color_batch_vertices(color_batches, ParticleBlendMode2d::Screen);
        append_soft_circle(
            highlight_vertices,
            viewport,
            Vec2::new(center.x - radius_px * 0.18, center.y - radius_px * 0.18),
            radius_px * 0.42,
            14,
            ColorRgba::new(0.7, 0.78, 0.86, (opacity * 0.12).clamp(0.0, 0.24)),
        );

        if lens.streaks_enabled && hash_unit(index as u32 * 61 + 5) <= lens.streak_chance {
            let length =
                lens.max_streak_length * (0.2 + 0.8 * hash_unit(index as u32 * 71 + 23));
            let offset = (hash_unit(index as u32 * 83 + 13) - 0.5) * lens.wobble * 24.0;
            let tail = Vec2::new(center.x + offset, (center.y + length).min(viewport.size().y));
            let half_width = (radius_px * 0.12).max(1.0);
            let a = ndc_from_ui_screen(Vec2::new(center.x - half_width, center.y), viewport);
            let b = ndc_from_ui_screen(Vec2::new(center.x + half_width, center.y), viewport);
            let c = ndc_from_ui_screen(Vec2::new(tail.x + half_width, tail.y), viewport);
            let d = ndc_from_ui_screen(Vec2::new(tail.x - half_width, tail.y), viewport);
            let vertices = color_batch_vertices(color_batches, ParticleBlendMode2d::Multiply);
            push_quad(
                vertices,
                a,
                b,
                c,
                d,
                ColorRgba::new(
                    0.0,
                    0.0,
                    0.0,
                    (opacity * lens.dirt_opacity * 0.35).clamp(0.0, 0.18),
                ),
            );
        }
    }
}

#[allow(dead_code)]
fn append_soft_circle(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    center_px: Vec2,
    radius_px: f32,
    segments: usize,
    color: ColorRgba,
) {
    let center = ndc_from_ui_screen(center_px, viewport);
    let x_scale = radius_px / viewport.half_width;
    let y_scale = radius_px / viewport.half_height;
    let segments = segments.max(6);
    for index in 0..segments {
        let a0 = (index as f32 / segments as f32) * std::f32::consts::TAU;
        let a1 = ((index + 1) as f32 / segments as f32) * std::f32::consts::TAU;
        let p0 = Vec2::new(center.x + a0.cos() * x_scale, center.y + a0.sin() * y_scale);
        let p1 = Vec2::new(center.x + a1.cos() * x_scale, center.y + a1.sin() * y_scale);
        push_triangle(vertices, [center, p0, p1], color);
    }
}

#[allow(dead_code)]
fn hash_unit(seed: u32) -> f32 {
    let x = ((seed as f32 * 12.9898).sin() * 43758.5453).abs();
    x.fract()
}

#[allow(dead_code)]
fn lerp_f32(min: f32, max: f32, t: f32) -> f32 {
    min + (max - min) * t.clamp(0.0, 1.0)
}
