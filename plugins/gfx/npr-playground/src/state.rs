use amigo_render_npr::{
    BrushLibrary, ComicInk, ComicInkOverrides, NprConstructionMark, NprMotionPolicy,
    NprPreparedSurface, NprStyleLayerOverrides, NprStyleLayers, NprSurfaceAnchorError,
    NprSurfaceIntent, NprSurfaceMode, NprToneMode, StrokeMotionMode, StrokeTool,
};
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Mutex};

pub const MODELS: &[&str] = &["cube", "wedge", "cylinder", "sphere", "suzanne", "avocado"];

pub fn style_preset(name: &str) -> Option<ComicInk> {
    let mut style = ComicInk::default();
    match name {
        "Comic Ink" => {}
        "Hand Ink" => {
            style.tool = StrokeTool::Fineliner;
            style.outline_width = 2.8;
            style.boundary_width = 2.0;
            style.crease_width = 1.1;
            style.crease_angle = 0.85;
            style.taper = 0.22;
            style.wobble = 0.85;
            style.gesture_confidence = 0.82;
            style.gesture_simplification = 0.08;
            style.gesture_correction = 0.12;
            style.gesture_overstroke = 0.14;
            style.tool_pressure = 0.82;
            style.tool_hardness = 0.72;
            style.paper_tooth = 0.12;
            style.paper_grain = 0.08;
            style.ink_dryness = 0.10;
        }
        "Rough Pencil Keys" => {
            style.tool = StrokeTool::Pencil;
            style.tone_mode = NprToneMode::Hatching;
            style.outline_width = 3.2;
            style.boundary_width = 2.3;
            style.crease_width = 1.2;
            style.taper = 0.34;
            style.wobble = 1.8;
            style.gesture_confidence = 0.52;
            style.gesture_simplification = 0.15;
            style.gesture_correction = 0.48;
            style.gesture_overstroke = 0.30;
            style.tool_pressure = 0.68;
            style.tool_hardness = 0.38;
            style.paper_tooth = 0.55;
            style.paper_grain = 0.42;
            style.ink_dryness = 0.24;
            style.tone_density = 0.58;
            style.hatching_spacing = 8.0;
            style.hatching_cross = 0.16;
        }
        "Pencil Art Animation" => {
            style.tool = StrokeTool::Pencil;
            style.tone_mode = NprToneMode::Hatching;
            style.surface_mode = NprSurfaceMode::Smooth;
            style.ink = glam::Vec4::new(0.11, 0.095, 0.078, 0.76);
            style.paper = glam::Vec4::new(0.93, 0.90, 0.84, 1.0);
            style.shadow = glam::Vec4::new(0.24, 0.25, 0.27, 1.0);
            style.mid = glam::Vec4::new(0.55, 0.55, 0.56, 1.0);
            style.light = glam::Vec4::new(0.86, 0.84, 0.79, 1.0);
            style.outline_width = 3.1;
            style.boundary_width = 2.1;
            style.crease_width = 1.25;
            style.crease_angle = 0.82;
            style.taper = 0.38;
            style.wobble = 1.65;
            style.gesture_confidence = 0.48;
            style.gesture_simplification = 0.14;
            style.gesture_correction = 0.38;
            style.gesture_overstroke = 0.22;
            style.tool_pressure = 0.72;
            style.tool_hardness = 0.30;
            style.paper_tooth = 0.70;
            style.paper_grain = 0.82;
            style.ink_dryness = 0.26;
            style.tone_density = 0.52;
            style.hatching_spacing = 8.0;
            style.hatching_cross = 0.18;
            style.suggestive_contours = true;
            style.suggestive_contour_confidence = 0.28;
            style.suggestive_contour_width_scale = 0.52;
            style.suggestive_contour_opacity = 0.42;
            style.form_line_width_scale = 0.42;
            style.form_line_opacity = 0.34;
            style.min_form_line_confidence = 0.30;
        }
        "Pencil Character" => {
            style.tool = StrokeTool::Pencil;
            style.tone_mode = NprToneMode::Hatching;
            style.ink = Vec4::new(0.075, 0.062, 0.050, 0.86);
            style.paper = Vec4::new(0.93, 0.90, 0.84, 1.0);
            style.shadow = Vec4::new(0.20, 0.19, 0.18, 1.0);
            style.outline_width = 3.2;
            style.boundary_width = 2.3;
            style.crease_width = 1.15;
            style.taper = 0.42;
            style.wobble = 2.1;
            style.gesture_confidence = 0.42;
            style.gesture_overstroke = 0.20;
            style.tool_pressure = 0.74;
            style.tool_hardness = 0.34;
            style.paper_tooth = 0.72;
            style.paper_grain = 0.86;
            style.ink_dryness = 0.22;
            style.tone_density = 0.55;
            style.hatching_spacing = 7.5;
            style.hatching_cross = 0.20;
        }
        "Pencil Architecture" => {
            style.tool = StrokeTool::Pencil;
            style.tone_mode = NprToneMode::Hatching;
            style.ink = Vec4::new(0.10, 0.09, 0.08, 0.68);
            style.shadow = Vec4::new(0.30, 0.28, 0.25, 1.0);
            style.outline_width = 2.1;
            style.boundary_width = 1.55;
            style.crease_width = 0.85;
            style.taper = 0.20;
            // Architectural lines are confident, not ruler-perfect. The
            // renderer keeps the realization stable until camera/motion
            // invalidates it, so this remains a quiet pencil bias.
            style.wobble = 0.72;
            style.gesture_confidence = 0.90;
            style.gesture_simplification = 0.05;
            style.gesture_overstroke = 0.02;
            style.tool_pressure = 0.62;
            style.tool_hardness = 0.52;
            style.paper_tooth = 0.54;
            style.paper_grain = 0.58;
            style.ink_dryness = 0.12;
            style.tone_density = 0.28;
            style.hatching_spacing = 10.0;
            style.hatching_cross = 0.10;
            style.suggestive_contours = true;
            style.suggestive_contour_confidence = 0.22;
            style.suggestive_contour_width_scale = 0.34;
            style.suggestive_contour_opacity = 0.24;
            style.form_line_width_scale = 0.30;
            style.form_line_opacity = 0.22;
            style.min_form_line_confidence = 0.42;
        }
        "Pencil Background" => {
            style.tool = StrokeTool::Pencil;
            style.tone_mode = NprToneMode::Hatching;
            style.ink = Vec4::new(0.18, 0.17, 0.16, 0.42);
            style.shadow = Vec4::new(0.42, 0.40, 0.37, 1.0);
            style.outline_width = 1.25;
            style.boundary_width = 0.9;
            style.crease_width = 0.5;
            style.taper = 0.12;
            style.wobble = 0.42;
            style.gesture_confidence = 0.96;
            style.gesture_overstroke = 0.0;
            style.tool_pressure = 0.48;
            style.tool_hardness = 0.62;
            style.paper_tooth = 0.42;
            style.paper_grain = 0.40;
            style.ink_dryness = 0.08;
            style.tone_density = 0.14;
            style.hatching_spacing = 13.0;
            style.hatching_cross = 0.035;
        }
        "Clean TV Ink" => {
            style.tool = StrokeTool::Fineliner;
            style.outline_width = 2.4;
            style.boundary_width = 1.8;
            style.crease_width = 0.9;
            style.taper = 0.16;
            style.wobble = 0.35;
            style.gesture_confidence = 0.94;
            style.gesture_simplification = 0.04;
            style.gesture_overstroke = 0.05;
            style.tool_pressure = 0.86;
            style.tool_hardness = 0.88;
            style.ink_dryness = 0.04;
        }
        "Pencil Study" => {
            style.tool = StrokeTool::Pencil;
            style.tone_mode = NprToneMode::Hatching;
            // Graphite remains warm, but the primary contour must establish
            // the drawing before the paper tooth takes away local coverage.
            style.ink = glam::Vec4::new(0.035, 0.028, 0.022, 1.0);
            style.paper = glam::Vec4::new(0.94, 0.91, 0.84, 1.0);
            style.shadow = glam::Vec4::new(0.25, 0.27, 0.30, 1.0);
            style.mid = glam::Vec4::new(0.58, 0.59, 0.60, 1.0);
            style.light = glam::Vec4::new(0.88, 0.86, 0.80, 1.0);
            style.outline_width = 3.6;
            style.crease_width = 1.6;
            style.boundary_width = 2.4;
            style.taper = 0.26;
            style.wobble = 1.25;
            style.gesture_confidence = 0.72;
            style.gesture_simplification = 0.08;
            style.gesture_correction = 0.18;
            style.gesture_overstroke = 0.12;
            style.tool_pressure = 0.88;
            style.tool_hardness = 0.38;
            style.paper_tooth = 0.58;
            style.paper_grain = 0.72;
            style.ink_dryness = 0.18;
            style.tone_density = 0.44;
            style.hatching_spacing = 7.0;
            style.hatching_cross = 0.10;
            // Smooth suggestive contours supply readable form detail between
            // the silhouette and topology creases; keep them subordinate to
            // the main contour hierarchy.
            style.suggestive_contours = true;
            style.suggestive_contour_confidence = 0.28;
            style.suggestive_contour_width_scale = 0.52;
            style.suggestive_contour_opacity = 0.42;
            style.form_line_width_scale = 0.42;
            style.form_line_opacity = 0.34;
            style.min_form_line_confidence = 0.30;
            // Keep the reviewed Pencil Study streamlines stable. Surface
            // cleanup is an authored object policy (`Natural Smooth`), not an
            // incidental consequence of picking a pigment tool.
            style.smooth_draw_creases = true;
            style.min_smooth_contour_length_pixels = 0.0;
            style.smooth_contour_simplification_pixels = 0.0;
        }
        "Loose Study" => {
            style.tool = StrokeTool::Pencil;
            style.tone_mode = NprToneMode::Hatching;
            style.ink = glam::Vec4::new(0.10, 0.085, 0.07, 0.78);
            style.outline_width = 3.0;
            style.crease_width = 1.15;
            style.boundary_width = 1.8;
            style.taper = 0.34;
            style.wobble = 2.4;
            style.gesture_confidence = 0.28;
            style.gesture_simplification = 0.16;
            style.gesture_correction = 0.82;
            style.gesture_overstroke = 0.34;
            style.tool_pressure = 0.66;
            style.tool_hardness = 0.25;
            style.paper_tooth = 0.72;
            style.paper_grain = 0.90;
            style.ink_dryness = 0.30;
            style.tone_density = 0.68;
            style.hatching_spacing = 5.5;
            style.hatching_cross = 0.32;
        }
        "Confident Ink" => {
            style.tool = StrokeTool::Fineliner;
            style.outline_width = 4.2;
            style.crease_width = 1.75;
            style.boundary_width = 2.8;
            style.taper = 0.22;
            style.gesture_confidence = 0.96;
            style.gesture_simplification = 0.04;
            style.gesture_correction = 0.0;
            style.gesture_overstroke = 0.0;
            style.tool_pressure = 0.82;
            style.tool_hardness = 0.92;
            style.ink_dryness = 0.08;
            style.tone_density = 0.0;
        }
        "Broad Nib" => {
            style.tool = StrokeTool::Nib;
            style.outline_width = 3.8;
            style.crease_width = 1.35;
            style.boundary_width = 2.6;
            style.gesture_confidence = 0.86;
            style.tool_pressure = 0.78;
            style.tool_hardness = 0.74;
            style.nib_angle = -18.0;
            style.nib_aspect = 0.68;
            style.ink_dryness = 0.14;
            style.tone_density = 0.18;
            style.hatching_spacing = 8.0;
        }
        "Blueprint" => {
            style.ink = glam::Vec4::new(0.75, 0.9, 1.0, 1.0);
            style.shadow = glam::Vec4::new(0.025, 0.07, 0.15, 1.0);
            style.mid = glam::Vec4::new(0.04, 0.13, 0.26, 1.0);
            style.light = glam::Vec4::new(0.08, 0.23, 0.38, 1.0);
            style.outline_width = 1.5;
            style.crease_width = 0.8;
            style.wobble = 0.0;
        }
        "Soft Toon" => {
            style.outline_width = 1.2;
            style.crease_width = 0.0;
            style.wobble = 0.0;
            style.taper = 0.0;
        }
        "Watercolour Wash" => {
            style.tool = StrokeTool::Brush;
            style.tone_mode = NprToneMode::ThreeBand;
            style.ink = glam::Vec4::new(0.09, 0.12, 0.16, 0.82);
            style.paper = glam::Vec4::new(0.94, 0.92, 0.86, 1.0);
            style.shadow = glam::Vec4::new(0.22, 0.34, 0.48, 1.0);
            style.mid = glam::Vec4::new(0.48, 0.66, 0.72, 1.0);
            style.light = glam::Vec4::new(0.78, 0.84, 0.76, 1.0);
            style.outline_width = 2.4;
            style.crease_width = 0.7;
            style.wobble = 0.55;
            style.paper_grain = 0.44;
            style.paper_tooth = 0.28;
            style.ink_dryness = 0.08;
        }
        _ => return None,
    }
    Some(style)
}

