use amigo_2d_post_fx::RainGlass2d;

use super::types::{
    reference_spawn_radius, RainGlassDrop, RainGlassDropKind, RainGlassInstance,
    RainGlassTrailSegment,
};

#[derive(Default)]
pub(crate) struct RainGlassSimulation {
    drops: Vec<RainGlassDrop>,
    microdrops: Vec<RainGlassDrop>,
    trail_segments: Vec<RainGlassTrailSegment>,
    next_id: u64,
    spawn_accumulator: f32,
    micro_spawn_accumulator: f32,
    micro_preseeded: bool,
    elapsed: f32,
    rng: RainGlassRng,
}

impl RainGlassSimulation {
    pub(crate) fn new(seed: u32, width: f32, height: f32) -> Self {
        let simulation = Self {
            rng: RainGlassRng::new(seed),
            ..Self::default()
        };
        let _ = (width, height);
        simulation
    }

    pub(crate) fn update(&mut self, cfg: RainGlass2d, dt: f32, width: f32, height: f32) {
        let cfg = cfg.normalized();
        self.preseed_micro_drops(cfg, width, height);
        self.spawn_main_drops(cfg, dt, width, height);
        self.spawn_micro_drops(cfg, dt, width, height);
        self.update_drops(cfg, dt);
        self.split_trails(cfg);
        self.merge_collisions(cfg);
        self.enhance_water_streaks(cfg, dt);
        self.emit_trail_ribbons(cfg);
        self.update_trail_segments(cfg, dt);
        self.destroy_out_of_bounds(width, height);
        self.limit_drop_count(effective_drop_limit(cfg));
        self.limit_trail_segments(cfg.spawn_limit as usize);
        self.elapsed += dt;
    }

    pub(crate) fn live_instances(&self, cfg: RainGlass2d) -> Vec<RainGlassInstance> {
        self.drops
            .iter()
            .filter(|drop| {
                drop.kind == RainGlassDropKind::Main
                    || (cfg.reference_mode && drop.kind == RainGlassDropKind::Trail)
            })
            .map(|drop| drop.to_instance(cfg))
            .collect()
    }

    pub(crate) fn persistent_instances(&self, cfg: RainGlass2d) -> Vec<RainGlassInstance> {
        self.microdrops
            .iter()
            .map(|drop| drop.to_instance(cfg))
            .collect()
    }

    pub(crate) fn trail_instances(&self, cfg: RainGlass2d) -> Vec<RainGlassInstance> {
        if cfg.reference_mode {
            return self
                .trail_segments
                .iter()
                .map(|segment| segment.to_instance(cfg))
                .collect();
        }

        let mut instances = self
            .trail_segments
            .iter()
            .map(|segment| segment.to_instance(cfg))
            .collect::<Vec<_>>();

        instances.extend(
            self.drops
                .iter()
                .filter(|drop| drop.kind == RainGlassDropKind::Trail)
                .map(|drop| drop.to_instance(cfg)),
        );

        instances
    }

    fn spawn_main_drops(&mut self, cfg: RainGlass2d, dt: f32, width: f32, height: f32) {
        if cfg.spawn_rate <= 0.0 || self.drops.len() >= cfg.spawn_limit as usize {
            return;
        }

        let base = 1.0 / cfg.spawn_rate.max(0.001);
        if self.spawn_accumulator <= 0.0 {
            self.spawn_accumulator = self.rng.range(base * 0.72, base * 1.36);
        }

        self.spawn_accumulator -= dt;
        while self.spawn_accumulator <= 0.0 && self.drops.len() < cfg.spawn_limit as usize {
            let size = self.rng.range(cfg.min_radius_px, cfg.max_radius_px);
            let radius = reference_spawn_radius(size, cfg);
            let x = self.rng.range(0.0, width);
            let y = self.rng.range(0.0, height);
            self.spawn_drop(x, y, radius, 1.0, RainGlassDropKind::Main, cfg);
            self.spawn_accumulator += self.rng.range(base * 0.72, base * 1.36);
        }
    }

    fn preseed_micro_drops(&mut self, cfg: RainGlass2d, width: f32, height: f32) {
        if self.micro_preseeded
            || !cfg.micro_droplets_enabled
            || cfg.micro_droplets_per_second <= 0.0
        {
            self.micro_preseeded = true;
            return;
        }

        let area_scale = ((width * height) / (1280.0 * 720.0)).clamp(0.35, 2.25);
        let target =
            (cfg.micro_droplets_per_second * 0.68 * area_scale).clamp(72.0, 1400.0) as usize;
        for _ in 0..target {
            let size = self
                .rng
                .range(cfg.micro_droplet_min_px, cfg.micro_droplet_max_px);
            let radius = reference_spawn_radius(size, cfg);
            let x = self.rng.range(0.0, width);
            let y = self.rng.range(0.0, height);
            self.spawn_microdrop(x, y, radius);
        }
        self.micro_preseeded = true;
    }

