use std::sync::Arc;

use amigo_2d_post_fx::{
    ColorQuantize2d, PostFx2d, PostFx2dService, RainGlass2d, RainGlassDebugView,
    RainGlassRaindropCompose,
};

#[derive(Clone)]
pub struct PostFxApi {
    pub(crate) post_fx: Option<Arc<PostFx2dService>>,
}

impl PostFxApi {
    pub fn count(&mut self) -> rhai::INT {
        self.post_fx
            .as_ref()
            .map(|service| service.scene_effect_count() as rhai::INT)
            .unwrap_or(0)
    }

    pub fn list(&mut self) -> rhai::Array {
        self.post_fx
            .as_ref()
            .map(|service| {
                service
                    .scene_effects()
                    .into_iter()
                    .enumerate()
                    .map(|(index, effect)| item_map(index, effect))
                    .collect::<rhai::Array>()
            })
            .unwrap_or_default()
    }

    pub fn item(&mut self, index: rhai::INT) -> PostFxItemRef {
        PostFxItemRef {
            post_fx: self.post_fx.clone(),
            index: index.max(0) as usize,
        }
    }

    pub fn color_quantize_palette_size(&mut self) -> rhai::INT {
        self.post_fx
            .as_ref()
            .and_then(|service| {
                service
                    .scene_effects()
                    .into_iter()
                    .find_map(|effect| match effect {
                        PostFx2d::ColorQuantize(effect) => Some(effect.palette_size as rhai::INT),
                        _ => None,
                    })
            })
            .unwrap_or(0)
    }

    pub fn set_color_quantize_palette_size(&mut self, value: rhai::INT) -> rhai::INT {
        self.update_color_quantize(|effect| {
            effect.palette_size = value.clamp(2, 256) as u32;
        })
    }

    pub fn adjust_color_quantize_palette_size(&mut self, delta: rhai::INT) -> rhai::INT {
        self.update_color_quantize(|effect| {
            let next = effect.palette_size as rhai::INT + delta;
            effect.palette_size = next.clamp(2, 256) as u32;
        })
    }

    pub fn set_rain_glass(&mut self, updates: &str) -> bool {
        let mut applied = false;
        for token in updates.split_whitespace() {
            let Some((field, value)) = token.split_once('=') else {
                continue;
            };

            let ok = match field {
                "preset" => self.apply_rain_glass_preset(value),
                "debug" | "debug_view" => self.set_rain_glass_debug(value),
                "compose" | "raindrop_compose" => self.set_rain_glass_compose(value),
                "enabled"
                | "trails"
                | "trails_enabled"
                | "micro"
                | "micro_enabled"
                | "micro_droplets_enabled"
                | "mist"
                | "mist_enabled"
                | "reference"
                | "reference_mode" => parse_bool(value)
                    .map(|value| self.set_rain_glass_bool(field, value))
                    .unwrap_or(false),
                "spawn_limit"
                | "seed"
                | "blur_steps"
                | "background_blur_steps"
                | "mist_blur_step" => value
                    .parse::<rhai::INT>()
                    .map(|value| self.set_rain_glass_int(field, value))
                    .unwrap_or(false),
                _ => value
                    .parse::<rhai::FLOAT>()
                    .map(|value| self.set_rain_glass_float(field, value))
                    .unwrap_or(false),
            };
            applied |= ok;
        }
        applied
    }

    pub fn set_rain_glass_bool(&mut self, field: &str, value: bool) -> bool {
        if !matches!(
            field,
            "enabled"
                | "trails"
                | "trails_enabled"
                | "micro"
                | "micro_enabled"
                | "micro_droplets_enabled"
                | "mist"
                | "mist_enabled"
                | "receives_scene_light"
                | "scene_lighting"
                | "light_react"
                | "reference"
                | "reference_mode"
        ) {
            return false;
        }

        self.update_rain_glass(|rain| match field {
            "enabled" => rain.enabled = value,
            "trails" | "trails_enabled" => rain.trails_enabled = value,
            "micro" | "micro_enabled" | "micro_droplets_enabled" => {
                rain.micro_droplets_enabled = value
            }
            "mist" | "mist_enabled" => rain.mist_enabled = value,
            "receives_scene_light" | "scene_lighting" | "light_react" => {
                rain.receives_scene_light = value
            }
            "reference" | "reference_mode" => rain.reference_mode = value,
            _ => {}
        })
    }