#[cfg(test)]
mod runtime_mesh_style_tests {
    use super::{style_preset, NprPlaygroundState};
    use crate::documents::NprRuntimeObjectStyleDocument;
    use glam::Vec4;

    #[test]
    fn runtime_redraw_advances_on_stop_and_detects_deformation() {
        let state = NprPlaygroundState::default();
        state.settings.lock().unwrap().runtime_objects.insert("hero".into(),
            NprRuntimeObjectStyleDocument { temporal: Some(true), ..Default::default() });
        let mut mesh = amigo_render_api::MeshDrawCommand {
            entity_id: 1, entity_name: "hero".into(),
            mesh: amigo_render_api::Mesh3d {
                mesh_asset: amigo_assets::AssetKey::new("hero.glb"),
                transform: Default::default(),
                geometry: Some(std::sync::Arc::new(amigo_render_api::MeshGeometry3d {
                    positions: vec![[0.0; 3]], indices: vec![], material_indices: vec![], material_colors: std::sync::Arc::new([]), topology_edges: vec![], triangle_edges: vec![],
                    reference_positions: vec![[0.0; 3]].into(),
                })),
            },
        };
        let epoch = |mesh: &amigo_render_api::MeshDrawCommand| state.runtime_mesh_styles(std::slice::from_ref(mesh))[0].artistic_frame;
        assert_eq!(epoch(&mesh), 0);
        state.tick(0.13);
        let stationary = epoch(&mesh);
        assert!(stationary > 0, "time alone must redraw a stationary subject");
        mesh.mesh.transform.translation.x = 0.1;
        state.tick(0.13);
        let moved = epoch(&mesh);
        assert!(moved > stationary);
        // Reallocation of identical geometry must not look like deformation.
        mesh.mesh.geometry = mesh.mesh.geometry.as_ref().map(|g| std::sync::Arc::new((**g).clone()));
        assert_eq!(epoch(&mesh), moved);
        std::sync::Arc::make_mut(mesh.mesh.geometry.as_mut().unwrap()).positions[0][1] = 0.2;
        state.tick(0.13);
        let deformed = epoch(&mesh);
        assert!(deformed > moved);
        assert_eq!(epoch(&mesh), deformed, "multiple extraction consumers share a drawing");
        state.runtime_mesh_styles(&[]);
        assert!(state.runtime_drawing_poses.lock().unwrap().is_empty());
    }

