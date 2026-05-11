use crate::renderer::*;

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

pub(crate) fn append_particle_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    particle: &Particle2dDrawCommand,
    lights: &[ParticleRenderLight],
    lightmaps: &[LightMap2dSampler],
    global_lights: &[GlobalLight2dCommand],
    light_groups: &[LightGroup2dCommand],
    light_routes: &[LightRoute2dCommand],
) {
    let size = particle.size.max(0.0);
    if size <= f32::EPSILON || particle.color.a <= 0.0 {
        return;
    }
    let particle_color = lit_particle_color(
        particle,
        lights,
        lightmaps,
        global_lights,
        light_groups,
        light_routes,
    );
    let shape = match particle.shape {
        ParticleShape2d::Circle { segments } => VectorShape2d {
            kind: VectorShapeKind2d::Circle {
                radius: size * 0.5,
                segments: segments.max(3),
            },
            style: VectorStyle2d {
                stroke_color: particle_color,
                stroke_width: 0.0,
                fill_color: Some(particle_color),
            },
        },
        ParticleShape2d::Quad => {
            let half = size * 0.5;
            VectorShape2d {
                kind: VectorShapeKind2d::Polygon {
                    points: vec![
                        Vec2::new(-half, -half),
                        Vec2::new(half, -half),
                        Vec2::new(half, half),
                        Vec2::new(-half, half),
                    ],
                },
                style: VectorStyle2d {
                    stroke_color: particle_color,
                    stroke_width: 0.0,
                    fill_color: Some(particle_color),
                },
            }
        }
        ParticleShape2d::Line { length } => {
            let mut line_length = length;
            let mut rotation_radians = particle.transform.rotation_radians;
            if let Some(stretch) = particle.motion_stretch {
                let delta = Vec2::new(
                    particle.position.x - particle.previous_position.x,
                    particle.position.y - particle.previous_position.y,
                );
                let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
                if stretch.enabled && distance > f32::EPSILON {
                    line_length = (length + distance * stretch.velocity_scale)
                        .min(stretch.max_length.max(length));
                    rotation_radians = delta.y.atan2(delta.x);
                }
            }
            return append_particle_line_vertices(
                vertices,
                viewport,
                camera,
                particle.position,
                rotation_radians,
                line_length,
                size.max(1.0),
                particle.line_anchor,
                particle_color,
            );
        }
    };
    append_vector_shape_vertices(
        vertices,
        viewport,
        camera,
        Transform2 {
            translation: particle.position,
            rotation_radians: particle.transform.rotation_radians,
            scale: particle.transform.scale,
        },
        &shape,
    );
}

pub(crate) fn particle_render_lights(
    particles: &[Particle2dDrawCommand],
) -> Vec<ParticleRenderLight> {
    let mut lights = Vec::new();
    let mut source_lights = BTreeMap::<String, ParticleRenderLight>::new();

    for particle in particles {
        let Some(light) = particle.light else {
            continue;
        };
        let radius = light.radius.max(0.0);
        let intensity = light.intensity.max(0.0);
        if radius <= f32::EPSILON || intensity <= 0.0 || particle.color.a <= 0.0 {
            continue;
        }

        match light.mode {
            ParticleLightMode2d::Particle => {
                lights.push(ParticleRenderLight {
                    position: particle.position,
                    color: particle.color,
                    radius,
                    intensity,
                });
            }
            ParticleLightMode2d::Source => {
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

pub(crate) fn append_particle_light_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    particle: &Particle2dDrawCommand,
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
    let shape = VectorShape2d {
        kind: VectorShapeKind2d::Circle {
            radius,
            segments: 16,
        },
        style: VectorStyle2d {
            stroke_color: glow_color,
            stroke_width: 0.0,
            fill_color: Some(glow_color),
        },
    };

    append_vector_shape_vertices(
        vertices,
        viewport,
        camera,
        Transform2 {
            translation: particle.position,
            rotation_radians: 0.0,
            scale: Vec2::new(1.0, 1.0),
        },
        &shape,
    );
}

fn line_points_for_anchor(length: f32, anchor: ParticleLineAnchor2d) -> (Vec2, Vec2) {
    let length = length.max(0.0);
    match anchor {
        ParticleLineAnchor2d::Center => (Vec2::new(-length * 0.5, 0.0), Vec2::new(length * 0.5, 0.0)),
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
    color: ColorRgba,
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
    let normal = Vec2::new(-dy / segment_length * half_width, dx / segment_length * half_width);
    let a = Vec2::new(start.x + normal.x, start.y + normal.y);
    let b = Vec2::new(end.x + normal.x, end.y + normal.y);
    let c = Vec2::new(end.x - normal.x, end.y - normal.y);
    let d = Vec2::new(start.x - normal.x, start.y - normal.y);
    push_quad(
        vertices,
        ndc_from_world_2d(a, camera, viewport),
        ndc_from_world_2d(b, camera, viewport),
        ndc_from_world_2d(c, camera, viewport),
        ndc_from_world_2d(d, camera, viewport),
        color,
    );
}