    pub fn set_rain_glass_int(&mut self, field: &str, value: rhai::INT) -> bool {
        if !matches!(
            field,
            "spawn_limit" | "seed" | "blur_steps" | "background_blur_steps" | "mist_blur_step"
        ) {
            return false;
        }

        self.update_rain_glass(|rain| match field {
            "spawn_limit" => rain.spawn_limit = value.max(0) as u32,
            "seed" => rain.seed = value.max(0) as u32,
            "blur_steps" | "background_blur_steps" => {
                rain.background_blur_steps = value.max(0) as u32
            }
            "mist_blur_step" => rain.mist_blur_step = value.max(0) as u32,
            _ => {}
        })
    }

    pub fn set_rain_glass_float(&mut self, field: &str, value: rhai::FLOAT) -> bool {
        if !is_rain_glass_float_field(field) {
            return false;
        }

        let value = value as f32;
        self.update_rain_glass(|rain| match field {
            "spawn_rate" => rain.spawn_rate = value,
            "radius_min" | "min_radius_px" => rain.min_radius_px = value,
            "radius_max" | "max_radius_px" => rain.max_radius_px = value,
            "gravity" | "gravity_px_per_sec2" => rain.gravity_px_per_sec2 = value,
            "slip_rate" => rain.slip_rate = value,
            "motion_min" | "motion_interval_min" => rain.motion_interval_min = value,
            "motion_max" | "motion_interval_max" => rain.motion_interval_max = value,
            "x_shift_min" => rain.x_shift_min = value,
            "x_shift_max" => rain.x_shift_max = value,
            "collider" | "collider_scale" => rain.collider_scale = value,
            "initial_spread" | "impact_spread" => rain.initial_spread = value,
            "shrink_rate" | "spread_shrink" => rain.shrink_rate = value,
            "velocity_stretch" | "velocity_spread" => rain.velocity_spread = value,
            "evaporate" => rain.evaporate = value,
            "refract_base" => rain.refract_base = value,
            "refract_scale" => rain.refract_scale = value,
            "opacity" => rain.opacity = value,
            "blur" | "background_blur_px" => rain.background_blur_px = value,
            "chroma" | "chromatic_aberration" => rain.chromatic_aberration = value,
            "distortion" | "distortion_px" => rain.distortion_px = value,
            "normal" | "normal_strength" => rain.normal_strength = value,
            "focus_blur" | "focus_blur_strength" => rain.focus_blur_strength = value,
            "body" | "body_opacity" => rain.body_opacity = value,
            "blend" | "scene_blend" => rain.scene_blend = value,
            "drop_blur" | "plane_blur" | "drop_plane_blur_px" => rain.drop_plane_blur_px = value,
            "light_tint" | "tint" | "scene_light_tint_strength" => {
                rain.scene_light_tint_strength = value
            }
            "shadow_floor" | "scene_shadow_floor" => rain.scene_shadow_floor = value,
            "blood_amount" => rain.blood_amount = value,
            "scene_darken" => rain.scene_darken = value,
            "blood_r" => rain.blood_tint[0] = value,
            "blood_g" => rain.blood_tint[1] = value,
            "blood_b" => rain.blood_tint[2] = value,
            "trail_refract" | "trail_refract_scale" => rain.trail_refract_scale = value,
            "trail_opacity" => rain.trail_opacity = value,
            "scene_light" | "scene_light_response" => rain.scene_light_response = value,
            "rim" | "rim_strength" => rain.rim_strength = value,
            "smooth_edge_min" => rain.smooth_edge_min = value,
            "smooth_edge_max" => rain.smooth_edge_max = value,
            "trail_taper" => rain.trail_taper = value,
            "trail_spread" => rain.trail_spread = value,
            "trail_density" | "trail_drop_density" => rain.trail_drop_density = value,
            "trail_evaporate" => rain.trail_evaporate = value,
            "trail_shrink_rate" => rain.trail_shrink_rate = value,
            "streak" | "streak_boost" => rain.streak_boost = value,
            "streak_len" | "streak_length" => rain.streak_length = value,
            "trail_dist_min" | "trail_distance_min_px" => rain.trail_distance_min_px = value,
            "trail_dist_max" | "trail_distance_max_px" => rain.trail_distance_max_px = value,
            "trail_size_min" | "trail_drop_size_min" => rain.trail_drop_size_min = value,
            "trail_size_max" | "trail_drop_size_max" => rain.trail_drop_size_max = value,
            "micro" | "micro_droplets_per_second" => rain.micro_droplets_per_second = value,
            "micro_min" | "micro_droplet_min_px" => rain.micro_droplet_min_px = value,
            "micro_max" | "micro_droplet_max_px" => rain.micro_droplet_max_px = value,
            "mist_opacity" => rain.mist_opacity = value,
            "mist_time" => rain.mist_time = value,
            "mist_strength" => {
                rain.mist_color_strength = value;
                rain.mist_accumulation = value;
            }
            "mist_color_strength" => rain.mist_color_strength = value,
            "mist_blur" | "mist_blur_px" => rain.mist_blur_px = value,
            "mist_accumulation" => rain.mist_accumulation = value,
            "eraser" | "raindrop_eraser_size" => rain.raindrop_eraser_size = [value, value],
            "eraser_min" | "raindrop_eraser_min" => rain.raindrop_eraser_size[0] = value,
            "eraser_max" | "raindrop_eraser_max" => rain.raindrop_eraser_size[1] = value,
            "light_bump" => rain.light_bump = value,
            "diffuse" | "diffuse_light" => rain.diffuse_light = [value, value, value],
            "specular" | "specular_light" | "specular_color" => {
                rain.specular_light = [value, value, value]
            }
            "shininess" | "specular_shininess" => rain.specular_shininess = value,
            "shadow_offset" => rain.shadow_offset = value,
            _ => {}
        })
    }