    #[test]
    fn artistic_frame_advances_at_redraw_rate_instead_of_host_rate() {
        let state = NprPlaygroundState::default();
        state.settings.lock().unwrap().runtime_objects.insert(
            "moving".into(),
            NprRuntimeObjectStyleDocument {
                temporal: Some(true),
                ..Default::default()
            },
        );
        assert_eq!(state.runtime_mesh_style_for("moving").artistic_frame, 0);
        state.tick(0.05);
        assert_eq!(state.runtime_mesh_style_for("moving").artistic_frame, 0);
        state.tick(0.08);
        assert_eq!(state.runtime_mesh_style_for("moving").artistic_frame, 1);
        assert_eq!(state.runtime_mesh_style_for("moving").artistic_frame, 1);
    }

    #[test]
    fn static_entities_receive_the_global_drawing_epoch() {
        let state = NprPlaygroundState::default();
        state.tick(1.0);
        let static_style = state.runtime_mesh_style_for("building");
        assert!(static_style.temporal);
        assert!(static_style.artistic_frame > 0);
    }

    #[test]
    fn pencil_art_animation_preset_uses_graphite_parameters() {
        let state = NprPlaygroundState::default();
        state.settings.lock().unwrap().runtime_objects.insert(
            "hero".into(),
            NprRuntimeObjectStyleDocument {
                preset: Some("Pencil Art Animation".into()),
                temporal: Some(true),
                ..Default::default()
            },
        );
        let style = state.runtime_mesh_style_for("hero");
        assert!(style.temporal);
        assert!(style.pencil_grain > 0.7);
        assert!(style.hardness < 0.5);
        assert!(style.pressure_variation > 0.3);
        assert!(style.draw_form_lines);
        assert!(style.form_line_width_pixels > 0.0);
    }

    #[test]
    fn pencil_role_presets_keep_character_and_background_distinct() {
        let character = style_preset("Pencil Character").unwrap();
        let architecture = style_preset("Pencil Architecture").unwrap();
        let background = style_preset("Pencil Background").unwrap();
        assert!(character.wobble > architecture.wobble);
        assert!(architecture.outline_width > background.outline_width);
        assert!(character.paper_grain > background.paper_grain);
        assert!(character.hatching_spacing < background.hatching_spacing);
        assert!(architecture.suggestive_contours);
    }

    #[test]
    fn runtime_preset_inherits_scene_palette_and_can_enable_role_creases() {
        let state = NprPlaygroundState::default();
        let mut settings = state.settings.lock().unwrap();
        settings.global.paper = Vec4::new(0.12, 0.13, 0.14, 1.0);
        settings.global.ink = Vec4::new(0.21, 0.22, 0.23, 1.0);
        settings.runtime_objects.insert(
            "building".into(),
            NprRuntimeObjectStyleDocument {
                preset: Some("Pencil Architecture".into()),
                creases: Some(true),
                ..Default::default()
            },
        );
        drop(settings);
        let style = state.runtime_mesh_style_for("building");
        assert_eq!(style.background, [0.12, 0.13, 0.14, 1.0]);
        assert_eq!(style.ink, [0.21, 0.22, 0.23, 1.0]);
        assert!(style.draw_creases);
        assert!(style.draw_contact_lines);
    }

    #[test]
    fn runtime_entity_style_is_explicit_and_inherits_its_preset() {
        let state = NprPlaygroundState::default();
        let mut authored = NprRuntimeObjectStyleDocument {
            preset: Some("Rough Pencil Keys".into()),
            hatching: Some(true),
            stroke_segments: Some(7),
            pressure_variation: Some(0.42),
            ..Default::default()
        };
        authored.style.outline_width = Some(5.0);
        state
            .settings
            .lock()
            .unwrap()
            .runtime_objects
            .insert("hero-officer".into(), authored);

        let hero = state.runtime_mesh_style_for("hero-officer");
        let building = state.runtime_mesh_style_for("city-block-a");
        assert_eq!(hero.outline_width_pixels, 5.0);
        assert!(hero.hatching_enabled);
        assert_eq!(hero.stroke_segments, 7);
        assert!((hero.pressure_variation - 0.42).abs() < f32::EPSILON);
        assert!(hero.wobble_pixels > building.wobble_pixels);
        assert_ne!(building.outline_width_pixels, 5.0);
    }

    #[test]
    fn runtime_entity_rejects_an_unknown_preset() {
        let mut settings = super::Settings::for_scene();
        settings.runtime_objects.insert(
            "hero-officer".into(),
            NprRuntimeObjectStyleDocument {
                preset: Some("Missing Brush".into()),
                ..Default::default()
            },
        );
        assert!(settings.validate().unwrap_err().contains("unknown preset"));
    }
}

/// Stable diagnostic id for the effective typed look. The render contract
/// carries an id rather than the UI label so diagnostics remain useful when
/// labels are localized or renamed.
pub fn style_preset_id(style: ComicInk) -> &'static str {
    let defaults = ComicInk::default();
    let normalize = |mut value: ComicInk| {
        value.paper = defaults.paper;
        value.light_direction = defaults.light_direction;
        value.surface_mode = defaults.surface_mode;
        value
    };
    let normalized = normalize(style);
    [
        ("comic-ink", "Comic Ink"),
        ("hand-ink", "Hand Ink"),
        ("rough-pencil-keys", "Rough Pencil Keys"),
        ("pencil-art-animation", "Pencil Art Animation"),
        ("clean-tv-ink", "Clean TV Ink"),
        ("pencil-study", "Pencil Study"),
        ("loose-study", "Loose Study"),
        ("confident-ink", "Confident Ink"),
        ("broad-nib", "Broad Nib"),
        ("blueprint", "Blueprint"),
        ("soft-toon", "Soft Toon"),
        ("watercolour-wash", "Watercolour Wash"),
    ]
    .into_iter()
    .find_map(|(id, label)| {
        let candidate = normalize(style_preset(label)?);
        (candidate == normalized).then_some(id)
    })
    .unwrap_or("custom")
}
fn default_surface_subdivision_level() -> u8 {
    1
}
fn default_smooth_weld_relative_tolerance() -> f32 {
    1.0e-5
}

/// A point on an authored model surface.  It deliberately omits the runtime
/// surface revision: RenderExtract attaches that identity after it selects the
/// immutable prepared source mesh.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionAnchorSettings {
    pub triangle: u32,
    pub barycentric: [f32; 3],
}

/// A scene/editor-facing construction mark.  This is the stable authored form;
/// `NprConstructionMark` is the validated render-domain form.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionMarkSettings {
    pub id: u32,
    pub anchors: Vec<ConstructionAnchorSettings>,
    #[serde(default)]
    pub closed: bool,
    #[serde(default = "default_construction_width_scale")]
    pub width_scale: f32,
    #[serde(default = "default_construction_opacity")]
    pub opacity: f32,
}

fn default_construction_width_scale() -> f32 {
    0.5
}

fn default_construction_opacity() -> f32 {
    0.35
}

