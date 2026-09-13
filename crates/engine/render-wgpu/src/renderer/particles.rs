use crate::renderer::*;
use amigo_render_api::{
    LightSource2dCommon, Particle2dPrimitive, ParticleBlendMode2dPrimitive,
    ParticleLightMode2dPrimitive, ParticleLineAnchor2dPrimitive, ParticleShape2dPrimitive,
    RenderPrimitive2d, Renderable2dItem, VectorShape2dKindPrimitive, VectorShape2dPrimitive,
    VectorShape2dStylePrimitive, VectorShape2dViewportFit,
};

fn with_alpha_scale(color: ColorRgba, scale: f32) -> ColorRgba {
    ColorRgba::new(
        color.r,
        color.g,
        color.b,
        (color.a * scale.clamp(0.0, 1.0)).clamp(0.0, 1.0),
    )
}

fn push_particle_line_quad_gradient(
    vertices: &mut Vec<ColorVertex>,
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
    tail_color: ColorRgba,
    head_color: ColorRgba,
) {
    vertices.push(ColorVertex::new(a, tail_color));
    vertices.push(ColorVertex::new(b, head_color));
    vertices.push(ColorVertex::new(c, head_color));
    vertices.push(ColorVertex::new(a, tail_color));
    vertices.push(ColorVertex::new(c, head_color));
    vertices.push(ColorVertex::new(d, tail_color));
}

pub(crate) fn color_batch_vertices(
    batches: &mut Vec<ColorBatch>,
    blend_mode: ParticleBlendMode2d,
) -> &mut Vec<ColorVertex> {
    let needs_new_batch = batches
        .last()
        .map(|batch| batch.blend_mode != blend_mode)
        .unwrap_or(true);
    if needs_new_batch {
        batches.push(ColorBatch {
            blend_mode,
            vertices: Vec::new(),
        });
    }
    &mut batches
        .last_mut()
        .expect("color batch should exist after push")
        .vertices
}

pub(crate) fn particle_blend_mode(mode: ParticleBlendMode2dPrimitive) -> ParticleBlendMode2d {
    match mode {
        ParticleBlendMode2dPrimitive::Alpha => ParticleBlendMode2d::Alpha,
        ParticleBlendMode2dPrimitive::Additive => ParticleBlendMode2d::Additive,
        ParticleBlendMode2dPrimitive::Multiply => ParticleBlendMode2d::Multiply,
        ParticleBlendMode2dPrimitive::Screen => ParticleBlendMode2d::Screen,
    }
}

