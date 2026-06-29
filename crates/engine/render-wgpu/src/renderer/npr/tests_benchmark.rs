use std::collections::BTreeMap;

use amigo_math::{Transform3, Vec3};

use crate::renderer::*;

#[test]
#[ignore = "manual NPR benchmark; run with --ignored --nocapture"]
fn benchmark_playground_npr_soldier_rotation_cpu_stroke_workload() {
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();
    let path = workspace_root.join("mods/playground-npr/source-models/threejs/Soldier.glb");
    let geometry = load_glb_geometry(&path).expect("playground npr soldier glb should import");
    let viewport = Viewport::from_dimensions(1280.0, 720.0);
    let camera = Transform3 {
        translation: Vec3::new(0.0, 0.0, 4.2),
        ..Transform3::default()
    };
    let cinematic_settings = amigo_render_api::NprLineSettings3d {
        style_preset: amigo_render_api::NprStylePreset3d::RoughComicInk,
        stroke_tool: amigo_render_api::NprStrokeTool3d::InkPen,
        boundary: true,
        silhouette: true,
        feature: true,
        suggestive: true,
        contact: true,
        contact_ground_y: -0.5,
        contact_threshold: 0.08,
        min_screen_length_px: 2.0,
        feature_angle_degrees: 32.0,
        width_px: 2.35,
        passes: 2,
        search_line_count: 1,
        temporal_stability: 0.92,
        visibility_hysteresis_frames: 3,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let target_60fps_settings = amigo_render_api::NprLineSettings3d {
        style_preset: amigo_render_api::NprStylePreset3d::RoughComicInk,
        stroke_tool: amigo_render_api::NprStrokeTool3d::TechnicalPen,
        boundary: true,
        silhouette: true,
        feature: true,
        suggestive: false,
        contact: false,
        contact_ground_y: -0.5,
        contact_threshold: 0.06,
        min_screen_length_px: 3.4,
        feature_angle_degrees: 42.0,
        width_px: 2.0,
        silhouette_width_multiplier: 1.50,
        boundary_width_multiplier: 0.92,
        feature_width_multiplier: 0.48,
        depth_pressure: 0.08,
        humanization: 0.22,
        line_confidence: 0.88,
        temporal_stability: 0.94,
        visibility_hysteresis_frames: 2,
        visibility_max_dimension_px: 720.0,
        endpoint_snap_px: 2.4,
        path_simplify_px: 1.45,
        passes: 1,
        search_line_count: 0,
        dropout: 0.0,
        dropout_segment_min_px: 12.0,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let low_120fps_settings = amigo_render_api::NprLineSettings3d {
        style_preset: amigo_render_api::NprStylePreset3d::RoughComicInk,
        stroke_tool: amigo_render_api::NprStrokeTool3d::TechnicalPen,
        boundary: true,
        silhouette: true,
        feature: true,
        suggestive: false,
        contact: false,
        contact_ground_y: -0.5,
        contact_threshold: 0.04,
        min_screen_length_px: 5.0,
        feature_angle_degrees: 52.0,
        width_px: 1.85,
        silhouette_width_multiplier: 1.55,
        boundary_width_multiplier: 0.85,
        feature_width_multiplier: 0.32,
        depth_pressure: 0.04,
        humanization: 0.08,
        line_confidence: 0.95,
        temporal_stability: 0.96,
        visibility_hysteresis_frames: 1,
        visibility_max_dimension_px: 512.0,
        endpoint_snap_px: 3.0,
        path_simplify_px: 2.0,
        passes: 1,
        search_line_count: 0,
        dropout: 0.0,
        dropout_segment_min_px: 16.0,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let cases = [
        ("cinematic_12fps", cinematic_settings),
        ("target_60fps", target_60fps_settings),
        ("low_120fps", low_120fps_settings),
    ];

    let frames = 120u64;
    let warmup = 12usize;
    println!(
        "NPR soldier rotation benchmark: frames={} warmup={} measured={} viewport=1280x720 vertices={} triangles={} edges={}",
        frames,
        warmup,
        frames as usize - warmup,
        geometry.vertices.len(),
        geometry.triangles.len(),
        geometry.edges.len()
    );
    for (case_name, settings) in cases {
        let mut history = BTreeMap::new();
        let mut timings_us = Vec::new();
        let mut vertices = Vec::new();
        let mut npr_stroke_segments = Vec::new();
        let mut totals = NprStrokeFrameStats3d::default();

        for frame in 0..frames {
            vertices.clear();
            npr_stroke_segments.clear();
            let angle = frame as f32 / frames as f32 * std::f32::consts::TAU;
            let transform = Transform3 {
                rotation_euler: Vec3::new(-std::f32::consts::FRAC_PI_2, angle, 0.0),
                ..Transform3::default()
            };
            let start = std::time::Instant::now();
            let stats = append_mesh_npr_line_vertices_with_history_and_stats(
                &mut history,
                frame + 1,
                "playground-npr-model-1-soldier",
                &mut vertices,
                Some(&mut npr_stroke_segments),
                &viewport,
                camera,
                amigo_render_api::Camera3dRenderSettings::default(),
                &geometry,
                transform,
                &settings,
            );
            let elapsed = start.elapsed();
            if frame as usize >= warmup {
                timings_us.push(elapsed.as_secs_f64() * 1_000_000.0);
                totals.add(stats);
            }
        }

        timings_us.sort_by(|left, right| {
            left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
        });
        let measured = timings_us.len().max(1);
        let measured_f64 = measured as f64;
        let mean_us = timings_us.iter().sum::<f64>() / measured_f64;
        let median_us = timings_us[measured / 2];
        let p95_us = timings_us[((measured as f32 * 0.95).floor() as usize).min(measured - 1)];
        let avg_paths = totals.paths as f64 / measured_f64;
        let avg_samples = totals.brush_samples as f64 / measured_f64;
        let avg_vertices = totals.strip_vertices as f64 / measured_f64;
        let avg_search_passes = totals.search_passes as f64 / measured_f64;
        let avg_path_build_us = totals.path_build_us / measured_f64;
        let avg_stabilize_us = totals.stabilize_us / measured_f64;
        let avg_stroke_vertices_us = totals.stroke_vertices_us / measured_f64;
        let avg_path_project_us = totals.path_project_us / measured_f64;
        let avg_path_visibility_us = totals.path_visibility_us / measured_f64;
        let avg_path_edge_sample_us = totals.path_edge_sample_us / measured_f64;
        let avg_path_stitch_us = totals.path_stitch_us / measured_f64;
        let avg_visible_edges = totals.path_visible_edges as f64 / measured_f64;
        let avg_fragments = totals.path_fragments as f64 / measured_f64;
        let cache_total = totals.cached_plan_hits + totals.cached_plan_misses;
        let cache_hit_rate = if cache_total == 0 {
            0.0
        } else {
            totals.cached_plan_hits as f64 / cache_total as f64 * 100.0
        };

        println!(
            "case={case_name} cpu_stroke_us mean={mean_us:.2} median={median_us:.2} p95={p95_us:.2}"
        );
        println!(
            "case={case_name} workload avg_paths={avg_paths:.2} avg_samples={avg_samples:.2} avg_vertices={avg_vertices:.2} avg_search_passes={avg_search_passes:.2} cache_hit_rate={cache_hit_rate:.1}% visibility_max_dimension_px={:.0}",
            settings.visibility_max_dimension_px
        );
        println!(
            "case={case_name} stage_cpu_us path_build={avg_path_build_us:.2} stabilize={avg_stabilize_us:.2} stroke_vertices={avg_stroke_vertices_us:.2}"
        );
        println!(
            "case={case_name} path_build_breakdown_us project={avg_path_project_us:.2} visibility={avg_path_visibility_us:.2} edge_sample={avg_path_edge_sample_us:.2} stitch={avg_path_stitch_us:.2} visible_edges={avg_visible_edges:.2} fragments={avg_fragments:.2}"
        );
        println!(
            "case={case_name} path_mix boundary={} silhouette={} crease={} seam={} feature={} contact={}",
            totals.boundary_paths,
            totals.silhouette_paths,
            totals.crease_paths,
            totals.seam_paths,
            totals.feature_paths,
            totals.contact_paths
        );

        assert!(mean_us.is_finite());
        assert!(avg_paths > 0.0);
        assert!(avg_vertices > 0.0);
    }
}