impl ConstructionMarkSettings {
    fn validate(&self) -> Result<(), String> {
        let minimum_anchors = if self.closed { 3 } else { 2 };
        if self.anchors.len() < minimum_anchors {
            return Err(format!(
                "construction mark {} needs at least {minimum_anchors} anchors",
                self.id
            ));
        }
        if self.anchors.iter().any(|anchor| {
            !anchor.barycentric.iter().all(|value| value.is_finite())
                || anchor.barycentric.iter().any(|value| *value < 0.0)
                || (anchor.barycentric.iter().sum::<f32>() - 1.0).abs() > 1e-4
        }) {
            return Err(format!(
                "construction mark {} has invalid barycentric coordinates",
                self.id
            ));
        }
        if !self.width_scale.is_finite() || !(0.0..=2.0).contains(&self.width_scale) {
            return Err(format!(
                "construction mark {} has invalid width scale",
                self.id
            ));
        }
        if !self.opacity.is_finite() || !(0.0..=1.0).contains(&self.opacity) {
            return Err(format!("construction mark {} has invalid opacity", self.id));
        }
        Ok(())
    }

    pub fn resolve(
        &self,
        source: &NprPreparedSurface,
    ) -> Result<NprConstructionMark, NprSurfaceAnchorError> {
        Ok(NprConstructionMark {
            id: self.id,
            anchors: self
                .anchors
                .iter()
                .map(|anchor| source.anchor(anchor.triangle, anchor.barycentric))
                .collect::<Result<_, _>>()?,
            closed: self.closed,
            width_scale: self.width_scale,
            opacity: self.opacity,
        })
    }
}

/// Transient two-point authoring state. It is intentionally not serialized:
/// only a complete, source-anchored construction mark becomes scene data.
#[derive(Debug, Default)]
struct ConstructionAuthoringState {
    object_id: Option<String>,
    anchors: Vec<ConstructionAnchorSettings>,
    waiting_for_release: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectSettings {
    pub model: String,
    /// Explicit material contribution used only when an authored NPR layer
    /// selects `model-base-colour`. It is object data, never inferred by WGPU.
    pub material_base_color: Vec4,
    /// Semantic source of the drawing policy.  It is authored data: WGPU only
    /// receives the resolved mode and never guesses from a model identifier.
    #[serde(default)]
    pub surface_intent: NprSurfaceIntent,
    #[serde(default)]
    pub surface_mode: NprSurfaceMode,
    /// Fixed smooth-proxy level, prepared per source revision rather than per
    /// camera frame. Zero means use the authored source surface directly.
    #[serde(default = "default_surface_subdivision_level")]
    pub surface_subdivision_level: u8,
    /// Seam-weld radius relative to model scale, used only by Smooth proxies.
    #[serde(default = "default_smooth_weld_relative_tolerance")]
    pub smooth_weld_relative_tolerance: f32,
    pub visible: bool,
    pub rotating: bool,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: f32,
    pub angular_speed: Vec3,
    /// A user-authored gesture epoch. It is separate from motion-driven redraw
    /// and changes only after the explicit workshop action.
    #[serde(default)]
    pub gesture_variant: u32,
    /// Sparse local values. `None` preserves the scene style, while a value
    /// shadows only that parameter.
    #[serde(default)]
    pub style_overrides: ComicInkOverrides,
    /// A whole layer stack may be intentionally replaced for an object. Layer
    /// parameter overrides are introduced only once every layer has a stable
    /// renderer execution path.
    #[serde(default)]
    pub style_layer_overrides: NprStyleLayerOverrides,
    /// Authored marks resolve against the selected source surface only during
    /// RenderExtract, so authored data never stores an internal mesh revision.
    #[serde(default)]
    pub construction_marks: Vec<ConstructionMarkSettings>,
}

impl ObjectSettings {
    pub fn effective_style(&self, scene_style: ComicInk) -> ComicInk {
        self.style_overrides.resolve(scene_style)
    }

    pub fn has_style_overrides(&self) -> bool {
        !self.style_overrides.is_empty()
    }

    pub fn effective_layers(&self, scene_layers: &NprStyleLayers) -> NprStyleLayers {
        self.style_layer_overrides
            .resolve(scene_layers)
            .expect("object layer overrides are validated with the scene stack")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// Runtime transport only: excluded from presets, drafts and authored history.
    #[serde(skip)]
    pub playback: Option<crate::playback::ModelPlayback>,
    pub global: ComicInk,
    /// Ordered scene-owned drawing layers. Object-level layer overrides are
    /// introduced separately from style-parameter inheritance so stable layer
    /// identities can be preserved across preset changes.
    #[serde(default)]
    pub style_layers: NprStyleLayers,
    /// Resolved brush definitions used by the extractor. References remain
    /// pinned in layers; this runtime copy supplies their defaults.
    #[serde(default)]
    pub brushes: BrushLibrary,
    pub objects: BTreeMap<String, ObjectSettings>,
    #[serde(default)]
    pub runtime_objects: BTreeMap<String, crate::documents::NprRuntimeObjectStyleDocument>,
    pub selected: String,
    pub paused: bool,
    /// Stops time-driven gesture variants without affecting model playback or
    /// camera interaction.
    #[serde(default)]
    pub sketch_paused: bool,
    pub speed: f32,
    pub step: bool,
    #[serde(default)]
    pub motion: NprMotionPolicy,
    pub seed: u64,
    pub debug: String,
    pub camera_target: Vec3,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_distance: f32,
    pub camera_fov: f32,
    pub preset_name: String,
    pub style_scope: String,
    pub preset_kind: String,
}
impl Settings {
    /// Whether time alone can change the rendered scene.
    pub fn needs_temporal_frames(&self) -> bool {
        let visible =
            |(id, object): (&String, &ObjectSettings)| object.visible && id == &self.selected;
        self.objects.iter().any(|entry| visible(entry))
            && ((!self.paused
                && self.speed != 0.0
                && self
                    .objects
                    .iter()
                    .any(|entry| visible(entry) && entry.1.rotating)
                && self.playback.is_none())
                || self
                    .playback
                    .as_ref()
                    .is_some_and(|p| p.playing && p.speed > 0.0)
                || (!self.sketch_paused
                    && self.motion.mode == StrokeMotionMode::RedrawContinuously))
    }

    pub fn for_scene() -> Self {
        let objects = MODELS
            .iter()
            .take(1)
            .enumerate()
            .map(|(_i, id)| {
                (
                    (*id).into(),
                    ObjectSettings {
                        model: (*id).into(),
                        material_base_color: default_material_base_color(id),
                        surface_intent: match *id {
                            "cube" | "wedge" => NprSurfaceIntent::HardSurface,
                            _ => NprSurfaceIntent::Organic,
                        },
                        surface_mode: match *id {
                            "cube" | "wedge" => NprSurfaceMode::Polygonal,
                            _ => NprSurfaceMode::Smooth,
                        },
                        surface_subdivision_level: match *id {
                            "cube" | "wedge" => 0,
                            _ => default_surface_subdivision_level(),
                        },
                        smooth_weld_relative_tolerance: default_smooth_weld_relative_tolerance(),
                        visible: true,
                        // Drawing Studio starts from a stable authored pose.
                        // Rotation remains an explicit opt-in in the motion
                        // controls and in serialized scene documents.
                        rotating: false,
                        position: Vec3::ZERO,
                        rotation: Vec3::new(0.36_f32.to_degrees(), 0.71_f32.to_degrees(), 0.0),
                        scale: 1.0,
                        angular_speed: Vec3::new(21.2, 40.7, 0.0),
                        gesture_variant: 0,
                        style_overrides: ComicInkOverrides::default(),
                        style_layer_overrides: NprStyleLayerOverrides::default(),
                        construction_marks: vec![],
                    },
                )
            })
            .collect();
        Self {
            global: ComicInk::default(),
            playback: None,
            style_layers: NprStyleLayers::default(),
            brushes: BrushLibrary::drawing_studio(),
            objects,
            runtime_objects: BTreeMap::new(),
            selected: "cube".into(),
            paused: false,
            sketch_paused: false,
            speed: 1.0,
            step: false,
            motion: NprMotionPolicy::default(),
            seed: 0x4e5052,
            debug: "Final".into(),
            camera_target: Vec3::ZERO,
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_distance: 5.0,
            camera_fov: 45.0,
            preset_name: "my-preset".into(),
            style_scope: "scene".into(),
            preset_kind: "scene".into(),
        }
    }