    pub fn set_rain_glass_debug(&mut self, value: &str) -> bool {
        self.update_rain_glass(|rain| {
            rain.debug_view = parse_rain_glass_debug_view(value);
        })
    }

    pub fn set_rain_glass_compose(&mut self, value: &str) -> bool {
        self.update_rain_glass(|rain| {
            rain.raindrop_compose = match value.trim().to_ascii_lowercase().as_str() {
                "hard" | "harder" => RainGlassRaindropCompose::Harder,
                _ => RainGlassRaindropCompose::Smoother,
            };
        })
    }

    pub fn apply_rain_glass_preset(&mut self, value: &str) -> bool {
        self.update_rain_glass(|rain| match value.trim().to_ascii_lowercase().as_str() {
            "debug" => {
                rain.enabled = true;
                rain.spawn_rate = 2.0;
                rain.spawn_limit = 40;
                rain.min_radius_px = 32.0;
                rain.max_radius_px = 96.0;
                rain.micro_droplets_enabled = false;
                rain.micro_droplets_per_second = 0.0;
                rain.mist_enabled = false;
                rain.mist_opacity = 0.0;
                rain.opacity = 0.75;
                rain.refract_scale = 1.6;
                rain.background_blur_px = 8.0;
                rain.debug_view = RainGlassDebugView::Final;
            }
            "cinematic" | "html_current_controls" => apply_html_current_controls(rain),
            "reference_cinematic" | "html_cinematic" | "html_cinematic_button" => {
                apply_html_cinematic_button(rain)
            }
            "html_storm_button" => apply_html_storm_button(rain),
            "storm" => {
                apply_html_storm_button(rain);
                rain.spawn_rate = 24.0;
                rain.spawn_limit = 1200;
                rain.min_radius_px = 8.0;
                rain.max_radius_px = 76.0;
                rain.mist_opacity = 0.48;
                rain.refract_scale = 1.25;
                rain.distortion_px = 44.0;
                rain.body_opacity = 0.58;
                rain.scene_light_response = 1.70;
                rain.rim_strength = 1.25;
                rain.debug_view = RainGlassDebugView::Final;
            }
            "lens_streaks" => {
                apply_html_current_controls(rain);
                rain.spawn_rate = 5.0;
                rain.spawn_limit = 360;
                rain.min_radius_px = 24.0;
                rain.max_radius_px = 88.0;
                rain.trails_enabled = true;
                rain.trail_distance_min_px = 8.0;
                rain.trail_distance_max_px = 18.0;
                rain.trail_taper = 0.72;
                rain.trail_spread = 1.15;
                rain.trail_evaporate = 26.0;
                rain.trail_shrink_rate = 0.94;
                rain.distortion_px = 36.0;
                rain.normal_strength = 7.2;
                rain.focus_blur_strength = 0.85;
                rain.body_opacity = 0.58;
                rain.trail_refract_scale = 0.62;
                rain.trail_opacity = 0.78;
                rain.scene_light_response = 1.55;
                rain.rim_strength = 1.18;
                rain.mist_enabled = false;
                rain.mist_opacity = 0.0;
                rain.debug_view = RainGlassDebugView::Final;
            }
            "subtle" => {
                apply_html_current_controls(rain);
                rain.spawn_rate = 3.0;
                rain.spawn_limit = 130;
                rain.min_radius_px = 10.0;
                rain.max_radius_px = 44.0;
                rain.micro_droplets_enabled = true;
                rain.micro_droplets_per_second = 90.0;
                rain.mist_enabled = true;
                rain.mist_opacity = 0.08;
                rain.opacity = 0.58;
                rain.refract_scale = 0.62;
                rain.background_blur_px = 2.0;
                rain.debug_view = RainGlassDebugView::Final;
            }
            "optics_debug" => {
                rain.enabled = true;
                rain.spawn_rate = 1.0;
                rain.spawn_limit = 12;
                rain.min_radius_px = 72.0;
                rain.max_radius_px = 140.0;
                rain.micro_droplets_enabled = false;
                rain.micro_droplets_per_second = 0.0;
                rain.mist_enabled = false;
                rain.mist_opacity = 0.0;
                rain.trails_enabled = true;
                rain.trail_distance_min_px = 8.0;
                rain.trail_distance_max_px = 16.0;
                rain.trail_spread = 1.35;
                rain.trail_taper = 0.78;
                rain.trail_shrink_rate = 0.92;
                rain.trail_evaporate = 28.0;
                rain.trail_opacity = 0.85;
                rain.trail_refract_scale = 0.72;
                rain.opacity = 1.0;
                rain.refract_base = 0.80;
                rain.refract_scale = 3.0;
                rain.distortion_px = 48.0;
                rain.normal_strength = 8.0;
                rain.background_blur_px = 12.0;
                rain.focus_blur_strength = 1.0;
                rain.body_opacity = 0.95;
                rain.scene_light_response = 2.2;
                rain.rim_strength = 1.6;
                rain.debug_view = RainGlassDebugView::Final;
            }
            _ => {}
        })
    }

