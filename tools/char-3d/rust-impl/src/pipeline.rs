use crate::{
    math::{
        EPS, bary_inside, bary2, clamp01, deg, hash01, lerp, noise, norm2, parse_hex_rgb, rot2,
        tri_area2,
    },
    mesh::Mesh,
    state::{AppState, ControlMode, ProjectionMode, ToolMode},
};
use glam::{Vec2, Vec3};
use std::time::Instant;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ProjectedVertex {
    pub world: Vec3,
    pub camera: Vec3,
    pub screen: Vec2,
    pub in_front: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct FrameFace {
    pub id: usize,
    pub p: [ProjectedVertex; 3],
    pub normal: Vec3,
    pub flow: Vec2,
    pub area: f32,
    pub center: Vec2,
    pub depth: f32,
    pub tone: f32,
    pub ndotl: f32,
    pub front: bool,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct ContourSegment {
    pub a: Vec2,
    pub b: Vec2,
    pub visible: bool,
    pub kind: ContourKind,
}

#[derive(Clone, Copy, Debug)]
pub enum ContourKind {
    Contour,
    Crease,
    Suggestive,
    Hidden,
}

#[derive(Clone, Debug)]
pub enum Mark {
    Line {
        pts: Vec<Vec2>,
        color: [f32; 4],
        width: f32,
        alpha: f32,
    },
    Dot {
        center: Vec2,
        radius: f32,
        color: [f32; 4],
        alpha: f32,
    },
}

#[derive(Clone, Debug)]
pub struct PaintRegion {
    pub points: Vec<Vec2>,
    pub color: [f32; 4],
    pub alpha: f32,
}

#[derive(Clone, Debug, Default)]
pub struct FrameStats {
    pub total_faces: usize,
    pub screen_faces: usize,
    pub visible_faces: usize,
    pub contours: usize,
    pub marks: usize,
    pub paint_regions: usize,
    pub frame_ms: f32,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RenderFrame {
    pub width: u32,
    pub height: u32,
    pub paper: [f32; 4],
    pub faces: Vec<FrameFace>,
    pub paint_regions: Vec<PaintRegion>,
    pub contours: Vec<ContourSegment>,
    pub marks: Vec<Mark>,
    pub stats: FrameStats,
}

pub fn compute_frame(mesh: &Mesh, state: &AppState, width: u32, height: u32) -> RenderFrame {
    let start = Instant::now();
    let ctx = ProjectionContext::new(state, width.max(1), height.max(1));
    let light = light_vector(state);
    let verts = mesh.deformed_vertices(state);
    let projected: Vec<_> = verts
        .iter()
        .enumerate()
        .map(|(i, p)| project_world_point(&ctx, *p, i))
        .collect();
    let face_ids = selected_face_indices(
        mesh.faces.len(),
        state.vector_budget_enabled,
        state.vector_max_projected_faces.max(0.0) as usize,
    );
    let mut faces = Vec::new();
    for id in face_ids {
        let face = &mesh.faces[id];
        let p = [
            projected[face.v[0]].clone(),
            projected[face.v[1]].clone(),
            projected[face.v[2]].clone(),
        ];
        let area = tri_area2(p[0].screen, p[1].screen, p[2].screen);
        if area <= EPS || area < state.vector_min_face_area_px {
            continue;
        }
        if offscreen(&p, width, height, 96.0) {
            continue;
        }
        let ab = p[1].camera - p[0].camera;
        let ac = p[2].camera - p[0].camera;
        let normal = ab.cross(ac).normalize_or_zero();
        let front = match state.control_mode {
            ControlMode::Freelook => normal.z < 0.0,
            ControlMode::Orbit => normal.z > 0.0,
        };
        if state.backface && !front {
            continue;
        }
        let center = (p[0].screen + p[1].screen + p[2].screen) / 3.0;
        let depth = (p[0].camera.z + p[1].camera.z + p[2].camera.z) / 3.0;
        let ndotl = normal.dot(light);
        let shade = 1.0 - clamp01(ndotl * 0.5 + 0.5);
        let rim = 1.0 - normal.z.abs();
        let contact = contact_score(center.y, normal, height as f32);
        let mut tone =
            clamp01(shade * 0.86 + rim * state.edge_dark * 0.36 + contact * state.contact * 0.42);
        tone = tone.powf(lerp(1.55, 0.58, clamp01(state.core / 2.0)));
        if state.simplify > 0.01 {
            let bands = lerp(10.0, 3.0, state.simplify).round().max(1.0);
            tone = (tone * bands).round() / bands;
        }
        let flow = compute_flow(&p, center, light, state, width, height);
        faces.push(FrameFace {
            id,
            p,
            normal,
            flow,
            area,
            center,
            depth,
            tone,
            ndotl,
            front,
            visible: true,
        });
    }
    if state.sort_faces {
        faces.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    }
    let contours = if state.contours {
        compute_contours(mesh, &projected, &faces, state)
    } else {
        Vec::new()
    };
    let paint_regions = if state.paint_enabled || state.face_wash || state.tone_debug {
        compute_paint_regions(&faces, state)
    } else {
        Vec::new()
    };
    let marks = if state.shadows_enabled {
        generate_marks(&faces, state)
    } else {
        Vec::new()
    };
    let stats = FrameStats {
        total_faces: mesh.faces.len(),
        screen_faces: faces.len(),
        visible_faces: faces.len(),
        contours: contours.len(),
        marks: marks.len(),
        paint_regions: paint_regions.len(),
        frame_ms: start.elapsed().as_secs_f32() * 1000.0,
    };
    RenderFrame {
        width,
        height,
        paper: parse_hex_rgb(&state.paint_paper_color, [0.965, 0.949, 0.91, 1.0]),
        faces,
        paint_regions,
        contours,
        marks,
        stats,
    }
}

fn offscreen(p: &[ProjectedVertex; 3], width: u32, height: u32, margin: f32) -> bool {
    let min = p[0].screen.min(p[1].screen).min(p[2].screen);
    let max = p[0].screen.max(p[1].screen).max(p[2].screen);
    max.x < -margin
        || max.y < -margin
        || min.x > width as f32 + margin
        || min.y > height as f32 + margin
}

fn light_vector(state: &AppState) -> Vec3 {
    let az = deg(state.light_az);
    let el = deg(state.light_el);
    Vec3::new(az.sin() * el.cos(), el.sin(), az.cos() * el.cos()).normalize_or_zero()
}

fn contact_score(y: f32, n: Vec3, height: f32) -> f32 {
    let low = clamp01((y - height * 0.50) / (height * 0.34));
    let grazing = clamp01(1.0 - n.z.abs());
    low * grazing
}

fn compute_flow(
    p: &[ProjectedVertex; 3],
    center: Vec2,
    light: Vec3,
    state: &AppState,
    width: u32,
    height: u32,
) -> Vec2 {
    let mut best = p[1].screen - p[0].screen;
    for i in 0..3 {
        let edge = p[(i + 1) % 3].screen - p[i].screen;
        if edge.length_squared() > best.length_squared() {
            best = edge;
        }
    }
    let form = norm2(best);
    let radial = norm2(center - Vec2::new(width as f32 * 0.5, height as f32 * 0.5));
    let cross = Vec2::new(-radial.y, radial.x);
    let light2 = norm2(Vec2::new(light.x, -light.y));
    let term = Vec2::new(-light2.y, light2.x);
    match state.flow_mode.as_str() {
        "parallel" => Vec2::new(deg(-22.0).cos(), deg(-22.0).sin()),
        "form" => form,
        "crossContour" => norm2(cross * 0.82 + form * 0.18),
        "silhouette" => cross,
        "light" => light2,
        "terminator" => term,
        _ => norm2(form * 0.50 + cross * 0.32 + term * 0.20),
    }
}

fn compute_contours(
    mesh: &Mesh,
    projected: &[ProjectedVertex],
    faces: &[FrameFace],
    state: &AppState,
) -> Vec<ContourSegment> {
    let mut face_by_id = vec![None; mesh.faces.len()];
    for (index, face) in faces.iter().enumerate() {
        face_by_id[face.id] = Some(index);
    }
    let mut out = Vec::new();
    let budget = if state.vector_budget_enabled {
        state.vector_max_contour_lines as usize
    } else {
        usize::MAX
    };
    for edge in &mesh.edges {
        if out.len() >= budget {
            break;
        }
        let f0 = edge.f0.and_then(|id| face_by_id.get(id).and_then(|v| *v));
        let f1 = edge.f1.and_then(|id| face_by_id.get(id).and_then(|v| *v));
        let a = projected[edge.a].screen;
        let b = projected[edge.b].screen;
        let len = a.distance(b);
        if len < state.cleanup_min_line_length_px
            || len < state.vector_min_edge_length_px
            || len > state.cleanup_max_edge_length_px
        {
            continue;
        }
        let (kind, visible) = match (f0, f1) {
            (Some(left), Some(right)) => {
                let l = &faces[left];
                let r = &faces[right];
                if l.front != r.front {
                    (ContourKind::Contour, true)
                } else if state.creases && l.normal.dot(r.normal) < 0.70 {
                    (ContourKind::Crease, true)
                } else if state.suggestive && (l.tone - r.tone).abs() > 0.32 {
                    (ContourKind::Suggestive, true)
                } else {
                    continue;
                }
            }
            (Some(_), None) | (None, Some(_)) => (ContourKind::Contour, true),
            _ => {
                if state.show_hidden {
                    (ContourKind::Hidden, false)
                } else {
                    continue;
                }
            }
        };
        out.push(ContourSegment {
            a,
            b,
            visible,
            kind,
        });
    }
    out
}

fn generate_marks(faces: &[FrameFace], state: &AppState) -> Vec<Mark> {
    let mut out = Vec::new();
    let budget = if state.vector_budget_enabled {
        state.vector_max_shadow_marks as usize
    } else {
        2600
    };
    let shadow = parse_hex_rgb(&state.paint_shadow_color, [0.18, 0.16, 0.14, 1.0]);
    for face in faces {
        if out.len() >= budget {
            break;
        }
        let tone = clamp01((face.tone - state.threshold) / (1.0 - state.threshold).max(0.05));
        if tone <= 0.018 && state.method != "stipple" && state.method != "halftone" {
            continue;
        }
        let spacing =
            (state.spacing * lerp(1.45, 0.55, tone) * lerp(0.58, 1.85, state.economy)).max(3.0);
        let raw = face.area / (spacing * spacing) * state.density * lerp(0.6, 2.25, tone);
        let count = raw.floor() as usize + (hash01(face.id as f32 + 991.7) < raw.fract()) as usize;
        let count = count.min(42).min(budget - out.len());
        for i in 0..count {
            let seed = (face.id as f32 + 1.0) * 1009.133 + i as f32 * 73.19;
            let r1 = hash01(seed + i as f32 * 2.17);
            let r2 = hash01(seed + i as f32 * 5.91 + 11.3);
            let s = r1.sqrt();
            let bary = Vec3::new(1.0 - s, s * (1.0 - r2), s * r2);
            let mut c =
                face.p[0].screen * bary.x + face.p[1].screen * bary.y + face.p[2].screen * bary.z;
            c.x += noise(seed, 10.0) * state.jitter * spacing * 0.35;
            c.y += noise(seed, 20.0) * state.jitter * spacing * 0.35;
            if !bary_inside(
                bary2(c, face.p[0].screen, face.p[1].screen, face.p[2].screen),
                0.035,
            ) && state.clip_to_faces
            {
                continue;
            }
            if state.method == "stipple" || state.method == "halftone" {
                let radius = state.dot_size
                    * lerp(
                        0.55,
                        if state.method == "halftone" {
                            2.0
                        } else {
                            1.22
                        },
                        tone,
                    );
                out.push(Mark::Dot {
                    center: c,
                    radius,
                    color: shadow,
                    alpha: lerp(0.12, 0.55, tone),
                });
            } else {
                out.push(make_stroke(face, c, tone, seed, state, shadow));
            }
        }
    }
    out
}

fn make_stroke(
    face: &FrameFace,
    center: Vec2,
    tone: f32,
    seed: f32,
    state: &AppState,
    color: [f32; 4],
) -> Mark {
    let mut dir = face.flow;
    if state.method == "crosshatch" && hash01(seed + 4.0) > 0.5 {
        dir = rot2(dir, deg(state.cross_angle));
    }
    let len = (state.stroke_len
        * lerp(0.52, 1.25, tone)
        * (1.0 + noise(seed, 1.0) * state.length_var * 0.65))
        .max(2.0);
    let steps = ((len / 9.0).round() as usize + 3).clamp(4, 18);
    let perp = Vec2::new(-dir.y, dir.x);
    let mut pts = Vec::with_capacity(steps);
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let q = lerp(-0.5, 0.5, t);
        let wob = (t * std::f32::consts::PI).sin() * state.curvature * state.wobble * len * 0.16
            + noise(seed, i as f32) * state.wobble * len * 0.040;
        pts.push(center + dir * len * q + perp * wob);
    }
    let mode_mul = match state.mode {
        ToolMode::Ink => 1.0,
        ToolMode::Pencil => 0.72,
        ToolMode::Brush => 1.45,
    };
    Mark::Line {
        pts,
        color,
        width: (state.stroke_width * mode_mul * (1.0 + noise(seed, 7.0) * state.width_var))
            .max(0.15),
        alpha: lerp(0.08, 0.42, tone),
    }
}

fn compute_paint_regions(faces: &[FrameFace], state: &AppState) -> Vec<PaintRegion> {
    let mut out = Vec::new();
    let base = parse_hex_rgb(&state.paint_base_color, [0.84, 0.68, 0.52, 1.0]);
    let shadow = parse_hex_rgb(&state.paint_shadow_color, [0.36, 0.44, 0.58, 1.0]);
    let highlight = parse_hex_rgb(&state.paint_highlight_color, [1.0, 0.94, 0.76, 1.0]);
    let mut candidates: Vec<_> = faces
        .iter()
        .filter(|face| face.area >= state.region_min_projected_area_px)
        .collect();
    candidates.sort_by(|a, b| b.area.total_cmp(&a.area));
    for face in candidates
        .into_iter()
        .take(state.region_max_paint_regions.max(0.0) as usize)
    {
        if state.base_wash_enabled && state.paint_enabled {
            out.push(PaintRegion {
                points: face.p.iter().map(|p| p.screen).collect(),
                color: base,
                alpha: state.paint_base_opacity * 0.45,
            });
        }
        if state.shadow_region_enabled && face.tone > 0.24 {
            out.push(PaintRegion {
                points: face.p.iter().map(|p| p.screen).collect(),
                color: shadow,
                alpha: state.paint_cel_strength * face.tone * 0.55,
            });
        }
        if state.highlight_region_enabled && face.tone < 0.16 {
            out.push(PaintRegion {
                points: face.p.iter().map(|p| p.screen).collect(),
                color: highlight,
                alpha: state.paint_highlight_amount * 0.45,
            });
        }
    }
    out
}

fn selected_face_indices(total: usize, budget_enabled: bool, max_faces: usize) -> Vec<usize> {
    if !budget_enabled || total <= max_faces || max_faces == 0 {
        return (0..total).collect();
    }

    let step = total as f32 / max_faces as f32;
    let mut out = Vec::with_capacity(max_faces);
    let mut last = usize::MAX;
    for i in 0..max_faces {
        let id = ((i as f32 * step).floor() as usize).min(total - 1);
        if id != last {
            out.push(id);
            last = id;
        }
    }
    out
}

struct ProjectionContext {
    state: AppState,
    #[allow(dead_code)]
    width: f32,
    #[allow(dead_code)]
    height: f32,
    center: Vec2,
    scale: f32,
    fwd: Vec3,
    rgt: Vec3,
    up: Vec3,
}

impl ProjectionContext {
    fn new(state: &AppState, width: u32, height: u32) -> Self {
        let width = width as f32;
        let height = height as f32;
        if state.control_mode == ControlMode::Freelook {
            let yaw = deg(state.camera_yaw);
            let pitch = deg(state.camera_pitch);
            let fwd = Vec3::new(
                yaw.sin() * pitch.cos(),
                -pitch.sin(),
                -yaw.cos() * pitch.cos(),
            )
            .normalize_or_zero();
            let rgt = Vec3::new(yaw.cos(), 0.0, yaw.sin()).normalize_or_zero();
            let up = rgt.cross(fwd).normalize_or_zero();
            return Self {
                state: state.clone(),
                width,
                height,
                center: Vec2::new(width * 0.5, height * 0.5),
                scale: width.min(height) * 0.1 * state.zoom,
                fwd,
                rgt,
                up,
            };
        }
        Self {
            state: state.clone(),
            width,
            height,
            center: Vec2::new(width * 0.5, height * 0.5),
            scale: width.min(height) * 0.36 * state.zoom,
            fwd: Vec3::NEG_Z,
            rgt: Vec3::X,
            up: Vec3::Y,
        }
    }
}

fn project_world_point(ctx: &ProjectionContext, p: Vec3, index: usize) -> ProjectedVertex {
    if ctx.state.control_mode == ControlMode::Freelook {
        let rel = p - Vec3::new(ctx.state.camera_x, ctx.state.camera_y, ctx.state.camera_z);
        let camera = Vec3::new(rel.dot(ctx.rgt), rel.dot(ctx.up), rel.dot(ctx.fwd));
        let perspective = if ctx.state.projection_mode == ProjectionMode::Perspective {
            ctx.state.focal_length / camera.z.max(0.1)
        } else {
            ctx.state.focal_length / 10.0
        };
        let mut screen = Vec2::new(
            ctx.center.x + camera.x * ctx.scale * perspective,
            ctx.center.y - camera.y * ctx.scale * perspective,
        );
        screen += wobble(ctx, index);
        return ProjectedVertex {
            world: p,
            camera,
            screen,
            in_front: camera.z >= 0.1,
        };
    }
    let yaw = deg(ctx.state.yaw);
    let pitch = deg(ctx.state.pitch);
    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    let x1 = p.x * cy + p.z * sy;
    let z1 = -p.x * sy + p.z * cy;
    let y2 = p.y * cp - z1 * sp;
    let z2 = p.y * sp + z1 * cp;
    let camera = Vec3::new(
        x1 - ctx.state.camera_x,
        y2 - ctx.state.camera_y,
        z2 - ctx.state.camera_z,
    );
    let mut screen = Vec2::new(
        ctx.center.x + camera.x * ctx.scale,
        ctx.center.y - camera.y * ctx.scale,
    );
    screen += wobble(ctx, index);
    ProjectedVertex {
        world: p,
        camera,
        screen,
        in_front: true,
    }
}

fn wobble(ctx: &ProjectionContext, index: usize) -> Vec2 {
    if ctx.state.projection_wobble <= 0.0 {
        return Vec2::ZERO;
    }
    let seed = (index as f32 + 1.0) * 409.17;
    Vec2::new(noise(seed, 1.0), noise(seed, 2.0)) * ctx.state.projection_wobble
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Mesh;

    #[test]
    fn projection_builds_frame() {
        let mesh = Mesh::from_obj_text("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n", "tri").unwrap();
        let frame = compute_frame(&mesh, &AppState::default(), 800, 600);
        assert_eq!(frame.stats.total_faces, 1);
    }

    #[test]
    fn face_budget_samples_across_full_mesh() {
        let ids = selected_face_indices(100, true, 10);
        assert_eq!(ids.len(), 10);
        assert_eq!(ids[0], 0);
        assert!(ids.contains(&90));
        assert_ne!(ids, (0..10).collect::<Vec<_>>());
    }
}
