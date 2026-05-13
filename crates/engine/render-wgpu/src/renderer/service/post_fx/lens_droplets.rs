use amigo_2d_post_fx::PostFxLensDroplets2d;

use crate::{WgpuOffscreenTarget, renderer::service::WgpuSceneRenderer, renderer::*};

pub(crate) fn execute_lens_droplets(
    renderer: &mut WgpuSceneRenderer,
    lens: PostFxLensDroplets2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> amigo_core::AmigoResult<()> {
    let lens = lens.normalized();

    renderer.copy_offscreen_to_offscreen(output, input_view)?;

    if !lens.is_active() {
        return Ok(());
    }

    render_lens_droplets_overlay_to_offscreen(renderer, output, lens)
}

fn render_lens_droplets_overlay_to_offscreen(
    renderer: &mut WgpuSceneRenderer,
    target: &mut WgpuOffscreenTarget,
    lens: PostFxLensDroplets2d,
) -> amigo_core::AmigoResult<()> {
    let width = target.width.max(1) as f32;
    let height = target.height.max(1) as f32;
    let mut primitives = Vec::new();

    let darken = lens.darken.clamp(0.0, 1.0);
    if darken > 0.0 {
        primitives.push(UiDrawPrimitive::Quad {
            rect: crate::ui_overlay::UiRect::new(0.0, 0.0, width, height),
            color: ColorRgba::new(0.0, 0.0, 0.0, darken),
        });
    }

    let dirt = lens.dirt_opacity.clamp(0.0, 1.0);
    if dirt > 0.0 {
        primitives.push(UiDrawPrimitive::Quad {
            rect: crate::ui_overlay::UiRect::new(
                width * 0.04,
                height * 0.08,
                width * 0.18,
                height * 0.015,
            ),
            color: ColorRgba::new(0.75, 0.9, 1.0, dirt * 0.18),
        });
        primitives.push(UiDrawPrimitive::Quad {
            rect: crate::ui_overlay::UiRect::new(
                width * 0.72,
                height * 0.18,
                width * 0.22,
                height * 0.012,
            ),
            color: ColorRgba::new(0.75, 0.9, 1.0, dirt * 0.14),
        });
        primitives.push(UiDrawPrimitive::Quad {
            rect: crate::ui_overlay::UiRect::new(
                width * 0.12,
                height * 0.78,
                width * 0.28,
                height * 0.010,
            ),
            color: ColorRgba::new(0.75, 0.9, 1.0, dirt * 0.12),
        });
    }

    let droplet_count = (lens.max_droplets.min(14) as usize).min(14);
    if droplet_count > 0 {
        let min_radius = lens.min_radius_px.min(lens.max_radius_px).max(1.0);
        let max_radius = lens.max_radius_px.max(min_radius);
        let opacity_min = lens.min_opacity.min(lens.max_opacity).clamp(0.0, 1.0);
        let opacity_max = lens.max_opacity.max(opacity_min).clamp(0.0, 1.0);

        for index in 0..droplet_count {
            let i = index as f32;
            let t = (i + 1.0) / (droplet_count as f32 + 1.0);
            let hash_a = ((index.wrapping_mul(37).wrapping_add(17)) % 101) as f32 / 101.0;
            let hash_b = ((index.wrapping_mul(53).wrapping_add(29)) % 103) as f32 / 103.0;

            let radius = min_radius + (max_radius - min_radius) * hash_a;
            let x = (width * (0.08 + 0.84 * hash_a)).clamp(radius, width - radius);
            let y = (height * (0.10 + 0.80 * t + 0.05 * hash_b)).clamp(radius, height - radius);
            let opacity = opacity_min + (opacity_max - opacity_min) * hash_b;

            primitives.push(UiDrawPrimitive::Quad {
                rect: crate::ui_overlay::UiRect::new(
                    x - radius,
                    y - radius,
                    radius * 2.0,
                    radius * 2.0,
                ),
                color: ColorRgba::new(0.82, 0.94, 1.0, opacity * 0.42),
            });

            if lens.streaks_enabled && index % 3 == 0 {
                let streak_length = lens
                    .max_streak_length
                    .min(height * 0.28)
                    .max(radius)
                    .max(1.0);
                primitives.push(UiDrawPrimitive::Quad {
                    rect: crate::ui_overlay::UiRect::new(
                        x - radius * 0.18,
                        y + radius * 0.55,
                        radius * 0.36,
                        streak_length,
                    ),
                    color: ColorRgba::new(0.82, 0.94, 1.0, opacity * 0.18),
                });
            }
        }
    }

    if primitives.is_empty() {
        return Ok(());
    }

    let viewport = Viewport::from_offscreen(target);
    let mut color_batches = Vec::new();
    let mut ui_color_primitives = Vec::with_capacity(primitives.len());
    ui_color_primitives.extend(primitives);

    let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
    append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);
    color_batches.retain(|batch| !batch.vertices.is_empty());
    if color_batches.is_empty() {
        return Ok(());
    }

    let mut encoder = target
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("amigo-lens-droplets-overlay-encoder"),
        });

    for batch in &color_batches {
        let vertex_buffer = target
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("amigo-lens-droplets-overlay-vertices"),
                contents: vertices_as_bytes(&batch.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-lens-droplets-overlay-pass"),
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

            pass.set_pipeline(renderer.color_pipeline_for(batch.blend_mode));
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..batch.vertices.len() as u32, 0..1);
        }
    }

    target.queue.submit(Some(encoder.finish()));
    Ok(())
}