    fn update_rain_glass(&self, update: impl FnOnce(&mut RainGlass2d)) -> bool {
        let Some(service) = self.post_fx.as_ref() else {
            return false;
        };

        let mut stack = service.scene_stack();
        let index = stack
            .effects
            .iter()
            .position(|effect| matches!(effect, PostFx2d::RainGlass(_)))
            .unwrap_or_else(|| {
                stack
                    .effects
                    .push(PostFx2d::RainGlass(RainGlass2d::default()));
                stack.effects.len() - 1
            });

        let mut rain = match stack.effects[index] {
            PostFx2d::RainGlass(rain) => rain,
            _ => RainGlass2d::default(),
        };
        update(&mut rain);
        stack.effects[index] = PostFx2d::RainGlass(rain.normalized());
        service.set_scene_stack(stack.normalized());
        true
    }

    fn update_color_quantize(&self, update: impl FnOnce(&mut ColorQuantize2d)) -> rhai::INT {
        let Some(service) = self.post_fx.as_ref() else {
            return 0;
        };

        let mut stack = service.scene_stack();
        let index = stack
            .effects
            .iter()
            .position(|effect| matches!(effect, PostFx2d::ColorQuantize(_)))
            .unwrap_or_else(|| {
                stack
                    .effects
                    .push(PostFx2d::ColorQuantize(ColorQuantize2d::default()));
                stack.effects.len() - 1
            });

        let mut effect = match stack.effects[index] {
            PostFx2d::ColorQuantize(effect) => effect,
            _ => ColorQuantize2d::default(),
        };
        update(&mut effect);
        let effect = effect.normalized();
        let palette_size = effect.palette_size as rhai::INT;
        stack.effects[index] = PostFx2d::ColorQuantize(effect);
        service.set_scene_stack(stack.normalized());
        palette_size
    }
}

