use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolMode {
    Ink,
    Pencil,
    Brush,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionMode {
    Perspective,
    Ortho,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlMode {
    Orbit,
    Freelook,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    pub model_source: String,
    pub control_mode: ControlMode,
    pub angle_snap: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub focal_length: f32,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_x: f32,
    pub camera_y: f32,
    pub camera_z: f32,
    pub projection_mode: ProjectionMode,
    pub light_az: f32,
    pub light_el: f32,
    pub mode: ToolMode,
    pub preset: String,
    pub method: String,
    pub flow_mode: String,
    pub auto: bool,
    pub imprecise_tween: bool,
    pub tween_jitter_frames: f32,
    pub anim_fps: f32,
    pub anim_time: f32,
    pub anim_frame_index: u64,
    pub anim_loop_index: u64,
    pub anim_sample_time: f32,
    pub anim_jitter_frames: f32,
    pub anim_accumulator: f32,
    pub density: f32,
    pub layers: f32,
    pub threshold: f32,
    pub core: f32,
    pub contact: f32,
    pub edge_dark: f32,
    pub simplify: f32,
    pub economy: f32,
    pub stroke_len: f32,
    pub spacing: f32,
    pub stroke_width: f32,
    pub curvature: f32,
    pub cross_angle: f32,
    pub dot_size: f32,
    pub wobble: f32,
    pub jitter: f32,
    pub stroke_crookedness: f32,
    pub stroke_kink_chance: f32,
    pub stroke_tone_ramp: f32,
    pub shadow_frame_drift: f32,
    pub shadow_loop_redraw: f32,
    pub shadow_layout_jitter: f32,
    pub projection_wobble: f32,
    pub spacing_var: f32,
    pub length_var: f32,
    pub width_var: f32,
    pub taper: f32,
    pub breakup: f32,
    pub overdraw: f32,
    pub contour_humanize: bool,
    pub contour_drift: f32,
    pub contour_wobble: f32,
    pub contour_gaps: f32,
    pub contour_frame_variance: f32,
    pub paint_enabled: bool,
    pub face_wash: bool,
    pub paint_brush: String,
    pub paint_palette: String,
    pub paint_paper_color: String,
    pub paint_base_color: String,
    pub paint_shadow_color: String,
    pub paint_highlight_color: String,
    pub paint_base_opacity: f32,
    pub paint_wash_opacity: f32,
    pub paint_cel_strength: f32,
    pub paint_cel_steps: f32,
    pub paint_highlight_amount: f32,
    pub paint_halftone: f32,
    pub paint_halftone_scale: f32,
    pub paint_registration: f32,
    pub paint_bleed: f32,
    pub paint_grain: f32,
    pub ink_dominance: f32,
    pub paint_region_resolution: f32,
    pub paint_region_simplify: f32,
    pub paint_edge_bleed: f32,
    pub paint_pigment_granulation: f32,
    pub paint_region_jitter: f32,
    pub paint_wet_mix: f32,
    pub contours: bool,
    pub shadows_enabled: bool,
    pub hide_occluded: bool,
    pub backface: bool,
    pub depth_clip_strokes: bool,
    pub clip_to_faces: bool,
    pub show_hidden: bool,
    pub sort_faces: bool,
    pub depth_eps: f32,
    pub creases: bool,
    pub suggestive: bool,
    pub contact_lines: bool,
    pub depth_fade: bool,
    pub scene_partition_enabled: bool,
    pub scene_partition_cell_size: f32,
    pub scene_partition_max_units: f32,
    pub visibility_culling_enabled: bool,
    pub visibility_margin_px: f32,
    pub visibility_min_area_px: f32,
    pub visibility_min_radius_px: f32,
    pub detail_policy_enabled: bool,
    pub detail_tier0_radius_px: f32,
    pub detail_tier1_radius_px: f32,
    pub detail_tier2_radius_px: f32,
    pub detail_tier3_radius_px: f32,
    pub detail_density_penalty: f32,
    pub detail_importance_bias: f32,
    pub vector_budget_enabled: bool,
    pub vector_max_projected_faces: f32,
    pub vector_max_visible_edges: f32,
    pub vector_max_contour_lines: f32,
    pub vector_max_shadow_marks: f32,
    pub vector_min_face_area_px: f32,
    pub vector_min_edge_length_px: f32,
    pub cleanup_min_face_area_px: f32,
    pub cleanup_min_line_length_px: f32,
    pub cleanup_max_edge_length_px: f32,
    pub cleanup_density_clamp: f32,
    pub cleanup_region_min_area_px: f32,
    pub cleanup_region_min_faces: f32,
    pub cleanup_region_max_aspect: f32,
    pub hair_region_suppression: f32,
    pub shadow_band_count: f32,
    pub shadow_region_bleed: f32,
    pub shadow_color_jitter: f32,
    pub stroke_pressure_jitter: f32,
    pub temporal_coherence: f32,
    pub projection_human_error: f32,
    pub region_budget_enabled: bool,
    pub region_min_projected_area_px: f32,
    pub region_max_paint_regions: f32,
    pub region_allow_far_fills: bool,
    pub main_contour_enabled: bool,
    pub crease_accent_enabled: bool,
    pub suggestive_contour_enabled: bool,
    pub hidden_line_enabled: bool,
    pub shadow_hatch_enabled: bool,
    pub base_wash_enabled: bool,
    pub shadow_region_enabled: bool,
    pub highlight_region_enabled: bool,
    pub tone_debug: bool,
    pub flow_debug: bool,
    pub depth_debug: bool,
    pub seed_debug: bool,
    pub region_debug: bool,
    pub cleanup_debug: bool,
    pub density_debug: bool,
    pub visibility_debug: bool,
    pub detail_debug: bool,
    pub budget_debug: bool,
    pub skip_simulation: bool,
    pub raw_yaw: f32,
    pub raw_pitch: f32,
    pub raw_camera_yaw: f32,
    pub raw_camera_pitch: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            model_source: "suzanne".to_owned(),
            control_mode: ControlMode::Orbit,
            angle_snap: 0.0,
            yaw: -24.0,
            pitch: 12.0,
            zoom: 1.0,
            focal_length: 35.0,
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_x: 0.0,
            camera_y: 0.0,
            camera_z: 5.0,
            projection_mode: ProjectionMode::Perspective,
            light_az: -42.0,
            light_el: 42.0,
            mode: ToolMode::Ink,
            preset: "cleanInk".to_owned(),
            method: "hatching".to_owned(),
            flow_mode: "mixed".to_owned(),
            auto: false,
            imprecise_tween: true,
            tween_jitter_frames: 1.25,
            anim_fps: 24.0,
            anim_time: 0.0,
            anim_frame_index: 0,
            anim_loop_index: 0,
            anim_sample_time: 0.0,
            anim_jitter_frames: 0.0,
            anim_accumulator: 0.0,
            density: 0.72,
            layers: 1.0,
            threshold: 0.20,
            core: 1.0,
            contact: 0.35,
            edge_dark: 0.24,
            simplify: 0.25,
            economy: 0.34,
            stroke_len: 44.0,
            spacing: 13.0,
            stroke_width: 1.05,
            curvature: 0.25,
            cross_angle: 55.0,
            dot_size: 2.1,
            wobble: 0.34,
            jitter: 0.23,
            stroke_crookedness: 0.18,
            stroke_kink_chance: 0.10,
            stroke_tone_ramp: 0.22,
            shadow_frame_drift: 0.18,
            shadow_loop_redraw: 0.65,
            shadow_layout_jitter: 0.25,
            projection_wobble: 0.45,
            spacing_var: 0.28,
            length_var: 0.30,
            width_var: 0.18,
            taper: 0.58,
            breakup: 0.05,
            overdraw: 0.12,
            contour_humanize: true,
            contour_drift: 1.35,
            contour_wobble: 0.22,
            contour_gaps: 0.08,
            contour_frame_variance: 0.35,
            paint_enabled: true,
            face_wash: true,
            paint_brush: "watercolor".to_owned(),
            paint_palette: "cleanComic".to_owned(),
            paint_paper_color: "#f6f2e8".to_owned(),
            paint_base_color: "#d7ad85".to_owned(),
            paint_shadow_color: "#5d6f95".to_owned(),
            paint_highlight_color: "#fff0c2".to_owned(),
            paint_base_opacity: 0.58,
            paint_wash_opacity: 0.34,
            paint_cel_strength: 0.42,
            paint_cel_steps: 3.0,
            paint_highlight_amount: 0.18,
            paint_halftone: 0.22,
            paint_halftone_scale: 14.0,
            paint_registration: 1.15,
            paint_bleed: 0.65,
            paint_grain: 0.16,
            ink_dominance: 1.0,
            paint_region_resolution: 384.0,
            paint_region_simplify: 0.45,
            paint_edge_bleed: 0.65,
            paint_pigment_granulation: 0.35,
            paint_region_jitter: 0.25,
            paint_wet_mix: 0.45,
            contours: true,
            shadows_enabled: true,
            hide_occluded: true,
            backface: true,
            depth_clip_strokes: true,
            clip_to_faces: true,
            show_hidden: false,
            sort_faces: true,
            depth_eps: 0.018,
            creases: true,
            suggestive: true,
            contact_lines: true,
            depth_fade: true,
            scene_partition_enabled: true,
            scene_partition_cell_size: 24.0,
            scene_partition_max_units: 4096.0,
            visibility_culling_enabled: true,
            visibility_margin_px: 80.0,
            visibility_min_area_px: 2.0,
            visibility_min_radius_px: 1.5,
            detail_policy_enabled: true,
            detail_tier0_radius_px: 180.0,
            detail_tier1_radius_px: 80.0,
            detail_tier2_radius_px: 28.0,
            detail_tier3_radius_px: 8.0,
            detail_density_penalty: 0.35,
            detail_importance_bias: 1.0,
            vector_budget_enabled: true,
            vector_max_projected_faces: 18_000.0,
            vector_max_visible_edges: 12_000.0,
            vector_max_contour_lines: 7_000.0,
            vector_max_shadow_marks: 2_600.0,
            vector_min_face_area_px: 0.8,
            vector_min_edge_length_px: 1.6,
            cleanup_min_face_area_px: 2.0,
            cleanup_min_line_length_px: 3.0,
            cleanup_max_edge_length_px: 500.0,
            cleanup_density_clamp: 0.65,
            cleanup_region_min_area_px: 80.0,
            cleanup_region_min_faces: 3.0,
            cleanup_region_max_aspect: 16.0,
            hair_region_suppression: 0.5,
            shadow_band_count: 3.0,
            shadow_region_bleed: 0.2,
            shadow_color_jitter: 0.25,
            stroke_pressure_jitter: 0.22,
            temporal_coherence: 0.85,
            projection_human_error: 0.12,
            region_budget_enabled: true,
            region_min_projected_area_px: 24.0,
            region_max_paint_regions: 600.0,
            region_allow_far_fills: false,
            main_contour_enabled: true,
            crease_accent_enabled: true,
            suggestive_contour_enabled: true,
            hidden_line_enabled: true,
            shadow_hatch_enabled: true,
            base_wash_enabled: true,
            shadow_region_enabled: true,
            highlight_region_enabled: true,
            tone_debug: false,
            flow_debug: false,
            depth_debug: false,
            seed_debug: false,
            region_debug: false,
            cleanup_debug: false,
            density_debug: false,
            visibility_debug: false,
            detail_debug: false,
            budget_debug: false,
            skip_simulation: false,
            raw_yaw: -24.0,
            raw_pitch: 12.0,
            raw_camera_yaw: 0.0,
            raw_camera_pitch: 0.0,
        }
    }
}

