use std::collections::BTreeMap;

pub const POST_FX_2D_CAPABILITY: &str = "post_fx_2d";
pub const POST_FX_2D_PLUGIN_LABEL: &str = "amigo-2d-post-fx";

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PostFx2dStack {
    pub effects: Vec<PostFx2d>,
}

impl PostFx2dStack {
    pub fn single(effect: PostFx2d) -> Self {
        Self {
            effects: vec![effect],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn normalized(self) -> Self {
        Self {
            effects: self
                .effects
                .into_iter()
                .map(PostFx2d::normalized)
                .filter(PostFx2d::is_active)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PostFx2d {
    Blur(PostFxBlur2d),
    ColorQuantize(ColorQuantize2d),
    Crt(Crt2d),
    Downscale(Downscale2d),
    DirtyBloom(DirtyBloom2d),
    EmbossEdges(PostFxEmbossEdges2d),
    FilmNoise(FilmNoise2d),
    LensDroplets(PostFxLensDroplets2d),
    RainGlass(RainGlass2d),
    ShutterBlur(ShutterBlur2d),
    WetReflections(PostFxWetReflections2d),
}

impl PostFx2d {
    pub fn kind(self) -> &'static str {
        match self {
            Self::Blur(_) => "blur",
            Self::ColorQuantize(_) => "color_quantize",
            Self::Crt(_) => "crt",
            Self::Downscale(_) => "downscale",
            Self::DirtyBloom(_) => "dirty_bloom",
            Self::EmbossEdges(_) => "embossed_edges",
            Self::FilmNoise(_) => "film_noise",
            Self::LensDroplets(_) => "lens_droplets",
            Self::RainGlass(_) => "rain_glass",
            Self::ShutterBlur(_) => "shutter_blur",
            Self::WetReflections(_) => "wet_reflections",
        }
    }

    pub fn normalized(self) -> Self {
        match self {
            Self::Blur(blur) => Self::Blur(blur.normalized()),
            Self::ColorQuantize(effect) => Self::ColorQuantize(effect.normalized()),
            Self::Crt(crt) => Self::Crt(crt.normalized()),
            Self::Downscale(effect) => Self::Downscale(effect.normalized()),
            Self::DirtyBloom(bloom) => Self::DirtyBloom(bloom.normalized()),
            Self::EmbossEdges(emboss) => Self::EmbossEdges(emboss.normalized()),
            Self::FilmNoise(noise) => Self::FilmNoise(noise.normalized()),
            Self::LensDroplets(lens) => Self::LensDroplets(lens.normalized()),
            Self::RainGlass(rain) => Self::RainGlass(rain.normalized()),
            Self::ShutterBlur(effect) => Self::ShutterBlur(effect.normalized()),
            Self::WetReflections(effect) => Self::WetReflections(effect.normalized()),
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            Self::Blur(blur) => blur.is_active(),
            Self::ColorQuantize(effect) => effect.is_active(),
            Self::Crt(crt) => crt.is_active(),
            Self::Downscale(effect) => effect.is_active(),
            Self::DirtyBloom(bloom) => bloom.is_active(),
            Self::EmbossEdges(emboss) => emboss.is_active(),
            Self::FilmNoise(noise) => noise.is_active(),
            Self::LensDroplets(lens) => lens.is_active(),
            Self::RainGlass(rain) => rain.is_active(),
            Self::ShutterBlur(effect) => effect.is_active(),
            Self::WetReflections(effect) => effect.is_active(),
        }
    }
}

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
            min_radius_px: 12.0,
            max_radius_px: 72.0,
            seed: 121713,
            gravity_px_per_sec2: 2400.0,
            slip_rate: 0.34,
            motion_interval_min: 0.10,
            motion_interval_max: 0.40,
            x_shift_min: 0.0,
            x_shift_max: 0.08,
            collider_scale: 1.0,
            initial_spread: 0.52,
            shrink_rate: 0.014,
            velocity_spread: 0.34,
            evaporate: 11.0,
            trails_enabled: true,
            trail_drop_density: 0.20,
            trail_drop_size_min: 0.28,
            trail_drop_size_max: 0.48,
            trail_distance_min_px: 18.0,
            trail_distance_max_px: 36.0,
            trail_spread: 0.58,
            trail_shrink_rate: 0.975,
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
            body_opacity: 0.92,
            trail_refract_scale: 0.48,
            trail_opacity: 0.72,
            reference_mode: true,
            raindrop_compose: RainGlassRaindropCompose::Smoother,
            raindrop_eraser_size: [0.93, 1.0],
            scene_light_response: 1.45,
            rim_strength: 1.15,
            light_pos: [-1.0, 1.0, 2.0, 0.0],
            diffuse_light: [0.035, 0.045, 0.055],
            shadow_offset: 0.76,
            specular_light: [0.018, 0.022, 0.028],
            specular_shininess: 300.0,
            light_bump: 0.78,
            debug_view: RainGlassDebugView::Final,
        }
    }
}

impl RainGlass2d {
    pub fn normalized(mut self) -> Self {
        let defaults = Self::default();
        self.spawn_rate = finite_or(self.spawn_rate, defaults.spawn_rate).clamp(0.0, 120.0);
        self.spawn_limit = self.spawn_limit.clamp(0, 3000);
        self.min_radius_px = finite_or(self.min_radius_px, 12.0).clamp(1.0, 256.0);
        self.max_radius_px = finite_or(self.max_radius_px, 72.0).clamp(self.min_radius_px, 256.0);
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
        self.trail_refract_scale =
            finite_or(self.trail_refract_scale, defaults.trail_refract_scale).clamp(0.0, 2.0);
        self.trail_opacity = finite_or(self.trail_opacity, defaults.trail_opacity).clamp(0.0, 1.0);
        self.reference_mode = self.reference_mode;
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
        self
    }