#[derive(Clone)]
pub struct PostFxItemRef {
    post_fx: Option<Arc<PostFx2dService>>,
    index: usize,
}

impl PostFxItemRef {
    pub fn exists(&mut self) -> bool {
        self.effect().is_some()
    }

    pub fn index(&mut self) -> rhai::INT {
        self.index as rhai::INT
    }

    pub fn name(&mut self) -> String {
        self.effect()
            .map(|effect| effect.kind().to_owned())
            .unwrap_or_default()
    }

    pub fn active(&mut self) -> bool {
        self.effect()
            .map(|effect| effect.is_active())
            .unwrap_or(false)
    }

    fn effect(&self) -> Option<PostFx2d> {
        self.post_fx
            .as_ref()
            .and_then(|service| service.scene_effect(self.index))
    }
}

fn item_map(index: usize, effect: PostFx2d) -> rhai::Dynamic {
    let mut map = rhai::Map::new();
    map.insert("index".into(), (index as rhai::INT).into());
    map.insert("name".into(), effect.clone().kind().to_owned().into());
    map.insert("active".into(), effect.is_active().into());
    map.into()
}

fn parse_rain_glass_debug_view(value: &str) -> RainGlassDebugView {
    match value.trim().to_ascii_lowercase().as_str() {
        "scene" | "scene_input" => RainGlassDebugView::SceneInput,
        "blur" | "blurred" | "blurred_scene" => RainGlassDebugView::BlurredScene,
        "raindrop_map" | "raindrops" => RainGlassDebugView::RaindropMap,
        "droplet_map" | "droplets" => RainGlassDebugView::DropletMap,
        "trail_map" | "streak_map" | "trails" | "streaks" => RainGlassDebugView::TrailMap,
        "drop_normals" | "normals" => RainGlassDebugView::DropNormals,
        "drop_mask" | "mask" => RainGlassDebugView::DropMask,
        "mist" => RainGlassDebugView::Mist,
        "refraction" => RainGlassDebugView::Refraction,
        _ => RainGlassDebugView::Final,
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enabled" => Some(true),
        "0" | "false" | "no" | "off" | "disabled" => Some(false),
        _ => None,
    }
}

fn is_rain_glass_float_field(field: &str) -> bool {
    matches!(
        field,
        "spawn_rate"
            | "radius_min"
            | "min_radius_px"
            | "radius_max"
            | "max_radius_px"
            | "gravity"
            | "gravity_px_per_sec2"
            | "slip_rate"
            | "motion_min"
            | "motion_interval_min"
            | "motion_max"
            | "motion_interval_max"
            | "x_shift_min"
            | "x_shift_max"
            | "collider"
            | "collider_scale"
            | "initial_spread"
            | "impact_spread"
            | "shrink_rate"
            | "spread_shrink"
            | "velocity_stretch"
            | "velocity_spread"
            | "evaporate"
            | "refract_base"
            | "refract_scale"
            | "opacity"
            | "blur"
            | "background_blur_px"
            | "chroma"
            | "chromatic_aberration"
            | "distortion"
            | "distortion_px"
            | "normal"
            | "normal_strength"
            | "focus_blur"
            | "focus_blur_strength"
            | "body"
            | "body_opacity"
            | "blend"
            | "scene_blend"
            | "drop_blur"
            | "plane_blur"
            | "drop_plane_blur_px"
            | "light_tint"
            | "tint"
            | "scene_light_tint_strength"
            | "shadow_floor"
            | "scene_shadow_floor"
            | "blood_amount"
            | "scene_darken"
            | "blood_r"
            | "blood_g"
            | "blood_b"
            | "trail_refract"
            | "trail_refract_scale"
            | "trail_opacity"
            | "scene_light"
            | "scene_light_response"
            | "rim"
            | "rim_strength"
            | "smooth_edge_min"
            | "smooth_edge_max"
            | "trail_taper"
            | "trail_spread"
            | "trail_density"
            | "trail_drop_density"
            | "trail_evaporate"
            | "trail_shrink_rate"
            | "streak"
            | "streak_boost"
            | "streak_len"
            | "streak_length"
            | "trail_dist_min"
            | "trail_distance_min_px"
            | "trail_dist_max"
            | "trail_distance_max_px"
            | "trail_size_min"
            | "trail_drop_size_min"
            | "trail_size_max"
            | "trail_drop_size_max"
            | "micro"
            | "micro_droplets_per_second"
            | "micro_min"
            | "micro_droplet_min_px"
            | "micro_max"
            | "micro_droplet_max_px"
            | "mist_opacity"
            | "mist_time"
            | "mist_strength"
            | "mist_color_strength"
            | "mist_blur"
            | "mist_blur_px"
            | "mist_accumulation"
            | "diffuse"
            | "diffuse_light"
            | "eraser"
            | "raindrop_eraser_size"
            | "eraser_min"
            | "raindrop_eraser_min"
            | "eraser_max"
            | "raindrop_eraser_max"
            | "light_bump"
            | "specular"
            | "specular_light"
            | "specular_color"
            | "shininess"
            | "specular_shininess"
            | "shadow_offset"
    )
}

