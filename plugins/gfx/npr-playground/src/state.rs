use amigo_render_npr::{
    ComicInk, NprConstructionMark, NprPreparedSurface, NprMotionPolicy, NprSurfaceAnchorError,
    NprSurfaceMode, NprToneMode, StrokeMotionMode, StrokeTool,
};
use amigo_runtime_control::*;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Mutex};
mod history;
pub mod look_presets;

pub const PREFIX: &str = "world.npr.settings.NprSettings.";
pub const MODELS: &[&str] = &["cube", "wedge", "cylinder", "sphere", "suzanne", "avocado"];
const ACTIONS: &[&str] = &[
    "fit",
    "focus_selected",
    "select_previous_object",
    "select_next_object",
    "undo",
    "redo",
    "capture_before",
    "reset_rotation",
    "reset_transform",
    "layout_grid",
    "camera_front",
    "camera_side",
    "camera_top",
    "reset_style",
    "new_gesture_variant",
    "begin_construction_mark",
    "commit_construction_mark",
    "close_construction_mark",
    "cancel_construction_mark",
    "undo_construction_anchor",
    "delete_selected_construction_mark",
    "select_previous_construction_mark",
    "select_next_construction_mark",
];
fn resolved_key(key: &str, settings: &Settings) -> String {
    if let Some(field) = key.strip_prefix("object.") {
        format!("objects.{}.{field}", settings.selected)
    } else if let Some(field) = key.strip_prefix("appearance.") {
        if settings.style_scope == "Obiekt" {
            format!("objects.{}.style.{field}", settings.selected)
        } else {
            format!("global.{field}")
        }
    } else {
        key.into()
    }
}
pub fn style_preset(name: &str) -> Option<ComicInk> {
    let mut style = ComicInk::default();
    match name {
        "Comic Ink" => {}
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
        _ => return None,
    }
    Some(style)
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
        ("pencil-study", "Pencil Study"),
        ("loose-study", "Loose Study"),
        ("confident-ink", "Confident Ink"),
        ("broad-nib", "Broad Nib"),
        ("blueprint", "Blueprint"),
        ("soft-toon", "Soft Toon"),
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
            return Err(format!("construction mark {} has invalid barycentric coordinates", self.id));
        }
        if !self.width_scale.is_finite() || !(0.0..=2.0).contains(&self.width_scale) {
            return Err(format!("construction mark {} has invalid width scale", self.id));
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
    #[serde(default)]
    pub surface_mode: NprSurfaceMode,
    /// Fixed smooth-proxy level, prepared per source revision rather than per
    /// camera frame. Zero means use the authored source surface directly.
    #[serde(default = "default_surface_subdivision_level")]
    pub surface_subdivision_level: u8,
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
    pub override_style: bool,
    pub style: ComicInk,
    /// Authored marks resolve against the selected source surface only during
    /// RenderExtract, so authored data never stores an internal mesh revision.
    #[serde(default)]
    pub construction_marks: Vec<ConstructionMarkSettings>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub global: ComicInk,
    pub objects: BTreeMap<String, ObjectSettings>,
    pub selected: String,
    pub gallery: bool,
    pub highlight_selected: bool,
    pub paused: bool,
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
    pub fn for_scene(gallery: bool) -> Self {
        let objects = MODELS
            .iter()
            .enumerate()
            .map(|(i, id)| {
                (
                    (*id).into(),
                    ObjectSettings {
                        model: (*id).into(),
                        surface_mode: match *id {
                            "cube" | "wedge" => NprSurfaceMode::Polygonal,
                            _ => NprSurfaceMode::Smooth,
                        },
                        surface_subdivision_level: match *id {
                            "cube" | "wedge" => 0,
                            _ => default_surface_subdivision_level(),
                        },
                        visible: true,
                        rotating: true,
                        position: if gallery {
                            Vec3::new((i % 3) as f32 * 3.0 - 3.0, 1.6 - (i / 3) as f32 * 3.2, 0.0)
                        } else {
                            Vec3::ZERO
                        },
                        rotation: Vec3::new(0.36_f32.to_degrees(), 0.71_f32.to_degrees(), 0.0),
                        scale: 1.0,
                        angular_speed: Vec3::new(21.2, 40.7, 0.0),
                        gesture_variant: 0,
                        override_style: false,
                        style: ComicInk::default(),
                        construction_marks: vec![],
                    },
                )
            })
            .collect();
        Self {
            global: ComicInk::default(),
            objects,
            selected: "cube".into(),
            gallery,
            highlight_selected: true,
            paused: false,
            speed: 1.0,
            step: false,
            motion: NprMotionPolicy::default(),
            seed: 0x4e5052,
            debug: "Final".into(),
            camera_target: Vec3::ZERO,
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_distance: if gallery { 14.0 } else { 5.0 },
            camera_fov: 45.0,
            preset_name: "my-preset".into(),
            style_scope: "Globalny".into(),
            preset_kind: "Scena".into(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if !["Globalny", "Obiekt"].contains(&self.style_scope.as_str()) {
            return Err("invalid style scope".into());
        }
        if !["Scena", "Wygląd"].contains(&self.preset_kind.as_str()) {
            return Err("invalid preset kind".into());
        }
        if !self.objects.contains_key(&self.selected)
            || self.objects.len() != MODELS.len()
            || MODELS.iter().any(|id| !self.objects.contains_key(*id))
        {
            return Err("invalid object selection/set".into());
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
            StrokeMotionMode::Stable | StrokeMotionMode::RedrawOnMotion
        ) {
            return Err("invalid stroke motion mode".into());
        }
        validate_style(self.global)?;
        for object in self.objects.values() {
            if !MODELS.contains(&object.model.as_str())
                || !object.position.is_finite()
                || !object.rotation.is_finite()
                || !object.angular_speed.is_finite()
                || !object.scale.is_finite()
                || !(0.01..=10.0).contains(&object.scale)
                || object.surface_subdivision_level > 2
            {
                return Err("invalid object parameters".into());
            }
            validate_style(object.style)?;
            let mut construction_ids = std::collections::BTreeSet::new();
            for mark in &object.construction_marks {
                mark.validate()?;
                if !construction_ids.insert(mark.id) {
                    return Err(format!("duplicate construction mark id {}", mark.id));
                }
            }
        }
        Ok(())
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
    defaults: Mutex<Settings>,
    pub fps: Mutex<f64>,
    last_frame: Mutex<Option<std::time::Instant>>,
    pub viewport: Mutex<[u32; 2]>,
    history: Mutex<history::History>,
    comparison: Mutex<Option<Settings>>,
    preview_before: Mutex<bool>,
    authored_scene: Mutex<Option<crate::scene::NprPlaygroundSceneDocument>>,
    construction_authoring: Mutex<ConstructionAuthoringState>,
    selected_construction_mark: Mutex<Option<usize>>,
    pub render_stats: Mutex<BTreeMap<String, u64>>,
}
impl Default for NprPlaygroundState {
    fn default() -> Self {
        let s = Settings::for_scene(false);
        Self {
            settings: Mutex::new(s.clone()),
            defaults: Mutex::new(s),
            fps: Mutex::new(0.0),
            last_frame: Mutex::new(None),
            viewport: Mutex::new([512, 512]),
            history: Mutex::new(history::History::default()),
            comparison: Mutex::new(None),
            preview_before: Mutex::new(false),
            authored_scene: Mutex::new(None),
            construction_authoring: Mutex::new(ConstructionAuthoringState::default()),
            selected_construction_mark: Mutex::new(None),
            render_stats: Mutex::new(
                [
                    "geometry",
                    "surface_source_triangles",
                    "surface_proxy_triangles",
                    "topology_edges",
                    "feature_segments",
                    "feature_candidates",
                    "feature_rejected",
                    "smooth_contour_spans",
                    "suggestive_contour_spans",
                    "silhouettes",
                    "creases",
                    "strokes",
                    "stroke_vertices",
                    "stroke_indices",
                    "hatching_strokes",
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
    ) -> Result<crate::scene::NprPlaygroundSceneDocument, String> {
        crate::scene::NprPlaygroundSceneDocument::from_settings(&self.snapshot())
    }

    /// Applies the small, scalar NPR scene surface exposed by the generic
    /// in-game editor. Structured objects and marks deliberately remain on the
    /// dedicated authoring path until scene-document transactions can persist
    /// them atomically.
    pub fn apply_editor_property(
        &self,
        field: &str,
        value: serde_yaml::Value,
    ) -> Result<bool, String> {
        let property_path = match field {
            "gallery" | "selected" | "seed" => field,
            "camera.distance" => "camera_distance",
            "camera.yaw" => "camera_yaw",
            "camera.pitch" => "camera_pitch",
            "camera.fov" => "camera_fov",
            "motion.mode" => "motion.mode",
            "motion.redraw_hz" => "motion.redraw_hz",
            "motion.redraw_strength" => "motion.redraw_strength",
            _ => return Ok(false),
        };
        let value = match value {
            serde_yaml::Value::Bool(value) => ControlValue::Bool(value),
            serde_yaml::Value::Number(value) if property_path == "seed" => {
                let value = value
                    .as_u64()
                    .or_else(|| {
                        value.as_f64().and_then(|value| {
                            (value.is_finite()
                                && value >= 0.0
                                && value <= u64::MAX as f64
                                && value.fract() == 0.0)
                                .then_some(value as u64)
                        })
                    })
                    .ok_or_else(|| "drawing seed must be a whole non-negative number".to_owned())?;
                ControlValue::U64(value)
            }
            serde_yaml::Value::Number(value) => ControlValue::F64(
                value
                    .as_f64()
                    .ok_or_else(|| "editor number is outside f64 range".to_owned())?,
            ),
            serde_yaml::Value::String(value) => ControlValue::String(value),
            _ => return Ok(false),
        };
        let value_type = value
            .value_type()
            .ok_or_else(|| "editor value has no runtime control type".to_owned())?;
        let property = RuntimeControlProperty {
            console_path: format!("{PREFIX}{property_path}"),
            target_path: "world.npr.settings".into(),
            component: Some("NprSettings".into()),
            property_path: property_path.to_owned(),
            value_type,
            range: property_range(property_path),
            writable: true,
            readable: true,
            animatable: false,
            source_file: None,
            source_pointer: None,
            provider_id: "npr-playground".into(),
            description: None,
        };
        <Self as RuntimeControlProvider>::set(self, &property, value)
            .map_err(|error| error.to_string())?;
        Ok(true)
    }

    pub fn stage_authored_scene(&self, authored: crate::scene::NprPlaygroundSceneDocument) {
        *self.authored_scene.lock().unwrap() = Some(authored);
    }

    /// Applies one pending authored document. This is consumed by the scene
    /// lifecycle rather than by the hydration command itself, so scene startup
    /// cannot overwrite declarative values with its fallback defaults.
    pub fn apply_staged_authored_scene(&self) -> Result<bool, String> {
        let Some(authored) = self.authored_scene.lock().unwrap().take() else {
            return Ok(false);
        };
        self.apply_authored_scene(authored)?;
        Ok(true)
    }

    pub fn apply_authored_scene(
        &self,
        authored: crate::scene::NprPlaygroundSceneDocument,
    ) -> Result<(), String> {
        let mut settings = Settings::for_scene(authored.gallery);
        authored.apply_to(&mut settings)?;
        *self.defaults.lock().unwrap() = settings.clone();
        *self.settings.lock().unwrap() = settings;
        *self.history.lock().unwrap() = history::History::default();
        *self.comparison.lock().unwrap() = None;
        *self.preview_before.lock().unwrap() = false;
        *self.construction_authoring.lock().unwrap() = ConstructionAuthoringState::default();
        *self.selected_construction_mark.lock().unwrap() = None;
        Ok(())
    }

    /// Starts an open construction line on the currently selected object.
    pub fn begin_construction_mark(&self) -> Result<(), String> {
        if *self.preview_before.lock().unwrap() {
            return Err("disable Before comparison to author a mark".into());
        }
        let selected = self.settings.lock().unwrap().selected.clone();
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
        self.construction_authoring.lock().unwrap().object_id.is_some()
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
    /// gallery click from silently joining anchors from separate meshes.
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
            return Err(format!("select {expected} before placing its construction mark"));
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
        let before = settings.clone();
        let object = settings
            .objects
            .get_mut(&object_id)
            .ok_or_else(|| format!("unknown construction-mark object {object_id}"))?;
        let mut id = 0x4000_0000u32;
        while object.construction_marks.iter().any(|mark| mark.id == id) {
            id = id.checked_add(1).ok_or("construction mark id space is exhausted")?;
        }
        object.construction_marks.push(ConstructionMarkSettings {
            id,
            anchors,
            closed,
            width_scale: default_construction_width_scale(),
            opacity: default_construction_opacity(),
        });
        settings.validate()?;
        self.history
            .lock()
            .unwrap()
            .record("add_construction_mark", &before, &settings);
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
            settings.objects[&settings.selected].construction_marks.len()
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

    /// Selects an authored scene object in stable gallery order. This is kept
    /// separate from the dropdown property so buttons, keyboard shortcuts and
    /// a future editor can share exactly the same camera/undo semantics.
    pub fn select_scene_object(&self, direction: isize) -> Result<(), String> {
        if direction == 0 {
            return Ok(());
        }
        let mut settings = self.settings.lock().unwrap();
        let before = settings.clone();
        let mut next_settings = before.clone();
        let current = MODELS
            .iter()
            .position(|id| *id == next_settings.selected)
            .ok_or_else(|| format!("unknown selected NPR object `{}`", next_settings.selected))?;
        let count = MODELS.len() as isize;
        let next = (current as isize + direction).rem_euclid(count) as usize;
        next_settings.selected = MODELS[next].to_owned();
        if !next_settings.gallery {
            Self::fit_candidate(&mut next_settings, *self.viewport.lock().unwrap())?;
        }
        next_settings.validate()?;
        self.history
            .lock()
            .unwrap()
            .record("select_scene_object", &before, &next_settings);
        *settings = next_settings;
        Ok(())
    }

    /// Removes the selected authored mark as one undoable settings change.
    pub fn delete_selected_construction_mark(&self) -> Result<(), String> {
        let selected_index = {
            let settings = self.settings.lock().unwrap();
            self.selected_construction_mark_index(
                settings.objects[&settings.selected].construction_marks.len(),
            )
            .ok_or("the selected object has no construction marks")?
        };
        let mut settings = self.settings.lock().unwrap();
        let before = settings.clone();
        let selected = settings.selected.clone();
        let remaining = {
            let object = settings
                .objects
                .get_mut(&selected)
                .ok_or_else(|| format!("unknown construction-mark object {selected}"))?;
            object.construction_marks.remove(selected_index);
            object.construction_marks.len()
        };
        self.history
            .lock()
            .unwrap()
            .record("delete_construction_mark", &before, &settings);
        drop(settings);
        *self.selected_construction_mark.lock().unwrap() = (remaining > 0)
            .then(|| selected_index.min(remaining - 1));
        Ok(())
    }

    fn set_selected_construction_mark_style(
        &self,
        field: &str,
        value: f32,
    ) -> Result<(), String> {
        if !value.is_finite() {
            return Err("construction mark value must be finite".into());
        }
        let selected_index = {
            let settings = self.settings.lock().unwrap();
            self.selected_construction_mark_index(
                settings.objects[&settings.selected].construction_marks.len(),
            )
            .ok_or("the selected object has no construction marks")?
        };
        let mut settings = self.settings.lock().unwrap();
        let before = settings.clone();
        let mut next = before.clone();
        let selected = next.selected.clone();
        let mark = next
            .objects
            .get_mut(&selected)
            .ok_or_else(|| format!("unknown construction-mark object {selected}"))?
            .construction_marks
            .get_mut(selected_index)
            .ok_or("the selected construction mark is out of range")?;
        match field {
            "width_scale" if (0.0..=2.0).contains(&value) => mark.width_scale = value,
            "opacity" if (0.0..=1.0).contains(&value) => mark.opacity = value,
            "width_scale" => return Err("construction mark width scale must be within 0..=2".into()),
            "opacity" => return Err("construction mark opacity must be within 0..=1".into()),
            _ => return Err(format!("unknown construction mark style field {field}")),
        }
        next.validate()?;
        self.history
            .lock()
            .unwrap()
            .record(&format!("construction_mark_{field}"), &before, &next);
        *settings = next;
        Ok(())
    }

    fn set_selected_construction_mark_closed(&self, closed: bool) -> Result<(), String> {
        let selected_index = {
            let settings = self.settings.lock().unwrap();
            self.selected_construction_mark_index(
                settings.objects[&settings.selected].construction_marks.len(),
            )
            .ok_or("the selected object has no construction marks")?
        };
        let mut settings = self.settings.lock().unwrap();
        let before = settings.clone();
        let mut next = before.clone();
        let selected = next.selected.clone();
        let mark = next
            .objects
            .get_mut(&selected)
            .ok_or_else(|| format!("unknown construction-mark object {selected}"))?
            .construction_marks
            .get_mut(selected_index)
            .ok_or("the selected construction mark is out of range")?;
        mark.closed = closed;
        next.validate()?;
        self.history
            .lock()
            .unwrap()
            .record("construction_mark_closed", &before, &next);
        *settings = next;
        Ok(())
    }

    fn control_values(&self, settings: &Settings) -> BTreeMap<String, ControlValue> {
        let mut props = values(settings);
        for (key, value) in props.clone() {
            if let Some(field) = key.strip_prefix(&format!("objects.{}.", settings.selected)) {
                props.insert(format!("object.{field}"), value);
            }
        }
        let prefix = if settings.style_scope == "Obiekt" {
            format!("objects.{}.style.", settings.selected)
        } else {
            "global.".into()
        };
        for (key, value) in props.clone() {
            if let Some(field) = key.strip_prefix(&prefix) {
                props.insert(format!("appearance.{field}"), value);
            }
        }
        for (id, object) in &settings.objects {
            let visible = object.visible && (settings.gallery || settings.selected == *id);
            props.insert(
                format!("objects.{id}.status"),
                ControlValue::String(format!(
                    "{} {}{}",
                    if visible { "W" } else { "-" },
                    if object.rotating && !settings.paused && settings.speed > 0.0 {
                        "R"
                    } else {
                        "-"
                    },
                    if object.override_style { " S" } else { "" }
                )),
            );
        }
        for action in ACTIONS {
            props.insert((*action).into(), ControlValue::Bool(false));
        }
        let (undo, redo) = self.history.lock().unwrap().available();
        let preview = *self.preview_before.lock().unwrap();
        props.insert("can_undo".into(), ControlValue::Bool(undo && !preview));
        props.insert("can_redo".into(), ControlValue::Bool(redo && !preview));
        props.insert(
            "can_compare".into(),
            ControlValue::Bool(self.comparison.lock().unwrap().is_some()),
        );
        props.insert("preview_before".into(), ControlValue::Bool(preview));
        props.insert("editable".into(), ControlValue::Bool(!preview));
        props.insert(
            "can_select_previous_object".into(),
            ControlValue::Bool(MODELS.len() > 1),
        );
        props.insert(
            "can_select_next_object".into(),
            ControlValue::Bool(MODELS.len() > 1),
        );
        let authoring = self.construction_authoring.lock().unwrap();
        props.insert(
            "construction_authoring_active".into(),
            ControlValue::Bool(authoring.object_id.is_some()),
        );
        props.insert(
            "construction_authoring_points".into(),
            ControlValue::U64(authoring.anchors.len() as u64),
        );
        props.insert(
            "construction_authoring_can_commit".into(),
            ControlValue::Bool(authoring.anchors.len() >= 2),
        );
        props.insert(
            "construction_authoring_can_close".into(),
            ControlValue::Bool(authoring.anchors.len() >= 3),
        );
        props.insert(
            "construction_authoring_can_undo_point".into(),
            ControlValue::Bool(!authoring.anchors.is_empty()),
        );
        let marks = &settings.objects[&settings.selected].construction_marks;
        let selected_mark = self
            .selected_construction_mark_index(marks.len())
            .and_then(|index| marks.get(index));
        props.insert(
            "construction_mark_count".into(),
            ControlValue::U64(marks.len() as u64),
        );
        props.insert(
            "construction_mark_last_id".into(),
            ControlValue::String(
                marks
                    .last()
                    .map(|mark| format!("0x{:08X}", mark.id))
                    .unwrap_or_else(|| "—".into()),
            ),
        );
        props.insert(
            "construction_mark_can_delete".into(),
            ControlValue::Bool(!marks.is_empty()),
        );
        props.insert(
            "construction_mark_can_edit".into(),
            ControlValue::Bool(!marks.is_empty()),
        );
        props.insert(
            "construction_mark_can_select_previous".into(),
            ControlValue::Bool(
                self.selected_construction_mark_index(marks.len())
                    .is_some_and(|index| index > 0),
            ),
        );
        props.insert(
            "construction_mark_can_select_next".into(),
            ControlValue::Bool(
                self.selected_construction_mark_index(marks.len())
                    .is_some_and(|index| index + 1 < marks.len()),
            ),
        );
        props.insert(
            "construction_mark_selected_width_scale".into(),
            ControlValue::F64(
                selected_mark
                    .map(|mark| f64::from(mark.width_scale))
                    .unwrap_or(f64::from(default_construction_width_scale())),
            ),
        );
        props.insert(
            "construction_mark_selected_opacity".into(),
            ControlValue::F64(
                selected_mark
                    .map(|mark| f64::from(mark.opacity))
                    .unwrap_or(f64::from(default_construction_opacity())),
            ),
        );
        props.insert(
            "construction_mark_selected_closed".into(),
            ControlValue::Bool(selected_mark.is_some_and(|mark| mark.closed)),
        );
        props.insert(
            "construction_mark_summary".into(),
            ControlValue::String(format!(
                "Linie: {} · wybrana: {}",
                marks.len(),
                selected_mark
                    .map(|mark| format!("0x{:08X}", mark.id))
                    .unwrap_or_else(|| "—".into())
            )),
        );
        props.insert(
            "construction_authoring_status".into(),
            ControlValue::String(match authoring.object_id.as_deref() {
                Some(_) if authoring.waiting_for_release => "Zwolnij przycisk myszy, aby rozpocząć wybór punktów.".into(),
                Some(object) => format!("{object}: {} punktów — dodaj kolejny lub zatwierdź", authoring.anchors.len()),
                None => "Wybierz „Dodaj linię”, potem punkty na modelu.".into(),
            }),
        );
        props.insert(
            "motion_redraw_editable".into(),
            ControlValue::Bool(
                !preview && settings.motion.mode == StrokeMotionMode::RedrawOnMotion,
            ),
        );
        props.insert(
            "appearance_editable".into(),
            ControlValue::Bool(
                !preview
                    && (settings.style_scope == "Globalny"
                        || settings.objects[&settings.selected].override_style),
            ),
        );
        let mut effective = if settings.style_scope == "Obiekt"
            && settings.objects[&settings.selected].override_style
        {
            settings.objects[&settings.selected].style
        } else {
            settings.global
        };
        effective.paper = ComicInk::default().paper;
        effective.light_direction = ComicInk::default().light_direction;
        let preset = [
            "Comic Ink",
            "Pencil Study",
            "Loose Study",
            "Confident Ink",
            "Broad Nib",
            "Blueprint",
            "Soft Toon",
        ]
        .into_iter()
        .find(|name| style_preset(name) == Some(effective))
        .unwrap_or("Własny");
        props.insert("style_preset".into(), ControlValue::String(preset.into()));
        props.insert(
            "preset_domain".into(),
            ControlValue::String(
                if settings.preset_kind == "Scena" {
                    "npr-playground"
                } else {
                    "npr-look"
                }
                .into(),
            ),
        );
        props.insert(
            "style_info".into(),
            ControlValue::String(if settings.style_scope == "Globalny" {
                "Styl sceny — obiekty bez własnego stylu".into()
            } else if settings.objects[&settings.selected].override_style {
                format!("Własny styl: {}", settings.selected)
            } else {
                format!(
                    "{} dziedziczy styl sceny — włącz własny styl w zakładce Obiekt",
                    settings.selected
                )
            }),
        );
        let fps = *self.fps.lock().unwrap();
        props.insert("fps".into(), ControlValue::F64(fps));
        props.insert(
            "frame_ms".into(),
            ControlValue::F64(if fps > 0.0 { 1000.0 / fps } else { 0.0 }),
        );
        for (key, value) in self.render_stats.lock().unwrap().iter() {
            props.insert(format!("stats.{key}"), ControlValue::U64(*value));
        }
        props
    }
    fn action(&self, action: &str) -> Result<(), String> {
        if *self.preview_before.lock().unwrap() {
            return Err("disable Before comparison to edit".into());
        }
        if action == "capture_before" {
            *self.comparison.lock().unwrap() = Some(self.snapshot());
            return Ok(());
        }
        if action == "undo" || action == "redo" {
            let mut settings = self.settings.lock().unwrap();
            return self
                .history
                .lock()
                .unwrap()
                .restore(&mut settings, action == "redo");
        }
        let before = self.snapshot();
        if action == "begin_construction_mark" {
            return self.begin_construction_mark();
        }
        if action == "commit_construction_mark" {
            return self.commit_construction_mark(false);
        }
        if action == "close_construction_mark" {
            return self.commit_construction_mark(true);
        }
        if action == "cancel_construction_mark" {
            self.cancel_construction_mark();
            return Ok(());
        }
        if action == "undo_construction_anchor" {
            return self.undo_construction_anchor();
        }
        if action == "delete_selected_construction_mark" {
            return self.delete_selected_construction_mark();
        }
        if action == "select_previous_construction_mark" {
            return self.select_construction_mark(-1);
        }
        if action == "select_next_construction_mark" {
            return self.select_construction_mark(1);
        }
        if action == "select_previous_object" {
            return self.select_scene_object(-1);
        }
        if action == "select_next_object" {
            return self.select_scene_object(1);
        }
        if action == "fit" {
            self.fit()?;
        } else {
            let mut s = self.settings.lock().unwrap();
            let selected = s.selected.clone();
            match action {
                "focus_selected" => s.gallery = false,
                "reset_rotation" => {
                    s.objects.get_mut(&selected).unwrap().rotation =
                        self.defaults.lock().unwrap().objects[&selected].rotation
                }
                "reset_transform" => {
                    let default = self.defaults.lock().unwrap();
                    let object = s.objects.get_mut(&selected).unwrap();
                    object.position = default.objects[&selected].position;
                    object.rotation = default.objects[&selected].rotation;
                    object.scale = default.objects[&selected].scale;
                }
                "layout_grid" => {
                    for (i, id) in MODELS.iter().enumerate() {
                        s.objects.get_mut(*id).unwrap().position =
                            Vec3::new((i % 3) as f32 * 3.0 - 3.0, 1.6 - (i / 3) as f32 * 3.2, 0.0);
                    }
                    s.gallery = true;
                }
                "camera_front" => {
                    s.camera_yaw = 0.0;
                    s.camera_pitch = 0.0;
                }
                "camera_side" => {
                    s.camera_yaw = 90.0;
                    s.camera_pitch = 0.0;
                }
                "camera_top" => {
                    s.camera_yaw = 0.0;
                    s.camera_pitch = 89.0;
                }
                "reset_style" => {
                    if s.style_scope == "Obiekt" {
                        s.objects.get_mut(&selected).unwrap().override_style = false;
                    } else {
                        let paper = s.global.paper;
                        let light = s.global.light_direction;
                        s.global = ComicInk::default();
                        s.global.paper = paper;
                        s.global.light_direction = light;
                    }
                }
                "new_gesture_variant" => {
                    let object = s.objects.get_mut(&selected).unwrap();
                    object.gesture_variant = object.gesture_variant.wrapping_add(1);
                }
                _ => return Err(format!("unknown action {action}")),
            }
            drop(s);
            if [
                "focus_selected",
                "layout_grid",
                "camera_front",
                "camera_side",
                "camera_top",
            ]
            .contains(&action)
            {
                if let Err(error) = self.fit() {
                    *self.settings.lock().unwrap() = before;
                    return Err(error);
                }
            }
        }
        let after = self.snapshot();
        self.history.lock().unwrap().record(action, &before, &after);
        Ok(())
    }
    pub fn configure_scene(&self, gallery: bool) {
        let settings = Settings::for_scene(gallery);
        *self.defaults.lock().unwrap() = settings.clone();
        *self.settings.lock().unwrap() = settings;
        *self.history.lock().unwrap() = history::History::default();
        *self.comparison.lock().unwrap() = None;
        *self.preview_before.lock().unwrap() = false;
        *self.construction_authoring.lock().unwrap() = ConstructionAuthoringState::default();
        *self.selected_construction_mark.lock().unwrap() = None;
    }
    pub fn tick(&self, dt: f32) {
        let mut s = self.settings.lock().unwrap();
        let delta = if s.step && s.paused {
            1.0 / 60.0
        } else if s.paused {
            0.0
        } else {
            dt * s.speed
        };
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
    pub fn render_snapshot(&self) -> Settings {
        let mut settings = self.snapshot();
        let preview_before = *self.preview_before.lock().unwrap();
        if preview_before {
            if let Some(before) = self.comparison.lock().unwrap().as_ref() {
                settings.global = before.global;
                for (id, object) in &mut settings.objects {
                    object.style = before.objects[id].style;
                    object.override_style = before.objects[id].override_style;
                }
            }
        }
        // A draft has no authored identity yet. Render it only after it has a
        // drawable open path, with an id outside the regular authoring range.
        // `snapshot()` and scene-document projection deliberately never see it.
        let authoring = self.construction_authoring.lock().unwrap();
        if !preview_before
            && let Some(object_id) = authoring.object_id.as_deref()
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
    fn fit(&self) -> Result<(), String> {
        let mut settings = self.settings.lock().unwrap();
        Self::fit_candidate(&mut settings, *self.viewport.lock().unwrap())
    }
    fn fit_candidate(settings: &mut Settings, viewport: [u32; 2]) -> Result<(), String> {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for (id, object) in &settings.objects {
            if object.visible && (settings.gallery || *id == settings.selected) {
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
fn values(settings: &Settings) -> BTreeMap<String, ControlValue> {
    fn flatten(prefix: String, value: serde_yaml::Value, out: &mut BTreeMap<String, ControlValue>) {
        let value = match value {
            serde_yaml::Value::Mapping(m) => {
                for (k, v) in m {
                    if let Some(k) = k.as_str() {
                        flatten(
                            if prefix.is_empty() {
                                k.into()
                            } else {
                                format!("{prefix}.{k}")
                            },
                            v,
                            out,
                        );
                    }
                }
                return;
            }
            serde_yaml::Value::Bool(v) => ControlValue::Bool(v),
            serde_yaml::Value::Number(v) => {
                if let Some(n) = v.as_i64() {
                    ControlValue::I64(n)
                } else {
                    ControlValue::F64(v.as_f64().unwrap_or_default())
                }
            }
            serde_yaml::Value::String(v) => ControlValue::String(v),
            serde_yaml::Value::Sequence(v) => {
                let n = v
                    .iter()
                    .map(|n| n.as_f64().unwrap_or_default() as f32)
                    .collect::<Vec<_>>();
                match n.as_slice() {
                    [x, y, z] => ControlValue::Vec3([*x, *y, *z]),
                    [r, g, b, a] => ControlValue::Color([*r, *g, *b, *a]),
                    _ => ControlValue::Null,
                }
            }
            _ => ControlValue::Null,
        };
        let value = if prefix.ends_with("crease_angle") {
            ControlValue::F64(value.as_f64().unwrap().to_degrees())
        } else {
            value
        };
        out.insert(prefix, value);
    }
    let mut out = BTreeMap::new();
    flatten(
        String::new(),
        serde_yaml::to_value(settings).unwrap(),
        &mut out,
    );
    out.insert("seed".into(), ControlValue::U64(settings.seed));
    out
}
fn control_yaml(value: ControlValue) -> serde_yaml::Value {
    match value {
        ControlValue::Bool(v) => serde_yaml::to_value(v),
        ControlValue::I64(v) => serde_yaml::to_value(v),
        ControlValue::U64(v) => serde_yaml::to_value(v),
        ControlValue::F64(v) => serde_yaml::to_value(v),
        ControlValue::String(v) | ControlValue::AssetRef(v) => serde_yaml::to_value(v),
        ControlValue::Vec2(v) => serde_yaml::to_value(v),
        ControlValue::Vec3(v) => serde_yaml::to_value(v),
        ControlValue::Color(v) => serde_yaml::to_value(v),
        ControlValue::Null => Ok(serde_yaml::Value::Null),
    }
    .unwrap()
}
fn property_range(key: &str) -> Option<ControlRange> {
    let (min, max) = match key.rsplit('.').next().unwrap_or(key) {
        "outline_width" | "crease_width" | "boundary_width" => (0.0, 20.0),
        "min_crease_length_pixels" => (0.0, 64.0),
        "surface_subdivision_level" => (0.0, 2.0),
        "crease_angle" | "smooth_crease_angle" => (0.0, 180.0),
        "taper" => (0.0, 1.0),
        "wobble" => (0.0, 10.0),
        "gesture_confidence"
        | "gesture_simplification"
        | "gesture_correction"
        | "gesture_overstroke"
        | "tool_pressure"
        | "tool_hardness"
        | "paper_tooth"
        | "paper_grain"
        | "ink_dryness"
        | "nib_aspect" => (0.0, 1.0),
        "nib_angle" => (-180.0, 180.0),
        "hatching_angle" => (-180.0, 180.0),
        "hatching_spacing" => (1.0, 40.0),
        "hatching_cross" => (0.0, 1.0),
        "min_form_line_confidence" => (0.0, 1.0),
        "suggestive_contour_confidence" => (0.0, 1.0),
        "suggestive_contour_width_scale" | "form_line_width_scale" => (0.0, 2.0),
        "suggestive_contour_opacity" | "form_line_opacity" => (0.0, 1.0),
        "scale" => (0.01, 10.0),
        "speed" => (0.0, 4.0),
        "appearance_fade_seconds" => (0.0, 2.0),
        "redraw_hz" => (0.25, 20.0),
        "redraw_strength" => (0.0, 1.0),
        "camera_distance" => (0.1, 100.0),
        "camera_pitch" => (-89.0, 89.0),
        "camera_fov" => (15.0, 90.0),
        _ => return None,
    };
    Some(ControlRange {
        min: Some(min),
        max: Some(max),
    })
}
impl RuntimeControlProvider for NprPlaygroundState {
    fn provider_id(&self) -> &'static str {
        "npr-playground"
    }
    fn rebuild_registry(
        &self,
        registry: &mut RuntimeControlRegistry,
    ) -> Result<(), RuntimeControlError> {
        registry.register_target(RuntimeControlTarget {
            console_path: "world.npr.settings".into(),
            source_id: None,
            label: "NPR Playground".into(),
            components: vec!["NprSettings".into()],
            aliases: vec![],
            source_file: None,
        });
        let snapshot = self.snapshot();
        let props = self.control_values(&snapshot);
        for (key, value) in props {
            registry.register_property(RuntimeControlProperty {
                console_path: format!("{PREFIX}{key}"),
                target_path: "world.npr.settings".into(),
                component: Some("NprSettings".into()),
                property_path: key.clone(),
                value_type: value.value_type().unwrap_or(ControlValueType::String),
                range: property_range(&key),
                writable: ![
                    "fps",
                    "frame_ms",
                    "can_undo",
                    "can_redo",
                    "can_compare",
                    "editable",
                    "can_select_previous_object",
                    "can_select_next_object",
                    "appearance_editable",
                    "style_info",
                    "preset_domain",
                    "construction_authoring_active",
                    "construction_authoring_points",
                    "construction_authoring_can_commit",
                    "construction_authoring_can_close",
                    "construction_authoring_can_undo_point",
                    "construction_authoring_status",
                    "construction_mark_count",
                    "construction_mark_last_id",
                    "construction_mark_can_delete",
                    "construction_mark_can_edit",
                    "construction_mark_can_select_previous",
                    "construction_mark_can_select_next",
                    "construction_mark_summary",
                ]
                .contains(&key.as_str())
                    && !key.ends_with(".status")
                    && !key.starts_with("stats."),
                readable: true,
                animatable: false,
                source_file: None,
                source_pointer: None,
                provider_id: self.provider_id().into(),
                description: Some(key),
            });
        }
        Ok(())
    }
    fn get(&self, path: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError> {
        self.control_values(&self.snapshot())
            .remove(&path.property_path)
            .ok_or_else(|| RuntimeControlError::UnknownProperty {
                path: path.console_path.clone(),
            })
    }
    fn get_many(
        &self,
        paths: &[RuntimeControlProperty],
    ) -> Result<Vec<ControlValue>, RuntimeControlError> {
        let values = self.control_values(&self.snapshot());
        paths
            .iter()
            .map(|path| {
                values.get(&path.property_path).cloned().ok_or_else(|| {
                    RuntimeControlError::UnknownProperty {
                        path: path.console_path.clone(),
                    }
                })
            })
            .collect()
    }
    fn set(
        &self,
        path: &RuntimeControlProperty,
        value: ControlValue,
    ) -> Result<(), RuntimeControlError> {
        let failure = |reason: String| RuntimeControlError::Unsupported {
            path: path.console_path.clone(),
            reason,
        };
        if path.property_path == "preview_before" {
            let enabled = value
                .as_bool()
                .ok_or_else(|| failure("boolean required".into()))?;
            if enabled && self.comparison.lock().unwrap().is_none() {
                return Err(failure("capture a comparison first".into()));
            }
            *self.preview_before.lock().unwrap() = enabled;
            if enabled {
                self.cancel_construction_mark();
            }
            return Ok(());
        }
        if *self.preview_before.lock().unwrap() {
            return Err(failure("disable Before comparison to edit".into()));
        }
        if ACTIONS.contains(&path.property_path.as_str()) {
            return if value == ControlValue::Bool(true) {
                self.action(&path.property_path).map_err(failure)
            } else {
                Ok(())
            };
        }
        if path.property_path == "construction_mark_selected_closed" {
            return self
                .set_selected_construction_mark_closed(
                    value
                        .as_bool()
                        .ok_or_else(|| failure("boolean required".into()))?,
                )
                .map_err(failure);
        }
        if let Some(field) = path
            .property_path
            .strip_prefix("construction_mark_selected_")
        {
            let value = value
                .as_f64()
                .ok_or_else(|| failure("number required".into()))? as f32;
            return self
                .set_selected_construction_mark_style(field, value)
                .map_err(failure);
        }
        let mut current = self.settings.lock().unwrap();
        if (path.property_path.starts_with("appearance.") || path.property_path == "style_preset")
            && current.style_scope == "Obiekt"
            && !current.objects[&current.selected].override_style
        {
            return Err(failure("enable an object style override first".into()));
        }
        if path.property_path == "style_preset" {
            let mut style = style_preset(value.as_string().unwrap_or(""))
                .ok_or_else(|| failure("unknown style preset".into()))?;
            let before = current.clone();
            let selected = current.selected.clone();
            style.paper = current.global.paper;
            style.light_direction = current.global.light_direction;
            if current.style_scope == "Obiekt" {
                current.objects.get_mut(&selected).unwrap().style = style;
            } else {
                current.global = style;
            }
            self.history
                .lock()
                .unwrap()
                .record("style_preset", &before, &current);
            return Ok(());
        }
        let mut yaml = serde_yaml::to_value(&*current).unwrap();
        let mut target = &mut yaml;
        let key = resolved_key(&path.property_path, &current);
        for part in key.split('.') {
            target = target
                .get_mut(part)
                .ok_or_else(|| RuntimeControlError::UnknownProperty {
                    path: path.console_path.clone(),
                })?;
        }
        let value = if key.ends_with("crease_angle") {
            ControlValue::F64(
                value
                    .as_f64()
                    .ok_or_else(|| RuntimeControlError::Unsupported {
                        path: path.console_path.clone(),
                        reason: "crease angle requires degrees".into(),
                    })?
                    .to_radians(),
            )
        } else {
            value
        };
        *target = control_yaml(value);
        let mut next: Settings =
            serde_yaml::from_value(yaml).map_err(|e| RuntimeControlError::Unsupported {
                path: path.console_path.clone(),
                reason: e.to_string(),
            })?;
        // Enabling an override starts from the currently effective global style.
        for (id, object) in &mut next.objects {
            if object.override_style && !current.objects[id].override_style {
                object.style = current.global;
            }
        }
        next.validate()
            .map_err(|reason| RuntimeControlError::Unsupported {
                path: path.console_path.clone(),
                reason,
            })?;
        if key == "gallery" || (key == "selected" && !next.gallery) {
            // Choosing hidden objects is valid; retain the camera when nothing is visible.
            if next
                .objects
                .iter()
                .any(|(id, o)| o.visible && (next.gallery || *id == next.selected))
            {
                Self::fit_candidate(&mut next, *self.viewport.lock().unwrap()).map_err(failure)?;
            }
        }
        self.history.lock().unwrap().record(&key, &current, &next);
        *current = next;
        Ok(())
    }
    fn reset(&self, path: &RuntimeControlProperty) -> Result<(), RuntimeControlError> {
        let key = resolved_key(&path.property_path, &self.snapshot());
        let value = values(&self.defaults.lock().unwrap())
            .remove(&key)
            .ok_or_else(|| RuntimeControlError::UnknownProperty {
                path: path.console_path.clone(),
            })?;
        self.set(path, value)
    }
}
impl amigo_panels::PresetProvider for NprPlaygroundState {
    fn id(&self) -> &'static str {
        "npr-playground"
    }
    fn snapshot(&self) -> Result<serde_yaml::Value, String> {
        let mut settings = self.snapshot();
        settings.step = false;
        serde_yaml::to_value(settings).map_err(|e| e.to_string())
    }
    fn apply(&self, value: serde_yaml::Value) -> Result<(), String> {
        if *self.preview_before.lock().unwrap() {
            return Err("disable Before comparison to load a preset".into());
        }
        let next: Settings = serde_yaml::from_value(value).map_err(|e| e.to_string())?;
        next.validate()?;
        let mut current = self.settings.lock().unwrap();
        self.history
            .lock()
            .unwrap()
            .record("load_preset", &current, &next);
        *current = next;
        Ok(())
    }
    fn reset(&self) -> Result<(), String> {
        if *self.preview_before.lock().unwrap() {
            return Err("disable Before comparison to reset".into());
        }
        let mut current = self.settings.lock().unwrap();
        let next = self.defaults.lock().unwrap().clone();
        self.history
            .lock()
            .unwrap()
            .record("reset_scene", &current, &next);
        *current = next;
        Ok(())
    }
}