    fn spawn_micro_drops(&mut self, cfg: RainGlass2d, dt: f32, width: f32, height: f32) {
        if !cfg.micro_droplets_enabled || cfg.micro_droplets_per_second <= 0.0 {
            return;
        }
        self.micro_spawn_accumulator += dt * cfg.micro_droplets_per_second;
        let max_microdrops = (cfg.micro_droplets_per_second * 2.2).clamp(96.0, 2400.0) as usize;
        while self.micro_spawn_accumulator >= 1.0 && self.microdrops.len() < max_microdrops {
            self.micro_spawn_accumulator -= 1.0;
            let size = self
                .rng
                .range(cfg.micro_droplet_min_px, cfg.micro_droplet_max_px);
            let radius = reference_spawn_radius(size, cfg);
            let x = self.rng.range(0.0, width);
            let y = self.rng.range(0.0, height);
            self.spawn_microdrop(x, y, radius);
        }
        for micro in &mut self.microdrops {
            micro.mass = (micro.mass - cfg.evaporate * 0.06 * dt).max(0.0);
            micro.age += dt;
        }
        self.microdrops
            .retain(|drop| drop.mass > 2.0 && drop.age < 40.0);
    }

    fn update_drops(&mut self, cfg: RainGlass2d, dt: f32) {
        for index in 0..self.drops.len() {
            if self.drops[index].destroyed {
                continue;
            }

            if self.elapsed >= self.drops[index].next_motion_time {
                self.randomize_motion(index, cfg);
            }

            let drop = &mut self.drops[index];
            drop.prev_x = drop.x;
            drop.prev_y = drop.y;

            drop.mass -= cfg.evaporate * dt;
            if drop.mass <= 2.0 {
                drop.destroyed = true;
                continue;
            }

            let force = cfg.gravity_px_per_sec2 * drop.mass - drop.resistance;
            let acceleration = force / drop.mass.max(0.001);
            drop.vy += acceleration * dt;
            if drop.vy < 0.0 {
                drop.vy = 0.0;
            }
            drop.vx = drop.vy.abs() * drop.shifting;
            drop.x += drop.vx * dt;
            drop.y += drop.vy * dt;

            let spread_from_velocity =
                cfg.velocity_spread * 2.0 * (drop.vy.abs() * 0.005).atan() / std::f32::consts::PI;
            drop.spread_y = drop.spread_y.max(spread_from_velocity);
            drop.spread_x *= cfg.shrink_rate.powf(dt);
            drop.spread_y *= cfg.shrink_rate.powf(dt);
            let mut visual_target = drop.mass;
            if cfg.reference_mode && drop.kind == RainGlassDropKind::Main && drop.vy.abs() > 95.0 {
                let sliding_age = (drop.age * (0.18 + drop.vy.abs() * 0.00035)).clamp(0.0, 0.55);
                visual_target *= 1.0 - sliding_age;
            }
            let visual_lerp = 1.0 - (-dt * 9.0).exp();
            drop.visual_mass += (visual_target - drop.visual_mass) * visual_lerp;
            drop.visual_mass = drop.visual_mass.max(1.0);
            drop.age += dt;
        }
    }

    fn randomize_motion(&mut self, index: usize, cfg: RainGlass2d) {
        let slip_t = (1.0 - cfg.slip_rate).clamp(0.0, 1.0);
        let reference_size = cfg.min_radius_px + (cfg.max_radius_px - cfg.min_radius_px) * slip_t;
        let resistance_scale = reference_size * reference_size * 4.0;
        let sign = if self.rng.next01() < 0.5 { -1.0 } else { 1.0 };
        let drop = &mut self.drops[index];
        drop.next_motion_time = self.elapsed
            + self
                .rng
                .range(cfg.motion_interval_min, cfg.motion_interval_max);
        drop.resistance = self.rng.next01() * cfg.gravity_px_per_sec2 * resistance_scale;
        drop.shifting = sign * self.rng.range(cfg.x_shift_min, cfg.x_shift_max);
    }