pub(crate) fn append_particle_primitive_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    particle: &Particle2dPrimitive,
    lights: &[ParticleRenderLight],
    lightmaps: &[LightMap2dSampler],
    light_sources: &[LightSource2dCommon],
    light_routes: &[LightRoute2dCommand],
) {
    let size = particle.size.max(0.0);
    if size <= f32::EPSILON || particle.color.a <= 0.0 {
        return;
    }
    let particle_color =
        lit_particle_color(particle, lights, lightmaps, light_sources, light_routes);
    let shape = match particle.shape {
        ParticleShape2dPrimitive::Circle { segments } => VectorShape2dPrimitive {
            shape: VectorShape2dKindPrimitive::Circle {
                radius: size * 0.5,
                segments: segments.max(3),
            },
            style: VectorShape2dStylePrimitive {
                stroke_color: particle_color,
                stroke_width: 0.0,
                fill_color: Some(particle_color),
            },
            transform: Transform2 {
                translation: particle.position,
                rotation_radians: particle.transform.rotation_radians,
                scale: particle.transform.scale,
            },
            viewport_fit: VectorShape2dViewportFit::Fixed,
            viewport_canvas_size: None,
            material: amigo_render_api::RenderMaterialBinding2d::none(
                amigo_material_api::MaterialCoverageKind2d::VectorCoverage,
            ),
        },
        ParticleShape2dPrimitive::Quad => {
            let half = size * 0.5;
            VectorShape2dPrimitive {
                shape: VectorShape2dKindPrimitive::Polygon {
                    points: vec![
                        Vec2::new(-half, -half),
                        Vec2::new(half, -half),
                        Vec2::new(half, half),
                        Vec2::new(-half, half),
                    ],
                },
                style: VectorShape2dStylePrimitive {
                    stroke_color: particle_color,
                    stroke_width: 0.0,
                    fill_color: Some(particle_color),
                },
                transform: Transform2 {
                    translation: particle.position,
                    rotation_radians: particle.transform.rotation_radians,
                    scale: particle.transform.scale,
                },
                viewport_fit: VectorShape2dViewportFit::Fixed,
                viewport_canvas_size: None,
                material: amigo_render_api::RenderMaterialBinding2d::none(
                    amigo_material_api::MaterialCoverageKind2d::VectorCoverage,
                ),
            }
        }
        ParticleShape2dPrimitive::Line { length } => {
            let mut line_length = length;
            let mut rotation_radians = particle.transform.rotation_radians;
            let mut tail_alpha = 1.0;
            let mut head_alpha = 1.0;
            if let Some(stretch) = particle.motion_stretch {
                let mut delta = Vec2::new(
                    particle.position.x - particle.previous_position.x,
                    particle.position.y - particle.previous_position.y,
                );
                if stretch.enabled {
                    if stretch.shutter_seconds > f32::EPSILON {
                        delta = Vec2::new(
                            particle.velocity.x * stretch.shutter_seconds,
                            particle.velocity.y * stretch.shutter_seconds,
                        );
                    }
                    delta = Vec2::new(
                        delta.x * stretch.velocity_scale.max(0.0),
                        delta.y * stretch.velocity_scale.max(0.0),
                    );
                    let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
                    if distance > f32::EPSILON {
                        line_length = (length + distance).min(stretch.max_length.max(length));
                        rotation_radians = delta.y.atan2(delta.x);
                    }
                    tail_alpha = stretch.tail_alpha;
                    head_alpha = stretch.head_alpha;
                }
            }
            let tail_color = with_alpha_scale(particle_color, tail_alpha);
            let head_color = with_alpha_scale(particle_color, head_alpha);
            return append_particle_line_vertices(
                vertices,
                viewport,
                camera,
                particle.position,
                rotation_radians,
                line_length,
                size.max(1.0),
                particle_line_anchor(particle.line_anchor),
                tail_color,
                head_color,
            );
        }
    };
    append_vector_primitive_vertices(vertices, viewport, camera, &shape, None, None, None);
}

pub(crate) fn particle_render_lights_from_renderables(
    renderables: &[Renderable2dItem],
) -> Vec<ParticleRenderLight> {
    let mut lights = Vec::new();
    let mut source_lights = BTreeMap::<String, ParticleRenderLight>::new();

    for renderable in renderables {
        let RenderPrimitive2d::ParticleBatch(particle) = &renderable.primitive else {
            continue;
        };
        let Some(light) = particle.light else {
            continue;
        };
        let radius = light.radius.max(0.0);
        let intensity = light.intensity.max(0.0);
        if radius <= f32::EPSILON || intensity <= 0.0 || particle.color.a <= 0.0 {
            continue;
        }

        match light.mode {
            ParticleLightMode2dPrimitive::Particle => {
                lights.push(ParticleRenderLight {
                    position: particle.position,
                    color: particle.color,
                    radius,
                    intensity,
                });
            }
            ParticleLightMode2dPrimitive::Source => {
                let Some(position) = particle.light_position else {
                    continue;
                };
                let candidate = ParticleRenderLight {
                    position,
                    color: particle.color,
                    radius,
                    intensity,
                };
                let replace = source_lights
                    .get(&particle.emitter_entity_name)
                    .map(|current| current.color.a < candidate.color.a)
                    .unwrap_or(true);
                if replace {
                    source_lights.insert(particle.emitter_entity_name.clone(), candidate);
                }
            }
        }
    }

    lights.extend(source_lights.into_values());
    lights
}

