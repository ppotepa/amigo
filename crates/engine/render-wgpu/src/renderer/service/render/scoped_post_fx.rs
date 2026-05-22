use super::world_filters::WorldPassLoad;
use super::world_selection::OwnedWorldRenderSelection;
use super::*;

enum ScopedPostFxTarget<'a> {
    DrawLayer {
        layer_id: &'a str,
    },
    SceneObject {
        scene_object_id: &'a str,
    },
    GroupSubtree {
        root_scene_object_id: &'a str,
    },
    SourceImage {
        asset: &'a str,
    },
    ImagePart {
        owner_scene_object_id: &'a str,
        part_id: &'a str,
    },
}

impl<'a> ScopedPostFxTarget<'a> {
    fn from_scope(
        scope: &'a amigo_render_api::PostFxScope2d,
        pipeline: amigo_render_api::PostFxPipelineKind,
    ) -> Option<Self> {
        match (scope, pipeline) {
            (
                amigo_render_api::PostFxScope2d::DrawLayer { draw_layer_id },
                amigo_render_api::PostFxPipelineKind::OffscreenDrawLayer,
            ) => Some(Self::DrawLayer {
                layer_id: draw_layer_id,
            }),
            (
                amigo_render_api::PostFxScope2d::SceneObjectPixels { scene_object_id },
                amigo_render_api::PostFxPipelineKind::OffscreenObject,
            ) => Some(Self::SceneObject { scene_object_id }),
            (
                amigo_render_api::PostFxScope2d::GroupSubtree {
                    root_scene_object_id,
                },
                amigo_render_api::PostFxPipelineKind::OffscreenGroup,
            ) => Some(Self::GroupSubtree {
                root_scene_object_id,
            }),
            (
                amigo_render_api::PostFxScope2d::SourceImage { asset },
                amigo_render_api::PostFxPipelineKind::CachedImage,
            ) => Some(Self::SourceImage { asset }),
            (
                amigo_render_api::PostFxScope2d::ImagePart {
                    owner_scene_object_id,
                    part_id,
                    ..
                },
                amigo_render_api::PostFxPipelineKind::CachedImage,
            ) => Some(Self::ImagePart {
                owner_scene_object_id,
                part_id,
            }),
            _ => None,
        }
    }

    fn labels(&self) -> (&'static str, &'static str) {
        match self {
            Self::DrawLayer { .. } => (
                "amigo-postfx-drawlayer-source",
                "amigo-postfx-drawlayer-result",
            ),
            Self::SceneObject { .. } => {
                ("amigo-postfx-object-source", "amigo-postfx-object-result")
            }
            Self::GroupSubtree { .. } => ("amigo-postfx-group-source", "amigo-postfx-group-result"),
            Self::SourceImage { .. } => (
                "amigo-postfx-cached-image-source",
                "amigo-postfx-cached-image-result",
            ),
            Self::ImagePart { .. } => (
                "amigo-postfx-image-part-source",
                "amigo-postfx-image-part-result",
            ),
        }
    }
}

pub(super) struct PostFxGraphNodeContext<'a> {
    pub(super) host_id: &'a amigo_render_api::PostFxHost2dId,
    pub(super) effect_id: &'a amigo_render_api::PostFx2dId,
    pub(super) scope: &'a amigo_render_api::PostFxScope2d,
    pub(super) pipeline: amigo_render_api::PostFxPipelineKind,
    pub(super) feature_id: amigo_render_api::RenderFeatureId,
}

