use crate::{
    assets::{BUILT_INS, ModelKind},
    pipeline::FrameStats,
    state::{AppState, ControlMode, ProjectionMode, ToolMode},
};

#[derive(Default)]
pub struct UiState {
    pub active_tab: usize,
}

pub enum UiAction {
    None,
    LoadBuiltIn(String),
    LoadObjFile,
    LoadFbxFile,
    ResetView,
    ExportSvg,
    ExportPng,
    ExportAtlas,
}

pub fn show(
    ui: &mut egui::Ui,
    state: &mut AppState,
    ui_state: &mut UiState,
    stats: Option<&FrameStats>,
    last_error: &str,
) -> UiAction {
    let mut action = UiAction::None;
    ui.heading("Susan/Suzanne - NPR Shadow Editor");
    ui.label(
        "OBJ/FBX -> projekcja wektorowa -> kontur, cien, tusz, kolor i losowosc recznej kreski.",
    );
    ui.separator();
    let tabs = [
        "Model",
        "Camera",
        "Features",
        "Stroke Tools",
        "Shadow",
        "Paint Regions",
        "Cleanup",
        "Randomness",
        "Lines",
        "Detail",
        "Debug",
        "Export",
    ];
    ui.horizontal_wrapped(|ui| {
        for (i, tab) in tabs.iter().enumerate() {
            if ui
                .selectable_label(ui_state.active_tab == i, *tab)
                .clicked()
            {
                ui_state.active_tab = i;
            }
        }
    });
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| match ui_state.active_tab {
        0 => model_tab(ui, state, &mut action),
        1 => camera_tab(ui, state),
        2 => features_tab(ui, state),
        3 => stroke_tab(ui, state),
        4 => shadow_tab(ui, state),
        5 => paint_tab(ui, state),
        6 => cleanup_tab(ui, state),
        7 => randomness_tab(ui, state),
        8 => lines_tab(ui, state),
        9 => detail_tab(ui, state),
        10 => debug_tab(ui, state),
        _ => export_tab(ui, state, &mut action),
    });
    ui.separator();
    if let Some(stats) = stats {
        ui.label(format!(
                "faces: {}/{}/{} visible/screen/total | strokes: {} | contours: {} | regions: {} | frame {:.2} ms",
                stats.visible_faces, stats.screen_faces, stats.total_faces, stats.marks, stats.contours, stats.paint_regions, stats.frame_ms
            ));
    }
    if !last_error.is_empty() {
        ui.colored_label(egui::Color32::from_rgb(150, 40, 30), last_error);
    }
    action
}

fn model_tab(ui: &mut egui::Ui, state: &mut AppState, action: &mut UiAction) {
    egui::ComboBox::from_label("Built-in")
        .selected_text(&state.model_source)
        .show_ui(ui, |ui| {
            for model in BUILT_INS {
                let suffix = match model.kind {
                    ModelKind::Obj => "",
                    ModelKind::Fbx => " (raw FBX)",
                    ModelKind::AnimClip => " (baked anim)",
                };
                if ui
                    .selectable_value(
                        &mut state.model_source,
                        model.id.to_owned(),
                        format!("{}{}", model.label, suffix),
                    )
                    .clicked()
                {
                    *action = UiAction::LoadBuiltIn(model.id.to_owned());
                }
            }
        });
    if ui.button("Load OBJ file...").clicked() {
        *action = UiAction::LoadObjFile;
    }
    if ui.button("Load FBX file...").clicked() {
        *action = UiAction::LoadFbxFile;
    }
    ui.checkbox(&mut state.auto, "auto rotate / play");
    ui.checkbox(&mut state.imprecise_tween, "imprecise tweening");
    slider(ui, &mut state.anim_fps, 2.0..=60.0, "Playback FPS");
    slider(
        ui,
        &mut state.tween_jitter_frames,
        0.0..=2.0,
        "Timing drift",
    );
    ui.label(format!(
        "Animation frame: {} | sample {:.3}s | loop {}",
        state.anim_frame_index, state.anim_sample_time, state.anim_loop_index
    ));
    ui.small("Built-in walking uses baked browser/Three.js vertex animation. Raw custom FBX is fallback only.");
}