impl AppState {
    pub fn reset_view_for_obj(&mut self) {
        self.control_mode = ControlMode::Orbit;
        self.angle_snap = 0.0;
        self.yaw = -24.0;
        self.pitch = 12.0;
        self.zoom = 1.0;
        self.camera_yaw = 0.0;
        self.camera_pitch = 0.0;
        self.camera_x = 0.0;
        self.camera_y = 0.0;
        self.camera_z = 5.0;
        self.raw_yaw = self.yaw;
        self.raw_pitch = self.pitch;
        self.raw_camera_yaw = self.camera_yaw;
        self.raw_camera_pitch = self.camera_pitch;
        self.focal_length = 35.0;
        self.projection_mode = ProjectionMode::Perspective;
        self.backface = true;
    }

    pub fn reset_view_for_fbx(&mut self) {
        self.control_mode = ControlMode::Freelook;
        self.angle_snap = 0.0;
        self.yaw = -24.0;
        self.pitch = 12.0;
        self.zoom = 1.0;
        self.camera_yaw = 0.0;
        self.camera_pitch = 0.0;
        self.camera_x = 0.0;
        self.camera_y = 0.6;
        self.camera_z = 6.5;
        self.raw_yaw = self.yaw;
        self.raw_pitch = self.pitch;
        self.raw_camera_yaw = self.camera_yaw;
        self.raw_camera_pitch = self.camera_pitch;
        self.focal_length = 35.0;
        self.projection_mode = ProjectionMode::Perspective;
        self.backface = false;
        self.auto = true;
        self.apply_preset("fbxBalanced");
        self.reset_animation();
    }