    fn emit_trail_ribbons(&mut self, cfg: RainGlass2d) {
        if !cfg.trails_enabled {
            return;
        }

        let mut segments = Vec::new();

        for drop in &mut self.drops {
            if drop.kind != RainGlassDropKind::Main {
                continue;
            }

            let speed = drop.vy.abs();
            if speed < 45.0 {
                continue;
            }

            let min_mass = (cfg.min_radius_px * cfg.min_radius_px * 0.65).clamp(90.0, 650.0);
            if drop.mass < min_mass {
                continue;
            }

            let dx = drop.x - drop.prev_x;
            let dy = drop.y - drop.prev_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 1.5 {
                continue;
            }

            let since_last_x = drop.x - drop.last_trail_x;
            let since_last_y = drop.y - drop.last_trail_y;
            let since_last = (since_last_x * since_last_x + since_last_y * since_last_y).sqrt();

            let ribbon_step = cfg.trail_distance_min_px.min(8.0).max(3.0);
            if since_last < ribbon_step && dist < ribbon_step {
                continue;
            }

            let size_x = drop.visible_size_x(cfg);
            let size_y = drop.visible_size_y(cfg);
            let width = (size_x * 0.62 * cfg.trail_spread).clamp(5.0, size_x.max(5.0) * 1.32);

            let len = (dist * 0.96 + size_y * 0.44).clamp(width * 2.4, size_y.max(width * 4.8));

            let x = (drop.prev_x + drop.x) * 0.5;
            let y = (drop.prev_y + drop.y) * 0.5 + size_y * 0.18;
            drop.last_trail_x = drop.x;
            drop.last_trail_y = drop.y;

            segments.push(RainGlassTrailSegment {
                parent_id: drop.id,
                x,
                y,
                half_width: width,
                half_len: len,
                opacity: cfg.trail_opacity.clamp(0.0, 1.0),
                age: 0.0,
                lifetime: (2.6 + cfg.trail_shrink_rate * 3.0).clamp(1.4, 6.2),
                seed: self.rng.next01(),
            });
        }

        self.trail_segments.extend(segments);
    }

    fn update_trail_segments(&mut self, cfg: RainGlass2d, dt: f32) {
        for segment in &mut self.trail_segments {
            segment.age += dt;

            let shrink =
                (1.0 - (1.0 - cfg.trail_shrink_rate).clamp(0.0, 1.0) * dt * 0.18).clamp(0.0, 1.0);
            segment.half_width *= shrink;
            segment.half_len *= (1.0 - cfg.trail_taper * dt * 0.18).clamp(0.0, 1.0);

            let evaporate = (-cfg.trail_evaporate * 0.024 * dt).exp();
            segment.opacity *= evaporate;
        }

        self.trail_segments.retain(|segment| {
            segment.age < segment.lifetime
                && segment.opacity > 0.025
                && segment.half_width > 0.75
                && segment.half_len > 2.0
        });
    }

    fn split_trails(&mut self, cfg: RainGlass2d) {
        if !cfg.trails_enabled {
            return;
        }
        let mut trails = Vec::new();
        for drop in &mut self.drops {
            if drop.kind != RainGlassDropKind::Main || drop.mass < 1000.0 {
                continue;
            }
            let dx = drop.x - drop.last_trail_x;
            let dy = drop.y - drop.last_trail_y;
            if dx * dx + dy * dy < drop.next_trail_distance * drop.next_trail_distance {
                continue;
            }
            let radius = drop.visible_radius(cfg)
                * self
                    .rng
                    .range(cfg.trail_drop_size_min, cfg.trail_drop_size_max);
            let mut trail = RainGlassDrop::new(
                self.next_id,
                drop.x + self.rng.range(-5.0, 5.0),
                drop.y + drop.visible_size_y(cfg) * 0.25,
                radius.max(1.0),
                cfg.trail_drop_density,
                RainGlassDropKind::Trail,
                &mut self.rng,
                cfg,
            );
            self.next_id += 1;
            trail.parent_id = Some(drop.id);
            trail.vy = drop.vy * 0.12;
            trail.spread_x = 0.16;
            trail.spread_y = (0.45 + drop.vy.abs() * 0.012) * cfg.trail_spread;
            trail.streak_mass0 = trail.mass;
            drop.mass = (drop.mass - trail.mass).max(1.0);
            drop.last_trail_x = drop.x;
            drop.last_trail_y = drop.y;
            drop.next_trail_distance = self
                .rng
                .range(cfg.trail_distance_min_px, cfg.trail_distance_max_px);
            trails.push(trail);

            if self.rng.next01() < 0.38 {
                let bead_radius = (radius * self.rng.range(0.26, 0.46)).max(1.0);
                let mut bead = RainGlassDrop::new(
                    self.next_id,
                    drop.x + self.rng.range(-6.0, 6.0),
                    drop.y + drop.visible_size_y(cfg) * self.rng.range(0.18, 0.42),
                    bead_radius,
                    (cfg.trail_drop_density * 1.10).max(0.05),
                    RainGlassDropKind::Trail,
                    &mut self.rng,
                    cfg,
                );
                self.next_id += 1;
                bead.parent_id = Some(drop.id);
                bead.vy = drop.vy * self.rng.range(0.05, 0.11);
                bead.vx = drop.vx * 0.15 + self.rng.range(-10.0, 10.0);
                bead.spread_x = 0.10 + self.rng.next01() * 0.08;
                bead.spread_y = (0.22 + drop.vy.abs() * 0.006) * cfg.trail_spread;
                bead.streak_mass0 = bead.mass;
                drop.mass = (drop.mass - bead.mass * 0.6).max(1.0);
                trails.push(bead);
            }
        }
        self.drops.extend(trails);
    }