fn camera_tab(ui: &mut egui::Ui, state: &mut AppState) {
    egui::ComboBox::from_label("Control mode")
        .selected_text(format!("{:?}", state.control_mode))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.control_mode, ControlMode::Orbit, "orbit model");
            ui.selectable_value(
                &mut state.control_mode,
                ControlMode::Freelook,
                "freelook camera",
            );
        });
    egui::ComboBox::from_label("Projection")
        .selected_text(format!("{:?}", state.projection_mode))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut state.projection_mode,
                ProjectionMode::Perspective,
                "perspective",
            );
            ui.selectable_value(&mut state.projection_mode, ProjectionMode::Ortho, "ortho");
        });
    slider(ui, &mut state.yaw, -180.0..=180.0, "Model yaw");
    slider(ui, &mut state.pitch, -85.0..=85.0, "Model pitch");
    slider(ui, &mut state.zoom, 0.55..=1.8, "Model zoom");
    slider(ui, &mut state.focal_length, 12.0..=200.0, "Focal length");
    slider(ui, &mut state.camera_x, -100.0..=100.0, "Camera X");
    slider(ui, &mut state.camera_y, -100.0..=100.0, "Camera Y");
    slider(ui, &mut state.camera_z, -100.0..=100.0, "Camera Z");
    slider(ui, &mut state.camera_yaw, -180.0..=180.0, "Camera Yaw");
    slider(ui, &mut state.camera_pitch, -85.0..=85.0, "Camera Pitch");
    slider(ui, &mut state.light_az, -180.0..=180.0, "Light azimuth");
    slider(ui, &mut state.light_el, -20.0..=85.0, "Light height");
}

fn features_tab(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.mode, ToolMode::Ink, "INK");
        ui.selectable_value(&mut state.mode, ToolMode::Pencil, "PENCIL");
        ui.selectable_value(&mut state.mode, ToolMode::Brush, "BRUSH");
    });
    egui::ComboBox::from_label("Preset")
        .selected_text(&state.preset)
        .show_ui(ui, |ui| {
            for key in [
                "cleanInk",
                "engraving",
                "loosePencil",
                "manga",
                "fbxBalanced",
                "pipelineCleanInk",
                "largeSceneBalanced",
            ] {
                if ui.selectable_label(state.preset == key, key).clicked() {
                    state.apply_preset(key);
                }
            }
        });
    text_choice(
        ui,
        "Method",
        &mut state.method,
        &[
            "hatching",
            "crosshatch",
            "contourHatch",
            "stipple",
            "halftone",
            "scribble",
            "scumble",
            "drybrush",
            "graphite",
            "comic",
            "hybrid",
        ],
    );
    text_choice(
        ui,
        "Flow mode",
        &mut state.flow_mode,
        &[
            "mixed",
            "parallel",
            "form",
            "crossContour",
            "silhouette",
            "light",
            "terminator",
        ],
    );
    ui.checkbox(&mut state.paint_enabled, "color passes");
    ui.checkbox(&mut state.face_wash, "soft face wash");
    ui.checkbox(&mut state.contours, "contour lines");
    ui.checkbox(&mut state.shadows_enabled, "shadow strokes");
    ui.checkbox(&mut state.skip_simulation, "skip hand-drawn simulation");
    slider(ui, &mut state.core, 0.0..=2.0, "Core emphasis");
    slider(ui, &mut state.contact, 0.0..=1.0, "Contact accent");
    slider(ui, &mut state.edge_dark, 0.0..=1.0, "Edge darkening");
    slider(ui, &mut state.simplify, 0.0..=1.0, "Tonal simplify");
}

fn stroke_tab(ui: &mut egui::Ui, state: &mut AppState) {
    slider(ui, &mut state.stroke_len, 5.0..=130.0, "Stroke length");
    slider(ui, &mut state.spacing, 3.0..=38.0, "Stroke spacing");
    slider(ui, &mut state.stroke_width, 0.25..=5.0, "Stroke width");
    slider(ui, &mut state.curvature, 0.0..=1.0, "Straight <-> curved");
    slider(ui, &mut state.cross_angle, 15.0..=90.0, "Crosshatch angle");
    slider(ui, &mut state.dot_size, 0.7..=7.0, "Dot size");
}

fn shadow_tab(ui: &mut egui::Ui, state: &mut AppState) {
    slider(ui, &mut state.density, 0.0..=2.0, "Density");
    slider(ui, &mut state.layers, 1.0..=4.0, "Layer count");
    slider(ui, &mut state.threshold, 0.0..=0.9, "Light threshold");
    slider(ui, &mut state.economy, 0.0..=1.0, "Shadow economy");
    slider(ui, &mut state.shadow_band_count, 1.0..=8.0, "Tone bands");
    slider(
        ui,
        &mut state.shadow_region_bleed,
        0.0..=1.0,
        "Region bleed",
    );
    slider(
        ui,
        &mut state.shadow_color_jitter,
        0.0..=1.0,
        "Color jitter",
    );
}

