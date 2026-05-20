use std::collections::BTreeMap;

use amigo_2d_spatial::{DepthCurve2d, DepthSpace2d, OpticalLayerRole2d};
use amigo_camera::CameraDebugViewId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisualSourceId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VisualSourceKind2d {
    SceneColor,
    SceneDepth,
    SceneNormal,
    SceneWetness,
    SceneEmissive,
    SceneHighlight,
    SceneMotion,
    LayerMask,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualSourceAvailability2d {
    Produced,
    Derived,
    Asset,
    Fallback,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisualSourceOrigin2d {
    WorldPass,
    DepthPass,
    LayerMaskPass,
    LayerRolePass,
    MaterialBuffer,
    LightBuffer,
    EmissiveBuffer,
    MotionBuffer,
    LightExtraction,
    BeaconExtraction,
    PostFxDerived { feature: String },
    Asset { path: String },
    ShutterHistory,
    DebugFallback,
}

impl VisualSourceKind2d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SceneColor => "scene_color",
            Self::SceneDepth => "scene_depth",
            Self::SceneNormal => "scene_normal",
            Self::SceneWetness => "scene_wetness",
            Self::SceneEmissive => "scene_emissive",
            Self::SceneHighlight => "scene_highlight",
            Self::SceneMotion => "scene_motion",
            Self::LayerMask => "layer_mask",
            Self::Debug => "debug",
        }
    }
}

/// Render-API contract for a source the camera may use.
/// This is not a WGPU texture handle. WGPU resolves it to concrete runtime sources.
#[derive(Debug, Clone, PartialEq)]
pub struct VisualSourceRef2d {
    pub id: VisualSourceId,
    pub kind: VisualSourceKind2d,
    pub availability: VisualSourceAvailability2d,
    pub origin: VisualSourceOrigin2d,
}

impl VisualSourceRef2d {
    pub fn produced(
        kind: VisualSourceKind2d,
        id: impl Into<String>,
        origin: VisualSourceOrigin2d,
    ) -> Self {
        Self {
            id: VisualSourceId(id.into()),
            kind,
            availability: VisualSourceAvailability2d::Produced,
            origin,
        }
    }

    pub fn derived(
        kind: VisualSourceKind2d,
        id: impl Into<String>,
        origin: VisualSourceOrigin2d,
    ) -> Self {
        Self {
            id: VisualSourceId(id.into()),
            kind,
            availability: VisualSourceAvailability2d::Derived,
            origin,
        }
    }

