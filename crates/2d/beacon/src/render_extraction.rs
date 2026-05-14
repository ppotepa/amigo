use amigo_math::{ColorRgba, Vec2};

use crate::BeaconLight2dSceneService;

pub struct Beacon2dRenderExtractionContext<'a> {
    pub beacon_scene_service: &'a BeaconLight2dSceneService,
}

pub struct Beacon2dRenderExtractor;

impl Beacon2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        "beacon_2d"
    }

    pub fn extract(
        &self,
        ctx: Beacon2dRenderExtractionContext<'_>,
        output: &mut impl amigo_2d_vector::Vector2dRenderOutput,
    ) {
        for beacon in ctx.beacon_scene_service.draw_commands() {
            for cmd in vectorize(&beacon) {
                output.push_vector2d_render_command(cmd);
            }
        }
    }
}

fn vectorize(
    beacon: &crate::BeaconLight2dDrawCommand,
) -> Vec<amigo_2d_vector::VectorShape2dDrawCommand> {
    let mut out = Vec::new();
    let mk_circle =
        |radius: f32, color: ColorRgba, z: f32| amigo_2d_vector::VectorShape2dDrawCommand {
            entity_id: amigo_scene::SceneEntityId::new(0),
            entity_name: beacon.entity_name.clone(),
            render_layer: beacon.render_layer.clone(),
            shape: amigo_2d_vector::VectorShape2d {
                kind: amigo_2d_vector::VectorShapeKind2d::Circle {
                    radius,
                    segments: 16,
                },
                style: amigo_2d_vector::VectorStyle2d {
                    stroke_color: color,
                    stroke_width: 0.0,
                    fill_color: Some(color),
                },
            },
            z_index: z,
            transform: amigo_math::Transform2 {
                translation: Vec2::new(beacon.center.x, beacon.center.y),
                ..Default::default()
            },
            viewport_fit: vector_viewport_fit(beacon.viewport_fit),
            viewport_canvas_size: beacon.viewport_canvas_size,
        };

    let c = scale_alpha(beacon.color, (beacon.intensity * 0.9).clamp(0.0, 1.0));
    let h1 = scale_alpha(beacon.color, (beacon.intensity * 0.30).clamp(0.0, 1.0));
    let h2 = scale_alpha(beacon.color, (beacon.intensity * 0.12).clamp(0.0, 1.0));
    out.push(mk_circle(beacon.core_radius_px.max(0.5), c, beacon.z_index));
    out.push(mk_circle(
        (beacon.halo_radius_px * 0.7).max(0.5),
        h1,
        beacon.z_index - 0.01,
    ));
    out.push(mk_circle(
        beacon.halo_radius_px.max(0.5),
        h2,
        beacon.z_index - 0.02,
    ));

    let ab = beacon.aberration_px.max(0.0);
    if ab > 0.01 {
        let mut r = mk_circle(
            (beacon.core_radius_px * 0.8).max(0.4),
            scale_alpha(beacon.color, beacon.intensity * 0.10),
            beacon.z_index + 0.01,
        );
        r.transform.translation.x += ab;
        out.push(r);
        let mut b = mk_circle(
            (beacon.core_radius_px * 0.8).max(0.4),
            scale_alpha(beacon.color, beacon.intensity * 0.06),
            beacon.z_index + 0.01,
        );
        b.transform.translation.x -= ab;
        out.push(b);
    }
    out
}

fn vector_viewport_fit(
    fit: amigo_scene::LayeredImageViewportFit2dSceneCommand,
) -> amigo_2d_vector::VectorViewportFit2d {
    match fit {
        amigo_scene::LayeredImageViewportFit2dSceneCommand::Fixed => {
            amigo_2d_vector::VectorViewportFit2d::Fixed
        }
        amigo_scene::LayeredImageViewportFit2dSceneCommand::Stretch => {
            amigo_2d_vector::VectorViewportFit2d::Stretch
        }
        amigo_scene::LayeredImageViewportFit2dSceneCommand::Contain => {
            amigo_2d_vector::VectorViewportFit2d::Contain
        }
        amigo_scene::LayeredImageViewportFit2dSceneCommand::Cover => {
            amigo_2d_vector::VectorViewportFit2d::Cover
        }
    }
}

fn scale_alpha(mut color: ColorRgba, alpha: f32) -> ColorRgba {
    color.a *= alpha.clamp(0.0, 1.0);
    color
}