    /// A scene-owned NPR workspace with no instantiated model. Catalog assets
    /// remain available, but no draw command is emitted until an asset is added.
    pub fn empty_scene() -> Self {
        let mut settings = Self::for_scene();
        settings.objects.clear();
        settings.selected.clear();
        settings
    }
    pub fn validate(&self) -> Result<(), String> {
        if !["scene", "object"].contains(&self.style_scope.as_str()) {
            return Err("invalid style scope".into());
        }
        if !["scene", "look"].contains(&self.preset_kind.as_str()) {
            return Err("invalid preset kind".into());
        }
        if (self.objects.is_empty() && !self.selected.is_empty())
            || (!self.objects.is_empty() && !self.objects.contains_key(&self.selected))
        {
            return Err("invalid object selection/set".into());
        }
        if self.objects.len() > 1 {
            return Err("Drawing Studio supports exactly one source model; select another model to start a new drawing".into());
        }
        if !["Final", "FeatureClasses", "StrokeIds"].contains(&self.debug.as_str()) {
            return Err("unknown debug view".into());
        }
        if !self.speed.is_finite()
            || !(0.0..=4.0).contains(&self.speed)
            || !self.motion.appearance_fade_seconds.is_finite()
            || !(0.0..=2.0).contains(&self.motion.appearance_fade_seconds)
            || !self.motion.redraw_hz.is_finite()
            || !(0.25..=20.0).contains(&self.motion.redraw_hz)
            || !self.motion.redraw_strength.is_finite()
            || !(0.0..=1.0).contains(&self.motion.redraw_strength)
            || !(15.0..=90.0).contains(&self.camera_fov)
            || !self.camera_fov.is_finite()
            || !(0.1..=100.0).contains(&self.camera_distance)
            || !self.camera_distance.is_finite()
            || !self.camera_target.is_finite()
            || !self.camera_yaw.is_finite()
            || !self.camera_pitch.is_finite()
            || self.camera_pitch.abs() > 89.0
        {
            return Err("invalid camera/animation parameters".into());
        }
        if !matches!(
            self.motion.mode,
            StrokeMotionMode::Stable
                | StrokeMotionMode::RedrawOnMotion
                | StrokeMotionMode::RedrawContinuously
        ) {
            return Err("invalid stroke motion mode".into());
        }
        validate_style(self.global)?;
        self.style_layers.validate()?;
        for object in self.objects.values() {
            if object.style_overrides.paper.is_some()
                || object.style_layer_overrides.layers.contains_key("paper")
            {
                return Err(
                    "Paper belongs to the scene and cannot be overridden by an object".into(),
                );
            }
            if crate::documents::validate_document_id(&object.model).is_err()
                || !object.position.is_finite()
                || !object.material_base_color.is_finite()
                || object.material_base_color.min_element() < 0.0
                || object.material_base_color.max_element() > 1.0
                || !object.rotation.is_finite()
                || !object.angular_speed.is_finite()
                || !object.scale.is_finite()
                || !(0.01..=10.0).contains(&object.scale)
                || object.surface_subdivision_level > 2
                || !object.smooth_weld_relative_tolerance.is_finite()
                || !(0.0..=1.0e-2).contains(&object.smooth_weld_relative_tolerance)
            {
                return Err("invalid object parameters".into());
            }
            validate_style(object.effective_style(self.global))?;
            object
                .style_layer_overrides
                .resolve(&self.style_layers)?
                .validate()?;
            let mut construction_ids = std::collections::BTreeSet::new();
            for mark in &object.construction_marks {
                mark.validate()?;
                if !construction_ids.insert(mark.id) {
                    return Err(format!("duplicate construction mark id {}", mark.id));
                }
            }
        }
        for (entity, runtime_style) in &self.runtime_objects {
            if entity.trim().is_empty() {
                return Err("runtime NPR object id cannot be empty".into());
            }
            let inherited = match runtime_style.preset.as_deref() {
                Some(preset) => style_preset(preset).ok_or_else(|| {
                    format!("runtime object `{entity}` uses unknown preset `{preset}`")
                })?,
                None => self.global,
            };
            validate_style(runtime_style.style.resolve(inherited))
                .map_err(|error| format!("invalid runtime object `{entity}` style: {error}"))?;
            if runtime_style.stroke_segments.is_some_and(|value| !(2..=8).contains(&value)) {
                return Err(format!("runtime object `{entity}` has invalid stroke_segments"));
            }
            if runtime_style
                .pressure_variation
                .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
            {
                return Err(format!("runtime object `{entity}` has invalid pressure_variation"));
            }
        }
        Ok(())
    }
}
fn default_material_base_color(model: &str) -> Vec4 {
    match model {
        // These are authored defaults for the built-in study models. They do
        // not influence a palette layer; they become visible only after a
        // layer explicitly chooses `model-base-colour`.
        "cube" => Vec4::new(0.82, 0.30, 0.20, 1.0),
        "wedge" => Vec4::new(0.94, 0.66, 0.20, 1.0),
        "cylinder" => Vec4::new(0.22, 0.59, 0.43, 1.0),
        "sphere" => Vec4::new(0.22, 0.48, 0.78, 1.0),
        "suzanne" => Vec4::new(0.56, 0.32, 0.68, 1.0),
        "avocado" => Vec4::new(0.30, 0.65, 0.24, 1.0),
        _ => Vec4::ONE,
    }
}
fn validate_style(s: ComicInk) -> Result<(), String> {
    if !s.light_direction.is_finite() || s.light_direction.length_squared() < 1e-8 {
        return Err("light direction must be finite and nonzero".into());
    }
    for c in [s.paper, s.shadow, s.mid, s.light, s.ink] {
        if !c.is_finite() || c.min_element() < 0.0 || c.max_element() > 1.0 {
            return Err("color must be finite RGBA in 0..1".into());
        }
    }
    if [s.outline_width, s.crease_width, s.boundary_width]
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=20.0).contains(v))
        || !s.min_crease_length_pixels.is_finite()
        || !(0.0..=64.0).contains(&s.min_crease_length_pixels)
        || !s.min_smooth_contour_length_pixels.is_finite()
        || !(0.0..=64.0).contains(&s.min_smooth_contour_length_pixels)
        || !s.smooth_contour_simplification_pixels.is_finite()
        || !(0.0..=8.0).contains(&s.smooth_contour_simplification_pixels)
        || !s.taper.is_finite()
        || !(0.0..=1.0).contains(&s.taper)
        || !s.wobble.is_finite()
        || !(0.0..=10.0).contains(&s.wobble)
        || !s.crease_angle.is_finite()
        || !(0.0..=std::f32::consts::PI).contains(&s.crease_angle)
        || !s.smooth_crease_angle.is_finite()
        || !(0.0..=std::f32::consts::PI).contains(&s.smooth_crease_angle)
        || [
            s.gesture_confidence,
            s.gesture_simplification,
            s.gesture_correction,
            s.gesture_overstroke,
            s.tool_pressure,
            s.tool_hardness,
            s.paper_tooth,
            s.paper_grain,
            s.ink_dryness,
            s.tone_density,
            s.min_form_line_confidence,
            s.suggestive_contour_confidence,
            s.suggestive_contour_opacity,
            s.form_line_opacity,
            s.hatching_cross,
        ]
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        || [s.suggestive_contour_width_scale, s.form_line_width_scale]
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=2.0).contains(v))
        || !s.nib_angle.is_finite()
        || !(-180.0..=180.0).contains(&s.nib_angle)
        || !s.nib_aspect.is_finite()
        || !(0.0..=1.0).contains(&s.nib_aspect)
        || !s.hatching_angle.is_finite()
        || !(-180.0..=180.0).contains(&s.hatching_angle)
        || !s.hatching_spacing.is_finite()
        || !(1.0..=40.0).contains(&s.hatching_spacing)
    {
        return Err("invalid ink parameters".into());
    }
    Ok(())
}
pub struct NprPlaygroundState {
    pub settings: Mutex<Settings>,
    /// Editor-only progressive reveal; never persisted in authored documents.
    pub build_up: Mutex<f32>,
    /// Editor-only layer isolation; paper remains visible beneath the solo layer.
    pub solo_layer: Mutex<Option<String>>,
    defaults: Mutex<Settings>,
    pub fps: Mutex<f64>,
    artistic_elapsed_seconds: Mutex<f32>,
    runtime_drawing_poses: Mutex<BTreeMap<u64, (amigo_render_api::MeshDrawCommand, u64)>>,
    last_frame: Mutex<Option<std::time::Instant>>,
    benchmark: Mutex<Option<(std::time::Instant, Vec<f64>)>>,
    pub viewport: Mutex<[u32; 2]>,
    authored_scene: Mutex<Option<crate::scene::NprPlaygroundSceneDocument>>,
    construction_authoring: Mutex<ConstructionAuthoringState>,
    selected_construction_mark: Mutex<Option<usize>>,
    pub render_stats: Mutex<BTreeMap<String, u64>>,
}
impl Default for NprPlaygroundState {
    fn default() -> Self {
        let s = Settings::for_scene();
        Self {
            settings: Mutex::new(s.clone()),
            build_up: Mutex::new(1.0),
            solo_layer: Mutex::new(None),
            defaults: Mutex::new(s),
            fps: Mutex::new(0.0),
            artistic_elapsed_seconds: Mutex::new(0.0),
            runtime_drawing_poses: Mutex::new(BTreeMap::new()),
            last_frame: Mutex::new(None),
            benchmark: Mutex::new(
                std::env::var_os("AMIGO_NPR_BENCHMARK")
                    .map(|_| (std::time::Instant::now(), Vec::new())),
            ),
            viewport: Mutex::new([512, 512]),
            authored_scene: Mutex::new(None),
            construction_authoring: Mutex::new(ConstructionAuthoringState::default()),
            selected_construction_mark: Mutex::new(None),
            render_stats: Mutex::new(
                [
                    "geometry",
                    "surface_source_vertices",
                    "surface_proxy_vertices",
                    "surface_source_triangles",
                    "surface_proxy_triangles",
                    "topology_edges",
                    "feature_segments",
                    "feature_candidates",
                    "feature_rejected",
                    "smooth_contour_rejected",
                    "suggestive_contour_rejected",
                    "smooth_contour_spans",
                    "suggestive_contour_spans",
                    "silhouettes",
                    "creases",
                    "strokes",
                    "underpainting_triangles",
                    "stroke_vertices",
                    "stroke_indices",
                    "hatching_strokes",
                    "form_line_strokes",
                    "hatching_confidence_rejected",
                    "construction_marks",
                    "construction_rejected",
                    "temporal_retained_strokes",
                    "temporal_entering_strokes",
                    "stroke_budget_rejected",
                    "stroke_budget_exhausted",
                    "viewport_width",
                    "viewport_height",
                ]
                .into_iter()
                .map(|key| (key.into(), 0))
                .collect(),
            ),
        }
    }
}
impl NprPlaygroundState {
    /// Returns a typed scene payload for the current declarative NPR intent.
    /// A scene/editor owner decides when and where that payload is persisted.
    pub fn authored_scene_document(
        &self,
    ) -> Result<crate::documents::NprSceneProfileDocument, String> {
        crate::documents::NprSceneProfileDocument::from_settings(&self.snapshot(), None)
    }