    fn merge_collisions(&mut self, cfg: RainGlass2d) {
        let len = self.drops.len();
        for i in 0..len {
            if self.drops[i].destroyed {
                continue;
            }
            for j in (i + 1)..len {
                if self.drops[j].destroyed {
                    continue;
                }

                let both_trails = self.drops[i].kind == RainGlassDropKind::Trail
                    && self.drops[j].kind == RainGlassDropKind::Trail;
                if both_trails {
                    continue;
                }

                let parent_related = self.drops[i].parent_id == Some(self.drops[j].id)
                    || self.drops[j].parent_id == Some(self.drops[i].id);
                let same_parent = self.drops[i].parent_id.is_some()
                    && self.drops[i].parent_id == self.drops[j].parent_id;
                if parent_related || same_parent {
                    continue;
                }

                let dx = self.drops[i].x - self.drops[j].x;
                let dy = self.drops[i].y - self.drops[j].y;
                let overlaps = if cfg.reference_mode {
                    let sx = (self.drops[i].visible_size_x(cfg)
                        + self.drops[j].visible_size_x(cfg))
                        * 0.64
                        * cfg.collider_scale.max(0.05);
                    let sy = (self.drops[i].visible_size_y(cfg)
                        + self.drops[j].visible_size_y(cfg))
                        * 0.52
                        * cfg.collider_scale.max(0.05);
                    let nx = dx / sx.max(1.0);
                    let ny = dy / sy.max(1.0);
                    nx * nx + ny * ny <= 1.0
                } else {
                    let distance = (dx * dx + dy * dy).sqrt();
                    let merge_distance = reference_merge_distance(&self.drops[i], cfg)
                        + reference_merge_distance(&self.drops[j], cfg);
                    distance - merge_distance < 0.0
                };

                if overlaps {
                    let (a, b) = if self.drops[i].mass >= self.drops[j].mass {
                        (i, j)
                    } else {
                        (j, i)
                    };
                    let total = self.drops[a].mass + self.drops[b].mass;
                    let mass_a = self.drops[a].mass;
                    let mass_b = self.drops[b].mass;
                    self.drops[a].x = (self.drops[a].x * mass_a + self.drops[b].x * mass_b) / total;
                    self.drops[a].y = (self.drops[a].y * mass_a + self.drops[b].y * mass_b) / total;
                    self.drops[a].vx =
                        (self.drops[a].vx * mass_a + self.drops[b].vx * mass_b) / total;
                    self.drops[a].vy =
                        (self.drops[a].vy * mass_a + self.drops[b].vy * mass_b) / total;
                    self.drops[a].mass = total;
                    self.drops[a].visual_mass = self.drops[a].visual_mass.max(mass_a);
                    self.drops[a].initial_mass = self.drops[a].initial_mass.max(total);
                    self.drops[a].spread_y = self.drops[a].spread_y.max(0.85);
                    self.drops[a].spread_x = self.drops[a].spread_x.max(0.20);
                    self.drops[a].vy += 35.0;
                    self.drops[b].destroyed = true;
                }
            }
        }
    }