pub(super) fn execute_post_fx_graph_node(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    node: &amigo_render_api::FrameGraphNode,
    post_fx: PostFxGraphNodeContext<'_>,
    resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
) -> AmigoResult<()> {
    let read = graph_nodes::first_read(node, "post-fx")?;
    let write = graph_nodes::first_write(node, "post-fx")?;

    let source = resources
        .target(read)
        .ok_or_else(|| amigo_core::AmigoError::Message("post-fx read target unavailable".into()))?
        .view
        .clone();
    let target = resources.target_mut(write).ok_or_else(|| {
        amigo_core::AmigoError::Message("post-fx write target unavailable".into())
    })?;

    if request.camera_debug_view.as_str() == "camera.raw_scene_color" {
        return renderer.copy_offscreen_to_offscreen(target, &source);
    }

    if amigo_relight_2d_plugin::is_plate_relight_render_debug_view(&request.camera_debug_view) {
        return renderer.copy_offscreen_to_offscreen(target, &source);
    }

    if matches!(
        post_fx.pipeline,
        amigo_render_api::PostFxPipelineKind::Unsupported
    ) {
        return renderer.copy_offscreen_to_offscreen(target, &source);
    }

    if let Some(scoped_target) = ScopedPostFxTarget::from_scope(post_fx.scope, post_fx.pipeline) {
        return execute_scoped_post_fx(
            renderer,
            request,
            target,
            &source,
            scoped_target,
            &post_fx,
        );
    }

    let visual_sources = request.camera_capture_input_2d.as_ref().map(|input| {
        crate::renderer::service::WgpuCameraVisualSources2d::from_capture_input(input)
    });

    if matches!(
        request.camera_debug_view.as_str(),
        "camera.scene_depth" | "camera.computed_z_depth"
    ) {
        let supports_depth_debug = amigo_render_api::PostFxRenderDescriptor::for_kind(
            post_fx.feature_id.as_str(),
        )
        .is_some_and(|descriptor| descriptor.debug_policy.supports_depth_debug_view);
        if !supports_depth_debug {
            return renderer.copy_offscreen_to_offscreen(target, &source);
        }
        if let Some(mut effect) = super::depth_debug_post_fx_for(
            request.post_fx_stacks,
            post_fx.host_id,
            post_fx.effect_id,
        )
        {
            effect.debug_view = amigo_render_api::FocusBlurDebugView2d::Depth;
            return crate::renderer::service::post_fx::focus_blur::execute_focus_blur(
                renderer, request, effect, &source, target,
            );
        }
    }

    if super::visual_debug::should_bypass_for_camera_debug_view(
        &request.camera_debug_view,
        post_fx.feature_id.as_str(),
    ) {
        return renderer.copy_offscreen_to_offscreen(target, &source);
    }

    if super::visual_debug::try_execute_camera_debug_view(
        renderer,
        request,
        target,
        &source,
        visual_sources.as_ref(),
        post_fx.host_id,
        post_fx.effect_id,
        post_fx.feature_id.as_str(),
    )? {
        return Ok(());
    }

    crate::renderer::service::post_fx::execute_screen_space_post_fx(
        renderer,
        request,
        post_fx.host_id,
        post_fx.effect_id,
        post_fx.scope,
        post_fx.pipeline,
        &post_fx.feature_id,
        &source,
        target,
    )?;

    let replays_scoped_layers = amigo_render_api::PostFxRenderDescriptor::for_kind(
        post_fx.feature_id.as_str(),
    )
    .is_some_and(|descriptor| {
        descriptor.output == amigo_render_api::PostFxRenderOutput::ReplayScopedLayers
    });
    if !replays_scoped_layers {
        return Ok(());
    }

    let Some(plan) = super::replay_scoped_layers_plan_for_effect(
        request.post_fx_stacks,
        request.world_2d.render_layers,
        request.camera_capture_input_2d,
        post_fx.host_id,
        post_fx.effect_id,
    ) else {
        return Ok(());
    };

    if plan.has_explicit_render_depth {
        let world_ctx = WorldRenderContext::from_request(request);
        for z_depth_layer in &plan.z_depth_layers {
            let mut z_depth_source = super::offscreen_ops::compatible_offscreen_target(
                target,
                "amigo-focus-blur-z-depth-source",
            );
            let mut z_depth_blurred = super::offscreen_ops::compatible_offscreen_target(
                target,
                "amigo-focus-blur-z-depth-blurred",
            );
            let selection = OwnedWorldRenderSelection::draw_layer(
                &z_depth_layer.layer_id,
                WorldPassLoad::ClearTransparent,
            );
            world::execute_world_to_offscreen(
                renderer,
                &mut z_depth_source,
                world_ctx,
                selection.borrowed(),
                &[],
            )?;
            let highlight_view = renderer
                .visual_source_targets_2d
                .scene_highlight
                .as_ref()
                .map(|target| target.view.clone());
            crate::renderer::service::post_fx::focus_blur::execute_focus_blur_with_depth_source(
                renderer,
                request,
                super::focus_blur_effect_for(
                    request.post_fx_stacks,
                    post_fx.host_id,
                    post_fx.effect_id,
                )
                    .expect("focus blur effect should exist for explicit layer plan"),
                &z_depth_source.view,
                highlight_view.as_ref(),
                &mut z_depth_blurred,
                crate::renderer::service::post_fx::focus_blur::FocusBlurDepthSource::ZDepth {
                    z_depth: z_depth_layer.z_depth,
                    blur_scale: z_depth_layer.blur_scale,
                },
            )?;
            renderer.composite_offscreen_over_offscreen(target, &z_depth_blurred.view)?;
        }

        if !plan.overlay_layers.is_empty() {
            let selection = OwnedWorldRenderSelection::include_layers(
                plan.overlay_layers.clone(),
                false,
                WorldPassLoad::Load,
            );
            return world::execute_world_to_offscreen(
                renderer,
                target,
                world_ctx,
                selection.borrowed(),
                &[],
            );
        }

        return Ok(());
    }

    let Some(affected_layers) = plan.implicit_affected_layers.as_ref() else {
        return Ok(());
    };

    let world_ctx = WorldRenderContext::from_request(request);
    let selection =
        OwnedWorldRenderSelection::exclude_layers(affected_layers.clone(), WorldPassLoad::Load);
    world::execute_world_to_offscreen(renderer, target, world_ctx, selection.borrowed(), &[])
}

