use amigo_2d_lighting_beacon::BeaconLight2dDrawCommand;
use amigo_scene::LayeredImageViewportFit2dSceneCommand;

use crate::renderer::*;

pub(crate) fn append_beacon_vfx_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    command: &BeaconLight2dDrawCommand,
) {
    let fitted = fit_beacon_to_viewport(command, viewport);
    let distance_factor = optical_distance_factor(command.distance_m);
    let energy = command.intensity.max(0.0) * (0.75 + 0.25 * distance_factor);
    if energy <= 0.001 {
        return;
    }

    let pulse = command.pulse.clamp(0.0, 1.5);
    if command.beam_enabled && command.beam_length_px > 0.0 && command.beam_strength > 0.0 {
        append_beam(vertices, viewport, camera, &fitted, command, energy, pulse);
    }

    let glow = command.glow_strength.max(0.0);
    append_radial_glow(
        vertices,
        viewport,
        camera,
        fitted.center,
        command.halo_radius_px * fitted.scale * 2.6,
        tint(
            command.color,
            0.025 * energy * glow * command.bloom.max(0.0),
        ),
    );
    append_radial_glow(
        vertices,
        viewport,
        camera,
        fitted.center,
        command.halo_radius_px * fitted.scale * 1.45,
        tint(command.color, 0.060 * energy * glow),
    );
    append_radial_glow(
        vertices,
        viewport,
        camera,
        fitted.center,
        command.halo_radius_px * fitted.scale * 0.72,
        tint(command.color, 0.125 * energy * glow),
    );
    append_radial_glow(
        vertices,
        viewport,
        camera,
        fitted.center,
        command.core_radius_px * fitted.scale * 4.2,
        tint(command.color, 0.35 * energy),
    );
    append_core(
        vertices,
        viewport,
        camera,
        fitted.center,
        command,
        fitted.scale,
        energy,
    );
    append_lens_flare(
        vertices,
        viewport,
        camera,
        &fitted,
        command,
        pulse,
        distance_factor,
    );
}

#[derive(Clone, Copy)]
struct FittedBeacon {
    center: Vec2,
    scale: f32,
}

fn fit_beacon_to_viewport(command: &BeaconLight2dDrawCommand, viewport: &Viewport) -> FittedBeacon {
    let Some(canvas_size) = command.viewport_canvas_size else {
        return FittedBeacon {
            center: command.center,
            scale: 1.0,
        };
    };
    if canvas_size.x <= 0.0 || canvas_size.y <= 0.0 {
        return FittedBeacon {
            center: command.center,
            scale: 1.0,
        };
    }

    let viewport_size = viewport.size();
    let scale_x = viewport_size.x / canvas_size.x;
    let scale_y = viewport_size.y / canvas_size.y;
    let scale = match command.viewport_fit {
        LayeredImageViewportFit2dSceneCommand::Fixed => {
            return FittedBeacon {
                center: command.center,
                scale: 1.0,
            };
        }
        LayeredImageViewportFit2dSceneCommand::Stretch => (scale_x.abs() + scale_y.abs()) * 0.5,
        LayeredImageViewportFit2dSceneCommand::Contain => scale_x.min(scale_y),
        LayeredImageViewportFit2dSceneCommand::Cover => scale_x.max(scale_y),
    };

    FittedBeacon {
        center: Vec2::new(command.center.x * scale_x, command.center.y * scale_y),
        scale,
    }
}

fn append_beam(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    fitted: &FittedBeacon,
    command: &BeaconLight2dDrawCommand,
    energy: f32,
    pulse: f32,
) {
    const LAYERS: [(f32, f32, f32); 3] = [(1.0, 1.0, 0.22), (0.72, 0.56, 0.25), (0.42, 0.26, 0.28)];
    let width = command
        .beam_width_degrees
        .to_radians()
        .clamp(0.01, std::f32::consts::PI);
    for (length_scale, width_scale, alpha_scale) in LAYERS {
        let half = width * width_scale * 0.5;
        let length = command.beam_length_px * fitted.scale * length_scale;
        let alpha = command.beam_strength * energy * pulse * alpha_scale;
        append_world_wedge(
            vertices,
            viewport,
            camera,
            fitted.center,
            command.rotation_radians,
            half,
            length,
            tint(command.color, alpha),
        );
    }
}