fn paint_tab(ui: &mut egui::Ui, state: &mut AppState) {
    text_choice(
        ui,
        "Palette",
        &mut state.paint_palette,
        &[
            "cleanComic",
            "pulp",
            "mangaWash",
            "noirTint",
            "blueShadow",
            "retroPrint",
        ],
    );
    text_choice(
        ui,
        "Brush",
        &mut state.paint_brush,
        &["watercolor", "gouache", "comicCel", "inkWash"],
    );
    color_text(ui, "Paper", &mut state.paint_paper_color);
    color_text(ui, "Base fill", &mut state.paint_base_color);
    color_text(ui, "Shadow", &mut state.paint_shadow_color);
    color_text(ui, "Highlight", &mut state.paint_highlight_color);
    slider(ui, &mut state.paint_base_opacity, 0.0..=1.0, "Base opacity");
    slider(ui, &mut state.paint_wash_opacity, 0.0..=1.0, "Soft wash");
    slider(ui, &mut state.paint_cel_strength, 0.0..=1.0, "Cel shadow");
    slider(
        ui,
        &mut state.paint_highlight_amount,
        0.0..=1.0,
        "Highlight amount",
    );
    slider(ui, &mut state.ink_dominance, 0.35..=1.35, "Ink dominance");
    ui.checkbox(&mut state.base_wash_enabled, "base wash region set");
    ui.checkbox(&mut state.shadow_region_enabled, "shadow region set");
    ui.checkbox(&mut state.highlight_region_enabled, "highlight region set");
}

fn cleanup_tab(ui: &mut egui::Ui, state: &mut AppState) {
    slider(
        ui,
        &mut state.cleanup_min_face_area_px,
        0.0..=30.0,
        "Min face area",
    );
    slider(
        ui,
        &mut state.cleanup_min_line_length_px,
        0.0..=40.0,
        "Min line length",
    );
    slider(
        ui,
        &mut state.cleanup_max_edge_length_px,
        20.0..=500.0,
        "Max edge length",
    );
    slider(
        ui,
        &mut state.cleanup_density_clamp,
        0.0..=1.0,
        "Density clamp",
    );
    slider(
        ui,
        &mut state.cleanup_region_min_area_px,
        0.0..=4000.0,
        "Region min area",
    );
    slider(
        ui,
        &mut state.cleanup_region_min_faces,
        1.0..=80.0,
        "Region min faces",
    );
    slider(
        ui,
        &mut state.cleanup_region_max_aspect,
        2.0..=40.0,
        "Region max aspect",
    );
    slider(
        ui,
        &mut state.hair_region_suppression,
        0.0..=1.0,
        "Hair suppression",
    );
}

fn randomness_tab(ui: &mut egui::Ui, state: &mut AppState) {
    slider(
        ui,
        &mut state.temporal_coherence,
        0.0..=1.0,
        "Temporal coherence",
    );
    slider(
        ui,
        &mut state.projection_human_error,
        0.0..=1.0,
        "Projection human error",
    );
    slider(
        ui,
        &mut state.stroke_pressure_jitter,
        0.0..=1.0,
        "Pressure jitter",
    );
    slider(ui, &mut state.wobble, 0.0..=1.0, "Wobble");
    slider(ui, &mut state.jitter, 0.0..=1.0, "Jitter");
    slider(
        ui,
        &mut state.stroke_crookedness,
        0.0..=1.0,
        "Line crookedness",
    );
    slider(ui, &mut state.stroke_kink_chance, 0.0..=0.55, "Kink chance");
    slider(ui, &mut state.spacing_var, 0.0..=1.0, "Spacing variance");
    slider(ui, &mut state.length_var, 0.0..=1.0, "Length variance");
    slider(ui, &mut state.width_var, 0.0..=1.0, "Width variance");
    slider(ui, &mut state.taper, 0.0..=1.0, "Taper");
    slider(ui, &mut state.breakup, 0.0..=1.0, "Breakup / depletion");
    slider(ui, &mut state.overdraw, 0.0..=1.0, "Overdraw");
    ui.checkbox(&mut state.contour_humanize, "humanized contour");
}

fn lines_tab(ui: &mut egui::Ui, state: &mut AppState) {
    ui.checkbox(&mut state.hide_occluded, "hide occluded regions");
    ui.checkbox(&mut state.backface, "back-face culling");
    ui.checkbox(&mut state.depth_clip_strokes, "depth-clip strokes");
    ui.checkbox(&mut state.clip_to_faces, "clip strokes to faces");
    ui.checkbox(&mut state.show_hidden, "show hidden dashed");
    ui.checkbox(&mut state.sort_faces, "depth-sort faces");
    slider(ui, &mut state.depth_eps, 0.0..=0.08, "Depth epsilon");
    ui.checkbox(&mut state.main_contour_enabled, "main contour line set");
    ui.checkbox(&mut state.creases, "crease accents");
    ui.checkbox(&mut state.crease_accent_enabled, "crease line set");
    ui.checkbox(&mut state.suggestive, "suggestive accents");
    ui.checkbox(&mut state.suggestive_contour_enabled, "suggestive line set");
    ui.checkbox(&mut state.contact_lines, "contact accents");
    ui.checkbox(&mut state.hidden_line_enabled, "hidden line set");
    ui.checkbox(&mut state.shadow_hatch_enabled, "shadow hatch line set");
    ui.checkbox(&mut state.depth_fade, "depth fade hidden");
}