    pub fn is_active(&self) -> bool {
        self.enabled
            && self.opacity > 0.0
            && (self.spawn_limit > 0 || self.micro_droplets_enabled || self.mist_enabled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Downscale2d {
    pub factor: f32,
    pub opacity: f32,
}

impl Default for Downscale2d {
    fn default() -> Self {
        Self {
            factor: 2.0,
            opacity: 1.0,
        }
    }
}

impl Downscale2d {
    pub fn normalized(mut self) -> Self {
        self.factor = finite_or(self.factor, 2.0).clamp(1.0, 16.0);
        self.opacity = finite_or(self.opacity, 1.0).clamp(0.0, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.factor > 1.0 && self.opacity > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShutterBlur2d {
    pub fps: f32,
    pub shutter_angle: f32,
    pub opacity: f32,
    pub edge_rejection: f32,
    pub luma_threshold: f32,
    pub frame_hold: bool,
}

impl Default for ShutterBlur2d {
    fn default() -> Self {
        Self {
            fps: 24.0,
            shutter_angle: 180.0,
            opacity: 0.72,
            edge_rejection: 0.35,
            luma_threshold: 0.04,
            frame_hold: false,
        }
    }
}

impl ShutterBlur2d {
    pub fn normalized(mut self) -> Self {
        let defaults = Self::default();
        self.fps = finite_or(self.fps, defaults.fps).clamp(1.0, 240.0);
        self.shutter_angle =
            finite_or(self.shutter_angle, defaults.shutter_angle).clamp(0.0, 360.0);
        self.opacity = finite_or(self.opacity, defaults.opacity).clamp(0.0, 1.0);
        self.edge_rejection =
            finite_or(self.edge_rejection, defaults.edge_rejection).clamp(0.0, 1.0);
        self.luma_threshold =
            finite_or(self.luma_threshold, defaults.luma_threshold).clamp(0.0, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.opacity > 0.0 && self.shutter_angle > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorQuantize2d {
    pub palette_size: u32,
    pub dither_strength: f32,
    pub opacity: f32,
    pub luma_preserve: f32,
    pub highlight_bias: f32,
    pub gamma: f32,
    pub seed: u32,
}

impl Default for ColorQuantize2d {
    fn default() -> Self {
        Self {
            palette_size: 64,
            dither_strength: 0.35,
            opacity: 1.0,
            luma_preserve: 0.2,
            highlight_bias: 0.0,
            gamma: 2.2,
            seed: 911,
        }
    }
}

impl ColorQuantize2d {
    pub fn normalized(mut self) -> Self {
        self.palette_size = self.palette_size.clamp(2, 256);
        self.dither_strength = finite_or(self.dither_strength, 0.35).clamp(0.0, 1.0);
        self.opacity = finite_or(self.opacity, 1.0).clamp(0.0, 1.0);
        self.luma_preserve = finite_or(self.luma_preserve, 0.2).clamp(0.0, 1.0);
        self.highlight_bias = finite_or(self.highlight_bias, 0.0).clamp(0.0, 1.0);
        self.gamma = finite_or(self.gamma, 2.2).clamp(1.0, 3.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.palette_size >= 2 && self.opacity > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilmNoise2d {
    pub iso: f32,
    pub grain_size: f32,
    pub chroma_noise: f32,
    pub color_shift: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub flicker: f32,
    pub vignette: f32,
    pub opacity: f32,
    pub seed: u32,
}

impl Default for FilmNoise2d {
    fn default() -> Self {
        Self {
            iso: 800.0,
            grain_size: 1.0,
            chroma_noise: 0.04,
            color_shift: 0.03,
            contrast: 1.0,
            saturation: 1.0,
            flicker: 0.12,
            vignette: 0.08,
            opacity: 0.35,
            seed: 1337,
        }
    }
}

impl FilmNoise2d {
    pub fn normalized(mut self) -> Self {
        self.iso = finite_or(self.iso, 800.0).clamp(50.0, 25600.0);
        self.grain_size = finite_or(self.grain_size, 1.0).clamp(0.25, 8.0);
        self.chroma_noise = finite_or(self.chroma_noise, 0.04).clamp(0.0, 1.0);
        self.color_shift = finite_or(self.color_shift, 0.03).clamp(-1.0, 1.0);
        self.contrast = finite_or(self.contrast, 1.0).clamp(0.25, 4.0);
        self.saturation = finite_or(self.saturation, 1.0).clamp(0.0, 4.0);
        self.flicker = finite_or(self.flicker, 0.12).clamp(0.0, 1.0);
        self.vignette = finite_or(self.vignette, 0.08).clamp(0.0, 1.0);
        self.opacity = finite_or(self.opacity, 0.35).clamp(0.0, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.iso > 50.0 && self.opacity > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirtyBloom2d {
    pub threshold: f32,
    pub strength: f32,
    pub small_radius_px: f32,
    pub medium_radius_px: f32,
    pub large_radius_px: f32,
    pub dirty_noise: f32,
    pub halation_strength: f32,
    pub reflection_smear_x_px: f32,
    pub reflection_smear_y_px: f32,
    pub seed: u32,
}

impl Default for DirtyBloom2d {
    fn default() -> Self {
        Self {
            threshold: 0.62,
            strength: 0.75,
            small_radius_px: 3.0,
            medium_radius_px: 12.0,
            large_radius_px: 32.0,
            dirty_noise: 0.18,
            halation_strength: 0.22,
            reflection_smear_x_px: 6.0,
            reflection_smear_y_px: 28.0,
            seed: 4242,
        }
    }
}

impl DirtyBloom2d {
    pub fn normalized(mut self) -> Self {
        self.threshold = finite_or(self.threshold, 0.62).clamp(0.0, 2.0);
        self.strength = finite_or(self.strength, 0.75).clamp(0.0, 4.0);
        self.small_radius_px = finite_or(self.small_radius_px, 3.0).clamp(0.0, 64.0);
        self.medium_radius_px = finite_or(self.medium_radius_px, 12.0).clamp(0.0, 128.0);
        self.large_radius_px = finite_or(self.large_radius_px, 32.0).clamp(0.0, 256.0);
        self.dirty_noise = finite_or(self.dirty_noise, 0.18).clamp(0.0, 1.0);
        self.halation_strength = finite_or(self.halation_strength, 0.22).clamp(0.0, 2.0);
        self.reflection_smear_x_px = finite_or(self.reflection_smear_x_px, 6.0).clamp(0.0, 128.0);
        self.reflection_smear_y_px = finite_or(self.reflection_smear_y_px, 28.0).clamp(0.0, 256.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.strength > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crt2d {
    pub scanline_opacity: f32,
    pub scanline_frequency_px: f32,
    pub rgb_split_px: f32,
    pub curvature: f32,
    pub vignette: f32,
    pub phosphor_mask: f32,
    pub brightness_compensation: f32,
}

impl Default for Crt2d {
    fn default() -> Self {
        Self {
            scanline_opacity: 0.12,
            scanline_frequency_px: 1.5,
            rgb_split_px: 1.0,
            curvature: 0.03,
            vignette: 0.22,
            phosphor_mask: 0.04,
            brightness_compensation: 1.05,
        }
    }
}

impl Crt2d {
    pub fn normalized(mut self) -> Self {
        self.scanline_opacity = finite_or(self.scanline_opacity, 0.12).clamp(0.0, 1.0);
        self.scanline_frequency_px = finite_or(self.scanline_frequency_px, 1.5).clamp(0.5, 8.0);
        self.rgb_split_px = finite_or(self.rgb_split_px, 1.0).clamp(0.0, 8.0);
        self.curvature = finite_or(self.curvature, 0.03).clamp(0.0, 0.5);
        self.vignette = finite_or(self.vignette, 0.22).clamp(0.0, 1.0);
        self.phosphor_mask = finite_or(self.phosphor_mask, 0.04).clamp(0.0, 1.0);
        self.brightness_compensation =
            finite_or(self.brightness_compensation, 1.05).clamp(0.0, 4.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.scanline_opacity > 0.0
            || self.rgb_split_px > 0.0
            || self.curvature > 0.0
            || self.vignette > 0.0
            || self.phosphor_mask > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WetReflectionsDebugView {
    Final,
    Mask,
    Edges,
    Light,
    Distortion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostFxWetReflections2d {
    pub enabled: bool,
    pub reflection_mask: String,
    pub reflection_mask_invert: bool,
    pub edge_map: Option<String>,
    pub reflection_color: Option<String>,
    pub noise_normal: Option<String>,
    pub blur_px: f32,
    pub distortion_px: f32,
    pub shimmer_strength: f32,
    pub ripple_strength: f32,
    pub wet_darken: f32,
    pub specular_boost: f32,
    pub edge_power: f32,
    pub light_reflection_strength: f32,
    pub foreground_strength: f32,
    pub background_strength: f32,
    pub horizon_y: f32,
    pub noise_scale: f32,
    pub noise_speed: f32,
    pub ripple_speed: f32,
    pub debug_view: WetReflectionsDebugView,
}

impl Default for PostFxWetReflections2d {
    fn default() -> Self {
        Self {
            enabled: true,
            reflection_mask: String::new(),
            reflection_mask_invert: true,
            edge_map: None,
            reflection_color: None,
            noise_normal: None,
            blur_px: 1.5,
            distortion_px: 0.8,
            shimmer_strength: 0.04,
            ripple_strength: 0.02,
            wet_darken: 0.06,
            specular_boost: 0.25,
            edge_power: 1.35,
            light_reflection_strength: 0.65,
            foreground_strength: 1.0,
            background_strength: 0.12,
            horizon_y: 0.42,
            noise_scale: 2.5,
            noise_speed: 0.035,
            ripple_speed: 0.08,
            debug_view: WetReflectionsDebugView::Final,
        }
    }
}

impl PostFxWetReflections2d {
    pub fn normalized(mut self) -> Self {
        self.blur_px = self.blur_px.clamp(0.0, 12.0);
        self.distortion_px = self.distortion_px.clamp(0.0, 16.0);
        self.shimmer_strength = self.shimmer_strength.clamp(0.0, 1.0);
        self.ripple_strength = self.ripple_strength.clamp(0.0, 1.0);
        self.wet_darken = self.wet_darken.clamp(0.0, 1.0);
        self.specular_boost = self.specular_boost.clamp(0.0, 4.0);
        self.edge_power = self.edge_power.clamp(0.25, 8.0);
        self.light_reflection_strength = self.light_reflection_strength.clamp(0.0, 4.0);
        self.foreground_strength = self.foreground_strength.clamp(0.0, 4.0);
        self.background_strength = self.background_strength.clamp(0.0, 4.0);
        self.horizon_y = self.horizon_y.clamp(0.0, 1.0);
        self.noise_scale = self.noise_scale.clamp(0.01, 64.0);
        self.noise_speed = self.noise_speed.clamp(-8.0, 8.0);
        self.ripple_speed = self.ripple_speed.clamp(-8.0, 8.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.enabled && !self.reflection_mask.trim().is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensDroplets2dStage {
    AfterWorldBeforeUi,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostFxLensDroplets2d {
    pub enabled: bool,
    pub stage: LensDroplets2dStage,
    pub max_droplets: u32,
    pub spawn_rate: f32,
    pub min_radius_px: f32,
    pub max_radius_px: f32,
    pub min_opacity: f32,
    pub max_opacity: f32,
    pub min_lifetime: f32,
    pub max_lifetime: f32,
    pub dirt_opacity: f32,
    pub darken: f32,
    pub blur_px: f32,
    pub blur_samples: u32,
    pub distortion: f32,
    pub downsample: f32,
    pub streaks_enabled: bool,
    pub streak_chance: f32,
    pub gravity_px_per_sec: f32,
    pub max_streak_length: f32,
    pub wobble: f32,
    pub affects_world: bool,
    pub affects_game_ui: bool,
    pub affects_debug_ui: bool,
    pub strict_certification: bool,
}

impl Default for PostFxLensDroplets2d {
    fn default() -> Self {
        Self {
            enabled: true,
            stage: LensDroplets2dStage::AfterWorldBeforeUi,
            max_droplets: 48,
            spawn_rate: 0.25,
            min_radius_px: 10.0,
            max_radius_px: 42.0,
            min_opacity: 0.18,
            max_opacity: 0.52,
            min_lifetime: 4.0,
            max_lifetime: 12.0,
            dirt_opacity: 0.16,
            darken: 0.08,
            blur_px: 3.0,
            blur_samples: 4,
            distortion: 0.015,
            downsample: 1.0,
            streaks_enabled: true,
            streak_chance: 0.16,
            gravity_px_per_sec: 24.0,
            max_streak_length: 160.0,
            wobble: 0.35,
            affects_world: true,
            affects_game_ui: false,
            affects_debug_ui: false,
            strict_certification: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensDroplets2dCertificationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LensDroplets2dCertificationIssue {
    pub severity: LensDroplets2dCertificationSeverity,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LensDroplets2dCertificationReport {
    pub accepted: bool,
    pub cost_score: f32,
    pub issues: Vec<LensDroplets2dCertificationIssue>,
    pub normalized: PostFxLensDroplets2d,
}

impl PostFxLensDroplets2d {
    pub fn normalized(self) -> Self {
        let defaults = Self::default();

        let mut min_radius =
            finite_or(self.min_radius_px, defaults.min_radius_px).clamp(1.0, 256.0);
        let mut max_radius =
            finite_or(self.max_radius_px, defaults.max_radius_px).clamp(1.0, 256.0);
        if min_radius > max_radius {
            std::mem::swap(&mut min_radius, &mut max_radius);
        }

        let mut min_opacity = finite_or(self.min_opacity, defaults.min_opacity).clamp(0.0, 1.0);
        let mut max_opacity = finite_or(self.max_opacity, defaults.max_opacity).clamp(0.0, 1.0);
        if min_opacity > max_opacity {
            std::mem::swap(&mut min_opacity, &mut max_opacity);
        }

        let mut min_lifetime =
            finite_or(self.min_lifetime, defaults.min_lifetime).clamp(0.1, 120.0);
        let mut max_lifetime =
            finite_or(self.max_lifetime, defaults.max_lifetime).clamp(0.1, 120.0);
        if min_lifetime > max_lifetime {
            std::mem::swap(&mut min_lifetime, &mut max_lifetime);
        }

        Self {
            enabled: self.enabled,
            stage: self.stage,
            max_droplets: self.max_droplets.min(256),
            spawn_rate: finite_or(self.spawn_rate, defaults.spawn_rate).clamp(0.0, 32.0),
            min_radius_px: min_radius,
            max_radius_px: max_radius,
            min_opacity,
            max_opacity,
            min_lifetime,
            max_lifetime,
            dirt_opacity: finite_or(self.dirt_opacity, defaults.dirt_opacity).clamp(0.0, 1.0),
            darken: finite_or(self.darken, defaults.darken).clamp(0.0, 1.0),
            blur_px: finite_or(self.blur_px, defaults.blur_px).clamp(0.0, 32.0),
            blur_samples: self.blur_samples.min(16),
            distortion: finite_or(self.distortion, defaults.distortion).clamp(0.0, 0.1),
            downsample: finite_or(self.downsample, defaults.downsample).clamp(0.25, 1.0),
            streaks_enabled: self.streaks_enabled,
            streak_chance: finite_or(self.streak_chance, defaults.streak_chance).clamp(0.0, 1.0),
            gravity_px_per_sec: finite_or(self.gravity_px_per_sec, defaults.gravity_px_per_sec)
                .clamp(0.0, 512.0),
            max_streak_length: finite_or(self.max_streak_length, defaults.max_streak_length)
                .clamp(0.0, 1024.0),
            wobble: finite_or(self.wobble, defaults.wobble).clamp(0.0, 4.0),
            affects_world: self.affects_world,
            affects_game_ui: self.affects_game_ui,
            affects_debug_ui: self.affects_debug_ui,
            strict_certification: self.strict_certification,
        }
    }

    pub fn is_active(&self) -> bool {
        self.enabled
            && self.affects_world
            && (self.max_droplets > 0 || self.dirt_opacity > 0.0 || self.darken > 0.0)
    }

    pub fn certify(self) -> LensDroplets2dCertificationReport {
        let normalized = self.normalized();
        let mut issues = Vec::new();

        if normalized.affects_debug_ui {
            issues.push(LensDroplets2dCertificationIssue {
                severity: LensDroplets2dCertificationSeverity::Error,
                code: "lens_droplets_debug_ui_forbidden",
                message: "LensDroplets2D must not affect debug UI in the MVP renderer.".to_owned(),
            });
        }

        if normalized.max_droplets > 96 {
            issues.push(LensDroplets2dCertificationIssue {
                severity: LensDroplets2dCertificationSeverity::Warning,
                code: "lens_droplets_high_droplet_count",
                message: format!(
                    "max_droplets={} exceeds recommended budget 96.",
                    normalized.max_droplets
                ),
            });
        }

        if normalized.blur_samples > 8 {
            issues.push(LensDroplets2dCertificationIssue {
                severity: LensDroplets2dCertificationSeverity::Warning,
                code: "lens_droplets_high_blur_samples",
                message: format!(
                    "blur_samples={} exceeds recommended budget 8.",
                    normalized.blur_samples
                ),
            });
        }

        if normalized.blur_px > 12.0 {
            issues.push(LensDroplets2dCertificationIssue {
                severity: LensDroplets2dCertificationSeverity::Warning,
                code: "lens_droplets_high_blur_radius",
                message: format!(
                    "blur_px={} exceeds recommended budget 12px.",
                    normalized.blur_px
                ),
            });
        }

        let blur_factor = normalized.blur_px.max(1.0) / 3.0;
        let downsample_factor = 1.0 / normalized.downsample.max(0.25);
        let cost_score = normalized.max_droplets as f32
            * normalized.blur_samples.max(1) as f32
            * blur_factor
            * downsample_factor;

        if cost_score > 1536.0 {
            issues.push(LensDroplets2dCertificationIssue {
                severity: if normalized.strict_certification {
                    LensDroplets2dCertificationSeverity::Error
                } else {
                    LensDroplets2dCertificationSeverity::Warning
                },
                code: "lens_droplets_cost_budget_exceeded",
                message: format!(
                    "estimated LensDroplets2D cost score {cost_score:.1} exceeds high budget."
                ),
            });
        }

        let accepted = !issues
            .iter()
            .any(|issue| issue.severity == LensDroplets2dCertificationSeverity::Error);

        LensDroplets2dCertificationReport {
            accepted,
            cost_score,
            issues,
            normalized,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostFxBlur2d {
    pub radius: f32,
    pub downsample: f32,
    pub intensity: f32,
}

impl Default for PostFxBlur2d {
    fn default() -> Self {
        Self {
            radius: 12.0,
            downsample: 0.5,
            intensity: 1.0,
        }
    }
}

impl PostFxBlur2d {
    pub fn normalized(self) -> Self {
        let defaults = Self::default();
        Self {
            radius: finite_or(self.radius, defaults.radius).clamp(0.0, 128.0),
            downsample: finite_or(self.downsample, defaults.downsample).clamp(0.125, 1.0),
            intensity: finite_or(self.intensity, defaults.intensity).clamp(0.0, 4.0),
        }
    }

    pub fn is_active(&self) -> bool {
        self.radius > 0.0 && self.intensity > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostFxEmbossEdges2d {
    pub mode: PostFxEmbossMode2d,
    pub intensity: f32,
    pub edge_strength: f32,
    pub sample_offset_px: f32,
    pub luma_threshold: f32,
    pub luma_gamma: f32,
    pub specular_radius_px: f32,
    pub distance_falloff: f32,
    pub tint: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostFxEmbossMode2d {
    PrebakedImage,
    LightAwareRuntime,
}

impl Default for PostFxEmbossEdges2d {
    fn default() -> Self {
        Self {
            mode: PostFxEmbossMode2d::PrebakedImage,
            intensity: 0.35,
            edge_strength: 1.25,
            sample_offset_px: 1.0,
            luma_threshold: 0.22,
            luma_gamma: 2.2,
            specular_radius_px: 6.0,
            distance_falloff: 0.18,
            tint: [1.0, 1.0, 1.0],
        }
    }
}

impl PostFxEmbossEdges2d {
    pub fn normalized(self) -> Self {
        let defaults = Self::default();
        Self {
            mode: self.mode,
            intensity: finite_or(self.intensity, defaults.intensity).clamp(0.0, 2.0),
            edge_strength: finite_or(self.edge_strength, defaults.edge_strength).clamp(0.0, 4.0),
            sample_offset_px: finite_or(self.sample_offset_px, defaults.sample_offset_px)
                .clamp(1.0, 4.0),
            luma_threshold: finite_or(self.luma_threshold, defaults.luma_threshold).clamp(0.0, 1.0),
            luma_gamma: finite_or(self.luma_gamma, defaults.luma_gamma).clamp(0.5, 4.0),
            specular_radius_px: finite_or(self.specular_radius_px, defaults.specular_radius_px)
                .clamp(1.0, 24.0),
            distance_falloff: finite_or(self.distance_falloff, defaults.distance_falloff)
                .clamp(0.01, 2.0),
            tint: [
                finite_or(self.tint[0], defaults.tint[0]).clamp(0.0, 1.0),
                finite_or(self.tint[1], defaults.tint[1]).clamp(0.0, 1.0),
                finite_or(self.tint[2], defaults.tint[2]).clamp(0.0, 1.0),
            ],
        }
    }

    pub fn is_active(&self) -> bool {
        self.intensity > 0.0 && self.edge_strength > 0.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostFx2dCacheKey {
    pub source_id: String,
    pub effect_kind: &'static str,
    pub radius_milli: u32,
    pub downsample_milli: u32,
    pub intensity_milli: u32,
}

impl PostFx2dCacheKey {
    pub fn blur(source_id: impl Into<String>, blur: PostFxBlur2d) -> Self {
        let blur = blur.normalized();
        Self {
            source_id: source_id.into(),
            effect_kind: "blur",
            radius_milli: quantize_milli(blur.radius),
            downsample_milli: quantize_milli(blur.downsample),
            intensity_milli: quantize_milli(blur.intensity),
        }
    }

    pub fn embossed_edges(source_id: impl Into<String>, emboss: PostFxEmbossEdges2d) -> Self {
        let emboss = emboss.normalized();
        Self {
            source_id: source_id.into(),
            effect_kind: "embossed_edges",
            radius_milli: quantize_milli(emboss.sample_offset_px + emboss.specular_radius_px),
            downsample_milli: quantize_milli(emboss.edge_strength + emboss.distance_falloff),
            intensity_milli: quantize_milli(emboss.intensity),
        }
    }
}

pub fn post_fx_stack_from_flat_metadata(
    metadata: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<PostFx2dStack> {
    let mut effects = Vec::new();

    if metadata.contains_key(&format!("{prefix}.kind")) {
        if let Some(effect) = post_fx_from_flat_metadata(metadata, prefix) {
            effects.push(effect);
        }
    }

    let effect_count = infer_indexed_count(metadata, &format!("{prefix}.effects"));
    for index in 0..effect_count {
        if let Some(effect) =
            post_fx_from_flat_metadata(metadata, &format!("{prefix}.effects.{index}"))
        {
            effects.push(effect);
        }
    }

    if effects.is_empty() {
        None
    } else {
        Some(PostFx2dStack { effects }.normalized())
    }
}

pub fn post_fx_from_flat_metadata(
    metadata: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<PostFx2d> {
    let kind = metadata_string(metadata, &format!("{prefix}.kind"))?;
    match kind.trim().to_ascii_lowercase().as_str() {
        "blur" | "gaussian_blur" | "lens_blur" => {
            let defaults = PostFxBlur2d::default();
            Some(PostFx2d::Blur(
                PostFxBlur2d {
                    radius: metadata_f32(metadata, &format!("{prefix}.radius"))
                        .unwrap_or(defaults.radius),
                    downsample: metadata_f32(metadata, &format!("{prefix}.downsample"))
                        .unwrap_or(defaults.downsample),
                    intensity: metadata_f32(metadata, &format!("{prefix}.intensity"))
                        .unwrap_or(defaults.intensity),
                }
                .normalized(),
            ))
        }
        "embossed_edges" | "emboss_edges" | "emboss" => {
            let defaults = PostFxEmbossEdges2d::default();
            Some(PostFx2d::EmbossEdges(
                PostFxEmbossEdges2d {
                    mode: metadata_string(metadata, &format!("{prefix}.mode"))
                        .as_deref()
                        .map(parse_emboss_mode)
                        .unwrap_or(defaults.mode),
                    intensity: metadata_f32(metadata, &format!("{prefix}.intensity"))
                        .unwrap_or(defaults.intensity),
                    edge_strength: metadata_f32(metadata, &format!("{prefix}.edge_strength"))
                        .unwrap_or(defaults.edge_strength),
                    sample_offset_px: metadata_f32(metadata, &format!("{prefix}.sample_offset_px"))
                        .unwrap_or(defaults.sample_offset_px),
                    luma_threshold: metadata_f32(metadata, &format!("{prefix}.luma_threshold"))
                        .unwrap_or(defaults.luma_threshold),
                    luma_gamma: metadata_f32(metadata, &format!("{prefix}.luma_gamma"))
                        .unwrap_or(defaults.luma_gamma),
                    specular_radius_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.specular_radius_px"),
                    )
                    .unwrap_or(defaults.specular_radius_px),
                    distance_falloff: metadata_f32(metadata, &format!("{prefix}.distance_falloff"))
                        .unwrap_or(defaults.distance_falloff),
                    tint: metadata_string(metadata, &format!("{prefix}.tint"))
                        .and_then(parse_color_triplet)
                        .unwrap_or(defaults.tint),
                }
                .normalized(),
            ))
        }
        "dirty_bloom" | "dirtybloom" => {
            let defaults = DirtyBloom2d::default();
            Some(PostFx2d::DirtyBloom(
                DirtyBloom2d {
                    threshold: metadata_f32(metadata, &format!("{prefix}.threshold"))
                        .unwrap_or(defaults.threshold),
                    strength: metadata_f32(metadata, &format!("{prefix}.strength"))
                        .unwrap_or(defaults.strength),
                    small_radius_px: metadata_f32(metadata, &format!("{prefix}.small_radius_px"))
                        .unwrap_or(defaults.small_radius_px),
                    medium_radius_px: metadata_f32(metadata, &format!("{prefix}.medium_radius_px"))
                        .unwrap_or(defaults.medium_radius_px),
                    large_radius_px: metadata_f32(metadata, &format!("{prefix}.large_radius_px"))
                        .unwrap_or(defaults.large_radius_px),
                    dirty_noise: metadata_f32(metadata, &format!("{prefix}.dirty_noise"))
                        .unwrap_or(defaults.dirty_noise),
                    halation_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.halation_strength"),
                    )
                    .unwrap_or(defaults.halation_strength),
                    reflection_smear_x_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.reflection_smear_x_px"),
                    )
                    .unwrap_or(defaults.reflection_smear_x_px),
                    reflection_smear_y_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.reflection_smear_y_px"),
                    )
                    .unwrap_or(defaults.reflection_smear_y_px),
                    seed: metadata_u32(metadata, &format!("{prefix}.seed"))
                        .unwrap_or(defaults.seed),
                }
                .normalized(),
            ))
        }
        "color_quantize" | "quantize" | "palette_dither" | "gif_dither" => {
            let defaults = ColorQuantize2d::default();
            Some(PostFx2d::ColorQuantize(
                ColorQuantize2d {
                    palette_size: metadata_u32(metadata, &format!("{prefix}.palette_size"))
                        .or_else(|| metadata_u32(metadata, &format!("{prefix}.colors")))
                        .unwrap_or(defaults.palette_size),
                    dither_strength: metadata_f32(metadata, &format!("{prefix}.dither_strength"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.dither")))
                        .unwrap_or(defaults.dither_strength),
                    opacity: metadata_f32(metadata, &format!("{prefix}.opacity"))
                        .unwrap_or(defaults.opacity),
                    luma_preserve: metadata_f32(metadata, &format!("{prefix}.luma_preserve"))
                        .unwrap_or(defaults.luma_preserve),
                    highlight_bias: metadata_f32(metadata, &format!("{prefix}.highlight_bias"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.light_bias")))
                        .unwrap_or(defaults.highlight_bias),
                    gamma: metadata_f32(metadata, &format!("{prefix}.gamma"))
                        .unwrap_or(defaults.gamma),
                    seed: metadata_u32(metadata, &format!("{prefix}.seed"))
                        .unwrap_or(defaults.seed),
                }
                .normalized(),
            ))
        }
        "downscale" | "pixelate" | "pixel_scale" => {
            let defaults = Downscale2d::default();
            Some(PostFx2d::Downscale(
                Downscale2d {
                    factor: metadata_f32(metadata, &format!("{prefix}.factor"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.scale")))
                        .unwrap_or(defaults.factor),
                    opacity: metadata_f32(metadata, &format!("{prefix}.opacity"))
                        .unwrap_or(defaults.opacity),
                }
                .normalized(),
            ))
        }
        "shutter_blur" | "shutter" | "temporal_blur" | "motion_blur_24fps" => {
            let defaults = ShutterBlur2d::default();
            Some(PostFx2d::ShutterBlur(
                ShutterBlur2d {
                    fps: metadata_f32(metadata, &format!("{prefix}.fps"))
                        .unwrap_or(defaults.fps),
                    shutter_angle: metadata_f32(metadata, &format!("{prefix}.shutter_angle"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.angle")))
                        .unwrap_or(defaults.shutter_angle),
                    opacity: metadata_f32(metadata, &format!("{prefix}.opacity"))
                        .unwrap_or(defaults.opacity),
                    edge_rejection: metadata_f32(metadata, &format!("{prefix}.edge_rejection"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.edge_reject")))
                        .unwrap_or(defaults.edge_rejection),
                    luma_threshold: metadata_f32(metadata, &format!("{prefix}.luma_threshold"))
                        .unwrap_or(defaults.luma_threshold),
                    frame_hold: metadata_bool(metadata, &format!("{prefix}.frame_hold"))
                        .unwrap_or(defaults.frame_hold),
                }
                .normalized(),
            ))
        }
        "crt" | "crt_screen" => {
            let defaults = Crt2d::default();
            Some(PostFx2d::Crt(
                Crt2d {
                    scanline_opacity: metadata_f32(metadata, &format!("{prefix}.scanline_opacity"))
                        .unwrap_or(defaults.scanline_opacity),
                    scanline_frequency_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.scanline_frequency_px"),
                    )
                    .unwrap_or(defaults.scanline_frequency_px),
                    rgb_split_px: metadata_f32(metadata, &format!("{prefix}.rgb_split_px"))
                        .unwrap_or(defaults.rgb_split_px),
                    curvature: metadata_f32(metadata, &format!("{prefix}.curvature"))
                        .unwrap_or(defaults.curvature),
                    vignette: metadata_f32(metadata, &format!("{prefix}.vignette"))
                        .unwrap_or(defaults.vignette),
                    phosphor_mask: metadata_f32(metadata, &format!("{prefix}.phosphor_mask"))
                        .unwrap_or(defaults.phosphor_mask),
                    brightness_compensation: metadata_f32(
                        metadata,
                        &format!("{prefix}.brightness_compensation"),
                    )
                    .unwrap_or(defaults.brightness_compensation),
                }
                .normalized(),
            ))
        }
        "film_noise" | "film_grain" | "noise_overlay" => {
            let defaults = FilmNoise2d::default();
            Some(PostFx2d::FilmNoise(
                FilmNoise2d {
                    iso: metadata_f32(metadata, &format!("{prefix}.iso")).unwrap_or(defaults.iso),
                    grain_size: metadata_f32(metadata, &format!("{prefix}.grain_size"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.scale")))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.grain_scale")))
                        .unwrap_or(defaults.grain_size),
                    chroma_noise: metadata_f32(metadata, &format!("{prefix}.chroma_noise"))
                        .unwrap_or(defaults.chroma_noise),
                    color_shift: metadata_f32(metadata, &format!("{prefix}.color_shift"))
                        .unwrap_or(defaults.color_shift),
                    contrast: metadata_f32(metadata, &format!("{prefix}.contrast"))
                        .unwrap_or(defaults.contrast),
                    saturation: metadata_f32(metadata, &format!("{prefix}.saturation"))
                        .unwrap_or(defaults.saturation),
                    flicker: metadata_f32(metadata, &format!("{prefix}.flicker"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.speed")))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.flicker_speed")))
                        .unwrap_or(defaults.flicker),
                    vignette: metadata_f32(metadata, &format!("{prefix}.vignette"))
                        .unwrap_or(defaults.vignette),
                    opacity: metadata_f32(metadata, &format!("{prefix}.opacity"))
                        .unwrap_or(defaults.opacity),
                    seed: metadata_u32(metadata, &format!("{prefix}.seed"))
                        .unwrap_or(defaults.seed),
                }
                .normalized(),
            ))
        }
        "lens_droplets" | "lens_drops" | "droplets" => {
            let defaults = PostFxLensDroplets2d::default();
            Some(PostFx2d::LensDroplets(
                PostFxLensDroplets2d {
                    enabled: metadata_bool(metadata, &format!("{prefix}.enabled"))
                        .unwrap_or(defaults.enabled),
                    stage: LensDroplets2dStage::AfterWorldBeforeUi,
                    max_droplets: metadata_u32(metadata, &format!("{prefix}.droplets.max"))
                        .unwrap_or(defaults.max_droplets),
                    spawn_rate: metadata_f32(metadata, &format!("{prefix}.droplets.spawn_rate"))
                        .unwrap_or(defaults.spawn_rate),
                    min_radius_px: metadata_range_min(
                        metadata,
                        &format!("{prefix}.droplets.radius_range"),
                    )
                    .unwrap_or(defaults.min_radius_px),
                    max_radius_px: metadata_range_max(
                        metadata,
                        &format!("{prefix}.droplets.radius_range"),
                    )
                    .unwrap_or(defaults.max_radius_px),
                    min_opacity: metadata_range_min(
                        metadata,
                        &format!("{prefix}.droplets.opacity_range"),
                    )
                    .unwrap_or(defaults.min_opacity),
                    max_opacity: metadata_range_max(
                        metadata,
                        &format!("{prefix}.droplets.opacity_range"),
                    )
                    .unwrap_or(defaults.max_opacity),
                    min_lifetime: metadata_range_min(
                        metadata,
                        &format!("{prefix}.droplets.lifetime_range"),
                    )
                    .unwrap_or(defaults.min_lifetime),
                    max_lifetime: metadata_range_max(
                        metadata,
                        &format!("{prefix}.droplets.lifetime_range"),
                    )
                    .unwrap_or(defaults.max_lifetime),
                    dirt_opacity: metadata_f32(metadata, &format!("{prefix}.surface.dirt_opacity"))
                        .unwrap_or(defaults.dirt_opacity),
                    darken: metadata_f32(metadata, &format!("{prefix}.surface.darken"))
                        .unwrap_or(defaults.darken),
                    blur_px: metadata_f32(metadata, &format!("{prefix}.surface.blur_px"))
                        .unwrap_or(defaults.blur_px),
                    blur_samples: metadata_u32(metadata, &format!("{prefix}.surface.blur_samples"))
                        .unwrap_or(defaults.blur_samples),
                    distortion: metadata_f32(metadata, &format!("{prefix}.surface.distortion"))
                        .unwrap_or(defaults.distortion),
                    downsample: metadata_f32(metadata, &format!("{prefix}.surface.downsample"))
                        .unwrap_or(defaults.downsample),
                    streaks_enabled: metadata_bool(metadata, &format!("{prefix}.streaks.enabled"))
                        .unwrap_or(defaults.streaks_enabled),
                    streak_chance: metadata_f32(metadata, &format!("{prefix}.streaks.chance"))
                        .unwrap_or(defaults.streak_chance),
                    gravity_px_per_sec: metadata_f32(
                        metadata,
                        &format!("{prefix}.streaks.gravity_px_per_sec"),
                    )
                    .unwrap_or(defaults.gravity_px_per_sec),
                    max_streak_length: metadata_f32(
                        metadata,
                        &format!("{prefix}.streaks.max_length"),
                    )
                    .unwrap_or(defaults.max_streak_length),
                    wobble: metadata_f32(metadata, &format!("{prefix}.streaks.wobble"))
                        .unwrap_or(defaults.wobble),
                    affects_world: metadata_bool(metadata, &format!("{prefix}.affects.world"))
                        .unwrap_or(defaults.affects_world),
                    affects_game_ui: metadata_bool(metadata, &format!("{prefix}.affects.game_ui"))
                        .unwrap_or(defaults.affects_game_ui),
                    affects_debug_ui: metadata_bool(
                        metadata,
                        &format!("{prefix}.affects.debug_ui"),
                    )
                    .unwrap_or(defaults.affects_debug_ui),
                    strict_certification: metadata_bool(
                        metadata,
                        &format!("{prefix}.certification.strict"),
                    )
                    .unwrap_or(defaults.strict_certification),
                }
                .normalized(),
            ))
        }
        "rain_glass" | "rainglass" | "rain_drops" => {
            let defaults = RainGlass2d::default();
            Some(PostFx2d::RainGlass(
                RainGlass2d {
                    enabled: metadata_bool(metadata, &format!("{prefix}.enabled"))
                        .unwrap_or(defaults.enabled),
                    spawn_rate: metadata_f32(metadata, &format!("{prefix}.spawn_rate"))
                        .unwrap_or(defaults.spawn_rate),
                    spawn_limit: metadata_u32(metadata, &format!("{prefix}.spawn_limit"))
                        .unwrap_or(defaults.spawn_limit),
                    min_radius_px: metadata_range_min(metadata, &format!("{prefix}.spawn_size"))
                        .unwrap_or(defaults.min_radius_px),
                    max_radius_px: metadata_range_max(metadata, &format!("{prefix}.spawn_size"))
                        .unwrap_or(defaults.max_radius_px),
                    refract_base: metadata_f32(metadata, &format!("{prefix}.refract_base"))
                        .unwrap_or(defaults.refract_base),
                    refract_scale: metadata_f32(metadata, &format!("{prefix}.refract_scale"))
                        .unwrap_or(defaults.refract_scale),
                    opacity: metadata_f32(metadata, &format!("{prefix}.opacity"))
                        .unwrap_or(defaults.opacity),
                    light_bump: metadata_f32(metadata, &format!("{prefix}.light_bump"))
                        .unwrap_or(defaults.light_bump),
                    seed: metadata_u32(metadata, &format!("{prefix}.seed"))
                        .unwrap_or(defaults.seed),
                    ..defaults
                }
                .normalized(),
            ))
        }
        "wet_reflections" | "wet_reflection" | "wet_surface" => {
            let defaults = PostFxWetReflections2d::default();
            Some(PostFx2d::WetReflections(
                PostFxWetReflections2d {
                    enabled: metadata_bool(metadata, &format!("{prefix}.enabled"))
                        .unwrap_or(defaults.enabled),
                    reflection_mask: metadata_string(metadata, &format!("{prefix}.mask"))
                        .or_else(|| metadata_string(metadata, &format!("{prefix}.reflection_mask")))
                        .unwrap_or_default(),
                    reflection_mask_invert: metadata_bool(
                        metadata,
                        &format!("{prefix}.mask_invert"),
                    )
                    .unwrap_or(defaults.reflection_mask_invert),
                    edge_map: metadata_string(metadata, &format!("{prefix}.edge_map")),
                    reflection_color: metadata_string(
                        metadata,
                        &format!("{prefix}.reflection_color"),
                    ),
                    noise_normal: metadata_string(metadata, &format!("{prefix}.noise_normal")),
                    blur_px: metadata_f32(metadata, &format!("{prefix}.surface.blur_px"))
                        .unwrap_or(defaults.blur_px),
                    distortion_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.surface.distortion_px"),
                    )
                    .unwrap_or(defaults.distortion_px),
                    shimmer_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.surface.shimmer_strength"),
                    )
                    .unwrap_or(defaults.shimmer_strength),
                    ripple_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.surface.ripple_strength"),
                    )
                    .unwrap_or(defaults.ripple_strength),
                    wet_darken: metadata_f32(metadata, &format!("{prefix}.surface.wet_darken"))
                        .unwrap_or(defaults.wet_darken),
                    specular_boost: metadata_f32(
                        metadata,
                        &format!("{prefix}.surface.specular_boost"),
                    )
                    .unwrap_or(defaults.specular_boost),
                    edge_power: metadata_f32(metadata, &format!("{prefix}.surface.edge_power"))
                        .unwrap_or(defaults.edge_power),
                    light_reflection_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.surface.light_reflection_strength"),
                    )
                    .unwrap_or(defaults.light_reflection_strength),
                    foreground_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.perspective.foreground_strength"),
                    )
                    .unwrap_or(defaults.foreground_strength),
                    background_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.perspective.background_strength"),
                    )
                    .unwrap_or(defaults.background_strength),
                    horizon_y: metadata_f32(metadata, &format!("{prefix}.perspective.horizon_y"))
                        .unwrap_or(defaults.horizon_y),
                    noise_scale: metadata_f32(metadata, &format!("{prefix}.animation.noise_scale"))
                        .unwrap_or(defaults.noise_scale),
                    noise_speed: metadata_f32(metadata, &format!("{prefix}.animation.noise_speed"))
                        .unwrap_or(defaults.noise_speed),
                    ripple_speed: metadata_f32(
                        metadata,
                        &format!("{prefix}.animation.ripple_speed"),
                    )
                    .unwrap_or(defaults.ripple_speed),
                    debug_view: defaults.debug_view,
                }
                .normalized(),
            ))
        }
        _ => None,
    }
}

fn infer_indexed_count(metadata: &BTreeMap<String, String>, prefix: &str) -> usize {
    let mut max_index = None;

    for key in metadata.keys() {
        let Some(rest) = key.strip_prefix(&format!("{prefix}.")) else {
            continue;
        };
        let Some((raw_index, _field)) = rest.split_once('.') else {
            continue;
        };
        let Ok(index) = raw_index.parse::<usize>() else {
            continue;
        };
        max_index = Some(max_index.map_or(index, |current: usize| current.max(index)));
    }

    max_index.map_or(0, |index| index + 1)
}

fn metadata_string(metadata: &BTreeMap<String, String>, key: &str) -> Option<String> {
    metadata
        .get(key)
        .cloned()
        .filter(|value| !value.trim().is_empty())
}

fn metadata_f32(metadata: &BTreeMap<String, String>, key: &str) -> Option<f32> {
    metadata.get(key)?.parse::<f32>().ok()
}

fn metadata_bool(metadata: &BTreeMap<String, String>, key: &str) -> Option<bool> {
    metadata.get(key)?.parse::<bool>().ok()
}

fn metadata_u32(metadata: &BTreeMap<String, String>, key: &str) -> Option<u32> {
    metadata.get(key)?.parse::<u32>().ok()
}

fn metadata_range_min(metadata: &BTreeMap<String, String>, key: &str) -> Option<f32> {
    parse_range(metadata.get(key)?).map(|range| range.0)
}

fn metadata_range_max(metadata: &BTreeMap<String, String>, key: &str) -> Option<f32> {
    parse_range(metadata.get(key)?).map(|range| range.1)
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

fn quantize_milli(value: f32) -> u32 {
    (finite_or(value, 0.0).max(0.0) * 1000.0).round() as u32
}

fn parse_emboss_mode(value: &str) -> PostFxEmbossMode2d {
    match value.trim().to_ascii_lowercase().as_str() {
        "light_aware_runtime" | "runtime" | "light_aware" => PostFxEmbossMode2d::LightAwareRuntime,
        _ => PostFxEmbossMode2d::PrebakedImage,
    }
}

fn parse_color_triplet(value: String) -> Option<[f32; 3]> {
    let mut parts = value.split(',').map(str::trim);
    let r = parts.next()?.parse::<f32>().ok()?;
    let g = parts.next()?.parse::<f32>().ok()?;
    let b = parts.next()?.parse::<f32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some([r, g, b])
}

fn parse_range(value: &str) -> Option<(f32, f32)> {
    let value = value.trim().trim_start_matches('[').trim_end_matches(']');
    let mut parts = value.split(',').map(str::trim);
    let min = parts.next()?.parse::<f32>().ok()?;
    let max = parts.next()?.parse::<f32>().ok()?;
    Some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_blur_stack_effects() {
        let metadata = BTreeMap::from([
            (
                "layer.post_fx.effects.0.kind".to_owned(),
                "gaussian_blur".to_owned(),
            ),
            (
                "layer.post_fx.effects.0.radius".to_owned(),
                "24.0".to_owned(),
            ),
        ]);

        let stack = post_fx_stack_from_flat_metadata(&metadata, "layer.post_fx")
            .expect("stack should parse");

        assert_eq!(stack.effects.len(), 1);
        assert!(matches!(stack.effects[0], PostFx2d::Blur(_)));
    }

    #[test]
    fn parses_emboss_stack_effect() {
        let metadata = BTreeMap::from([
            ("layer.post_fx.kind".to_owned(), "embossed_edges".to_owned()),
            ("layer.post_fx.edge_strength".to_owned(), "1.6".to_owned()),
        ]);
        let stack = post_fx_stack_from_flat_metadata(&metadata, "layer.post_fx")
            .expect("stack should parse");
        assert_eq!(stack.effects.len(), 1);
        assert!(matches!(stack.effects[0], PostFx2d::EmbossEdges(_)));
    }

    #[test]
    fn parses_color_quantize_from_flat_metadata() {
        let metadata = BTreeMap::from([
            ("fx.kind".to_owned(), "gif_dither".to_owned()),
            ("fx.colors".to_owned(), "32".to_owned()),
            ("fx.dither".to_owned(), "0.5".to_owned()),
            ("fx.opacity".to_owned(), "0.8".to_owned()),
            ("fx.highlight_bias".to_owned(), "0.45".to_owned()),
        ]);

        let effect = post_fx_from_flat_metadata(&metadata, "fx").expect("effect should parse");
        let PostFx2d::ColorQuantize(effect) = effect else {
            panic!("expected color quantize effect");
        };
        assert_eq!(effect.palette_size, 32);
        assert_eq!(effect.dither_strength, 0.5);
        assert_eq!(effect.opacity, 0.8);
        assert_eq!(effect.highlight_bias, 0.45);
    }

    #[test]
    fn color_quantize_normalized_clamps_values() {
        let effect = ColorQuantize2d {
            palette_size: 999,
            dither_strength: -2.0,
            opacity: 4.0,
            luma_preserve: 9.0,
            highlight_bias: 9.0,
            gamma: 9.0,
            seed: 7,
        }
        .normalized();

        assert_eq!(effect.palette_size, 256);
        assert_eq!(effect.dither_strength, 0.0);
        assert_eq!(effect.opacity, 1.0);
        assert_eq!(effect.luma_preserve, 1.0);
        assert_eq!(effect.highlight_bias, 1.0);
        assert_eq!(effect.gamma, 3.0);
    }

    #[test]
    fn parses_downscale_from_flat_metadata() {
        let metadata = BTreeMap::from([
            ("fx.kind".to_owned(), "downscale".to_owned()),
            ("fx.factor".to_owned(), "2".to_owned()),
            ("fx.opacity".to_owned(), "0.75".to_owned()),
        ]);

        let effect = post_fx_from_flat_metadata(&metadata, "fx").expect("effect should parse");
        let PostFx2d::Downscale(effect) = effect else {
            panic!("expected downscale effect");
        };
        assert_eq!(effect.factor, 2.0);
        assert_eq!(effect.opacity, 0.75);
    }

    #[test]
    fn downscale_normalized_clamps_values() {
        let effect = Downscale2d {
            factor: 999.0,
            opacity: 9.0,
        }
        .normalized();

        assert_eq!(effect.factor, 16.0);
        assert_eq!(effect.opacity, 1.0);
    }

    #[test]
    fn normalizes_rain_glass_extended_parameters() {
        let effect = RainGlass2d {
            spawn_limit: 99_999,
            refract_scale: 99.0,
            trail_taper: 99.0,
            micro_droplets_per_second: 99_999.0,
            specular_shininess: 99_999.0,
            distortion_px: 999.0,
            normal_strength: 999.0,
            focus_blur_strength: 999.0,
            body_opacity: 999.0,
            trail_refract_scale: 999.0,
            trail_opacity: 999.0,
            scene_light_response: 999.0,
            rim_strength: 999.0,
            streak_boost: 999.0,
            streak_length: 999.0,
            mist_time: 999.0,
            mist_color_strength: 999.0,
            mist_blur_step: 999,
            background_blur_steps: 999,
            raindrop_eraser_size: [999.0, 999.0],
            ..RainGlass2d::default()
        }
        .normalized();

        assert_eq!(effect.spawn_limit, 3000);
        assert!(effect.refract_scale <= 4.0);
        assert!(effect.trail_taper <= 1.0);
        assert!(effect.micro_droplets_per_second <= 5000.0);
        assert!(effect.specular_shininess <= 1024.0);
        assert!(effect.distortion_px <= 128.0);
        assert!(effect.normal_strength <= 16.0);
        assert!(effect.focus_blur_strength <= 2.0);
        assert!(effect.body_opacity <= 1.0);
        assert!(effect.trail_refract_scale <= 2.0);
        assert!(effect.trail_opacity <= 1.0);
        assert!(effect.scene_light_response <= 5.0);
        assert!(effect.rim_strength <= 5.0);
        assert!(effect.streak_boost <= 2.0);
        assert!(effect.streak_length <= 4.0);
        assert!(effect.mist_time <= 120.0);
        assert!(effect.mist_color_strength <= 1.0);
        assert!(effect.mist_blur_step <= 8);
        assert!(effect.background_blur_steps <= 8);
        assert!(effect.raindrop_eraser_size[0] <= 4.0);
        assert!(effect.raindrop_eraser_size[1] <= 4.0);
    }

    #[test]
    fn normalizes_rain_glass_mist_blur() {
        let effect = RainGlass2d {
            mist_blur_px: 999.0,
            ..RainGlass2d::default()
        }
        .normalized();

        assert!(effect.mist_blur_px <= 32.0);
    }

    #[test]
    fn certifies_default_lens_droplets() {
        let report = PostFxLensDroplets2d::default().certify();
        assert!(report.accepted);
        assert!(report.cost_score > 0.0);
    }

    #[test]
    fn rejects_lens_droplets_affecting_debug_ui() {
        let report = PostFxLensDroplets2d {
            affects_debug_ui: true,
            ..PostFxLensDroplets2d::default()
        }
        .certify();

        assert!(!report.accepted);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "lens_droplets_debug_ui_forbidden")
        );
    }

    #[test]
    fn parses_lens_droplets_from_flat_metadata() {
        let metadata = BTreeMap::from([
            ("fx.kind".to_owned(), "lens_droplets".to_owned()),
            ("fx.droplets.max".to_owned(), "48".to_owned()),
            ("fx.surface.blur_samples".to_owned(), "4".to_owned()),
            ("fx.affects.debug_ui".to_owned(), "false".to_owned()),
        ]);

        let effect = post_fx_from_flat_metadata(&metadata, "fx").expect("effect should parse");
        assert!(matches!(effect, PostFx2d::LensDroplets(_)));
    }

    #[test]
    fn shutter_blur_normalized_clamps_values() {
        let effect = ShutterBlur2d {
            fps: 999.0,
            shutter_angle: 999.0,
            opacity: 999.0,
            edge_rejection: 999.0,
            luma_threshold: 999.0,
            frame_hold: true,
        }
        .normalized();

        assert_eq!(effect.fps, 240.0);
        assert_eq!(effect.shutter_angle, 360.0);
        assert_eq!(effect.opacity, 1.0);
        assert_eq!(effect.edge_rejection, 1.0);
        assert_eq!(effect.luma_threshold, 1.0);
        assert!(effect.frame_hold);
    }

    #[test]
    fn parses_shutter_blur_from_flat_metadata() {
        let metadata = BTreeMap::from([
            ("fx.kind".to_owned(), "shutter_blur".to_owned()),
            ("fx.fps".to_owned(), "24".to_owned()),
            ("fx.shutter_angle".to_owned(), "180".to_owned()),
            ("fx.opacity".to_owned(), "0.7".to_owned()),
            ("fx.frame_hold".to_owned(), "true".to_owned()),
        ]);

        let effect = post_fx_from_flat_metadata(&metadata, "fx").expect("effect should parse");
        let PostFx2d::ShutterBlur(effect) = effect else {
            panic!("expected shutter_blur");
        };
        assert_eq!(effect.fps, 24.0);
        assert_eq!(effect.shutter_angle, 180.0);
        assert!(effect.frame_hold);
    }

    #[test]
    fn wet_reflections_kind_is_stable() {
        assert_eq!(
            PostFx2d::WetReflections(PostFxWetReflections2d::default()).kind(),
            "wet_reflections"
        );
    }

    #[test]
    fn wet_reflections_inactive_without_mask() {
        assert!(!PostFxWetReflections2d::default().is_active());
    }

    #[test]
    fn wet_reflections_normalized_clamps_values() {
        let effect = PostFxWetReflections2d {
            enabled: true,
            reflection_mask: "mask.png".to_owned(),
            reflection_mask_invert: true,
            edge_map: None,
            reflection_color: None,
            noise_normal: None,
            blur_px: 99.0,
            distortion_px: -3.0,
            shimmer_strength: 4.0,
            ripple_strength: 4.0,
            wet_darken: -2.0,
            specular_boost: 9.0,
            edge_power: 0.0,
            light_reflection_strength: 9.0,
            foreground_strength: 9.0,
            background_strength: -4.0,
            horizon_y: 9.0,
            noise_scale: 0.0,
            noise_speed: 9.0,
            ripple_speed: -9.0,
            debug_view: WetReflectionsDebugView::Final,
        }
        .normalized();

        assert_eq!(effect.blur_px, 12.0);
        assert_eq!(effect.distortion_px, 0.0);
        assert_eq!(effect.shimmer_strength, 1.0);
        assert_eq!(effect.ripple_strength, 1.0);
        assert_eq!(effect.wet_darken, 0.0);
        assert_eq!(effect.specular_boost, 4.0);
        assert_eq!(effect.edge_power, 0.25);
        assert_eq!(effect.light_reflection_strength, 4.0);
        assert_eq!(effect.foreground_strength, 4.0);
        assert_eq!(effect.background_strength, 0.0);
        assert_eq!(effect.horizon_y, 1.0);
        assert_eq!(effect.noise_scale, 0.01);
        assert_eq!(effect.noise_speed, 8.0);
        assert_eq!(effect.ripple_speed, -8.0);
    }

    #[test]
    fn parses_wet_reflections_from_flat_metadata() {
        let metadata = BTreeMap::from([
            ("fx.kind".to_owned(), "wet_reflections".to_owned()),
            (
                "fx.reflection_mask".to_owned(),
                "rotten-club/layered-images/neon-alley/reflection_mask.png".to_owned(),
            ),
            (
                "fx.edge_map".to_owned(),
                "rotten-club/layered-images/neon-alley/edge_map_2.png".to_owned(),
            ),
        ]);

        let effect = post_fx_from_flat_metadata(&metadata, "fx").expect("effect should parse");
        assert!(matches!(effect, PostFx2d::WetReflections(_)));
    }
}
