use crate::renderer::*;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn append_mesh_triangles(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    light_settings: amigo_render_api::Light3dRenderSettings,
    transform: Transform3,
    geometry: Option<&amigo_render_api::MeshGeometry3d>,
    base_color: ColorRgba,
    render_order: i32,
) {
    if let Some(geometry) = geometry {
        for indices in geometry.indices.chunks_exact(3) {
            let Some(points) = indices
                .iter()
                .map(|index| geometry.positions.get(*index as usize).copied())
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            append_world_triangle(
                triangles,
                viewport,
                camera,
                camera_settings,
                light_settings,
                [
                    transform_point_3d(
                        Vec3::new(points[0][0], points[0][1], points[0][2]),
                        transform,
                    ),
                    transform_point_3d(
                        Vec3::new(points[1][0], points[1][1], points[1][2]),
                        transform,
                    ),
                    transform_point_3d(
                        Vec3::new(points[2][0], points[2][1], points[2][2]),
                        transform,
                    ),
                ],
                base_color,
                render_order,
            );
        }
        return;
    }
    let corners = [
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
    ]
    .map(|point| transform_point_3d(point, transform));
    let faces = [
        [[0usize, 2usize, 1usize], [0usize, 3usize, 2usize]],
        [[4usize, 5usize, 6usize], [4usize, 6usize, 7usize]],
        [[0usize, 1usize, 5usize], [0usize, 5usize, 4usize]],
        [[2usize, 3usize, 7usize], [2usize, 7usize, 6usize]],
        [[1usize, 2usize, 6usize], [1usize, 6usize, 5usize]],
        [[3usize, 0usize, 4usize], [3usize, 4usize, 7usize]],
    ];

    for face_triangles in faces {
        for [a, b, c] in face_triangles {
            append_world_triangle(
                triangles,
                viewport,
                camera,
                camera_settings,
                light_settings,
                [corners[a], corners[b], corners[c]],
                base_color,
                render_order,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_world_triangle(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    light_settings: amigo_render_api::Light3dRenderSettings,
    world: [Vec3; 3],
    base_color: ColorRgba,
    render_order: i32,
) {
    let projected = world.map(|point| {
        project_point_with_camera(
            point,
            camera,
            *viewport,
            camera_settings.fov_y_degrees,
            camera_settings.near_clip,
            camera_settings.far_clip,
        )
    });
    let [Some(a), Some(b), Some(c)] = projected else {
        return;
    };
    let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
    let center = triangle_center(world);
    if dot(normal, sub(camera.translation, center)) <= 0.0
        || !projected_triangle_is_sane([a.position, b.position, c.position])
    {
        return;
    }
    let light_dir = normalize(Vec3::new(
        -light_settings.direction.x,
        -light_settings.direction.y,
        -light_settings.direction.z,
    ));
    let lit = dot(normal, light_dir).max(0.0) * light_settings.intensity.max(0.0);
    let brightness: f32 = (light_settings.ambient.max(0.0) + lit).clamp(0.0, 1.25);
    let shaded = force_opaque(modulate_color(base_color, brightness));
    triangles.push(ProjectedTriangle {
        depths: [a.depth, b.depth, c.depth],
        points: [a.position, b.position, c.position],
        color: multiply_color(shaded, light_settings.color),
        depth: (a.depth + b.depth + c.depth) / 3.0,
        render_order,
        hatch: None,
        ink: None,
    });
}

/// Projects a model-space mesh every frame while deriving only its visible
/// boundary/silhouette edges. This deliberately avoids the heavyweight NPR
/// drawing pipeline (surface proxies, hatching and stroke tessellation).
pub(crate) fn append_npr_mesh_triangles(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    transform: Transform3,
    entity_id: u64,
    geometry: Option<&amigo_render_api::MeshGeometry3d>,
    style: amigo_render_api::NprMeshStyle,
    history: &mut NprStrokeHistory,
) {
    let Some(geometry) = geometry else {
        append_mesh_triangles(
            triangles,
            viewport,
            camera,
            camera_settings,
            amigo_render_api::Light3dRenderSettings::default(),
            transform,
            geometry,
            ColorRgba::new(style.fill[0], style.fill[1], style.fill[2], style.fill[3]),
            0,
        );
        return;
    };

    let world_positions = geometry
        .positions
        .iter()
        .map(|point| transform_point_3d(Vec3::new(point[0], point[1], point[2]), transform))
        .collect::<Vec<_>>();
    let (mut minimum, mut maximum) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for point in geometry.reference_positions.iter() {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(point[axis]);
            maximum[axis] = maximum[axis].max(point[axis]);
        }
    }
    let reference_extent = (0..3).map(|axis| maximum[axis] - minimum[axis]).fold(0.0_f32, f32::max);
    let hatch_scale = 256.0 / reference_extent.max(1.0e-5);
    let mut faces = Vec::with_capacity(geometry.indices.len() / 3);
    let mut front_faces = Vec::with_capacity(geometry.indices.len() / 3);
    let mut face_normals = Vec::with_capacity(geometry.indices.len() / 3);
    for indices in geometry.indices.chunks_exact(3) {
        let [a, b, c] = [indices[0], indices[1], indices[2]];
        let Some(world) = [a, b, c]
            .map(|index| world_positions.get(index as usize).copied())
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .map(|points| [points[0], points[1], points[2]])
        else {
            faces.push(None);
            front_faces.push(false);
            face_normals.push(None);
            continue;
        };
        let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
        let front = dot(normal, sub(camera.translation, triangle_center(world))) > 0.0;
        faces.push(Some(world));
        front_faces.push(front);
        face_normals.push(Some(normal));
    }

    let fill = ColorRgba::new(style.fill[0], style.fill[1], style.fill[2], style.fill[3]);
    let ink = ColorRgba::new(style.ink[0], style.ink[1], style.ink[2], style.ink[3]);
    let mut projected_outline_edges = BTreeMap::new();
    for (edge_index, edge) in geometry.topology_edges.iter().enumerate() {
        if !style.join_strokes {
            break;
        }
        let owner = match edge.faces {
            [Some(face), None] if front_faces.get(face as usize).copied().unwrap_or(false) => {
                Some(face)
            }
            [None, Some(face)] if front_faces.get(face as usize).copied().unwrap_or(false) => {
                Some(face)
            }
            [Some(left), Some(right)]
                if front_faces.get(left as usize) != front_faces.get(right as usize) =>
            {
                if front_faces.get(left as usize).copied().unwrap_or(false) {
                    Some(left)
                } else {
                    Some(right)
                }
            }
            _ => None,
        };
        let Some(owner) = owner else { continue };
        let Some(edge_style) = npr_edge_style(
            edge_index as u32,
            owner,
            geometry,
            &front_faces,
            &face_normals,
            style,
            ink,
            entity_id,
        ) else {
            continue;
        };
        let Some([a_world, b_world]) = edge
            .vertices
            .map(|index| world_positions.get(index as usize).copied())
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .and_then(|points| (points.len() == 2).then_some([points[0], points[1]]))
        else {
            continue;
        };
        let projected = [a_world, b_world].map(|point| {
            project_point_with_camera(
                point,
                camera,
                *viewport,
                camera_settings.fov_y_degrees,
                camera_settings.near_clip,
                camera_settings.far_clip,
            )
        });
        let Some([a, b]) = projected
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .and_then(|points| (points.len() == 2).then_some([points[0], points[1]]))
        else {
            continue;
        };
        let mut edge_style = edge_style;
        edge_style.depths = [a.depth, b.depth];
        projected_outline_edges.insert(
            edge_index as u32,
            ProjectedOutlineEdge {
                vertices: edge.vertices,
                points: [a.position, b.position],
                depth: (a.depth + b.depth) * 0.5,
                style: edge_style,
            },
        );
        history.remember(
            entity_id,
            edge_index as u32,
            [a.position, b.position],
            edge_style,
            (a.depth + b.depth) * 0.5,
            style.artistic_frame,
        );
    }
    let outline_chains = build_outline_chains(&projected_outline_edges);
    let chain_edge_ids = outline_chains
        .iter()
        .flat_map(|(_, _, edge_ids)| edge_ids.iter().copied())
        .collect::<BTreeSet<_>>();
    for (points, base_style, edge_ids) in outline_chains {
        let total_length = points
            .windows(2)
            .map(|pair| {
                let dx = pair[1].x - pair[0].x;
                let dy = pair[1].y - pair[0].y;
                (dx * dx + dy * dy).sqrt()
            })
            .sum::<f32>();
        if total_length <= 0.001 || points.len() < 2 {
            continue;
        }
        let mut travelled = 0.0;
        for (segment_index, pair) in points.windows(2).enumerate() {
            let dx = pair[1].x - pair[0].x;
            let dy = pair[1].y - pair[0].y;
            let length = (dx * dx + dy * dy).sqrt();
            let mut segment_style = base_style;
            let source_edge = &projected_outline_edges[&edge_ids[segment_index]];
            segment_style.depths = if pair[0] == source_edge.points[0] {
                source_edge.style.depths
            } else {
                [source_edge.style.depths[1], source_edge.style.depths[0]]
            };
            segment_style.seed = mix_npr_seed(
                entity_id,
                edge_ids[0],
                style.artistic_frame,
            );
            segment_style.stable_seed = mix_npr_seed(entity_id, edge_ids[0], 0);
            segment_style.gesture_t0 = travelled / total_length;
            travelled += length;
            segment_style.gesture_t1 = travelled / total_length;
            triangles.push(ProjectedTriangle {
                depths: [segment_style.depths[0], segment_style.depths[1], segment_style.depths[1]],
                points: [pair[0], pair[1], pair[1]],
                color: ColorRgba::new(0.0, 0.0, 0.0, 0.0),
                depth: projected_outline_edges
                    .get(&edge_ids[segment_index])
                    .map(|edge| edge.depth)
                    .unwrap_or(0.0),
                // A joined contour belongs to world geometry. Drawing it in
                // an overlay layer exposes it through nearer occluders.
                render_order: 0,
                hatch: None,
                ink: Some(ProjectedTriangleInk {
                    edges: [Some(segment_style), None, None],
                }),
            });
        }
    }
    for (face_index, world) in faces.into_iter().enumerate() {
        let Some(world) = world else {
            continue;
        };
        if !front_faces[face_index] {
            continue;
        }
        let projected = world.map(|point| {
            project_point_with_camera(
                point,
                camera,
                *viewport,
                camera_settings.fov_y_degrees,
                camera_settings.near_clip,
                camera_settings.far_clip,
            )
        });
        let [Some(pa), Some(pb), Some(pc)] = projected else {
            continue;
        };
        if !projected_triangle_is_sane([pa.position, pb.position, pc.position]) {
            continue;
        }
        let edge_style = |edge_index: u32| {
            npr_edge_style(
                edge_index,
                face_index as u32,
                geometry,
                &front_faces,
                &face_normals,
                style,
                ink,
                entity_id,
            )
        };
        let edge_mask = geometry
            .triangle_edges
            .get(face_index)
            .map(|edges| {
                edges.map(|edge_index| {
                    if chain_edge_ids.contains(&edge_index) {
                        None
                    } else {
                        edge_style(edge_index)
                    }
                })
            })
            .unwrap_or([None; 3]);
        if let Some(edge_indices) = geometry.triangle_edges.get(face_index) {
            for (slot, edge_style) in edge_mask.into_iter().enumerate() {
                let Some(edge_style) = edge_style else {
                    continue;
                };
                let edge_index = edge_indices[slot];
                let Some(edge) = geometry.topology_edges.get(edge_index as usize) else {
                    continue;
                };
                let Some([a, b]) = edge
                    .vertices
                    .map(|index| world_positions.get(index as usize).copied())
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .and_then(|points| (points.len() == 2).then_some([points[0], points[1]]))
                else {
                    continue;
                };
                let projected = [a, b].map(|point| {
                    project_point_with_camera(
                        point,
                        camera,
                        *viewport,
                        camera_settings.fov_y_degrees,
                        camera_settings.near_clip,
                        camera_settings.far_clip,
                    )
                });
                let Some([a, b]) = projected
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .and_then(|points| (points.len() == 2).then_some([points[0], points[1]]))
                else {
                    continue;
                };
                if edge_style.persistent {
                    let mut edge_style = edge_style;
                    edge_style.depths = [a.depth, b.depth];
                    history.remember(
                        entity_id,
                        edge_index,
                        [a.position, b.position],
                        edge_style,
                        (a.depth + b.depth) * 0.5,
                        style.artistic_frame,
                    );
                }
            }
        }
        let normal = face_normals[face_index].unwrap_or(Vec3::new(0.0, 1.0, 0.0));
        let light = normalize(Vec3::new(
            style.light_direction[0],
            style.light_direction[1],
            style.light_direction[2],
        ));
        let illumination = dot(normal, light).max(0.0);
        let material_tone = geometry
            .material_indices
            .get(face_index)
            .and_then(|index| geometry.material_colors.get(*index as usize))
            .map(|color| {
                (0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2])
                    .clamp(0.0, 1.0)
            })
            .unwrap_or(1.0);
        let shadow_amount = ((0.52 - illumination) / 0.52
            + (1.0 - material_tone) * 0.28)
            .clamp(0.0, 1.0);
        let fill_tone = 0.76 + material_tone * 0.24;
        let fill = ColorRgba::new(
            fill.r * fill_tone,
            fill.g * fill_tone,
            fill.b * fill_tone,
            fill.a,
        );
        let hatch = (style.hatching_enabled
            && style.hatching_density > 0.01
            && shadow_amount > 0.08)
            .then_some(ProjectedTriangleHatch {
                surface_coordinates: hatch_surface_coordinates(geometry, face_index, hatch_scale),
                depths: [pa.depth, pb.depth, pc.depth],
                pencil_grain: style.pencil_grain,
                wobble_pixels: (style.wobble_pixels * 0.32).clamp(0.0, 1.5),
                stroke_segments: style.stroke_segments.max(2),
                pressure_variation: (style.pressure_variation * 0.7).clamp(0.02, 0.45),
                hardness: style.hardness,
                dryness: style.dryness,
                color: ColorRgba::new(
                    style.shadow[0],
                    style.shadow[1],
                    style.shadow[2],
                    style.shadow[3]
                        * style.hatching_density.clamp(0.0, 1.0)
                        * shadow_amount
                        * (0.78 + (1.0 - material_tone) * 0.42),
                ),
                angle_degrees: style.hatching_angle_degrees,
                spacing_pixels: style.hatching_spacing_pixels.max(4.0),
                cross: style.hatching_cross.clamp(0.0, 1.0) * shadow_amount,
                width_pixels: (style.crease_width_pixels * 0.55).clamp(0.45, 1.25),
                seed: mix_npr_seed(entity_id, 0, 0),
            });
        triangles.push(ProjectedTriangle {
            points: [pa.position, pb.position, pc.position],
            depths: [pa.depth, pb.depth, pc.depth],
            color: fill,
            depth: (pa.depth + pb.depth + pc.depth) / 3.0,
            render_order: 0,
            hatch,
            ink: Some(ProjectedTriangleInk {
                edges: edge_mask,
            }),
        });
    }
    if style.temporal {
      for (points, edge_style, depth) in
          history.persistent_for(entity_id, style.artistic_frame)
      {
        triangles.push(ProjectedTriangle {
            points: [points[0], points[1], points[1]],
            depths: [edge_style.depths[0], edge_style.depths[1], edge_style.depths[1]],
            color: ColorRgba::new(0.0, 0.0, 0.0, 0.0),
            depth,
            render_order: 0,
            hatch: None,
            ink: Some(ProjectedTriangleInk {
                edges: [Some(edge_style), None, None],
            }),
        });
      }
    }
}

/// A stable box projection of the reference surface. The selected plane cannot
/// change when skinning changes the current face normal or when the camera moves.
fn hatch_surface_coordinates(geometry: &amigo_render_api::MeshGeometry3d, face: usize, drawing_scale: f32) -> [Vec2; 3] {
    let indices = &geometry.indices[face * 3..face * 3 + 3];
    let p = indices.iter().map(|&index| geometry.reference_positions[index as usize])
        .map(|p| Vec3::new(p[0], p[1], p[2])).collect::<Vec<_>>();
    let normal = cross(sub(p[1], p[0]), sub(p[2], p[0]));
    let project = |p: Vec3| {
        let uv = if normal.x.abs() >= normal.y.abs() && normal.x.abs() >= normal.z.abs() {
            Vec2::new(p.z, p.y)
        } else if normal.y.abs() >= normal.z.abs() {
            Vec2::new(p.x, p.z)
        } else {
            Vec2::new(p.x, p.y)
        };
        // The reference drawing spans 256 units over the longest source axis,
        // independent of asset unit conventions and of camera zoom.
        Vec2::new(uv.x * drawing_scale, uv.y * drawing_scale)
    };
    [project(p[0]), project(p[1]), project(p[2])]
}

#[derive(Clone, Copy)]
struct ProjectedOutlineEdge {
    vertices: [u32; 2],
    points: [Vec2; 2],
    depth: f32,
    style: ProjectedInkEdge,
}

fn npr_edge_style(
    edge_index: u32,
    face_index: u32,
    geometry: &amigo_render_api::MeshGeometry3d,
    front_faces: &[bool],
    face_normals: &[Option<Vec3>],
    style: amigo_render_api::NprMeshStyle,
    ink: ColorRgba,
    entity_id: u64,
) -> Option<ProjectedInkEdge> {
    let edge = geometry.topology_edges.get(edge_index as usize)?;
    let (width, wobble_scale, overstroke_scale, persistent) = match edge.faces {
        [Some(_), None] | [None, Some(_)] if style.draw_silhouettes => {
            (style.boundary_width_pixels, 1.0, 1.0, true)
        }
        [Some(left), Some(right)]
            if style.draw_silhouettes
                && front_faces.get(left as usize) != front_faces.get(right as usize) =>
        {
            (style.outline_width_pixels, 1.0, 1.0, true)
        }
        [Some(left), Some(right)]
            if style.draw_material_seams
                && geometry
                    .material_indices
                    .get(left as usize)
                    != geometry.material_indices.get(right as usize) =>
        {
            (style.boundary_width_pixels * 0.72, 0.55, 0.55, false)
        }
        [Some(left), Some(right)]
            if style.draw_contact_lines
                && front_faces.get(left as usize).copied().unwrap_or(false)
                && front_faces.get(right as usize).copied().unwrap_or(false)
                && face_normals
                    .get(left as usize)
                    .copied()
                    .flatten()
                    .zip(face_normals.get(right as usize).copied().flatten())
                    .map(|(left_normal, right_normal)| {
                        let angle = dot(left_normal, right_normal).clamp(-1.0, 1.0).acos();
                        let light = normalize(Vec3::new(
                            style.light_direction[0],
                            style.light_direction[1],
                            style.light_direction[2],
                        ));
                        let value_delta =
                            (dot(left_normal, light) - dot(right_normal, light)).abs();
                        value_delta >= 0.24 && angle < style.crease_angle_radians
                    })
                    .unwrap_or(false) =>
        {
            let left_normal = face_normals.get(left as usize).copied().flatten()?;
            let right_normal = face_normals.get(right as usize).copied().flatten()?;
            let light = normalize(Vec3::new(
                style.light_direction[0],
                style.light_direction[1],
                style.light_direction[2],
            ));
            let value_delta = (dot(left_normal, light) - dot(right_normal, light)).abs();
            (value_delta >= 0.24)
                .then_some((style.contact_line_width_pixels, 0.28, 0.0, false))?
        }
        [Some(left), Some(right)]
            if style.draw_form_lines
                && front_faces.get(left as usize).copied().unwrap_or(false)
                && front_faces.get(right as usize).copied().unwrap_or(false)
                && face_normals
                    .get(left as usize)
                    .copied()
                    .flatten()
                    .zip(face_normals.get(right as usize).copied().flatten())
                    .map(|(left_normal, right_normal)| {
                        let angle = dot(left_normal, right_normal).clamp(-1.0, 1.0).acos();
                        angle >= 0.22 && angle < style.crease_angle_radians
                    })
                    .unwrap_or(false) =>
        {
            let left_normal = face_normals.get(left as usize).copied().flatten()?;
            let right_normal = face_normals.get(right as usize).copied().flatten()?;
            let angle = dot(left_normal, right_normal).clamp(-1.0, 1.0).acos();
            // Form lines occupy the middle range: flatter than an authored
            // crease, but strong enough to describe a readable change of form.
            (angle >= 0.22 && angle < style.crease_angle_radians)
                .then_some((style.form_line_width_pixels, 0.34, 0.0, false))?
        }
        [Some(left), Some(right)]
            if style.draw_creases
                && face_index == left.min(right)
                && front_faces.get(left as usize).copied().unwrap_or(false)
                && front_faces.get(right as usize).copied().unwrap_or(false) =>
        {
            let left_normal = face_normals.get(left as usize).copied().flatten()?;
            let right_normal = face_normals.get(right as usize).copied().flatten()?;
            let angle = dot(left_normal, right_normal).clamp(-1.0, 1.0).acos();
            (angle >= style.crease_angle_radians)
                .then_some((style.crease_width_pixels, 0.2, 0.0, false))?
        }
        _ => return None,
    };
    (width > 0.0).then_some(ProjectedInkEdge {
        depths: [1.0; 2],
        color: ink,
        width_pixels: width * (0.65 + style.pressure.clamp(0.0, 1.0) * 0.7),
        wobble_pixels: style.wobble_pixels.max(0.0)
            * style.redraw_strength.clamp(0.0, 1.0)
            * wobble_scale,
        stroke_segments: style.stroke_segments,
        pressure_variation: style.pressure_variation.clamp(0.0, 1.0),
        gesture_t0: 0.0,
        gesture_t1: 1.0,
        taper: style.taper.clamp(0.0, 1.0),
        overstroke: style.overstroke.clamp(0.0, 1.0) * overstroke_scale,
        hardness: style.hardness.clamp(0.0, 1.0),
        dryness: style.dryness.clamp(0.0, 1.0),
        pencil_grain: style.pencil_grain.clamp(0.0, 1.0),
        persistent: persistent && style.temporal,
        stable_seed: mix_npr_seed(entity_id, edge_index, 0),
        seed: mix_npr_seed(entity_id, edge_index, style.artistic_frame),
    })
}

fn build_outline_chains(
    edges: &BTreeMap<u32, ProjectedOutlineEdge>,
) -> Vec<(Vec<Vec2>, ProjectedInkEdge, Vec<u32>)> {
    let mut adjacency = BTreeMap::<u32, Vec<u32>>::new();
    for (&edge_index, edge) in edges {
        adjacency
            .entry(edge.vertices[0])
            .or_default()
            .push(edge_index);
        adjacency
            .entry(edge.vertices[1])
            .or_default()
            .push(edge_index);
    }
    let mut remaining = edges.keys().copied().collect::<BTreeSet<_>>();
    let mut chains = Vec::new();
    while let Some(&seed_edge) = remaining.iter().next() {
        let seed = edges[&seed_edge];
        let start = if adjacency
            .get(&seed.vertices[0])
            .map_or(0, Vec::len)
            != 2
        {
            seed.vertices[0]
        } else {
            seed.vertices[1]
        };
        let mut current = start;
        let mut points = Vec::new();
        let mut edge_ids = Vec::new();
        let mut first = true;
        loop {
            let next_edge = adjacency
                .get(&current)
                .into_iter()
                .flatten()
                .copied()
                .find(|edge_index| remaining.contains(edge_index));
            let Some(next_edge) = next_edge else { break };
            let edge = edges[&next_edge];
            if first {
                let point = if edge.vertices[0] == current {
                    edge.points[0]
                } else {
                    edge.points[1]
                };
                points.push(point);
                first = false;
            }
            let (next_vertex, next_point) = if edge.vertices[0] == current {
                (edge.vertices[1], edge.points[1])
            } else {
                (edge.vertices[0], edge.points[0])
            };
            remaining.remove(&next_edge);
            edge_ids.push(next_edge);
            points.push(next_point);
            current = next_vertex;
            if current == start {
                break;
            }
        }
        if !edge_ids.is_empty() && points.len() >= 2 {
            chains.push((points, seed.style, edge_ids));
        }
    }
    chains
}

fn mix_npr_seed(entity_id: u64, edge_index: u32, artistic_frame: u64) -> u64 {
    let mut value = entity_id
        ^ u64::from(edge_index).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ artistic_frame.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn triangle_center(points: [Vec3; 3]) -> Vec3 {
    Vec3::new(
        (points[0].x + points[1].x + points[2].x) / 3.0,
        (points[0].y + points[1].y + points[2].y) / 3.0,
        (points[0].z + points[1].z + points[2].z) / 3.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hatch_lattice_is_shared_at_seams_and_held_through_deformation() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let mut geometry = amigo_render_api::MeshGeometry3d {
            reference_positions: positions.clone().into(), positions,
            indices: vec![0, 1, 2, 0, 2, 3], material_indices: vec![0, 0], material_colors: vec![[1.0; 4]].into(), topology_edges: vec![], triangle_edges: vec![],
        };
        let first = hatch_surface_coordinates(&geometry, 0, 256.0);
        let second = hatch_surface_coordinates(&geometry, 1, 256.0);
        assert_eq!(first[0], second[0]);
        assert_eq!(first[2], second[1]);
        // Bend the current surface into another projection plane. The drawing
        // remains attached to the original vertices instead of being reseeded.
        geometry.positions[2] = [0.0, 1.0, 2.0];
        assert_eq!(hatch_surface_coordinates(&geometry, 0, 256.0), first);
        assert_eq!(hatch_surface_coordinates(&geometry, 1, 256.0), second);
    }

    #[test]
    fn isolated_joined_contour_is_emitted_in_world_depth_order() {
        let geometry = amigo_render_api::MeshGeometry3d {
            positions: vec![[-1.0, -1.0, -3.0], [1.0, -1.0, -3.0], [0.0, 1.0, -3.0]],
            reference_positions: vec![[-1.0, -1.0, -3.0], [1.0, -1.0, -3.0], [0.0, 1.0, -3.0]].into(),
            indices: vec![0, 1, 2],
            material_indices: vec![0],
            material_colors: vec![[1.0; 4]].into(),
            topology_edges: vec![amigo_render_api::MeshTopologyEdge3d { vertices: [0, 1], faces: [Some(0), None] }],
            triangle_edges: vec![[0; 3]],
        };
        let style = amigo_render_api::NprMeshStyle {
            background: [1.0; 4], fill: [1.0; 4], shadow: [0.2; 4], ink: [0.1; 4],
            light_direction: [0.0, 0.0, 1.0], artistic_frame: 0, temporal: false,
            draw_silhouettes: true, draw_creases: false, draw_material_seams: false,
            draw_form_lines: false, draw_contact_lines: false,
            outline_width_pixels: 2.0, boundary_width_pixels: 2.0, crease_width_pixels: 1.0,
            form_line_width_pixels: 0.7, contact_line_width_pixels: 0.6,
            crease_angle_radians: 0.8, wobble_pixels: 0.5, stroke_segments: 3,
            join_strokes: true, pressure_variation: 0.2, taper: 0.2, overstroke: 0.0,
            pressure: 0.8, hardness: 0.5, dryness: 0.1, pencil_grain: 0.5,
            redraw_strength: 0.5, hatching_enabled: false, hatching_angle_degrees: 30.0,
            hatching_spacing_pixels: 10.0, hatching_cross: 0.1, hatching_density: 0.2,
        };
        let mut triangles = Vec::new();
        append_npr_mesh_triangles(&mut triangles, &Viewport::from_dimensions(512.0, 512.0),
            Transform3::default(), Default::default(), Transform3::default(), 1,
            Some(&geometry), style, &mut NprStrokeHistory::default());
        let strokes: Vec<_> = triangles.iter().filter(|t| t.color.a == 0.0 && t.ink.is_some()).collect();
        assert_eq!(strokes.len(), 1, "a one-edge gesture must survive chain extraction");
        assert_eq!(strokes[0].render_order, 0, "world strokes must not bypass nearer geometry");
        assert!((strokes[0].depth - 3.0).abs() < 1.0e-6);
    }

    #[test]
    fn material_luminance_changes_npr_tone_without_replacing_graphite_palette() {
        let geometry = amigo_render_api::MeshGeometry3d {
            positions: vec![[-1.0, -1.0, -3.0], [1.0, -1.0, -3.0], [0.0, 1.0, -3.0]],
            reference_positions: vec![[-1.0, -1.0, -3.0], [1.0, -1.0, -3.0], [0.0, 1.0, -3.0]].into(),
            indices: vec![0, 1, 2], material_indices: vec![0],
            material_colors: vec![[0.08, 0.08, 0.08, 1.0]].into(),
            topology_edges: vec![], triangle_edges: vec![],
        };
        let style = amigo_render_api::NprMeshStyle {
            background: [1.0; 4], fill: [0.8, 0.7, 0.6, 1.0], shadow: [0.1; 4], ink: [0.1; 4],
            light_direction: [0.0, 0.0, 1.0], artistic_frame: 0, temporal: false,
            draw_silhouettes: false, draw_creases: false, draw_material_seams: false,
            draw_form_lines: false, draw_contact_lines: false,
            outline_width_pixels: 2.0, boundary_width_pixels: 2.0, crease_width_pixels: 1.0,
            form_line_width_pixels: 0.7, contact_line_width_pixels: 0.6,
            crease_angle_radians: 0.8, wobble_pixels: 0.0, stroke_segments: 2,
            join_strokes: false, pressure_variation: 0.0, taper: 0.0, overstroke: 0.0,
            pressure: 1.0, hardness: 1.0, dryness: 0.0, pencil_grain: 0.0,
            redraw_strength: 0.0, hatching_enabled: false, hatching_angle_degrees: 0.0,
            hatching_spacing_pixels: 10.0, hatching_cross: 0.0, hatching_density: 0.0,
        };
        let mut triangles = Vec::new();
        append_npr_mesh_triangles(&mut triangles, &Viewport::from_dimensions(512.0, 512.0),
            Transform3::default(), Default::default(), Transform3::default(), 2,
            Some(&geometry), style, &mut NprStrokeHistory::default());
        let fill = triangles.iter().find(|triangle| triangle.color.a > 0.0).unwrap();
        assert!(fill.color.r < 0.8 && fill.color.g < 0.7 && fill.color.b < 0.6);
    }

    #[test]
    fn suggestive_form_line_uses_the_explicit_middle_angle_band() {
        let geometry = amigo_render_api::MeshGeometry3d {
            positions: vec![],
            reference_positions: vec![].into(),
            indices: vec![],
            material_indices: vec![],
            material_colors: vec![].into(),
            topology_edges: vec![amigo_render_api::MeshTopologyEdge3d {
                vertices: [0, 1],
                faces: [Some(0), Some(1)],
            }],
            triangle_edges: vec![],
        };
        let mut style = amigo_render_api::NprMeshStyle {
            background: [1.0; 4], fill: [1.0; 4], shadow: [0.2; 4], ink: [0.1; 4],
            light_direction: [0.0, 0.0, 1.0], artistic_frame: 0, temporal: false,
            draw_silhouettes: false, draw_creases: false, draw_material_seams: false,
            draw_form_lines: true, draw_contact_lines: false,
            outline_width_pixels: 2.0, boundary_width_pixels: 2.0,
            crease_width_pixels: 1.0, form_line_width_pixels: 0.65,
            contact_line_width_pixels: 0.6,
            crease_angle_radians: 0.8, wobble_pixels: 0.5, stroke_segments: 3,
            join_strokes: false, pressure_variation: 0.2, taper: 0.2, overstroke: 0.0,
            pressure: 0.8, hardness: 0.5, dryness: 0.1, pencil_grain: 0.5,
            redraw_strength: 0.5, hatching_enabled: false, hatching_angle_degrees: 30.0,
            hatching_spacing_pixels: 10.0, hatching_cross: 0.1, hatching_density: 0.2,
        };
        let half_angle = 0.5_f32;
        let normals = vec![
            Some(Vec3::new(0.0, 0.0, 1.0)),
            Some(Vec3::new(half_angle.sin(), 0.0, half_angle.cos())),
        ];
        let edge = npr_edge_style(
            0,
            0,
            &geometry,
            &[true, true],
            &normals,
            style,
            ColorRgba::new(0.1, 0.1, 0.1, 1.0),
            7,
        )
        .expect("middle-angle form edge should be emitted");
        assert!((edge.width_pixels - 0.65 * (0.65 + 0.8 * 0.7)).abs() < 1.0e-5);
        style.draw_form_lines = false;
        assert!(npr_edge_style(
            0,
            0,
            &geometry,
            &[true, true],
            &normals,
            style,
            ColorRgba::new(0.1, 0.1, 0.1, 1.0),
            7,
        )
        .is_none());
        style.draw_contact_lines = true;
        style.light_direction = [1.0, 0.0, 0.0];
        let contact = npr_edge_style(
            0,
            0,
            &geometry,
            &[true, true],
            &normals,
            style,
            ColorRgba::new(0.1, 0.1, 0.1, 1.0),
            7,
        )
        .expect("lighting transition should emit a contact line");
        assert!((contact.width_pixels - 0.6 * (0.65 + 0.8 * 0.7)).abs() < 1.0e-5);
    }

    fn outline_edge(vertices: [u32; 2], points: [Vec2; 2]) -> ProjectedOutlineEdge {
        ProjectedOutlineEdge {
            vertices,
            points,
            depth: 0.5,
            style: ProjectedInkEdge {
                depths: [1.0; 2],
                color: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
                width_pixels: 1.0,
                wobble_pixels: 1.0,
                stroke_segments: 4,
                pressure_variation: 0.2,
                gesture_t0: 0.0,
                gesture_t1: 1.0,
                taper: 0.1,
                overstroke: 0.1,
                hardness: 0.5,
                dryness: 0.0,
                pencil_grain: 0.0,
                persistent: true,
                stable_seed: 1,
                seed: 2,
            },
        }
    }

    #[test]
    fn outline_chain_joins_edges_at_shared_vertices() {
        let mut edges = BTreeMap::new();
        edges.insert(
            4,
            outline_edge([0, 1], [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)]),
        );
        edges.insert(
            9,
            outline_edge([1, 2], [Vec2::new(1.0, 0.0), Vec2::new(2.0, 0.4)]),
        );
        let chains = build_outline_chains(&edges);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].0.len(), 3);
        assert_eq!(chains[0].2, vec![4, 9]);
        assert_eq!(chains[0].0[1], Vec2::new(1.0, 0.0));
    }
}