fn detail_tab(ui: &mut egui::Ui, state: &mut AppState) {
    ui.checkbox(&mut state.scene_partition_enabled, "scene partition");
    slider(
        ui,
        &mut state.scene_partition_cell_size,
        4.0..=128.0,
        "Cell size",
    );
    slider(
        ui,
        &mut state.scene_partition_max_units,
        64.0..=8192.0,
        "Max units",
    );
    ui.checkbox(&mut state.visibility_culling_enabled, "visibility culling");
    slider(ui, &mut state.visibility_margin_px, 0.0..=260.0, "Margin");
    slider(
        ui,
        &mut state.visibility_min_area_px,
        0.0..=120.0,
        "Min area",
    );
    slider(
        ui,
        &mut state.visibility_min_radius_px,
        0.0..=40.0,
        "Min radius",
    );
    ui.checkbox(&mut state.detail_policy_enabled, "detail policy");
    slider(
        ui,
        &mut state.detail_tier0_radius_px,
        20.0..=500.0,
        "D0 radius",
    );
    slider(
        ui,
        &mut state.detail_tier1_radius_px,
        10.0..=300.0,
        "D1 radius",
    );
    slider(
        ui,
        &mut state.detail_tier2_radius_px,
        4.0..=160.0,
        "D2 radius",
    );
    slider(
        ui,
        &mut state.detail_tier3_radius_px,
        1.0..=80.0,
        "D3 radius",
    );
    ui.checkbox(&mut state.vector_budget_enabled, "vector budget");
    slider(
        ui,
        &mut state.vector_max_projected_faces,
        1000.0..=120000.0,
        "Max faces",
    );
    slider(
        ui,
        &mut state.vector_max_visible_edges,
        1000.0..=160000.0,
        "Max edges",
    );
    slider(
        ui,
        &mut state.vector_max_contour_lines,
        500.0..=60000.0,
        "Max contours",
    );
    slider(
        ui,
        &mut state.vector_max_shadow_marks,
        0.0..=16000.0,
        "Max marks",
    );
    ui.checkbox(&mut state.region_budget_enabled, "region budget");
    ui.checkbox(&mut state.region_allow_far_fills, "allow far fills");
    slider(
        ui,
        &mut state.region_min_projected_area_px,
        0.0..=2000.0,
        "Min region area",
    );
    slider(
        ui,
        &mut state.region_max_paint_regions,
        0.0..=2000.0,
        "Max regions",
    );
}

fn debug_tab(ui: &mut egui::Ui, state: &mut AppState) {
    ui.checkbox(&mut state.tone_debug, "tone debug");
    ui.checkbox(&mut state.flow_debug, "flow debug");
    ui.checkbox(&mut state.depth_debug, "depth debug");
    ui.checkbox(&mut state.seed_debug, "seed debug");
    ui.checkbox(&mut state.region_debug, "paint region IDs");
    ui.checkbox(&mut state.cleanup_debug, "rejected faces");
    ui.checkbox(&mut state.density_debug, "density heatmap");
    ui.checkbox(&mut state.visibility_debug, "visibility overlay");
    ui.checkbox(&mut state.detail_debug, "detail tier overlay");
    ui.checkbox(&mut state.budget_debug, "budget overlay");
}

fn export_tab(ui: &mut egui::Ui, _state: &mut AppState, action: &mut UiAction) {
    if ui.button("Reset view").clicked() {
        *action = UiAction::ResetView;
    }
    if ui.button("Export SVG").clicked() {
        *action = UiAction::ExportSvg;
    }
    if ui.button("Export PNG").clicked() {
        *action = UiAction::ExportPng;
    }
    if ui.button("PNG atlas stylow").clicked() {
        *action = UiAction::ExportAtlas;
    }
}

fn slider(ui: &mut egui::Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, label: &str) {
    ui.add(egui::Slider::new(value, range).text(label));
}

fn text_choice(ui: &mut egui::Ui, label: &str, value: &mut String, options: &[&str]) {
    egui::ComboBox::from_label(label)
        .selected_text(value.as_str())
        .show_ui(ui, |ui| {
            for option in options {
                ui.selectable_value(value, (*option).to_owned(), *option);
            }
        });
}

fn color_text(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(value);
    });
}