fn apply_html_current_controls(rain: &mut RainGlass2d) {
    let trail_density = 0.20_f32;
    let trail_size = 0.42_f32;
    let trail_spread = 0.58_f32;
    let streak_boost = 0.72_f32;
    let streak_length = 1.15_f32;
    let background_blur_steps = 2_u32;

    rain.enabled = true;
    rain.reference_mode = true;
    rain.raindrop_compose = RainGlassRaindropCompose::Smoother;
    rain.spawn_rate = 10.0;
    rain.spawn_limit = 850;
    rain.min_radius_px = 45.0;
    rain.max_radius_px = 118.0;
    rain.opacity = 1.0;
    rain.slip_rate = 0.34;
    rain.gravity_px_per_sec2 = 2400.0;
    rain.motion_interval_min = 0.08;
    rain.motion_interval_max = 0.32;
    rain.x_shift_min = 0.0;
    rain.x_shift_max = 0.12;
    rain.collider_scale = 1.0;
    rain.evaporate = 11.0;
    rain.initial_spread = 0.52;
    rain.velocity_spread = 0.34;
    rain.shrink_rate = 0.014;
    rain.trails_enabled = true;
    rain.trail_drop_density = trail_density * (1.0 + streak_boost * 0.85);
    rain.trail_drop_size_min = trail_size * 0.58;
    rain.trail_drop_size_max = trail_size * 1.22;
    rain.trail_spread = trail_spread * (1.0 + streak_boost * 1.15);
    rain.streak_boost = streak_boost;
    rain.streak_length = streak_length;
    rain.trail_taper = 0.68;
    rain.trail_evaporate = 18.0;
    rain.trail_shrink_rate = (0.35 + rain.shrink_rate).clamp(0.001, 1.0);
    rain.trail_distance_min_px = (19.0 - streak_boost * 12.0).max(5.0);
    rain.trail_distance_max_px = (34.0 - streak_boost * 17.0).max(9.0);
    rain.trail_refract_scale = 1.0;
    rain.trail_opacity = 1.0;
    rain.micro_droplets_enabled = true;
    rain.micro_droplets_per_second = 620.0;
    rain.micro_droplet_min_px = 8.0;
    rain.micro_droplet_max_px = 27.0;
    rain.background_blur_steps = background_blur_steps;
    rain.background_blur_px = background_blur_steps as f32;
    rain.mist_enabled = true;
    rain.mist_opacity = 1.0;
    rain.mist_blur_step = 4;
    rain.mist_blur_px = 4.0;
    rain.mist_time = 16.0;
    rain.mist_accumulation = 0.012;
    rain.mist_color_strength = 0.012;
    rain.smooth_edge_min = 0.945;
    rain.smooth_edge_max = 0.992;
    rain.refract_base = 0.34;
    rain.refract_scale = 0.76;
    rain.chromatic_aberration = 0.0;
    rain.distortion_px = 28.0;
    rain.normal_strength = 6.0;
    rain.focus_blur_strength = 0.85;
    rain.body_opacity = 1.0;
    rain.scene_blend = 1.0;
    rain.raindrop_eraser_size = [0.93, 1.0];
    rain.shadow_offset = 0.76;
    rain.diffuse_light = [0.22, 0.22, 0.22];
    rain.specular_light = [0.025, 0.025, 0.025];
    rain.specular_shininess = 300.0;
    rain.light_pos = [-1.0, 1.0, 2.0, 0.0];
    rain.light_bump = 0.78;
    rain.scene_light_response = 0.0;
    rain.rim_strength = 0.0;
    rain.debug_view = RainGlassDebugView::Final;
}

