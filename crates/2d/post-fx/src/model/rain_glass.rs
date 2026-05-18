use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RainGlass2d {
    pub enabled: bool,
    pub spawn_rate: f32,
    pub spawn_limit: u32,
    pub min_radius_px: f32,
    pub max_radius_px: f32,
    pub seed: u32,
    pub gravity_px_per_sec2: f32,
    pub slip_rate: f32,
    pub motion_interval_min: f32,
    pub motion_interval_max: f32,
    pub x_shift_min: f32,
    pub x_shift_max: f32,
    pub collider_scale: f32,
    pub initial_spread: f32,
    pub shrink_rate: f32,
    pub velocity_spread: f32,
    pub evaporate: f32,
    pub trails_enabled: bool,
    pub trail_drop_density: f32,
    pub trail_drop_size_min: f32,
    pub trail_drop_size_max: f32,
    pub trail_distance_min_px: f32,
    pub trail_distance_max_px: f32,
    pub trail_spread: f32,
    pub trail_shrink_rate: f32,
    pub trail_evaporate: f32,
    pub trail_taper: f32,
    pub streak_boost: f32,
    pub streak_length: f32,
    pub micro_droplets_enabled: bool,
    pub micro_droplets_per_second: f32,
    pub micro_droplet_min_px: f32,
    pub micro_droplet_max_px: f32,
    pub mist_enabled: bool,
    pub mist_opacity: f32,
    pub mist_blur_px: f32,
    pub mist_accumulation: f32,
    pub mist_time: f32,
    pub mist_color_strength: f32,
    pub mist_blur_step: u32,
    pub background_blur_px: f32,
    pub background_blur_steps: u32,
    pub smooth_edge_min: f32,
    pub smooth_edge_max: f32,
    pub refract_base: f32,
    pub refract_scale: f32,
    pub opacity: f32,
    pub chromatic_aberration: f32,
    pub distortion_px: f32,
    pub normal_strength: f32,
    pub focus_blur_strength: f32,
    pub body_opacity: f32,
    pub scene_blend: f32,
    pub drop_plane_blur_px: f32,
    pub receives_scene_light: bool,
    pub scene_light_tint_strength: f32,
    pub scene_shadow_floor: f32,
    pub blood_tint: [f32; 3],
    pub blood_amount: f32,
    pub scene_darken: f32,
    pub trail_refract_scale: f32,
    pub trail_opacity: f32,
    pub reference_mode: bool,
    pub raindrop_compose: RainGlassRaindropCompose,
    pub raindrop_eraser_size: [f32; 2],
    pub scene_light_response: f32,
    pub rim_strength: f32,
    pub light_pos: [f32; 4],
    pub diffuse_light: [f32; 3],
    pub shadow_offset: f32,
    pub specular_light: [f32; 3],
    pub specular_shininess: f32,
    pub light_bump: f32,
    pub z_depth: Option<f32>,
    pub z_depth_blur_scale: f32,
    pub z_depth_focus_response: f32,
    pub camera_focus_depth: f32,
    pub camera_focus_width: f32,
    pub camera_focus_enabled: bool,
    pub quality_scale: f32,
    pub debug_view: RainGlassDebugView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RainGlassRaindropCompose {
    Smoother,
    Harder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RainGlassDebugView {
    Final,
    SceneInput,
    BlurredScene,
    RaindropMap,
    DropletMap,
    TrailMap,
    DropNormals,
    DropMask,
    Mist,
    Refraction,
}

impl Default for RainGlass2d {
    fn default() -> Self {
        Self {
            enabled: true,
            spawn_rate: 10.0,
            spawn_limit: 850,
            min_radius_px: 45.0,
            max_radius_px: 118.0,
            seed: 121713,
            gravity_px_per_sec2: 2400.0,
            slip_rate: 0.34,
            motion_interval_min: 0.08,
            motion_interval_max: 0.32,
            x_shift_min: 0.0,
            x_shift_max: 0.12,
            collider_scale: 1.0,
            initial_spread: 0.52,
            shrink_rate: 0.014,
            velocity_spread: 0.34,
            evaporate: 11.0,
            trails_enabled: true,
            trail_drop_density: 0.3224,
            trail_drop_size_min: 0.2436,
            trail_drop_size_max: 0.5124,
            trail_distance_min_px: 10.36,
            trail_distance_max_px: 21.76,
            trail_spread: 1.06024,
            trail_shrink_rate: 0.364,
            trail_evaporate: 18.0,
            trail_taper: 0.68,
            streak_boost: 0.72,
            streak_length: 1.15,
            micro_droplets_enabled: true,
            micro_droplets_per_second: 620.0,
            micro_droplet_min_px: 8.0,
            micro_droplet_max_px: 27.0,
            mist_enabled: true,
            mist_opacity: 1.0,
            mist_blur_px: 4.0,
            mist_accumulation: 0.012,
            mist_time: 16.0,
            mist_color_strength: 0.012,
            mist_blur_step: 4,
            background_blur_px: 2.0,
            background_blur_steps: 2,
            smooth_edge_min: 0.945,
            smooth_edge_max: 0.992,
            refract_base: 0.34,
            refract_scale: 0.76,
            opacity: 1.0,
            chromatic_aberration: 0.0,
            distortion_px: 28.0,
            normal_strength: 6.0,
            focus_blur_strength: 0.85,
            body_opacity: 1.0,
            scene_blend: 1.0,
            drop_plane_blur_px: 0.0,
            receives_scene_light: true,
            scene_light_tint_strength: 0.35,
            scene_shadow_floor: 0.28,
            blood_tint: [0.55, 0.015, 0.01],
            blood_amount: 0.0,
            scene_darken: 0.0,
            trail_refract_scale: 1.0,
            trail_opacity: 1.0,
            reference_mode: true,
            raindrop_compose: RainGlassRaindropCompose::Smoother,
            raindrop_eraser_size: [0.93, 1.0],
            scene_light_response: 0.0,
            rim_strength: 0.0,
            light_pos: [-1.0, 1.0, 2.0, 0.0],
            diffuse_light: [0.22, 0.22, 0.22],
            shadow_offset: 0.76,
            specular_light: [0.025, 0.025, 0.025],
            specular_shininess: 300.0,
            light_bump: 0.78,
            z_depth: None,
            z_depth_blur_scale: 1.0,
            z_depth_focus_response: 0.0,
            camera_focus_depth: 0.5,
            camera_focus_width: 0.05,
            camera_focus_enabled: false,
            quality_scale: 1.0,
            debug_view: RainGlassDebugView::Final,
        }
    }
}

impl RainGlass2d {
    pub fn normalized(mut self) -> Self {
        let defaults = Self::default();
        self.spawn_rate = finite_or(self.spawn_rate, defaults.spawn_rate).clamp(0.0, 120.0);
        self.spawn_limit = self.spawn_limit.clamp(0, 3000);
        self.min_radius_px =
            finite_or(self.min_radius_px, defaults.min_radius_px).clamp(1.0, 256.0);
        self.max_radius_px =
            finite_or(self.max_radius_px, defaults.max_radius_px).clamp(self.min_radius_px, 256.0);
        self.gravity_px_per_sec2 =
            finite_or(self.gravity_px_per_sec2, defaults.gravity_px_per_sec2).clamp(0.0, 6000.0);
        self.slip_rate = finite_or(self.slip_rate, defaults.slip_rate).clamp(0.0, 1.0);
        self.motion_interval_min =
            finite_or(self.motion_interval_min, defaults.motion_interval_min).clamp(0.01, 5.0);
        self.motion_interval_max =
            finite_or(self.motion_interval_max, defaults.motion_interval_max)
                .clamp(self.motion_interval_min, 10.0);
        self.x_shift_min = finite_or(self.x_shift_min, defaults.x_shift_min).clamp(-2.0, 2.0);
        self.x_shift_max =
            finite_or(self.x_shift_max, defaults.x_shift_max).clamp(self.x_shift_min, 2.0);
        self.collider_scale =
            finite_or(self.collider_scale, defaults.collider_scale).clamp(0.05, 4.0);
        self.initial_spread =
            finite_or(self.initial_spread, defaults.initial_spread).clamp(0.0, 2.0);
        self.shrink_rate = finite_or(self.shrink_rate, defaults.shrink_rate).clamp(0.001, 1.0);
        self.velocity_spread =
            finite_or(self.velocity_spread, defaults.velocity_spread).clamp(0.0, 4.0);
        self.evaporate = finite_or(self.evaporate, defaults.evaporate).clamp(0.0, 200.0);
        self.trail_drop_density =
            finite_or(self.trail_drop_density, defaults.trail_drop_density).clamp(0.01, 1.0);
        self.trail_drop_size_min =
            finite_or(self.trail_drop_size_min, defaults.trail_drop_size_min).clamp(0.01, 4.0);
        self.trail_drop_size_max =
            finite_or(self.trail_drop_size_max, defaults.trail_drop_size_max)
                .clamp(self.trail_drop_size_min, 4.0);
        self.trail_distance_min_px =
            finite_or(self.trail_distance_min_px, defaults.trail_distance_min_px).clamp(1.0, 512.0);
        self.trail_distance_max_px =
            finite_or(self.trail_distance_max_px, defaults.trail_distance_max_px)
                .clamp(self.trail_distance_min_px, 1024.0);
        self.trail_spread = finite_or(self.trail_spread, defaults.trail_spread).clamp(0.0, 4.0);
        self.trail_shrink_rate =
            finite_or(self.trail_shrink_rate, defaults.trail_shrink_rate).clamp(0.001, 1.0);
        self.trail_evaporate =
            finite_or(self.trail_evaporate, defaults.trail_evaporate).clamp(0.0, 200.0);
        self.trail_taper = finite_or(self.trail_taper, defaults.trail_taper).clamp(0.0, 1.0);
        self.streak_boost = finite_or(self.streak_boost, defaults.streak_boost).clamp(0.0, 2.0);
        self.streak_length = finite_or(self.streak_length, defaults.streak_length).clamp(0.0, 4.0);
        self.micro_droplets_per_second = finite_or(
            self.micro_droplets_per_second,
            defaults.micro_droplets_per_second,
        )
        .clamp(0.0, 5000.0);
        self.micro_droplet_min_px =
            finite_or(self.micro_droplet_min_px, defaults.micro_droplet_min_px).clamp(1.0, 128.0);
        self.micro_droplet_max_px =
            finite_or(self.micro_droplet_max_px, defaults.micro_droplet_max_px)
                .clamp(self.micro_droplet_min_px, 256.0);
        self.mist_opacity = finite_or(self.mist_opacity, defaults.mist_opacity).clamp(0.0, 1.0);
        self.mist_blur_px = finite_or(self.mist_blur_px, defaults.mist_blur_px).clamp(0.0, 32.0);
        self.mist_accumulation =
            finite_or(self.mist_accumulation, defaults.mist_accumulation).clamp(0.0, 1.0);
        self.mist_time = finite_or(self.mist_time, defaults.mist_time).clamp(0.1, 120.0);
        self.mist_color_strength =
            finite_or(self.mist_color_strength, defaults.mist_color_strength).clamp(0.0, 1.0);
        self.mist_blur_step = self.mist_blur_step.clamp(0, 8);
        self.background_blur_px =
            finite_or(self.background_blur_px, defaults.background_blur_px).clamp(0.0, 32.0);
        self.background_blur_steps = self.background_blur_steps.clamp(0, 8);
        self.smooth_edge_min =
            finite_or(self.smooth_edge_min, defaults.smooth_edge_min).clamp(0.0, 1.0);
        self.smooth_edge_max = finite_or(self.smooth_edge_max, defaults.smooth_edge_max)
            .clamp(self.smooth_edge_min, 1.0);
        self.refract_base = finite_or(self.refract_base, defaults.refract_base).clamp(0.0, 2.0);
        self.refract_scale = finite_or(self.refract_scale, defaults.refract_scale).clamp(0.0, 4.0);
        self.opacity = finite_or(self.opacity, 1.0).clamp(0.0, 1.0);
        self.chromatic_aberration =
            finite_or(self.chromatic_aberration, defaults.chromatic_aberration).clamp(0.0, 4.0);
        self.distortion_px =
            finite_or(self.distortion_px, defaults.distortion_px).clamp(0.0, 128.0);
        self.normal_strength =
            finite_or(self.normal_strength, defaults.normal_strength).clamp(0.0, 16.0);
        self.focus_blur_strength =
            finite_or(self.focus_blur_strength, defaults.focus_blur_strength).clamp(0.0, 2.0);
        self.body_opacity = finite_or(self.body_opacity, defaults.body_opacity).clamp(0.0, 1.0);
        self.scene_blend = finite_or(self.scene_blend, defaults.scene_blend).clamp(0.0, 1.0);
        self.drop_plane_blur_px =
            finite_or(self.drop_plane_blur_px, defaults.drop_plane_blur_px).clamp(0.0, 8.0);
        self.scene_light_tint_strength = finite_or(
            self.scene_light_tint_strength,
            defaults.scene_light_tint_strength,
        )
        .clamp(0.0, 1.0);
        self.scene_shadow_floor =
            finite_or(self.scene_shadow_floor, defaults.scene_shadow_floor).clamp(0.0, 1.0);
        self.blood_tint = [
            finite_or(self.blood_tint[0], defaults.blood_tint[0]).clamp(0.0, 1.0),
            finite_or(self.blood_tint[1], defaults.blood_tint[1]).clamp(0.0, 1.0),
            finite_or(self.blood_tint[2], defaults.blood_tint[2]).clamp(0.0, 1.0),
        ];
        self.blood_amount = finite_or(self.blood_amount, defaults.blood_amount).clamp(0.0, 1.0);
        self.scene_darken = finite_or(self.scene_darken, defaults.scene_darken).clamp(0.0, 1.0);
        self.trail_refract_scale =
            finite_or(self.trail_refract_scale, defaults.trail_refract_scale).clamp(0.0, 2.0);
        self.trail_opacity = finite_or(self.trail_opacity, defaults.trail_opacity).clamp(0.0, 1.0);
        self.raindrop_eraser_size = [
            finite_or(
                self.raindrop_eraser_size[0],
                defaults.raindrop_eraser_size[0],
            )
            .clamp(0.0, 4.0),
            finite_or(
                self.raindrop_eraser_size[1],
                defaults.raindrop_eraser_size[1],
            )
            .clamp(0.0, 4.0),
        ];
        self.scene_light_response =
            finite_or(self.scene_light_response, defaults.scene_light_response).clamp(0.0, 5.0);
        self.rim_strength = finite_or(self.rim_strength, defaults.rim_strength).clamp(0.0, 5.0);
        self.shadow_offset = finite_or(self.shadow_offset, defaults.shadow_offset).clamp(0.0, 2.0);
        self.specular_shininess =
            finite_or(self.specular_shininess, defaults.specular_shininess).clamp(1.0, 1024.0);
        self.light_bump = finite_or(self.light_bump, defaults.light_bump).clamp(0.05, 4.0);
        self.z_depth = self
            .z_depth
            .map(|z_depth| finite_or(z_depth, defaults.camera_focus_depth).clamp(0.0, 1.0));
        self.z_depth_blur_scale =
            finite_or(self.z_depth_blur_scale, defaults.z_depth_blur_scale).clamp(0.0, 4.0);
        self.z_depth_focus_response =
            finite_or(self.z_depth_focus_response, defaults.z_depth_focus_response).clamp(0.0, 2.0);
        self.camera_focus_depth =
            finite_or(self.camera_focus_depth, defaults.camera_focus_depth).clamp(0.0, 1.0);
        self.camera_focus_width =
            finite_or(self.camera_focus_width, defaults.camera_focus_width).clamp(0.001, 1.0);
        self.quality_scale = finite_or(self.quality_scale, defaults.quality_scale).clamp(0.35, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.enabled
            && self.opacity > 0.0
            && (self.spawn_limit > 0 || self.micro_droplets_enabled || self.mist_enabled)
    }
}
