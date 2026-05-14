use amigo_2d_post_fx::{RainGlass2d, RainGlassDebugView, RainGlassRaindropCompose};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct RainGlassInstance {
    pub center_size: [f32; 4],
    pub params: [f32; 4],
}

impl RainGlassInstance {
    pub(crate) fn bytes(instances: &[Self]) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                instances.as_ptr() as *const u8,
                std::mem::size_of_val(instances),
            )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RainGlassDropKind {
    Main,
    Trail,
    Micro,
}

fn reference_visual_scale(kind: RainGlassDropKind, cfg: RainGlass2d) -> f32 {
    if !cfg.reference_mode {
        return 1.0;
    }

    match kind {
        RainGlassDropKind::Main | RainGlassDropKind::Trail => 0.58,
        RainGlassDropKind::Micro => 0.80,
    }
}

pub(crate) fn reference_spawn_radius(size_px: f32, cfg: RainGlass2d) -> f32 {
    if cfg.reference_mode {
        size_px * 0.5
    } else {
        size_px
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RainGlassDrop {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub x: f32,
    pub y: f32,
    pub prev_x: f32,
    pub prev_y: f32,
    pub vx: f32,
    pub vy: f32,
    pub mass: f32,
    pub visual_mass: f32,
    pub initial_mass: f32,
    pub density: f32,
    pub spread_x: f32,
    pub spread_y: f32,
    pub resistance: f32,
    pub shifting: f32,
    pub last_trail_x: f32,
    pub last_trail_y: f32,
    pub next_trail_distance: f32,
    pub next_motion_time: f32,
    pub streak_emit: f32,
    pub streak_age: f32,
    pub streak_mass0: f32,
    pub streak_seed: f32,
    pub reference_streak_child: bool,
    pub age: f32,
    pub seed: f32,
    pub kind: RainGlassDropKind,
    pub destroyed: bool,
}

impl RainGlassDrop {
    pub(crate) fn radius(&self) -> f32 {
        self.mass.max(0.0).sqrt() / self.density.max(0.001)
    }

    pub(crate) fn size_x(&self) -> f32 {
        (1.0 + self.spread_x).max(0.05) * self.radius()
    }

    #[cfg(test)]
    pub(crate) fn size_y(&self) -> f32 {
        (1.0 + self.spread_y).max(0.05) * self.radius()
    }

    fn visual_radius(&self) -> f32 {
        self.visual_mass.max(1.0).sqrt() / self.density.max(0.001)
    }

    fn visual_size_x(&self) -> f32 {
        (1.0 + self.spread_x) * self.visual_radius()
    }

    fn visual_size_y(&self) -> f32 {
        (1.0 + self.spread_y) * self.visual_radius()
    }

    pub(crate) fn visible_radius(&self, cfg: RainGlass2d) -> f32 {
        self.visual_radius() * reference_visual_scale(self.kind, cfg)
    }

    pub(crate) fn visible_size_x(&self, cfg: RainGlass2d) -> f32 {
        self.visual_size_x() * reference_visual_scale(self.kind, cfg)
    }

    pub(crate) fn visible_size_y(&self, cfg: RainGlass2d) -> f32 {
        self.visual_size_y() * reference_visual_scale(self.kind, cfg)
    }

    pub(crate) fn opacity(&self) -> f32 {
        let mass_alpha = (self.mass / self.initial_mass.max(1.0))
            .sqrt()
            .clamp(0.0, 1.0);
        match self.kind {
            RainGlassDropKind::Main => mass_alpha.clamp(0.12, 1.0),
            RainGlassDropKind::Trail => (mass_alpha * (1.0 - self.age * 0.10)).clamp(0.04, 0.88),
            RainGlassDropKind::Micro => (mass_alpha * 0.68).clamp(0.05, 0.68),
        }
    }

    pub(crate) fn to_instance(&self, cfg: RainGlass2d) -> RainGlassInstance {
        let kind = match self.kind {
            RainGlassDropKind::Main => 0.0,
            RainGlassDropKind::Trail => 1.0,
            RainGlassDropKind::Micro => 2.0,
        };
        let mut size_x = self.visible_size_x(cfg).max(1.0);
        let mut size_y = self.visible_size_y(cfg).max(1.0);

        if self.kind == RainGlassDropKind::Trail && !cfg.reference_mode {
            size_x = (size_x * 0.72).max(2.0);
            size_y = (size_y * 1.65).max(size_x * 3.0);
        }

        let opacity = if self.kind == RainGlassDropKind::Trail && !cfg.reference_mode {
            (self.opacity() * 1.25).clamp(0.0, 1.0)
        } else {
            self.opacity()
        };

        RainGlassInstance {
            center_size: [self.x, self.y, size_x, size_y],
            params: [
                opacity,
                (self.visible_radius(cfg)
                    / reference_spawn_radius(cfg.max_radius_px.max(1.0), cfg))
                .clamp(0.05, 1.0),
                self.seed,
                kind,
            ],
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(kind: RainGlassDropKind) -> Self {
        Self {
            id: 1,
            parent_id: None,
            x: 100.0,
            y: 100.0,
            prev_x: 100.0,
            prev_y: 100.0,
            vx: 0.0,
            vy: 0.0,
            mass: 1600.0,
            visual_mass: 1600.0,
            initial_mass: 1600.0,
            density: 1.0,
            spread_x: 0.2,
            spread_y: 0.2,
            resistance: 0.0,
            shifting: 0.0,
            last_trail_x: 100.0,
            last_trail_y: 0.0,
            next_trail_distance: 8.0,
            next_motion_time: 0.0,
            streak_emit: 0.0,
            streak_age: 0.0,
            streak_mass0: 1600.0,
            streak_seed: 0.5,
            reference_streak_child: false,
            age: 0.0,
            seed: 0.5,
            kind,
            destroyed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RainGlassTrailSegment {
    pub parent_id: u64,
    pub x: f32,
    pub y: f32,
    pub half_width: f32,
    pub half_len: f32,
    pub opacity: f32,
    pub age: f32,
    pub lifetime: f32,
    pub seed: f32,
}

impl RainGlassTrailSegment {
    pub(crate) fn to_instance(&self, cfg: RainGlass2d) -> RainGlassInstance {
        let seed = self.seed + self.parent_id as f32 * 0.000_001;
        RainGlassInstance {
            center_size: [
                self.x,
                self.y,
                self.half_width.max(1.5),
                self.half_len.max(self.half_width * 2.5),
            ],
            params: [
                self.opacity.clamp(0.0, 1.0),
                (self.half_width / cfg.max_radius_px.max(1.0)).clamp(0.05, 0.75),
                seed,
                1.0,
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RainGlassUniform {
    pub params0: [f32; 4],
    pub params1: [f32; 4],
    pub params2: [f32; 4],
    pub params3: [f32; 4],
    pub params4: [f32; 4],
    pub params5: [f32; 4],
    pub params6: [f32; 4],
    pub params7: [f32; 4],
    pub params8: [f32; 4],
    pub params9: [f32; 4],
    pub diffuse: [f32; 4],
    pub specular: [f32; 4],
}

impl RainGlassUniform {
    pub(crate) fn new(cfg: RainGlass2d, width: u32, height: u32, dt: f32) -> Self {
        Self {
            params0: [
                width.max(1) as f32,
                height.max(1) as f32,
                1.0 / width.max(1) as f32,
                1.0 / height.max(1) as f32,
            ],
            params1: [
                cfg.refract_base,
                cfg.refract_scale,
                cfg.opacity,
                cfg.light_bump,
            ],
            params2: [
                cfg.smooth_edge_min,
                cfg.smooth_edge_max,
                cfg.shadow_offset,
                cfg.background_blur_px,
            ],
            params3: [
                cfg.light_pos[0],
                cfg.light_pos[1],
                cfg.light_pos[2],
                cfg.light_pos[3],
            ],
            params4: [
                cfg.specular_shininess,
                cfg.mist_opacity,
                cfg.chromatic_aberration,
                debug_view_id(cfg.debug_view),
            ],
            params5: [
                cfg.distortion_px,
                cfg.normal_strength,
                cfg.focus_blur_strength,
                cfg.body_opacity,
            ],
            params6: [
                cfg.scene_light_response,
                cfg.rim_strength,
                cfg.trail_refract_scale,
                cfg.trail_opacity,
            ],
            params7: [
                cfg.mist_blur_px,
                if cfg.mist_enabled { 1.0 } else { 0.0 },
                cfg.scene_blend,
                if cfg.micro_droplets_enabled { 1.0 } else { 0.0 },
            ],
            params8: [
                cfg.mist_time,
                cfg.mist_color_strength,
                cfg.background_blur_steps as f32,
                cfg.mist_blur_step as f32,
            ],
            params9: [
                cfg.raindrop_eraser_size[0],
                cfg.raindrop_eraser_size[1],
                match cfg.raindrop_compose {
                    RainGlassRaindropCompose::Smoother => 0.0,
                    RainGlassRaindropCompose::Harder => 1.0,
                },
                if cfg.reference_mode { 1.0 } else { 0.0 },
            ],
            diffuse: [
                cfg.diffuse_light[0],
                cfg.diffuse_light[1],
                cfg.diffuse_light[2],
                dt,
            ],
            specular: [
                cfg.specular_light[0],
                cfg.specular_light[1],
                cfg.specular_light[2],
                cfg.mist_accumulation,
            ],
        }
    }
}

pub(crate) fn debug_view_id(view: RainGlassDebugView) -> f32 {
    match view {
        RainGlassDebugView::Final => 0.0,
        RainGlassDebugView::SceneInput => 1.0,
        RainGlassDebugView::BlurredScene => 2.0,
        RainGlassDebugView::RaindropMap => 3.0,
        RainGlassDebugView::DropletMap => 4.0,
        RainGlassDebugView::TrailMap => 5.0,
        RainGlassDebugView::DropNormals => 6.0,
        RainGlassDebugView::DropMask => 7.0,
        RainGlassDebugView::Mist => 8.0,
        RainGlassDebugView::Refraction => 9.0,
    }
}

pub(crate) fn bytes_of<T>(value: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((value as *const T) as *const u8, std::mem::size_of::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_mode_scales_visual_instance_size_without_changing_depth() {
        let mut drop = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        drop.mass = 10_000.0;
        drop.visual_mass = 10_000.0;
        drop.initial_mass = 10_000.0;
        drop.density = 1.0;
        drop.spread_x = 0.0;
        drop.spread_y = 0.0;

        let reference = RainGlass2d {
            reference_mode: true,
            max_radius_px: 100.0,
            ..RainGlass2d::default()
        }
        .normalized();
        let custom = RainGlass2d {
            reference_mode: false,
            max_radius_px: 100.0,
            ..RainGlass2d::default()
        }
        .normalized();

        let reference_instance = drop.to_instance(reference);
        let custom_instance = drop.to_instance(custom);

        assert!(reference_instance.center_size[2] < custom_instance.center_size[2] * 0.7);
        assert!(reference_instance.center_size[3] < custom_instance.center_size[3] * 0.7);
        assert_eq!(reference_instance.params[1], custom_instance.params[1]);
    }
}