fn projected_triangle_is_sane(points: [Vec2; 3]) -> bool {
    points.iter().all(|point| {
        point.x.is_finite() && point.y.is_finite() && point.x.abs() < 8.0 && point.y.abs() < 8.0
    })
}

fn force_opaque(color: ColorRgba) -> ColorRgba {
    ColorRgba::new(color.r, color.g, color.b, 1.0)
}

pub(crate) fn append_text_3d_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform3,
    content: &str,
    transform: Transform3,
    size: f32,
    color: ColorRgba,
) {
    let pixel_size = (size * 0.18).max(0.05);
    let advance = 6.0 * pixel_size;
    let text_width = content.chars().count() as f32 * advance;
    let start_x = -text_width * 0.5;
    let start_y = -3.5 * pixel_size;

    for (index, ch) in content.chars().enumerate() {
        let rows = glyph_rows(ch);
        let glyph_origin_x = start_x + index as f32 * advance;
        for (row_index, row_bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if row_bits & (1 << (4 - column)) == 0 {
                    continue;
                }

                let min = Vec3::new(
                    glyph_origin_x + column as f32 * pixel_size,
                    start_y + (6 - row_index) as f32 * pixel_size,
                    0.0,
                );
                let max = Vec3::new(min.x + pixel_size, min.y + pixel_size, 0.0);
                let quad = [
                    transform_point_3d(min, transform),
                    transform_point_3d(Vec3::new(max.x, min.y, 0.0), transform),
                    transform_point_3d(max, transform),
                    transform_point_3d(Vec3::new(min.x, max.y, 0.0), transform),
                ];
                let [Some(a), Some(b), Some(c), Some(d)] = quad.map(|point| {
                    project_point(point, camera, *viewport).map(|projected| projected.position)
                }) else {
                    continue;
                };
                push_quad(vertices, a, b, c, d, color);
            }
        }
    }
}