    pub fn apply_preset(&mut self, key: &str) {
        self.preset = key.to_owned();
        match key {
            "engraving" => {
                self.mode = ToolMode::Ink;
                self.method = "crosshatch".into();
                self.flow_mode = "parallel".into();
                self.density = 1.08;
                self.layers = 3.0;
                self.threshold = 0.14;
                self.core = 1.35;
                self.spacing = 9.0;
                self.stroke_len = 60.0;
                self.stroke_width = 0.72;
                self.curvature = 0.08;
                self.wobble = 0.06;
                self.jitter = 0.04;
            }
            "loosePencil" => {
                self.mode = ToolMode::Pencil;
                self.method = "hatching".into();
                self.flow_mode = "mixed".into();
                self.density = 0.95;
                self.layers = 2.0;
                self.threshold = 0.13;
                self.stroke_len = 38.0;
                self.stroke_width = 0.72;
                self.curvature = 0.36;
                self.wobble = 0.56;
                self.jitter = 0.45;
                self.overdraw = 0.28;
            }
            "manga" => {
                self.mode = ToolMode::Ink;
                self.method = "comic".into();
                self.flow_mode = "terminator".into();
                self.density = 0.92;
                self.layers = 2.0;
                self.threshold = 0.25;
                self.core = 1.9;
                self.spacing = 9.0;
                self.stroke_len = 52.0;
                self.stroke_width = 1.35;
                self.edge_dark = 0.55;
                self.contact = 0.7;
            }
            "pipelineCleanInk" => {
                self.mode = ToolMode::Ink;
                self.method = "hatching".into();
                self.flow_mode = "mixed".into();
                self.density = 0.76;
                self.layers = 1.0;
                self.threshold = 0.20;
                self.cleanup_min_line_length_px = 4.0;
                self.temporal_coherence = 0.9;
                self.projection_human_error = 0.08;
            }
            "largeSceneBalanced" => {
                self.scene_partition_enabled = true;
                self.scene_partition_cell_size = 32.0;
                self.visibility_culling_enabled = true;
                self.detail_policy_enabled = true;
                self.vector_budget_enabled = true;
                self.vector_max_projected_faces = 12_000.0;
                self.vector_max_visible_edges = 8_000.0;
                self.vector_max_contour_lines = 5_000.0;
                self.vector_max_shadow_marks = 900.0;
            }
            "fbxBalanced" => {
                self.mode = ToolMode::Ink;
                self.method = "hatching".into();
                self.flow_mode = "silhouette".into();
                self.density = 0.20;
                self.layers = 1.0;
                self.threshold = 0.24;
                self.core = 1.25;
                self.contact = 0.22;
                self.edge_dark = 0.30;
                self.simplify = 0.42;
                self.economy = 0.68;
                self.stroke_len = 30.0;
                self.spacing = 22.0;
                self.stroke_width = 0.82;
                self.curvature = 0.18;
                self.wobble = 0.16;
                self.jitter = 0.10;
                self.projection_wobble = 0.18;
                self.paint_base_opacity = 0.32;
                self.paint_wash_opacity = 0.14;
                self.contours = true;
                self.creases = false;
                self.suggestive = false;
                self.shadows_enabled = false;
                self.vector_budget_enabled = false;
                self.vector_max_projected_faces = 200_000.0;
                self.vector_max_visible_edges = 200_000.0;
                self.vector_max_contour_lines = 80_000.0;
                self.vector_max_shadow_marks = 2_600.0;
                self.vector_min_face_area_px = 0.6;
                self.cleanup_min_face_area_px = 1.5;
                self.cleanup_density_clamp = 0.45;
            }
            _ => {
                self.mode = ToolMode::Ink;
                self.method = "hatching".into();
                self.flow_mode = "mixed".into();
                self.density = 0.72;
                self.layers = 1.0;
                self.threshold = 0.20;
                self.core = 1.0;
                self.spacing = 13.0;
                self.stroke_len = 44.0;
                self.stroke_width = 1.05;
                self.curvature = 0.18;
                self.wobble = 0.20;
                self.jitter = 0.12;
            }
        }
    }

