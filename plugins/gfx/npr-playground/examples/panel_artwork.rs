//! Deterministic thumbnail authoring: emits neutral 2D triangles as YAML to stdout.
//! Run from any directory with `cargo run -p amigo-npr-playground-plugin --example panel_artwork`.
use amigo_npr_playground_plugin::{
    NprPlaygroundRenderService,
    state::{MODELS, Settings},
};
use amigo_panel_api::PreviewTriangle;
use std::collections::BTreeMap;
fn main() {
    let render = NprPlaygroundRenderService::default();
    render
        .load_models(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../mods/npr-playground"),
        )
        .unwrap();
    let mut artwork = BTreeMap::new();
    for model in MODELS.iter().copied().chain([
        "Comic Ink",
        "Pencil Study",
        "Loose Study",
        "Confident Ink",
        "Broad Nib",
        "Blueprint",
        "Soft Toon",
    ]) {
        let mut settings = Settings::for_scene(false);
        if let Some(style) = amigo_npr_playground_plugin::state::style_preset(model) {
            settings.global = style;
        } else {
            settings.selected = model.into();
        }
        settings.paused = true;
        settings.camera_distance = 4.5;
        render.rebuild(&settings, [128, 128]).unwrap();
        let packet = render.snapshot().unwrap().packet;
        let mut triangles = packet.fills.clone();
        if amigo_npr_playground_plugin::state::style_preset(model).is_some() {
            for stroke in &packet.strokes {
                for indices in stroke.indices.chunks_exact(3) {
                    let vertices =
                        [indices[0], indices[1], indices[2]].map(|i| stroke.vertices[i as usize]);
                    triangles.push(amigo_render_npr::NprFillTriangle {
                        positions: vertices.map(|v| v.position),
                        depths: vertices.map(|v| v.depth - 0.0001),
                        color: glam::Vec4::from_array(packet.stroke_color(stroke)),
                    });
                }
            }
        }
        triangles.sort_by(|a, b| {
            b.depths
                .iter()
                .sum::<f32>()
                .total_cmp(&a.depths.iter().sum::<f32>())
        });
        artwork.insert(
            model,
            triangles
                .into_iter()
                .map(|t| PreviewTriangle {
                    points: t.positions.map(|p| {
                        p.to_array()
                            .map(|v| ((v / 128.0).clamp(0.0, 1.0) * 10000.0).round() / 10000.0)
                    }),
                    color: [t.color.x, t.color.y, t.color.z].map(|c| (c * 255.0).round() as u8),
                })
                .collect::<Vec<_>>(),
        );
    }
    // Flow-style triangles keep the authored file compact and diffable.
    println!("artwork:");
    for (name, triangles) in artwork {
        println!("  {name}:");
        for triangle in triangles {
            println!(
                "    - {{points: {:?}, color: {:?}}}",
                triangle.points, triangle.color
            );
        }
    }
}