    pub fn stage_authored_scene(&self, authored: crate::scene::NprPlaygroundSceneDocument) {
        *self.authored_scene.lock().unwrap() = Some(authored);
    }

    pub fn take_staged_authored_scene(&self) -> Option<crate::scene::NprPlaygroundSceneDocument> {
        self.authored_scene.lock().unwrap().take()
    }

    pub fn apply_authored_scene(
        &self,
        authored: crate::documents::NprSceneProfileDocument,
    ) -> Result<(), String> {
        let settings = authored.resolve(&BTreeMap::new(), &BTreeMap::new(), &Default::default())?;
        *self.defaults.lock().unwrap() = settings.clone();
        *self.settings.lock().unwrap() = settings;
        *self.artistic_elapsed_seconds.lock().unwrap() = 0.0;
        self.runtime_drawing_poses.lock().unwrap().clear();
        *self.construction_authoring.lock().unwrap() = ConstructionAuthoringState::default();
        *self.selected_construction_mark.lock().unwrap() = None;
        Ok(())
    }

    /// Starts an open construction line on the currently selected object.
    pub fn begin_construction_mark(&self) -> Result<(), String> {
        let selected = self.settings.lock().unwrap().selected.clone();
        if selected.is_empty() {
            return Err("add and select a model before authoring a stroke".into());
        }
        *self.construction_authoring.lock().unwrap() = ConstructionAuthoringState {
            object_id: Some(selected),
            anchors: Vec::new(),
            // The same left press activated the panel button. Wait for it to
            // end so it cannot also become a point on the viewport.
            waiting_for_release: true,
        };
        Ok(())
    }

    /// Discards an incomplete construction line without changing authored data.
    pub fn cancel_construction_mark(&self) {
        *self.construction_authoring.lock().unwrap() = ConstructionAuthoringState::default();
    }

    /// Removes the most recently placed draft point without changing authored
    /// scene data or the document undo history.
    pub fn undo_construction_anchor(&self) -> Result<(), String> {
        let mut authoring = self.construction_authoring.lock().unwrap();
        if authoring.object_id.is_none() {
            return Err("construction mark authoring is not active".into());
        }
        authoring
            .anchors
            .pop()
            .ok_or("the construction mark has no points to remove")?;
        Ok(())
    }

    pub fn construction_authoring_active(&self) -> bool {
        self.construction_authoring
            .lock()
            .unwrap()
            .object_id
            .is_some()
    }

    /// Arms authoring after the panel-button press has been released.
    pub fn construction_authoring_accepts_click(&self, mouse_left_down: bool) -> bool {
        let mut authoring = self.construction_authoring.lock().unwrap();
        if authoring.object_id.is_none() {
            return false;
        }
        if authoring.waiting_for_release {
            if !mouse_left_down {
                authoring.waiting_for_release = false;
            }
            return false;
        }
        true
    }