fn execute_scoped_post_fx(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
    source: &wgpu::TextureView,
    scoped_target: ScopedPostFxTarget<'_>,
    post_fx: &PostFxGraphNodeContext<'_>,
) -> AmigoResult<()> {
    renderer.copy_offscreen_to_offscreen(target, source)?;
    let (source_label, result_label) = scoped_target.labels();
    let mut scoped_source = super::offscreen_ops::compatible_offscreen_target(target, source_label);
    let mut scoped_result = super::offscreen_ops::compatible_offscreen_target(target, result_label);

    {
        let world_ctx = WorldRenderContext::from_request(request);
        render_scoped_source(
            renderer,
            request.assets,
            &mut scoped_source,
            world_ctx,
            scoped_target,
        )?;
    }

    crate::renderer::service::post_fx::execute_screen_space_post_fx(
        renderer,
        request,
        post_fx.host_id,
        post_fx.effect_id,
        post_fx.scope,
        post_fx.pipeline,
        &post_fx.feature_id,
        &scoped_source.view,
        &mut scoped_result,
    )?;
    renderer.composite_offscreen_over_offscreen(target, &scoped_result.view)
}

fn render_scoped_source(
    renderer: &mut WgpuSceneRenderer,
    assets: &AssetCatalog,
    target: &mut WgpuOffscreenTarget,
    world_ctx: WorldRenderContext<'_>,
    scoped_target: ScopedPostFxTarget<'_>,
) -> AmigoResult<()> {
    match scoped_target {
        ScopedPostFxTarget::DrawLayer { layer_id } => {
            let selection =
                OwnedWorldRenderSelection::draw_layer(layer_id, WorldPassLoad::ClearTransparent);
            world::execute_world_to_offscreen(
                renderer,
                target,
                world_ctx,
                selection.borrowed(),
                &[],
            )
        }
        ScopedPostFxTarget::SceneObject { scene_object_id } => {
            let selection = OwnedWorldRenderSelection::scene_object(
                scene_object_id,
                WorldPassLoad::ClearTransparent,
            );
            world::execute_world_to_offscreen(
                renderer,
                target,
                world_ctx,
                selection.borrowed(),
                &[],
            )
        }
        ScopedPostFxTarget::GroupSubtree {
            root_scene_object_id,
        } => {
            let selection = OwnedWorldRenderSelection::group_subtree(
                root_scene_object_id,
                WorldPassLoad::ClearTransparent,
            );
            world::execute_world_to_offscreen(
                renderer,
                target,
                world_ctx,
                selection.borrowed(),
                &[],
            )
        }
        ScopedPostFxTarget::SourceImage { asset } => {
            crate::renderer::service::post_fx::wet_reflections::render_texture_asset_debug(
                renderer,
                assets,
                target,
                asset,
                [0.0, 0.0, 0.0, 0.0],
                "amigo-postfx-source-image",
            )
        }
        ScopedPostFxTarget::ImagePart {
            owner_scene_object_id,
            part_id,
        } => {
            let mut part_targets = BTreeMap::new();
            part_targets.insert(
                owner_scene_object_id.to_owned(),
                BTreeSet::from([part_id.to_owned()]),
            );
            super::execute_layered_image_parts_to_offscreen(
                renderer,
                target,
                world_ctx.renderables,
                world_ctx.assets,
                world_ctx.render_layers,
                &part_targets,
                WorldPassLoad::ClearTransparent,
            )
        }
    }
}