fn append_world_wedge(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    center: Vec2,
    angle: f32,
    half_width: f32,
    length: f32,
    color: ColorRgba,
) {
    if length <= 0.0 || color.a <= 0.001 {
        return;
    }

    let source = ndc_from_world_2d(center, camera, viewport);
    let source_color = color;
    let outer_color = ColorRgba::new(color.r, color.g, color.b, 0.0);
    let segments = 10;
    for index in 0..segments {
        let t0 = index as f32 / segments as f32;
        let t1 = (index + 1) as f32 / segments as f32;
        let a0 = angle - half_width + half_width * 2.0 * t0;
        let a1 = angle - half_width + half_width * 2.0 * t1;
        let p0 = vadd(center, Vec2::new(a0.cos() * length, a0.sin() * length));
        let p1 = vadd(center, Vec2::new(a1.cos() * length, a1.sin() * length));
        vertices.push(ColorVertex::new(source, source_color));
        vertices.push(ColorVertex::new(
            ndc_from_world_2d(p0, camera, viewport),
            outer_color,
        ));
        vertices.push(ColorVertex::new(
            ndc_from_world_2d(p1, camera, viewport),
            outer_color,
        ));
    }
}

fn append_radial_glow(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    center: Vec2,
    radius: f32,
    color: ColorRgba,
) {
    if radius <= 0.0 || color.a <= 0.001 {
        return;
    }

    let center_ndc = ndc_from_world_2d(center, camera, viewport);
    append_screen_radial_glow(vertices, viewport, center_ndc, radius, color);
}

fn append_screen_radial_glow(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    center_ndc: Vec2,
    radius: f32,
    color: ColorRgba,
) {
    if radius <= 0.0 || color.a <= 0.001 {
        return;
    }

    let outer = ColorRgba::new(color.r, color.g, color.b, 0.0);
    let segments = 36;
    for index in 0..segments {
        let a0 = (index as f32 / segments as f32) * std::f32::consts::TAU;
        let a1 = ((index + 1) as f32 / segments as f32) * std::f32::consts::TAU;
        let p0 = vadd(
            center_ndc,
            Vec2::new(
                a0.cos() * radius / viewport.half_width,
                a0.sin() * radius / viewport.half_height,
            ),
        );
        let p1 = vadd(
            center_ndc,
            Vec2::new(
                a1.cos() * radius / viewport.half_width,
                a1.sin() * radius / viewport.half_height,
            ),
        );
        vertices.push(ColorVertex::new(center_ndc, color));
        vertices.push(ColorVertex::new(p0, outer));
        vertices.push(ColorVertex::new(p1, outer));
    }
}

fn append_core(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    center: Vec2,
    command: &BeaconLight2dDrawCommand,
    scale: f32,
    energy: f32,
) {
    let radius = command.core_radius_px * scale;
    append_radial_glow(
        vertices,
        viewport,
        camera,
        center,
        radius * 2.8,
        ColorRgba::new(1.0, 1.0, 1.0, (0.88 * energy).clamp(0.0, 1.0)),
    );
    append_radial_glow(
        vertices,
        viewport,
        camera,
        center,
        radius * 2.0,
        tint(command.color, (0.58 * energy).clamp(0.0, 1.0)),
    );
}

fn append_lens_flare(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    fitted: &FittedBeacon,
    command: &BeaconLight2dDrawCommand,
    pulse: f32,
    distance_factor: f32,
) {
    let source = ndc_from_world_2d(fitted.center, camera, viewport);
    let center_distance = (source.x * source.x + source.y * source.y).sqrt();
    let visible_gate = if source.x.abs() < 1.18 && source.y.abs() < 1.18 {
        1.0
    } else {
        0.0
    };
    let center_gate = 1.0 - smoothstep(0.18, 0.98, center_distance);
    let amount = visible_gate
        * center_gate
        * command.flare_strength.max(0.0)
        * (command.lens_influence.max(0.0) * distance_factor)
        * command
            .z_depth
            .map(|z_depth| 0.45 + z_depth.clamp(0.0, 1.0) * 0.55)
            .unwrap_or(1.0)
        * pulse;
    if amount <= 0.001 {
        return;
    }

    let to_center = Vec2::new(-source.x, -source.y);
    let distance = (to_center.x * to_center.x + to_center.y * to_center.y)
        .sqrt()
        .max(0.001);
    let ux = to_center.x / distance;
    let uy = to_center.y / distance;
    let px = -uy;
    let py = ux;
    let length = command.flare_length_px * fitted.scale;
    let half_width = 1.2 + amount * 2.8;
    append_screen_line(
        vertices,
        viewport,
        source,
        Vec2::new(px, py),
        length,
        half_width,
        emission_tint(command.color, 0.30 * amount, 0.12),
    );

    let ghosts = [
        (0.34, 14.0, 0.10, 0.10),
        (0.58, 8.0, 0.13, 0.20),
        (0.82, 19.0, 0.07, 0.08),
        (1.12, 6.0, 0.13, 0.18),
    ];
    let far = Vec2::new(-source.x * 0.85, -source.y * 0.85);
    for (t, radius, alpha, white_mix) in ghosts {
        let point = vadd(source, vscale(vsub(far, source), t));
        append_screen_radial_glow(
            vertices,
            viewport,
            point,
            radius * fitted.scale * (1.0 + amount * 0.18),
            emission_tint(command.color, alpha * amount, white_mix),
        );
    }

    let aberration = command.aberration_px * fitted.scale;
    append_screen_radial_glow(
        vertices,
        viewport,
        vsub(
            source,
            screen_px_to_ndc(viewport, Vec2::new(px * aberration, py * aberration)),
        ),
        8.0 * fitted.scale,
        emission_tint(command.color, 0.12 * amount, 0.05),
    );
    append_screen_radial_glow(
        vertices,
        viewport,
        vadd(
            source,
            screen_px_to_ndc(viewport, Vec2::new(px * aberration, py * aberration)),
        ),
        8.0 * fitted.scale,
        emission_tint(command.color, 0.08 * amount, 0.25),
    );
}