    pub fn reset_animation(&mut self) {
        self.anim_time = 0.0;
        self.anim_frame_index = 0;
        self.anim_loop_index = 0;
        self.anim_sample_time = 0.0;
        self.anim_jitter_frames = 0.0;
        self.anim_accumulator = 0.0;
    }

    pub fn advance_animation(&mut self, dt: f32, duration: f32) {
        let step = 1.0 / self.anim_fps.max(1.0);
        let duration = duration.max(0.001);
        self.anim_accumulator += dt.clamp(0.0, 0.12);
        if self.anim_accumulator < step {
            return;
        }
        let ticks = (self.anim_accumulator / step).floor().max(1.0);
        self.anim_accumulator -= ticks * step;
        let next_time = self.anim_time + ticks * step;
        self.anim_loop_index += (next_time / duration).floor().max(0.0) as u64;
        self.anim_time = wrap_time(next_time, duration);
        self.anim_frame_index = self.anim_frame_index.saturating_add(ticks as u64);
        self.update_anim_sample_time(step, duration);
    }

    pub fn update_anim_sample_time(&mut self, step: f32, duration: f32) {
        if !self.imprecise_tween || self.tween_jitter_frames <= 0.0 {
            self.anim_jitter_frames = 0.0;
            self.anim_sample_time = wrap_time(self.anim_time, duration);
            return;
        }
        let frame = self.anim_frame_index as f32;
        let a = noise(41.73, frame);
        let b = noise(93.17, (frame / 3.0).floor());
        let hold_bias = noise(12.31, (frame / 5.0).floor()) * 0.35;
        let target = ((a * 0.72 + b * 0.28 + hold_bias) * self.tween_jitter_frames)
            .clamp(-self.tween_jitter_frames, self.tween_jitter_frames);
        self.anim_jitter_frames += (target - self.anim_jitter_frames) * 0.42;
        self.anim_sample_time =
            wrap_time(self.anim_time + self.anim_jitter_frames * step, duration);
    }
}

fn wrap_time(value: f32, duration: f32) -> f32 {
    if duration <= 0.0 {
        return value;
    }
    let mut out = value % duration;
    if out < 0.0 {
        out += duration;
    }
    out
}

fn noise(seed: f32, x: f32) -> f32 {
    ((x * 12.9898 + seed * 78.233).sin() * 43_758.547).fract()
}

#[cfg(test)]
mod tests {
    use super::{AppState, ControlMode};

    #[test]
    fn fbx_reset_uses_animated_balanced_defaults() {
        let mut state = AppState::default();
        state.reset_view_for_fbx();

        assert_eq!(state.control_mode, ControlMode::Freelook);
        assert_eq!(state.preset, "fbxBalanced");
        assert!(state.auto, "FBX should start animated by default");
        assert!(
            !state.backface,
            "FBX winding should not hide the imported body"
        );
        assert_eq!(state.zoom, 1.0);
        assert!(!state.vector_budget_enabled);
    }
}
