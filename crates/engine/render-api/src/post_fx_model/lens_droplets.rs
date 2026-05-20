use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensDroplets2dStage {
    AfterWorldBeforeUi,
}

#[derive(Debug, Clone, PartialEq)]
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