fn apply_html_cinematic_button(rain: &mut RainGlass2d) {
    apply_html_current_controls(rain);
    let trail_size = 0.38_f32;
    let trail_spread = 0.56_f32;
    let streak_boost = 0.55_f32;
    let streak_length = 1.05_f32;

    rain.slip_rate = 0.24;
    rain.gravity_px_per_sec2 = 2100.0;
    rain.evaporate = 13.0;
    rain.trail_drop_density = 0.18;
    rain.trail_drop_size_min = (trail_size * 0.58).max(0.05);
    rain.trail_drop_size_max = (trail_size * 1.22).max(0.06);
    rain.trail_spread = trail_spread * (1.0 + streak_boost * streak_length);
    rain.streak_boost = streak_boost;
    rain.streak_length = streak_length;
    rain.trail_taper = 0.72;
    rain.trail_evaporate = 19.0;
    rain.trail_distance_min_px = (19.0 - streak_boost * 12.0).max(5.0);
    rain.trail_distance_max_px = (34.0 - streak_boost * 17.0).max(9.0);
    rain.micro_droplets_enabled = true;
    rain.micro_droplets_per_second = 420.0;
    rain.background_blur_steps = 2;
    rain.background_blur_px = 8.0;
    rain.mist_enabled = true;
    rain.mist_opacity = 0.42;
    rain.mist_color_strength = 0.010;
    rain.mist_blur_step = 4;
    rain.mist_blur_px = 4.0;
    rain.refract_base = 0.32;
    rain.refract_scale = 0.70;
    rain.diffuse_light = [0.20, 0.20, 0.20];
    rain.shadow_offset = 0.76;
    rain.specular_light = [0.018, 0.018, 0.018];
    rain.body_opacity = 0.58;
    rain.trail_refract_scale = 0.62;
    rain.trail_opacity = 0.78;
}