pub(crate) fn append_particle_light_primitive_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    particle: &Particle2dPrimitive,
) {
    let Some(light) = particle.light else {
        return;
    };
    let radius = light.radius.max(0.0);
    let intensity = light.intensity.max(0.0);
    if radius <= f32::EPSILON || intensity <= 0.0 || particle.color.a <= 0.0 {
        return;
    }

    let alpha = (particle.color.a * intensity).clamp(0.0, 1.0);
    let glow_color = ColorRgba::new(particle.color.r, particle.color.g, particle.color.b, alpha);
    let shape = VectorShape2dPrimitive {
        shape: VectorShape2dKindPrimitive::Circle {
            radius,
            segments: 16,
        },
        style: VectorShape2dStylePrimitive {
            stroke_color: glow_color,
            stroke_width: 0.0,
            fill_color: Some(glow_color),
        },
        transform: Transform2 {
            translation: particle.position,
            rotation_radians: 0.0,
            scale: Vec2::new(1.0, 1.0),
        },
        viewport_fit: VectorShape2dViewportFit::Fixed,
        viewport_canvas_size: None,
        material: amigo_render_api::RenderMaterialBinding2d::none(
            amigo_material_api::MaterialCoverageKind2d::VectorCoverage,
        ),
    };

    append_vector_primitive_vertices(vertices, viewport, camera, &shape, None, None, None);
}

fn particle_line_anchor(anchor: ParticleLineAnchor2dPrimitive) -> ParticleLineAnchor2d {
    match anchor {
        ParticleLineAnchor2dPrimitive::Center => ParticleLineAnchor2d::Center,
        ParticleLineAnchor2dPrimitive::Start => ParticleLineAnchor2d::Start,
        ParticleLineAnchor2dPrimitive::End => ParticleLineAnchor2d::End,
    }
}

fn line_points_for_anchor(length: f32, anchor: ParticleLineAnchor2d) -> (Vec2, Vec2) {
    let length = length.max(0.0);
    match anchor {
        ParticleLineAnchor2d::Center => {
            (Vec2::new(-length * 0.5, 0.0), Vec2::new(length * 0.5, 0.0))
        }
        ParticleLineAnchor2d::Start => (Vec2::ZERO, Vec2::new(length, 0.0)),
        ParticleLineAnchor2d::End => (Vec2::new(-length, 0.0), Vec2::ZERO),
    }
}

fn append_particle_line_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    position: Vec2,
    rotation_radians: f32,
    length: f32,
    width: f32,
    anchor: ParticleLineAnchor2d,
    tail_color: ColorRgba,
    head_color: ColorRgba,
) {
    let (start_local, end_local) = line_points_for_anchor(length, anchor);
    let transform = Transform2 {
        translation: position,
        rotation_radians,
        scale: Vec2::new(1.0, 1.0),
    };
    let start = transform_point_2d(start_local, transform);
    let end = transform_point_2d(end_local, transform);
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let segment_length = (dx * dx + dy * dy).sqrt();
    if segment_length <= f32::EPSILON {
        return;
    }

    let half_width = width * 0.5;
    let normal = Vec2::new(
        -dy / segment_length * half_width,
        dx / segment_length * half_width,
    );
    let a = Vec2::new(start.x + normal.x, start.y + normal.y);
    let b = Vec2::new(end.x + normal.x, end.y + normal.y);
    let c = Vec2::new(end.x - normal.x, end.y - normal.y);
    let d = Vec2::new(start.x - normal.x, start.y - normal.y);
    push_particle_line_quad_gradient(
        vertices,
        ndc_from_world_2d(a, camera, viewport),
        ndc_from_world_2d(b, camera, viewport),
        ndc_from_world_2d(c, camera, viewport),
        ndc_from_world_2d(d, camera, viewport),
        tail_color,
        head_color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_line_gradient_pushes_tail_and_head_alpha() {
        let mut vertices = Vec::new();
        push_particle_line_quad_gradient(
            &mut vertices,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
            ColorRgba::new(1.0, 1.0, 1.0, 0.0),
            ColorRgba::new(1.0, 1.0, 1.0, 1.0),
        );

        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].color[3], 0.0);
        assert_eq!(vertices[1].color[3], 1.0);
        assert_eq!(vertices[2].color[3], 1.0);
    }
}