    /// Adds one source-surface point to the in-progress construction line.
    ///
    /// The selected object is fixed when authoring starts. This prevents a
    /// source change from silently joining anchors from separate meshes.
    pub fn place_construction_anchor(
        &self,
        object_id: &str,
        anchor: ConstructionAnchorSettings,
    ) -> Result<(), String> {
        let mut authoring = self.construction_authoring.lock().unwrap();
        let expected = authoring
            .object_id
            .as_deref()
            .ok_or("construction mark authoring is not active")?;
        if expected != object_id {
            return Err(format!(
                "select {expected} before placing its construction mark"
            ));
        }
        authoring.anchors.push(anchor);
        Ok(())
    }

    /// Atomically commits the in-progress line. Closed marks require three
    /// points; open marks require two.
    pub fn commit_construction_mark(&self, closed: bool) -> Result<(), String> {
        let (object_id, anchors) = {
            let mut authoring = self.construction_authoring.lock().unwrap();
            if authoring.object_id.is_none() {
                return Err("construction mark authoring is not active".into());
            }
            let draft = ConstructionMarkSettings {
                id: 0,
                anchors: authoring.anchors.clone(),
                closed,
                width_scale: default_construction_width_scale(),
                opacity: default_construction_opacity(),
            };
            draft.validate()?;
            (
                std::mem::take(&mut authoring.object_id).expect("active authoring has an object"),
                std::mem::take(&mut authoring.anchors),
            )
        };
        let mut settings = self.settings.lock().unwrap();
        let object = settings
            .objects
            .get_mut(&object_id)
            .ok_or_else(|| format!("unknown construction-mark object {object_id}"))?;
        let mut id = 0x4000_0000u32;
        while object.construction_marks.iter().any(|mark| mark.id == id) {
            id = id
                .checked_add(1)
                .ok_or("construction mark id space is exhausted")?;
        }
        object.construction_marks.push(ConstructionMarkSettings {
            id,
            anchors,
            closed,
            width_scale: default_construction_width_scale(),
            opacity: default_construction_opacity(),
        });
        settings.validate()?;
        let selected_index = settings.objects[&object_id].construction_marks.len() - 1;
        drop(settings);
        *self.selected_construction_mark.lock().unwrap() = Some(selected_index);
        Ok(())
    }

    fn selected_construction_mark_index(&self, count: usize) -> Option<usize> {
        (count > 0).then(|| {
            self.selected_construction_mark
                .lock()
                .unwrap()
                .unwrap_or(count - 1)
                .min(count - 1)
        })
    }

    /// Selects a mark within the current object's authored list without
    /// changing scene data. The editor can later replace this navigation with
    /// a structured list while retaining the same selected-index contract.
    pub fn select_construction_mark(&self, direction: isize) -> Result<(), String> {
        let count = {
            let settings = self.settings.lock().unwrap();
            settings.objects[&settings.selected]
                .construction_marks
                .len()
        };
        let current = self
            .selected_construction_mark_index(count)
            .ok_or("the selected object has no construction marks")?;
        let next = if direction < 0 {
            current.saturating_sub(direction.unsigned_abs())
        } else {
            current.saturating_add(direction as usize).min(count - 1)
        };
        *self.selected_construction_mark.lock().unwrap() = Some(next);
        Ok(())
    }

    /// Removes the selected authored mark from the authored settings.
    pub fn delete_selected_construction_mark(&self) -> Result<(), String> {
        let selected_index = {
            let settings = self.settings.lock().unwrap();
            self.selected_construction_mark_index(
                settings.objects[&settings.selected]
                    .construction_marks
                    .len(),
            )
            .ok_or("the selected object has no construction marks")?
        };
        let mut settings = self.settings.lock().unwrap();
        let selected = settings.selected.clone();
        let remaining = {
            let object = settings
                .objects
                .get_mut(&selected)
                .ok_or_else(|| format!("unknown construction-mark object {selected}"))?;
            object.construction_marks.remove(selected_index);
            object.construction_marks.len()
        };
        drop(settings);
        *self.selected_construction_mark.lock().unwrap() =
            (remaining > 0).then(|| selected_index.min(remaining - 1));
        Ok(())
    }