    pub fn asset(kind: VisualSourceKind2d, path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            id: VisualSourceId(path.clone()),
            kind,
            availability: VisualSourceAvailability2d::Asset,
            origin: VisualSourceOrigin2d::Asset { path },
        }
    }

    pub fn fallback(kind: VisualSourceKind2d, id: impl Into<String>) -> Self {
        Self {
            id: VisualSourceId(id.into()),
            kind,
            availability: VisualSourceAvailability2d::Fallback,
            origin: VisualSourceOrigin2d::DebugFallback,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CameraDebugView2d(pub CameraDebugViewId);

impl CameraDebugView2d {
    pub fn new(value: impl Into<String>) -> Self {
        Self(CameraDebugViewId::new(value))
    }

    pub fn final_output() -> Self {
        Self(CameraDebugViewId::final_output())
    }

    pub fn raw_scene_color() -> Self {
        Self(CameraDebugViewId::raw_scene_color())
    }

    pub fn scene_depth() -> Self {
        Self(CameraDebugViewId::scene_depth())
    }

    pub fn stop_after_feature(&self) -> Option<&'static str> {
        match self.as_str() {
            "camera.after_exposure" => Some("camera_exposure"),
            "camera.after_optics" => Some("camera_optics"),
            "camera.after_dof" => Some("focus_blur"),
            "camera.after_lens_surface" => Some("rain_glass"),
            "camera.after_film" => Some("film_emulsion"),
            "camera.after_look" => Some("color_ramp"),
            _ => None,
        }
    }

    pub fn wants_visual_source_debug(&self) -> bool {
        matches!(
            self.as_str(),
            "camera.scene_depth"
                | "camera.computed_z_depth"
                | "camera.layer_optical_roles"
                | "camera.layer_mask"
                | "camera.scene_normal"
                | "camera.scene_wetness"
                | "camera.scene_emissive"
                | "camera.scene_highlight"
                | "camera.scene_motion"
        )
    }

    pub fn parse(value: &str) -> Self {
        Self(CameraDebugViewId::parse(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for CameraDebugView2d {
    fn default() -> Self {
        Self::final_output()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLayerOptics2d {
    pub layer_id: String,
    pub role: OpticalLayerRole2d,
    pub depth_mode: String,
    pub distance_m: Option<f32>,
    pub z_depth: f32,
    pub base_z_depth: f32,
    pub effective_z_depth: f32,
    pub effective_distance_m: Option<f32>,
    pub blur_scale: f32,
    pub camera_motion_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraCaptureInput2d {
    pub depth_space: DepthSpace2d,
    pub color: VisualSourceRef2d,
    pub depth: Option<VisualSourceRef2d>,
    pub layer_mask: Option<VisualSourceRef2d>,
    pub normal: Option<VisualSourceRef2d>,
    pub wetness: Option<VisualSourceRef2d>,
    pub emissive: Option<VisualSourceRef2d>,
    pub highlight: Option<VisualSourceRef2d>,
    pub motion: Option<VisualSourceRef2d>,
    pub layers: Vec<ResolvedLayerOptics2d>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraCaptureInputDiagnostic2d {
    pub code: &'static str,
    pub message: String,
    pub kind: VisualSourceKind2d,
}

pub struct CameraCaptureInput2dBuilder {
    input: CameraCaptureInput2d,
}

impl CameraCaptureInput2dBuilder {
    pub fn new(depth_space: DepthSpace2d, layers: Vec<ResolvedLayerOptics2d>) -> Self {
        Self {
            input: CameraCaptureInput2d {
                depth_space: depth_space.normalized(),
                color: VisualSourceRef2d::produced(
                    VisualSourceKind2d::SceneColor,
                    "world.color",
                    VisualSourceOrigin2d::WorldPass,
                ),
                depth: None,
                layer_mask: None,
                normal: None,
                wetness: None,
                emissive: None,
                highlight: None,
                motion: None,
                layers,
            },
        }
    }

    pub fn with_depth(mut self, id: impl Into<String>) -> Self {
        self.input.depth = Some(VisualSourceRef2d::produced(
            VisualSourceKind2d::SceneDepth,
            id,
            VisualSourceOrigin2d::DepthPass,
        ));
        self
    }

    pub fn with_layer_mask(mut self, id: impl Into<String>) -> Self {
        self.input.layer_mask = Some(VisualSourceRef2d::produced(
            VisualSourceKind2d::LayerMask,
            id,
            VisualSourceOrigin2d::LayerMaskPass,
        ));
        self
    }

    // Compatibility convenience: derived, not produced. Prefer explicit
    // with_normal_produced / with_normal_derived.
    pub fn with_normal(self, id: impl Into<String>) -> Self {
        self.with_normal_derived(id, "derived")
    }

    pub fn with_normal_produced(mut self, id: impl Into<String>) -> Self {
        self.input.normal = Some(VisualSourceRef2d::produced(
            VisualSourceKind2d::SceneNormal,
            id,
            VisualSourceOrigin2d::MaterialBuffer,
        ));
        self
    }

    pub fn with_normal_asset(mut self, path: impl Into<String>) -> Self {
        self.input.normal = Some(VisualSourceRef2d::asset(
            VisualSourceKind2d::SceneNormal,
            path,
        ));
        self
    }

    pub fn with_normal_derived(
        mut self,
        id: impl Into<String>,
        feature: impl Into<String>,
    ) -> Self {
        self.input.normal = Some(VisualSourceRef2d::derived(
            VisualSourceKind2d::SceneNormal,
            id,
            VisualSourceOrigin2d::PostFxDerived {
                feature: feature.into(),
            },
        ));
        self
    }

    // Compatibility convenience: derived, not produced. Prefer explicit
    // with_wetness_produced / with_wetness_derived.
    pub fn with_wetness(self, id: impl Into<String>) -> Self {
        self.with_wetness_derived(id, "derived")
    }

    pub fn with_wetness_produced(mut self, id: impl Into<String>) -> Self {
        self.input.wetness = Some(VisualSourceRef2d::produced(
            VisualSourceKind2d::SceneWetness,
            id,
            VisualSourceOrigin2d::MaterialBuffer,
        ));
        self
    }

    pub fn with_wetness_asset(mut self, path: impl Into<String>) -> Self {
        self.input.wetness = Some(VisualSourceRef2d::asset(
            VisualSourceKind2d::SceneWetness,
            path,
        ));
        self
    }

    pub fn with_wetness_derived(
        mut self,
        id: impl Into<String>,
        feature: impl Into<String>,
    ) -> Self {
        self.input.wetness = Some(VisualSourceRef2d::derived(
            VisualSourceKind2d::SceneWetness,
            id,
            VisualSourceOrigin2d::PostFxDerived {
                feature: feature.into(),
            },
        ));
        self
    }

    // Compatibility convenience: derived, not produced. Prefer explicit
    // with_emissive_produced / with_emissive_derived.
    pub fn with_emissive(self, id: impl Into<String>) -> Self {
        self.with_emissive_derived(id)
    }

    pub fn with_emissive_produced(mut self, id: impl Into<String>) -> Self {
        self.input.emissive = Some(VisualSourceRef2d::produced(
            VisualSourceKind2d::SceneEmissive,
            id,
            VisualSourceOrigin2d::EmissiveBuffer,
        ));
        self
    }

    pub fn with_emissive_derived(mut self, id: impl Into<String>) -> Self {
        self.input.emissive = Some(VisualSourceRef2d::derived(
            VisualSourceKind2d::SceneEmissive,
            id,
            VisualSourceOrigin2d::BeaconExtraction,
        ));
        self
    }

    // Compatibility convenience: derived, not produced. Prefer explicit
    // with_highlight_produced / with_highlight_derived.
    pub fn with_highlight(self, id: impl Into<String>) -> Self {
        self.with_highlight_derived(id)
    }

    pub fn with_highlight_produced(mut self, id: impl Into<String>) -> Self {
        self.input.highlight = Some(VisualSourceRef2d::produced(
            VisualSourceKind2d::SceneHighlight,
            id,
            VisualSourceOrigin2d::LightBuffer,
        ));
        self
    }

    pub fn with_highlight_derived(mut self, id: impl Into<String>) -> Self {
        self.input.highlight = Some(VisualSourceRef2d::derived(
            VisualSourceKind2d::SceneHighlight,
            id,
            VisualSourceOrigin2d::LightExtraction,
        ));
        self
    }

    // Compatibility convenience: derived, not produced. Prefer explicit
    // with_motion_produced / with_motion_derived.
    pub fn with_motion(self, id: impl Into<String>) -> Self {
        self.with_motion_derived(id)
    }

    pub fn with_motion_produced(mut self, id: impl Into<String>) -> Self {
        self.input.motion = Some(VisualSourceRef2d::produced(
            VisualSourceKind2d::SceneMotion,
            id,
            VisualSourceOrigin2d::MotionBuffer,
        ));
        self
    }

    pub fn with_motion_derived(mut self, id: impl Into<String>) -> Self {
        self.input.motion = Some(VisualSourceRef2d::derived(
            VisualSourceKind2d::SceneMotion,
            id,
            VisualSourceOrigin2d::ShutterHistory,
        ));
        self
    }

    pub fn build(self) -> CameraCaptureInput2d {
        self.input
    }
}

impl CameraCaptureInput2d {
    pub fn world_color(depth_space: DepthSpace2d, layers: Vec<ResolvedLayerOptics2d>) -> Self {
        CameraCaptureInput2dBuilder::new(depth_space, layers)
            .with_depth("world.depth")
            .build()
    }

    pub fn source(&self, kind: VisualSourceKind2d) -> Option<&VisualSourceRef2d> {
        match kind {
            VisualSourceKind2d::SceneColor => Some(&self.color),
            VisualSourceKind2d::SceneDepth => self.depth.as_ref(),
            VisualSourceKind2d::LayerMask => self.layer_mask.as_ref(),
            VisualSourceKind2d::SceneNormal => self.normal.as_ref(),
            VisualSourceKind2d::SceneWetness => self.wetness.as_ref(),
            VisualSourceKind2d::SceneEmissive => self.emissive.as_ref(),
            VisualSourceKind2d::SceneHighlight => self.highlight.as_ref(),
            VisualSourceKind2d::SceneMotion => self.motion.as_ref(),
            VisualSourceKind2d::Debug => None,
        }
    }

    pub fn missing_source_kinds(&self) -> Vec<VisualSourceKind2d> {
        [
            VisualSourceKind2d::LayerMask,
            VisualSourceKind2d::SceneNormal,
            VisualSourceKind2d::SceneWetness,
            VisualSourceKind2d::SceneEmissive,
            VisualSourceKind2d::SceneHighlight,
            VisualSourceKind2d::SceneMotion,
        ]
        .into_iter()
        .filter(|kind| self.source(*kind).is_none())
        .collect()
    }

    pub fn diagnostics(&self) -> Vec<CameraCaptureInputDiagnostic2d> {
        let mut diagnostics = Vec::new();
        for (kind, code, message) in [
            (
                VisualSourceKind2d::SceneNormal,
                "camera_capture_missing_scene_normal",
                "scene normal source is missing; debug views will fall back to a neutral normal color",
            ),
            (
                VisualSourceKind2d::SceneWetness,
                "camera_capture_missing_scene_wetness",
                "scene wetness source is missing; debug views will fall back to a wetness placeholder",
            ),
            (
                VisualSourceKind2d::SceneHighlight,
                "camera_capture_missing_scene_highlight",
                "scene highlight source is missing; highlight debug uses a fallback visualization",
            ),
            (
                VisualSourceKind2d::SceneEmissive,
                "camera_capture_missing_scene_emissive",
                "scene emissive source is missing; emissive debug uses a fallback visualization",
            ),
            (
                VisualSourceKind2d::SceneMotion,
                "camera_capture_missing_scene_motion",
                "scene motion source is missing; motion debug uses a fallback visualization",
            ),
        ] {
            match self.source(kind) {
                None => {
                    diagnostics.push(CameraCaptureInputDiagnostic2d {
                        code,
                        message: message.to_owned(),
                        kind,
                    });
                    diagnostics.push(CameraCaptureInputDiagnostic2d {
                        code: "visual_source_missing",
                        message: format!(
                            "{} is missing; camera debug falls back until a dedicated source exists",
                            kind.as_str()
                        ),
                        kind,
                    });
                }
                Some(source) => match source.availability {
                    VisualSourceAvailability2d::Produced => {}
                    VisualSourceAvailability2d::Derived => {
                        diagnostics.push(CameraCaptureInputDiagnostic2d {
                            code: "visual_source_not_produced",
                            message: format!(
                                "{} is available as derived data, not a dedicated produced buffer",
                                kind.as_str()
                            ),
                            kind,
                        });
                        diagnostics.push(CameraCaptureInputDiagnostic2d {
                            code: "visual_source_derived",
                            message: format!(
                                "{} comes from a derived runtime path ({})",
                                kind.as_str(),
                                origin_label(&source.origin)
                            ),
                            kind,
                        });
                    }
                    VisualSourceAvailability2d::Asset => {
                        diagnostics.push(CameraCaptureInputDiagnostic2d {
                            code: "visual_source_not_produced",
                            message: format!(
                                "{} is asset-backed, not a dedicated produced buffer",
                                kind.as_str()
                            ),
                            kind,
                        });
                        diagnostics.push(CameraCaptureInputDiagnostic2d {
                            code: "visual_source_asset_backed",
                            message: format!(
                                "{} resolves from an asset source ({})",
                                kind.as_str(),
                                origin_label(&source.origin)
                            ),
                            kind,
                        });
                    }
                    VisualSourceAvailability2d::Fallback | VisualSourceAvailability2d::Missing => {
                        diagnostics.push(CameraCaptureInputDiagnostic2d {
                            code: "visual_source_not_produced",
                            message: format!(
                                "{} is not backed by a dedicated produced buffer",
                                kind.as_str()
                            ),
                            kind,
                        });
                        diagnostics.push(CameraCaptureInputDiagnostic2d {
                            code: "visual_source_missing",
                            message: format!(
                                "{} currently resolves to fallback/missing state",
                                kind.as_str()
                            ),
                            kind,
                        });
                    }
                },
            }
        }
        diagnostics
    }

    pub fn debug_summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push("CameraCaptureInput2d:".to_owned());
        lines.push(format!(
            "  depth_space near={:.2} far={:.2} curve={}",
            self.depth_space.near_m,
            self.depth_space.far_m,
            depth_curve_label(self.depth_space.curve),
        ));
        lines.push(format!(
            "  source {} id={} availability={} origin={}",
            self.color.kind.as_str(),
            self.color.id.0,
            availability_label(self.color.availability),
            origin_label(&self.color.origin),
        ));
        for kind in [
            VisualSourceKind2d::SceneDepth,
            VisualSourceKind2d::LayerMask,
            VisualSourceKind2d::SceneNormal,
            VisualSourceKind2d::SceneWetness,
            VisualSourceKind2d::SceneEmissive,
            VisualSourceKind2d::SceneHighlight,
            VisualSourceKind2d::SceneMotion,
        ] {
            if let Some(source) = self.source(kind) {
                lines.push(format!(
                    "  source {} id={} availability={} origin={}",
                    kind.as_str(),
                    source.id.0,
                    availability_label(source.availability),
                    origin_label(&source.origin),
                ));
            } else {
                lines.push(format!(
                    "  source {} id=none availability={} origin=missing",
                    kind.as_str(),
                    availability_label(VisualSourceAvailability2d::Missing),
                ));
            }
        }
        let missing = self
            .missing_source_kinds()
            .into_iter()
            .map(VisualSourceKind2d::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "  missing: {}",
            if missing.is_empty() { "none" } else { &missing }
        ));
        lines.push(format!(
            "  missing_count={}",
            self.missing_source_kinds().len()
        ));
        lines.push(format!("  layers={}", self.layers.len()));
        if self.layers.is_empty() {
            lines.push("  layer_summary=none".to_owned());
        } else {
            let mut role_counts = BTreeMap::<&'static str, usize>::new();
            for layer in &self.layers {
                *role_counts
                    .entry(optical_role_label(layer.role))
                    .or_default() += 1;
                let distance = layer
                    .distance_m
                    .map(|value| format!("{value:.2}m"))
                    .unwrap_or_else(|| "none".to_owned());
                lines.push(format!(
                    "  layer={} role={} depth_mode={} distance_m/base={} effective_distance_m={} base_z_depth={:.3} effective_z_depth={:.3} z_depth={:.3} blur_scale={:.2} camera_motion_scale={:.2}",
                    layer.layer_id,
                    optical_role_label(layer.role),
                    layer.depth_mode,
                    distance,
                    layer
                        .effective_distance_m
                        .map(|value| format!("{value:.2}m"))
                        .unwrap_or_else(|| "none".to_owned()),
                    layer.base_z_depth,
                    layer.effective_z_depth,
                    layer.z_depth,
                    layer.blur_scale,
                    layer.camera_motion_scale
                ));
            }
            let role_summary = role_counts
                .into_iter()
                .map(|(role, count)| format!("{role}:{count}"))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("  optical_roles={role_summary}"));
        }
        let diagnostics = self.diagnostics();
        lines.push("Diagnostics:".to_owned());
        if diagnostics.is_empty() {
            lines.push("  - none".to_owned());
        } else {
            for diagnostic in diagnostics {
                lines.push(format!("  - {} {}", diagnostic.code, diagnostic.message));
            }
        }
        lines.join("\n")
    }
}

fn depth_curve_label(curve: DepthCurve2d) -> &'static str {
    match curve {
        DepthCurve2d::Linear => "linear",
        DepthCurve2d::Logarithmic => "logarithmic",
    }
}

fn availability_label(value: VisualSourceAvailability2d) -> &'static str {
    match value {
        VisualSourceAvailability2d::Produced => "produced",
        VisualSourceAvailability2d::Derived => "derived",
        VisualSourceAvailability2d::Asset => "asset",
        VisualSourceAvailability2d::Fallback => "fallback",
        VisualSourceAvailability2d::Missing => "missing",
    }
}

fn origin_label(origin: &VisualSourceOrigin2d) -> String {
    match origin {
        VisualSourceOrigin2d::WorldPass => "world_pass".to_owned(),
        VisualSourceOrigin2d::DepthPass => "depth_pass".to_owned(),
        VisualSourceOrigin2d::LayerMaskPass => "layer_mask_pass".to_owned(),
        VisualSourceOrigin2d::LayerRolePass => "layer_role_pass".to_owned(),
        VisualSourceOrigin2d::MaterialBuffer => "material_buffer".to_owned(),
        VisualSourceOrigin2d::LightBuffer => "light_buffer".to_owned(),
        VisualSourceOrigin2d::EmissiveBuffer => "emissive_buffer".to_owned(),
        VisualSourceOrigin2d::MotionBuffer => "motion_buffer".to_owned(),
        VisualSourceOrigin2d::LightExtraction => "light_extraction".to_owned(),
        VisualSourceOrigin2d::BeaconExtraction => "beacon_extraction".to_owned(),
        VisualSourceOrigin2d::PostFxDerived { feature } => format!("postfx:{feature}"),
        VisualSourceOrigin2d::Asset { path } => format!("asset:{path}"),
        VisualSourceOrigin2d::ShutterHistory => "shutter_history".to_owned(),
        VisualSourceOrigin2d::DebugFallback => "debug_fallback".to_owned(),
    }
}

fn optical_role_label(role: OpticalLayerRole2d) -> &'static str {
    match role {
        OpticalLayerRole2d::WorldSurface => "world_surface",
        OpticalLayerRole2d::SceneMedium => "scene_medium",
        OpticalLayerRole2d::ForegroundMedium => "foreground_medium",
        OpticalLayerRole2d::LensSurface => "lens_surface",
        OpticalLayerRole2d::Overlay => "overlay",
        OpticalLayerRole2d::Debug => "debug",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_debug_view_parses_snake_case_names() {
        assert_eq!(
            CameraDebugView2d::parse("computed_z_depth"),
            CameraDebugView2d::new("camera.computed_z_depth")
        );
        assert_eq!(
            CameraDebugView2d::parse("mask"),
            CameraDebugView2d::new("camera.layer_mask")
        );
        assert_eq!(
            CameraDebugView2d::parse("emissive"),
            CameraDebugView2d::new("camera.scene_emissive")
        );
        assert_eq!(
            CameraDebugView2d::parse("camera_after_dof").as_str(),
            "camera.after_dof"
        );
    }

    #[test]
    fn camera_debug_view_helpers_expose_stop_and_visual_source_intent() {
        assert_eq!(
            CameraDebugView2d::parse("camera_after_optics").stop_after_feature(),
            Some("camera_optics")
        );
        assert!(CameraDebugView2d::parse("scene_wetness").wants_visual_source_debug());
        assert!(CameraDebugView2d::parse("layer_mask").wants_visual_source_debug());
        assert!(!CameraDebugView2d::final_output().wants_visual_source_debug());
    }

    #[test]
    fn camera_capture_input_builder_tracks_present_and_missing_sources() {
        let input = CameraCaptureInput2dBuilder::new(DepthSpace2d::default(), Vec::new())
            .with_depth("world.depth")
            .with_layer_mask("world.layer_mask")
            .with_highlight("world.highlight")
            .with_emissive("world.emissive")
            .build();

        assert_eq!(
            input.depth.as_ref().map(|source| source.kind),
            Some(VisualSourceKind2d::SceneDepth)
        );
        assert_eq!(
            input.highlight.as_ref().map(|source| source.kind),
            Some(VisualSourceKind2d::SceneHighlight)
        );
        assert_eq!(
            input.layer_mask.as_ref().map(|source| source.kind),
            Some(VisualSourceKind2d::LayerMask)
        );
        assert!(
            input
                .missing_source_kinds()
                .contains(&VisualSourceKind2d::SceneNormal)
        );
        assert!(
            !input
                .missing_source_kinds()
                .contains(&VisualSourceKind2d::SceneHighlight)
        );
        assert_eq!(
            input
                .source(VisualSourceKind2d::SceneHighlight)
                .map(|source| source.id.0.as_str()),
            Some("world.highlight")
        );
        assert_eq!(
            input
                .source(VisualSourceKind2d::LayerMask)
                .map(|source| source.id.0.as_str()),
            Some("world.layer_mask")
        );
    }

    #[test]
    fn camera_capture_debug_summary_reports_sources_and_layers() {
        let input = CameraCaptureInput2dBuilder::new(
            DepthSpace2d::default(),
            vec![ResolvedLayerOptics2d {
                layer_id: "weather.rain.mid".to_owned(),
                role: OpticalLayerRole2d::SceneMedium,
                depth_mode: "distance".to_owned(),
                distance_m: Some(75.0),
                z_depth: 0.41,
                base_z_depth: 0.41,
                effective_z_depth: 0.41,
                effective_distance_m: Some(75.0),
                blur_scale: 0.25,
                camera_motion_scale: amigo_2d_spatial::z_depth_to_camera_motion_scale(0.41),
            }],
        )
        .with_depth("world.depth")
        .with_layer_mask("world.layer_mask")
        .with_emissive("world.emissive")
        .build();

        let summary = input.debug_summary();
        assert!(summary.contains("depth_space near="));
        assert!(summary.contains("curve=logarithmic"));
        assert!(summary.contains("source scene_depth id=world.depth"));
        assert!(summary.contains("source layer_mask id=world.layer_mask"));
        assert!(summary.contains("source scene_wetness id=none availability=missing"));
        assert!(summary.contains("layer=weather.rain.mid role=scene_medium"));
        assert!(summary.contains("base_z_depth="));
        assert!(summary.contains("effective_z_depth="));
        assert!(summary.contains("effective_distance_m="));
        assert!(summary.contains("optical_roles=scene_medium:1"));
    }
}