    fn enhance_water_streaks(&mut self, cfg: RainGlass2d, dt: f32) {
        if !cfg.trails_enabled || cfg.streak_boost <= 0.01 {
            self.update_reference_streak_children(cfg, dt);
            return;
        }

        let limit = effective_drop_limit(cfg);
        let base_len = self.drops.len();
        let mut additions = Vec::new();

        for index in 0..base_len {
            if self.drops.len() + additions.len() >= limit {
                break;
            }
            if self.drops[index].destroyed
                || self.drops[index].parent_id.is_some()
                || self.drops[index].kind != RainGlassDropKind::Main
            {
                continue;
            }

            let speed = self.drops[index].vy.abs();
            if speed < 95.0 || self.drops[index].mass < 900.0 {
                continue;
            }

            self.drops[index].streak_emit += dt * cfg.streak_boost * (1.18 + speed / 320.0);
            while self.drops[index].streak_emit >= 1.0 && self.drops.len() + additions.len() < limit
            {
                self.drops[index].streak_emit -= 1.0;

                let parent_id = self.drops[index].id;
                let parent_x = self.drops[index].x;
                let parent_y = self.drops[index].y;
                let parent_size_x = self.drops[index].visible_size_x(cfg);
                let parent_size_y = self.drops[index].visible_size_y(cfg);
                let parent_vx = self.drops[index].vx;
                let parent_vy = self.drops[index].vy;

                let x = parent_x + (self.rng.next01() - 0.5) * parent_size_x * 0.44;
                let y = parent_y + parent_size_y * (0.12 + self.rng.next01() * 0.72);
                let radius = parent_size_x
                    * (0.18 + self.rng.next01() * 0.34)
                    * (0.95 + cfg.streak_boost * 0.90);
                let density = (reference_raw_trail_density(cfg) * 1.08).max(0.05);

                let mut trail = RainGlassDrop::new(
                    self.next_id,
                    x,
                    y,
                    radius.max(1.0),
                    density,
                    RainGlassDropKind::Trail,
                    &mut self.rng,
                    cfg,
                );
                self.next_id += 1;

                trail.parent_id = Some(parent_id);
                trail.reference_streak_child = true;
                trail.streak_age = 0.0;
                trail.streak_mass0 = trail.mass;
                trail.streak_seed = self.rng.next01();
                trail.vy = parent_vy * (0.08 + self.rng.next01() * 0.18);
                trail.vx = parent_vx * 0.18 + (self.rng.next01() - 0.5) * 24.0;
                trail.spread_x = 0.14 + self.rng.next01() * 0.22;
                trail.spread_y = trail.spread_y.max(
                    cfg.streak_length * (0.95 + speed * 0.0060) * (0.9 + self.rng.next01() * 0.9),
                );

                self.drops[index].mass = (self.drops[index].mass - trail.mass * 0.24).max(1.0);
                additions.push(trail);
            }
        }

        self.drops.extend(additions);
        self.update_reference_streak_children(cfg, dt);
    }

    fn update_reference_streak_children(&mut self, cfg: RainGlass2d, dt: f32) {
        let parent_velocities = self
            .drops
            .iter()
            .map(|drop| (drop.id, drop.vy))
            .collect::<Vec<_>>();

        for drop in &mut self.drops {
            if drop.destroyed || !drop.reference_streak_child {
                continue;
            }

            drop.streak_age += dt;
            if drop.streak_mass0 <= 0.0 {
                drop.streak_mass0 = drop.mass;
            }

            let parent_vy = drop
                .parent_id
                .and_then(|id| {
                    parent_velocities
                        .iter()
                        .find(|(parent_id, _)| *parent_id == id)
                        .map(|(_, vy)| *vy)
                })
                .unwrap_or(drop.vy);
            let speed = drop.vy.abs().max(parent_vy.abs());
            let life01 = (drop.streak_age / (1.8 + cfg.streak_length * 1.3)).min(1.0);

            drop.spread_x = (drop.spread_x * (0.74 + cfg.shrink_rate * 0.2).powf(dt)).max(0.075);
            let target_y =
                cfg.streak_length * (0.62 + speed * 0.0036) * (1.0 - life01 * cfg.trail_taper * 0.72);
            drop.spread_y = (drop.spread_y * (0.82 + cfg.shrink_rate * 0.15).powf(dt))
                .max(target_y)
                .max(0.18);

            let mass_target =
                drop.streak_mass0 * (1.0 - life01 * cfg.trail_taper).max(0.02).powf(2.0);
            drop.mass = (drop.mass - cfg.trail_evaporate * 0.72 * dt)
                .min(mass_target)
                .max(0.01);
            if drop.mass < 8.0 || life01 >= 0.995 {
                drop.destroyed = true;
            }
        }
    }

    fn destroy_out_of_bounds(&mut self, width: f32, height: f32) {
        self.drops.retain(|drop| {
            !drop.destroyed
                && drop.mass > 2.0
                && drop.x > -256.0
                && drop.x < width + 256.0
                && drop.y < height + 256.0
        });
    }

    fn limit_drop_count(&mut self, limit: usize) {
        let max_drops = limit.saturating_mul(2).max(limit).max(1);
        if self.drops.len() <= max_drops {
            return;
        }
        let remove = self.drops.len() - max_drops;
        self.drops.drain(0..remove);
    }

    fn limit_trail_segments(&mut self, limit: usize) {
        let max_segments = (limit * 2).clamp(64, 1800);

        if self.trail_segments.len() <= max_segments {
            return;
        }

        let remove = self.trail_segments.len() - max_segments;
        self.trail_segments.drain(0..remove);
    }