    pub fn configure_scene(&self) {
        let settings = Settings::for_scene();
        *self.defaults.lock().unwrap() = settings.clone();
        *self.settings.lock().unwrap() = settings;
        *self.artistic_elapsed_seconds.lock().unwrap() = 0.0;
        self.runtime_drawing_poses.lock().unwrap().clear();
        *self.construction_authoring.lock().unwrap() = ConstructionAuthoringState::default();
        *self.selected_construction_mark.lock().unwrap() = None;
    }
    pub fn tick(&self, dt: f32) {
        let mut s = self.settings.lock().unwrap();
        if let Some(mut playback) = s.playback.take() {
            playback.advance(dt);
            let selected = s.selected.clone();
            if playback.source == crate::playback::PlaybackSource::Turntable {
                if let Some(object) = s.objects.get_mut(&selected) {
                    object.rotation = playback.rotation(object.angular_speed);
                }
            }
            s.playback = Some(playback);
            s.step = false;
            return;
        }
        let delta = if s.step && s.paused {
            1.0 / 60.0
        } else if s.paused {
            0.0
        } else {
            dt * s.speed
        };
        if delta > 0.0 {
            *self.artistic_elapsed_seconds.lock().unwrap() += delta;
        }
        s.step = false;
        for o in s.objects.values_mut() {
            if delta > 0.0 && o.rotating {
                o.rotation = (o.rotation + o.angular_speed * delta).map(|v| v.rem_euclid(360.0));
            }
        }
    }
    pub fn record_frame(&self) {
        let now = std::time::Instant::now();
        if let Some(last) = self.last_frame.lock().unwrap().replace(now) {
            let dt = now.duration_since(last).as_secs_f64();
            if dt > 0.0 {
                if let Some((started, intervals)) = self.benchmark.lock().unwrap().as_mut() {
                    intervals.push(dt * 1000.0);
                    if started.elapsed().as_secs_f64() >= 1.0 {
                        intervals.sort_by(f64::total_cmp);
                        eprintln!(
                            "npr-host-benchmark: {}",
                            serde_json::json!({
                                "fps": intervals.len() as f64 / started.elapsed().as_secs_f64(),
                                "interval_p95_ms": intervals[(intervals.len() * 95 / 100).min(intervals.len()-1)],
                            })
                        );
                        intervals.clear();
                        *started = now;
                    }
                }
                let mut fps = self.fps.lock().unwrap();
                *fps = if *fps == 0.0 {
                    1.0 / dt
                } else {
                    *fps * 0.9 + 0.1 / dt
                };
            }
        }
    }
    pub fn snapshot(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn runtime_mesh_style(&self) -> amigo_render_api::NprMeshStyle {
        self.runtime_mesh_style_for("")
    }

    /// Resolve a frame's drawing epochs in the NPR domain. The camera is not
    /// part of a subject pose: moving the camera must not redraw its graphite.
    pub fn runtime_mesh_styles(
        &self,
        meshes: &[amigo_render_api::MeshDrawCommand],
    ) -> Vec<amigo_render_api::NprMeshStyle> {
        let mut styles: Vec<_> = meshes.iter()
            .map(|mesh| self.runtime_mesh_style_for(&mesh.entity_name)).collect();
        let mut poses = self.runtime_drawing_poses.lock().unwrap();
        let active: std::collections::BTreeSet<_> = meshes.iter()
            .zip(&styles).filter(|(_, style)| style.temporal)
            .map(|(mesh, _)| mesh.entity_id).collect();
        poses.retain(|id, _| active.contains(id));
        for (mesh, style) in meshes.iter().zip(&mut styles) {
            if !style.temporal { continue; }
            let candidate_epoch = style.artistic_frame;
            let entry = poses.entry(mesh.entity_id).or_insert_with(|| (mesh.clone(), candidate_epoch));
            // Every NPR subject receives a new realization at the authored
            // drawing cadence. Pose changes still update the cached source,
            // but they are no longer required to permit a new sketch frame.
            if candidate_epoch < entry.1 || candidate_epoch > entry.1 {
                *entry = (mesh.clone(), candidate_epoch);
            }
            style.artistic_frame = entry.1;
        }
        styles
    }

    pub fn runtime_mesh_style_for(&self, entity_name: &str) -> amigo_render_api::NprMeshStyle {
        let settings = self.settings.lock().unwrap();
        let runtime_style = settings.runtime_objects.get(entity_name);
        let inherited = runtime_style
            .and_then(|runtime| runtime.preset.as_deref())
            .and_then(style_preset)
            .map(ComicInkOverrides::technique_only)
            .map(|overrides| overrides.resolve(settings.global))
            .unwrap_or(settings.global);
        let style = runtime_style
            .map(|runtime| runtime.style.resolve(inherited))
            .unwrap_or(inherited);
        let layer_enabled = |source| {
            settings
                .style_layers
                .layers
                .iter()
                .any(|layer| layer.enabled && layer.source == source)
        };
        let hatching_enabled = runtime_style
            .and_then(|runtime| runtime.hatching)
            .unwrap_or_else(|| layer_enabled(amigo_render_npr::NprGeometrySource::ShadowHatch));
        let redraw_hz = settings.motion.redraw_hz.clamp(0.25, 20.0);
        // NPR is an animated drawing medium: even a stationary mesh gets a
        // fresh hand realization at `redraw_hz`. The document field remains
        // accepted for authored data compatibility, but no longer suppresses
        // the global sketch cadence.
        let temporal = true;
        let artistic_frame = if temporal {
            (*self.artistic_elapsed_seconds.lock().unwrap() * redraw_hz).floor() as u64
        } else {
            0
        };
        let stroke_segments = if runtime_style.is_some() {
            if style.wobble >= 1.3 { 6 } else if style.wobble >= 0.7 { 5 } else { 4 }
        } else {
            // Long architectural strokes still receive a small gesture curve;
            // the higher-quality character path is reserved for explicit
            // runtime identities.
            3
        };
        let stroke_segments = runtime_style
            .and_then(|runtime| runtime.stroke_segments)
            .unwrap_or(stroke_segments)
            .clamp(2, 8);
        let pressure_variation = runtime_style
            .and_then(|runtime| runtime.pressure_variation)
            .unwrap_or_else(|| {
                (0.10
                    + (1.0 - style.tool_hardness).clamp(0.0, 1.0) * 0.35
                    + (1.0 - style.gesture_confidence).clamp(0.0, 1.0) * 0.15)
                    .clamp(0.08, 0.55)
            })
            .clamp(0.0, 1.0);
        amigo_render_api::NprMeshStyle {
            background: style.paper.to_array(),
            fill: style.light.to_array(),
            shadow: style.shadow.to_array(),
            ink: style.ink.to_array(),
            light_direction: style.light_direction.to_array(),
            artistic_frame,
            temporal,
            draw_silhouettes: layer_enabled(amigo_render_npr::NprGeometrySource::Silhouette),
            draw_creases: runtime_style
                .and_then(|runtime| runtime.creases)
                .unwrap_or_else(|| layer_enabled(amigo_render_npr::NprGeometrySource::Creases)),
            draw_material_seams: runtime_style
                .and_then(|runtime| runtime.material_seams)
                .unwrap_or(false),
            draw_form_lines: style.suggestive_contours,
            draw_contact_lines: style.tone_density >= 0.20,
            outline_width_pixels: style.outline_width,
            boundary_width_pixels: style.boundary_width,
            crease_width_pixels: style.crease_width,
            form_line_width_pixels: (style.crease_width * style.suggestive_contour_width_scale)
                .clamp(0.35, 1.5),
            contact_line_width_pixels: (style.crease_width * 0.42).clamp(0.35, 1.1),
            crease_angle_radians: style.crease_angle,
            wobble_pixels: style.wobble,
            stroke_segments,
            join_strokes: runtime_style
                .and_then(|runtime| runtime.join_strokes)
                .unwrap_or(false),
            pressure_variation,
            taper: style.taper,
            overstroke: style.gesture_overstroke,
            pressure: style.tool_pressure,
            hardness: style.tool_hardness,
            dryness: style.ink_dryness,
            pencil_grain: if style.tool == StrokeTool::Pencil {
                (style.paper_grain * 0.72 + style.paper_tooth * 0.28).clamp(0.0, 1.0)
            } else {
                0.0
            },
            redraw_strength: settings.motion.redraw_strength,
            hatching_enabled,
            hatching_angle_degrees: style.hatching_angle,
            hatching_spacing_pixels: style.hatching_spacing,
            hatching_cross: style.hatching_cross,
            hatching_density: style.tone_density,
        }
    }
    pub fn render_snapshot(&self) -> Settings {
        let mut settings = self.snapshot();
        let build_up = (*self.build_up.lock().unwrap()).clamp(0.0, 1.0);
        let visible_layers =
            ((settings.style_layers.layers.len() as f32) * build_up).ceil() as usize;
        for (index, layer) in settings.style_layers.layers.iter_mut().enumerate() {
            if index >= visible_layers {
                layer.enabled = false;
            }
        }
        if let Some(solo) = self.solo_layer.lock().unwrap().as_deref() {
            for layer in &mut settings.style_layers.layers {
                layer.enabled = layer.id == "paper" || layer.id == solo;
            }
        }
        // A draft has no authored identity yet. Render it only after it has a
        // drawable open path, with an id outside the regular authoring range.
        // `snapshot()` and scene-document projection deliberately never see it.
        let authoring = self.construction_authoring.lock().unwrap();
        if let Some(object_id) = authoring.object_id.as_deref()
            && authoring.anchors.len() >= 2
            && let Some(object) = settings.objects.get_mut(object_id)
        {
            let mut id = u32::MAX;
            while object.construction_marks.iter().any(|mark| mark.id == id) {
                let Some(next) = id.checked_sub(1) else {
                    return settings;
                };
                id = next;
            }
            object.construction_marks.push(ConstructionMarkSettings {
                id,
                anchors: authoring.anchors.clone(),
                closed: false,
                width_scale: default_construction_width_scale(),
                opacity: 0.5,
            });
        }
        settings
    }

    pub fn set_build_up(&self, value: f32) {
        *self.build_up.lock().unwrap() = value.clamp(0.0, 1.0);
    }
    pub fn set_solo_layer(&self, layer: Option<String>) {
        *self.solo_layer.lock().unwrap() = layer;
    }
    pub(crate) fn fit_candidate(settings: &mut Settings, viewport: [u32; 2]) -> Result<(), String> {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for (id, object) in &settings.objects {
            if object.visible && *id == settings.selected {
                let extent = Vec3::splat(3.0_f32.sqrt() * object.scale);
                min = min.min(object.position - extent);
                max = max.max(object.position + extent);
            }
        }
        if !min.is_finite() {
            return Err("no visible objects to fit".into());
        }
        let aspect = viewport[0].max(1) as f32 / viewport[1].max(1) as f32;
        let vertical = settings.camera_fov.to_radians() * 0.5;
        let half_angle = vertical.min((vertical.tan() * aspect).atan());
        settings.camera_target = (min + max) * 0.5;
        settings.camera_distance =
            ((max - min).length() * 0.5 / half_angle.sin() * 1.05).clamp(0.1, 100.0);
        Ok(())
    }
}