fn apply_html_storm_button(rain: &mut RainGlass2d) {
    apply_html_current_controls(rain);
    let trail_spread = 0.58_f32;
    let streak_boost = 0.96_f32;
    let streak_length = 1.78_f32;

    rain.streak_boost = streak_boost;
    rain.streak_length = streak_length;
    rain.trail_spread = trail_spread * (1.0 + streak_boost * streak_length);
    rain.trail_distance_min_px = (19.0 - streak_boost * 12.0).max(5.0);
    rain.trail_distance_max_px = (34.0 - streak_boost * 17.0).max(9.0);
    rain.trail_taper = 0.86;
    rain.trail_evaporate = 12.0;
    rain.micro_droplets_enabled = true;
    rain.micro_droplets_per_second = 1250.0;
    rain.background_blur_steps = 3;
    rain.background_blur_px = 12.0;
    rain.mist_enabled = true;
    rain.mist_opacity = 0.55;
    rain.mist_color_strength = 0.018;
    rain.refract_base = 0.48;
    rain.refract_scale = 0.94;
    rain.diffuse_light = [0.28, 0.28, 0.28];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_current_controls_apply_reference_trail_options() {
        let mut rain = RainGlass2d::default();

        apply_html_current_controls(&mut rain);

        assert!(rain.enabled);
        assert!(rain.reference_mode);
        assert_eq!(rain.spawn_rate, 10.0);
        assert_eq!(rain.spawn_limit, 850);
        assert_eq!(rain.min_radius_px, 45.0);
        assert_eq!(rain.max_radius_px, 118.0);
        assert_eq!(rain.slip_rate, 0.34);
        assert_eq!(rain.gravity_px_per_sec2, 2400.0);
        assert_eq!(rain.initial_spread, 0.52);
        assert_eq!(rain.velocity_spread, 0.34);
        assert_eq!(rain.shrink_rate, 0.014);
        assert_eq!(rain.evaporate, 11.0);
        assert!(rain.trails_enabled);
        assert!((rain.trail_drop_density - 0.3224).abs() < 0.0002);
        assert!((rain.trail_drop_size_min - 0.2436).abs() < 0.0001);
        assert!((rain.trail_drop_size_max - 0.5124).abs() < 0.0001);
        assert!((rain.trail_spread - 1.06024).abs() < 0.0001);
        assert!((rain.trail_distance_min_px - 10.36).abs() < 0.0001);
        assert!((rain.trail_distance_max_px - 21.76).abs() < 0.0001);
        assert_eq!(rain.streak_boost, 0.72);
        assert_eq!(rain.streak_length, 1.15);
        assert!(rain.micro_droplets_enabled);
        assert_eq!(rain.micro_droplets_per_second, 620.0);
        assert_eq!(rain.micro_droplet_min_px, 8.0);
        assert_eq!(rain.micro_droplet_max_px, 27.0);
        assert!(rain.mist_enabled);
        assert_eq!(rain.mist_opacity, 1.0);
        assert_eq!(rain.mist_accumulation, 0.012);
        assert_eq!(rain.mist_color_strength, 0.012);
        assert_eq!(rain.background_blur_steps, 2);
        assert_eq!(rain.background_blur_px, 2.0);
        assert_eq!(rain.mist_blur_step, 4);
        assert_eq!(rain.mist_time, 16.0);
        assert_eq!(rain.smooth_edge_min, 0.945);
        assert_eq!(rain.smooth_edge_max, 0.992);
        assert_eq!(rain.refract_base, 0.34);
        assert_eq!(rain.refract_scale, 0.76);
        assert_eq!(rain.diffuse_light, [0.22, 0.22, 0.22]);
        assert_eq!(rain.specular_light, [0.025, 0.025, 0.025]);
        assert_eq!(rain.specular_shininess, 300.0);
        assert_eq!(rain.scene_light_response, 0.0);
        assert_eq!(rain.rim_strength, 0.0);
    }

    #[test]
    fn html_cinematic_button_uses_button_values_not_engine_reference_values() {
        let mut rain = RainGlass2d::default();

        apply_html_cinematic_button(&mut rain);

        assert_eq!(rain.slip_rate, 0.24);
        assert_eq!(rain.gravity_px_per_sec2, 2100.0);
        assert_eq!(rain.trail_drop_density, 0.18);
        assert!((rain.trail_drop_size_min - 0.2204).abs() < 0.0001);
        assert!((rain.trail_drop_size_max - 0.4636).abs() < 0.0001);
        assert!((rain.trail_spread - 0.8834).abs() < 0.0001);
        assert!((rain.trail_distance_min_px - 12.4).abs() < 0.0001);
        assert!((rain.trail_distance_max_px - 24.65).abs() < 0.0001);
        assert_eq!(rain.micro_droplets_per_second, 420.0);
        assert_eq!(rain.background_blur_steps, 2);
        assert_eq!(rain.mist_color_strength, 0.010);
        assert_eq!(rain.refract_base, 0.32);
        assert_eq!(rain.refract_scale, 0.70);
    }

    #[test]
    fn html_storm_button_uses_reference_streak_boost_mapping() {
        let mut rain = RainGlass2d::default();

        apply_html_storm_button(&mut rain);

        assert_eq!(rain.streak_boost, 0.96);
        assert_eq!(rain.streak_length, 1.78);
        assert!((rain.trail_spread - 1.571_104).abs() < 0.0001);
        assert!((rain.trail_distance_min_px - 7.48).abs() < 0.0001);
        assert!((rain.trail_distance_max_px - 17.68).abs() < 0.0001);
        assert_eq!(rain.trail_taper, 0.86);
        assert_eq!(rain.trail_evaporate, 12.0);
        assert_eq!(rain.micro_droplets_per_second, 1250.0);
        assert_eq!(rain.background_blur_steps, 3);
        assert_eq!(rain.mist_color_strength, 0.018);
        assert_eq!(rain.refract_base, 0.48);
        assert_eq!(rain.refract_scale, 0.94);
    }
}