    fn spawn_drop(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        density: f32,
        kind: RainGlassDropKind,
        cfg: RainGlass2d,
    ) {
        let drop = RainGlassDrop::new(
            self.next_id,
            x,
            y,
            radius,
            density,
            kind,
            &mut self.rng,
            cfg,
        );
        self.next_id += 1;
        self.drops.push(drop);
    }

    fn spawn_microdrop(&mut self, x: f32, y: f32, radius: f32) {
        let mut drop = RainGlassDrop::micro(self.next_id, x, y, radius);
        self.next_id += 1;
        drop.seed = self.rng.next01();
        self.microdrops.push(drop);
    }

    #[cfg(test)]
    fn clear_for_test(&mut self) {
        self.drops.clear();
        self.microdrops.clear();
        self.trail_segments.clear();
        self.spawn_accumulator = 0.0;
        self.micro_spawn_accumulator = 0.0;
        self.micro_preseeded = false;
    }

    #[cfg(test)]
    fn push_for_test(&mut self, drop: RainGlassDrop) {
        self.drops.push(drop);
    }

    #[cfg(test)]
    fn drops_for_test(&self) -> &[RainGlassDrop] {
        &self.drops
    }
}

fn reference_merge_distance(drop: &RainGlassDrop, cfg: RainGlass2d) -> f32 {
    if cfg.reference_mode {
        drop.visible_size_x(cfg)
            .max(drop.visible_size_y(cfg) * 0.55)
            * 0.36
            * cfg.collider_scale
    } else {
        drop.size_x() * (1.0 + drop.spread_x) * 0.16 * cfg.collider_scale
    }
}

fn effective_drop_limit(cfg: RainGlass2d) -> usize {
    if cfg.reference_mode && cfg.trails_enabled && cfg.streak_boost > 0.01 {
        let streak_budget =
            (cfg.spawn_limit as f32 * cfg.streak_boost * 0.65).clamp(32.0, 900.0) as usize;
        cfg.spawn_limit as usize + streak_budget
    } else {
        cfg.spawn_limit as usize
    }
}

fn reference_raw_trail_density(cfg: RainGlass2d) -> f32 {
    cfg.trail_drop_density / (1.0 + cfg.streak_boost * 0.85).max(0.001)
}

impl RainGlassDrop {
    fn new(
        id: u64,
        x: f32,
        y: f32,
        radius: f32,
        density: f32,
        kind: RainGlassDropKind,
        rng: &mut RainGlassRng,
        cfg: RainGlass2d,
    ) -> Self {
        let mass = (radius * density.max(0.001)).powi(2);
        Self {
            id,
            parent_id: None,
            x,
            y,
            prev_x: x,
            prev_y: y,
            vx: rng.range(-12.0, 12.0),
            vy: rng.range(0.0, 34.0),
            mass,
            visual_mass: mass,
            initial_mass: mass,
            density: density.max(0.001),
            spread_x: cfg.initial_spread * rng.range(0.3, 1.0),
            spread_y: cfg.initial_spread * rng.range(0.5, 1.2),
            resistance: rng.range(0.0, 0.25) * mass,
            shifting: rng.range(cfg.x_shift_min, cfg.x_shift_max)
                * if rng.next01() < 0.5 { -1.0 } else { 1.0 },
            last_trail_x: x,
            last_trail_y: y,
            next_trail_distance: rng.range(cfg.trail_distance_min_px, cfg.trail_distance_max_px),
            next_motion_time: rng.range(cfg.motion_interval_min, cfg.motion_interval_max),
            streak_emit: 0.0,
            streak_age: 0.0,
            streak_mass0: mass,
            streak_seed: rng.next01(),
            reference_streak_child: false,
            age: 0.0,
            seed: rng.next01(),
            kind,
            destroyed: false,
        }
    }

    fn micro(id: u64, x: f32, y: f32, radius: f32) -> Self {
        Self {
            id,
            parent_id: None,
            x,
            y,
            prev_x: x,
            prev_y: y,
            vx: 0.0,
            vy: 0.0,
            mass: radius * radius,
            visual_mass: radius * radius,
            initial_mass: radius * radius,
            density: 1.0,
            spread_x: 0.0,
            spread_y: 0.0,
            resistance: 0.0,
            shifting: 0.0,
            last_trail_x: x,
            last_trail_y: y,
            next_trail_distance: 9999.0,
            next_motion_time: 9999.0,
            streak_emit: 0.0,
            streak_age: 0.0,
            streak_mass0: radius * radius,
            streak_seed: 0.0,
            reference_streak_child: false,
            age: 0.0,
            seed: 0.0,
            kind: RainGlassDropKind::Micro,
            destroyed: false,
        }
    }
}