fn optical_distance_factor(distance_m: Option<f32>) -> f32 {
    distance_m
        .map(|meters| (1.0 / (1.0 + meters.max(0.0) * 0.035)).clamp(0.08, 1.0))
        .unwrap_or(1.0)
}

fn append_screen_line(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    center: Vec2,
    direction: Vec2,
    length_px: f32,
    width_px: f32,
    color: ColorRgba,
) {
    if color.a <= 0.001 || length_px <= 0.0 || width_px <= 0.0 {
        return;
    }

    let dir = screen_px_to_ndc(
        viewport,
        Vec2::new(direction.x * length_px, direction.y * length_px),
    );
    let normal = screen_px_to_ndc(
        viewport,
        Vec2::new(-direction.y * width_px, direction.x * width_px),
    );
    let start = vsub(center, dir);
    let end = vadd(center, dir);
    let transparent = ColorRgba::new(color.r, color.g, color.b, 0.0);
    vertices.push(ColorVertex::new(vsub(start, normal), transparent));
    vertices.push(ColorVertex::new(vsub(center, normal), color));
    vertices.push(ColorVertex::new(vadd(center, normal), color));
    vertices.push(ColorVertex::new(vsub(start, normal), transparent));
    vertices.push(ColorVertex::new(vadd(center, normal), color));
    vertices.push(ColorVertex::new(vadd(start, normal), transparent));
    vertices.push(ColorVertex::new(vsub(center, normal), color));
    vertices.push(ColorVertex::new(vsub(end, normal), transparent));
    vertices.push(ColorVertex::new(vadd(end, normal), transparent));
    vertices.push(ColorVertex::new(vsub(center, normal), color));
    vertices.push(ColorVertex::new(vadd(end, normal), transparent));
    vertices.push(ColorVertex::new(vadd(center, normal), color));
}

fn screen_px_to_ndc(viewport: &Viewport, px: Vec2) -> Vec2 {
    Vec2::new(px.x / viewport.half_width, px.y / viewport.half_height)
}

fn vadd(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x + b.x, a.y + b.y)
}

fn vsub(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x - b.x, a.y - b.y)
}

fn vscale(a: Vec2, scale: f32) -> Vec2 {
    Vec2::new(a.x * scale, a.y * scale)
}

fn tint(color: ColorRgba, alpha: f32) -> ColorRgba {
    ColorRgba::new(color.r, color.g, color.b, alpha.clamp(0.0, 1.0))
}

fn emission_tint(color: ColorRgba, alpha: f32, white_mix: f32) -> ColorRgba {
    let white_mix = white_mix.clamp(0.0, 1.0);
    ColorRgba::new(
        color.r + (1.0 - color.r) * white_mix,
        color.g + (1.0 - color.g) * white_mix,
        color.b + (1.0 - color.b) * white_mix,
        alpha.clamp(0.0, 1.0),
    )
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::optical_distance_factor;

    #[test]
    fn beacon_optical_distance_factor_prefers_near_lights() {
        assert!(optical_distance_factor(Some(6.0)) > optical_distance_factor(Some(150.0)));
    }

    #[test]
    fn beacon_optical_distance_factor_defaults_to_neutral_without_distance() {
        assert_eq!(optical_distance_factor(None), 1.0);
    }

    #[test]
    fn beacon_optical_distance_factor_is_clamped() {
        assert_eq!(optical_distance_factor(Some(0.0)), 1.0);
        assert_eq!(optical_distance_factor(Some(10_000.0)), 0.08);
    }
}
