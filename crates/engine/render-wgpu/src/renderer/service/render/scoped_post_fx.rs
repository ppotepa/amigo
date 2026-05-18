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
        scope: &'a amigo_2d_post_fx::PostFxScope2d,
        pipeline: amigo_2d_post_fx::PostFxPipelineKind,
    ) -> Option<Self> {
        match (scope, pipeline) {
            (
                amigo_2d_post_fx::PostFxScope2d::DrawLayer { draw_layer_id },
                amigo_2d_post_fx::PostFxPipelineKind::OffscreenDrawLayer,
            ) => Some(Self::DrawLayer {
                layer_id: draw_layer_id,
            }),
            (
                amigo_2d_post_fx::PostFxScope2d::SceneObjectPixels { scene_object_id },
                amigo_2d_post_fx::PostFxPipelineKind::OffscreenObject,
            ) => Some(Self::SceneObject { scene_object_id }),
            (
                amigo_2d_post_fx::PostFxScope2d::GroupSubtree {
                    root_scene_object_id,
                },
                amigo_2d_post_fx::PostFxPipelineKind::OffscreenGroup,
            ) => Some(Self::GroupSubtree {
                root_scene_object_id,
            }),
            (
                amigo_2d_post_fx::PostFxScope2d::SourceImage { asset },
                amigo_2d_post_fx::PostFxPipelineKind::CachedImage,
            ) => Some(Self::SourceImage { asset }),
            (
                amigo_2d_post_fx::PostFxScope2d::ImagePart {
                    owner_scene_object_id,
                    part_id,
                    ..
                },
                amigo_2d_post_fx::PostFxPipelineKind::CachedImage,
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

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_post_fx_graph_node(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    node: &amigo_render_api::FrameGraphNode,
    host_id: &amigo_2d_post_fx::PostFxHost2dId,
    effect_id: &amigo_2d_post_fx::PostFx2dId,
    scope: &amigo_2d_post_fx::PostFxScope2d,
    pipeline: amigo_2d_post_fx::PostFxPipelineKind,
    feature_id: amigo_render_api::RenderFeatureId,
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

    if request.camera_debug_view == amigo_render_api::CameraDebugView2d::RawSceneColor {
        return renderer.copy_offscreen_to_offscreen(target, &source);
    }

    if request.camera_debug_view.wants_plate_relight_debug() {
        return renderer.copy_offscreen_to_offscreen(target, &source);
    }

    if matches!(pipeline, amigo_2d_post_fx::PostFxPipelineKind::Unsupported) {
        return renderer.copy_offscreen_to_offscreen(target, &source);
    }

    if let Some(scoped_target) = ScopedPostFxTarget::from_scope(scope, pipeline) {
        return execute_scoped_post_fx(
            renderer,
            request,
            target,
            &source,
            scoped_target,
            host_id,
            effect_id,
            scope,
            pipeline,
            &feature_id,
        );
    }

    let visual_sources = request.camera_capture_input_2d.as_ref().map(|input| {
        crate::renderer::service::WgpuCameraVisualSources2d::from_capture_input(input)
    });

    if matches!(
        request.camera_debug_view,
        amigo_render_api::CameraDebugView2d::SceneDepth
            | amigo_render_api::CameraDebugView2d::ComputedZDepth
    ) {
        if feature_id.as_str() != "focus_blur" {
            return renderer.copy_offscreen_to_offscreen(target, &source);
        }
        if let Some(mut effect) =
            super::focus_blur_effect_for(request.post_fx_stacks, host_id, effect_id)
        {
            effect.debug_view = amigo_2d_post_fx::FocusBlurDebugView2d::Depth;
            return crate::renderer::service::post_fx::focus_blur::execute_focus_blur(
                renderer, request, effect, &source, target,
            );
        }
    }

    if super::visual_debug::should_bypass_for_camera_debug_view(
        request.camera_debug_view,
        feature_id.as_str(),
    ) {
        return renderer.copy_offscreen_to_offscreen(target, &source);
    }

    if super::visual_debug::try_execute_camera_debug_view(
        renderer,
        request,
        target,
        &source,
        visual_sources.as_ref(),
        host_id,
        effect_id,
        feature_id.as_str(),
    )? {
        return Ok(());
    }

    crate::renderer::service::post_fx::execute_screen_space_post_fx(
        renderer,
        request,
        host_id,
        effect_id,
        scope,
        pipeline,
        &feature_id,
        &source,
        target,
    )?;

    let Some(plan) = super::focus_blur_layer_plan_for_effect(
        request.post_fx_stacks,
        request.world_2d.render_layers,
        host_id,
        effect_id,
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
                super::focus_blur_effect_for(request.post_fx_stacks, host_id, effect_id)
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

    let Some(affected_layers) = plan.legacy_affected_layers.as_ref() else {
        return Ok(());
    };

    let world_ctx = WorldRenderContext::from_request(request);
    let selection =
        OwnedWorldRenderSelection::exclude_layers(affected_layers.clone(), WorldPassLoad::Load);
    world::execute_world_to_offscreen(renderer, target, world_ctx, selection.borrowed(), &[])
}

#[allow(clippy::too_many_arguments)]
fn execute_scoped_post_fx(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
    source: &wgpu::TextureView,
    scoped_target: ScopedPostFxTarget<'_>,
    host_id: &amigo_2d_post_fx::PostFxHost2dId,
    effect_id: &amigo_2d_post_fx::PostFx2dId,
    scope: &amigo_2d_post_fx::PostFxScope2d,
    pipeline: amigo_2d_post_fx::PostFxPipelineKind,
    feature_id: &amigo_render_api::RenderFeatureId,
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
        host_id,
        effect_id,
        scope,
        pipeline,
        feature_id,
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
            world::execute_layered_image_parts_to_offscreen(
                renderer,
                target,
                world_ctx.scene,
                world_ctx.assets,
                world_ctx.layered_images,
                world_ctx.render_layers,
                world_ctx.active_camera_2d_entity,
                &part_targets,
                WorldPassLoad::ClearTransparent,
            )
        }
    }
}