#[derive(Clone, Copy)]
struct RainGlassRng {
    state: u32,
}

impl Default for RainGlassRng {
    fn default() -> Self {
        Self::new(1)
    }
}

impl RainGlassRng {
    fn new(seed: u32) -> Self {
        Self { state: seed.max(1) }
    }

    fn next01(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state as f32 / u32::MAX as f32
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next01()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_main_drops_respects_spawn_rate() {
        let cfg = RainGlass2d {
            spawn_rate: 10.0,
            spawn_limit: 4,
            gravity_px_per_sec2: 0.0,
            trails_enabled: false,
            ..RainGlass2d::default()
        }
        .normalized();
        let mut sim = RainGlassSimulation::new(1, 800.0, 600.0);
        sim.clear_for_test();
        sim.update(cfg, 0.4, 800.0, 600.0);
        assert!(
            (3..=4).contains(&sim.drops_for_test().len()),
            "reference spawn interval jitter should produce a bounded spawn count"
        );
    }

    #[test]
    fn gravity_moves_main_drop_down() {
        let cfg = RainGlass2d {
            spawn_rate: 0.0,
            gravity_px_per_sec2: 1000.0,
            ..RainGlass2d::default()
        }
        .normalized();
        let mut sim = RainGlassSimulation::new(1, 800.0, 600.0);
        sim.clear_for_test();
        let mut drop = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        drop.vy = 300.0;
        drop.mass = 1000.0;
        drop.next_motion_time = 10.0;
        sim.push_for_test(drop);
        sim.update(cfg, 0.5, 800.0, 600.0);
        assert!(sim
            .drops_for_test()
            .iter()
            .any(|drop| drop.kind == RainGlassDropKind::Main && drop.y > 100.0));
    }

    #[test]
    fn trail_split_creates_trail_drop() {
        let cfg = RainGlass2d {
            spawn_rate: 0.0,
            trails_enabled: true,
            ..RainGlass2d::default()
        }
        .normalized();
        let mut sim = RainGlassSimulation::new(1, 800.0, 600.0);
        sim.clear_for_test();
        let mut drop = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        drop.mass = 2500.0;
        drop.last_trail_y = 0.0;
        drop.next_trail_distance = 4.0;
        sim.push_for_test(drop);
        sim.update(cfg, 0.016, 800.0, 600.0);
        assert!(sim
            .drops_for_test()
            .iter()
            .any(|drop| drop.kind == RainGlassDropKind::Trail));
    }

    #[test]
    fn trail_split_uses_visible_density_for_cinematic_sizes() {
        let cfg = RainGlass2d {
            trails_enabled: true,
            gravity_px_per_sec2: 0.0,
            min_radius_px: 12.0,
            max_radius_px: 66.0,
            trail_drop_density: 0.72,
            trail_drop_size_min: 0.32,
            trail_drop_size_max: 0.58,
            trail_distance_min_px: 4.0,
            trail_distance_max_px: 8.0,
            trail_spread: 1.45,
            ..RainGlass2d::default()
        }
        .normalized();

        let mut sim = RainGlassSimulation::new(7, 800.0, 600.0);
        sim.clear_for_test();

        let mut drop = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        drop.mass = 2200.0;
        drop.initial_mass = 2200.0;
        drop.vy = 0.0;
        drop.last_trail_x = drop.x;
        drop.last_trail_y = drop.y - 32.0;
        drop.next_trail_distance = 4.0;
        sim.push_for_test(drop);

        sim.update(cfg, 1.0 / 60.0, 800.0, 600.0);

        assert!(sim
            .drops_for_test()
            .iter()
            .any(|drop| drop.kind == RainGlassDropKind::Trail));
    }

    #[test]
    fn moving_main_drop_emits_reference_child_trails() {
        let cfg = RainGlass2d {
            spawn_rate: 0.0,
            reference_mode: true,
            trails_enabled: true,
            streak_boost: 1.0,
            streak_length: 1.15,
            trail_opacity: 1.0,
            trail_spread: 1.4,
            trail_drop_density: 0.3224,
            trail_drop_size_min: 0.2436,
            trail_drop_size_max: 0.5124,
            trail_distance_min_px: 3.0,
            trail_distance_max_px: 8.0,
            gravity_px_per_sec2: 0.0,
            ..RainGlass2d::default()
        }
        .normalized();

        let mut sim = RainGlassSimulation::new(1, 800.0, 600.0);
        sim.clear_for_test();

        let mut drop = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        drop.mass = 2200.0;
        drop.initial_mass = 2200.0;
        drop.vy = 300.0;
        drop.prev_y = drop.y - 16.0;
        drop.last_trail_y = drop.y - 32.0;
        sim.push_for_test(drop);

        sim.update(cfg, 1.0, 800.0, 600.0);

        assert!(
            sim.drops_for_test()
                .iter()
                .any(|drop| drop.kind == RainGlassDropKind::Trail),
            "moving main drops should emit physical child trail drops"
        );
        assert!(
            !sim.trail_instances(cfg).is_empty(),
            "reference mode should render wet streak ribbon instances into the persistent streak map"
        );
        assert!(
            sim.live_instances(cfg).len() >= 2,
            "reference child trail drops should be routed through raindrop map"
        );
    }

    #[test]
    fn trail_taper_shrinks_spread() {
        let cfg = RainGlass2d {
            trail_taper: 0.8,
            trail_shrink_rate: 0.8,
            trail_evaporate: 10.0,
            spawn_rate: 0.0,
            gravity_px_per_sec2: 0.0,
            trails_enabled: false,
            ..RainGlass2d::default()
        }
        .normalized();
        let mut sim = RainGlassSimulation::new(1, 800.0, 600.0);
        sim.clear_for_test();
        let mut trail = RainGlassDrop::new_for_test(RainGlassDropKind::Trail);
        trail.spread_x = 1.0;
        trail.spread_y = 2.0;
        trail.mass = 400.0;
        sim.push_for_test(trail);
        sim.update(cfg, 1.0, 800.0, 600.0);
        let drop = &sim.drops_for_test()[0];
        assert!(drop.spread_x < 1.0);
        assert!(drop.spread_y < 2.0);
        assert!(drop.mass < 400.0);
    }

    #[test]
    fn merge_collisions_combines_mass() {
        let cfg = RainGlass2d {
            spawn_rate: 0.0,
            collider_scale: 4.0,
            gravity_px_per_sec2: 0.0,
            trails_enabled: false,
            ..RainGlass2d::default()
        }
        .normalized();
        let mut sim = RainGlassSimulation::new(1, 800.0, 600.0);
        sim.clear_for_test();
        let mut a = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        let mut b = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        a.mass = 1000.0;
        b.mass = 500.0;
        b.id = 2;
        b.x = a.x + 1.0;
        sim.push_for_test(a);
        sim.push_for_test(b);
        sim.update(cfg, 0.0, 800.0, 600.0);
        assert_eq!(sim.drops_for_test().len(), 1);
        assert!(sim.drops_for_test()[0].mass >= 1500.0);
    }

    #[test]
    fn sliding_drops_merge_with_visual_overlap() {
        let cfg = RainGlass2d {
            spawn_rate: 0.0,
            collider_scale: 1.65,
            gravity_px_per_sec2: 0.0,
            trails_enabled: false,
            ..RainGlass2d::default()
        }
        .normalized();

        let mut sim = RainGlassSimulation::new(1, 800.0, 600.0);
        sim.clear_for_test();

        let mut a = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        let mut b = RainGlassDrop::new_for_test(RainGlassDropKind::Main);

        a.mass = 900.0;
        a.initial_mass = 900.0;
        a.spread_x = 0.4;
        a.spread_y = 1.0;
        a.vy = 320.0;

        b.id = 2;
        b.mass = 500.0;
        b.initial_mass = 500.0;
        b.x = a.x + a.size_x() * 0.35;
        b.y = a.y + a.size_y() * 0.15;

        sim.push_for_test(a);
        sim.push_for_test(b);

        sim.update(cfg, 0.0, 800.0, 600.0);

        assert_eq!(sim.drops_for_test().len(), 1);
        assert!(sim.drops_for_test()[0].mass >= 1400.0);
    }

    #[test]
    fn limit_drop_count_removes_oldest_when_far_above_limit() {
        let cfg = RainGlass2d {
            spawn_rate: 0.0,
            spawn_limit: 1,
            trails_enabled: false,
            ..RainGlass2d::default()
        }
        .normalized();
        let mut sim = RainGlassSimulation::new(1, 800.0, 600.0);
        sim.clear_for_test();
        let a = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        let mut b = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        let mut c = RainGlassDrop::new_for_test(RainGlassDropKind::Main);
        b.id = 2;
        b.x = 300.0;
        c.id = 3;
        c.x = 500.0;
        sim.push_for_test(a);
        sim.push_for_test(b);
        sim.push_for_test(c);
        sim.update(cfg, 0.0, 800.0, 600.0);
        assert_eq!(sim.drops_for_test().len(), 2);
        assert_eq!(sim.drops_for_test()[0].id, 2);
    }
}
