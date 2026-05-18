#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DepthCurve2d {
    Linear,
    #[default]
    Logarithmic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DepthSpace2d {
    pub near_m: f32,
    pub far_m: f32,
    pub curve: DepthCurve2d,
}

impl Default for DepthSpace2d {
    fn default() -> Self {
        Self {
            near_m: 1.0,
            far_m: 1500.0,
            curve: DepthCurve2d::Logarithmic,
        }
    }
}

impl DepthSpace2d {
    pub fn normalized(mut self) -> Self {
        if !self.near_m.is_finite() || self.near_m <= 0.0 {
            self.near_m = 1.0;
        }
        if !self.far_m.is_finite() || self.far_m <= self.near_m {
            self.far_m = (self.near_m + 1.0).max(1500.0);
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DepthSource2d {
    DepthMap,
    Distance { meters: f32 },
    ZDepth { value: f32 },
    Infinity,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedDepth2d {
    pub z_depth: f32,
    pub distance_m: Option<f32>,
}

pub fn distance_to_z_depth(distance_m: f32, space: DepthSpace2d) -> f32 {
    let space = space.normalized();
    let distance_m = if distance_m.is_finite() {
        distance_m.clamp(space.near_m, space.far_m)
    } else {
        space.far_m
    };

    let t = match space.curve {
        DepthCurve2d::Linear => {
            let span = (space.far_m - space.near_m).max(f32::EPSILON);
            (distance_m - space.near_m) / span
        }
        DepthCurve2d::Logarithmic => {
            let ratio = (space.far_m / space.near_m).max(1.0001);
            (distance_m / space.near_m).ln() / ratio.ln()
        }
    };

    1.0 - t.clamp(0.0, 1.0)
}

pub fn z_depth_to_camera_motion_scale(z_depth: f32) -> f32 {
    z_depth.clamp(0.0, 1.0)
}

pub fn resolve_depth_source(source: DepthSource2d, space: DepthSpace2d) -> ResolvedDepth2d {
    match source {
        DepthSource2d::DepthMap => ResolvedDepth2d {
            z_depth: 0.5,
            distance_m: None,
        },
        DepthSource2d::Distance { meters } => ResolvedDepth2d {
            z_depth: distance_to_z_depth(meters, space),
            distance_m: Some(meters),
        },
        DepthSource2d::ZDepth { value } => ResolvedDepth2d {
            z_depth: value.clamp(0.0, 1.0),
            distance_m: None,
        },
        DepthSource2d::Infinity => ResolvedDepth2d {
            z_depth: 0.0,
            distance_m: None,
        },
        DepthSource2d::Overlay => ResolvedDepth2d {
            z_depth: 1.0,
            distance_m: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.015
    }

    #[test]
    fn logarithmic_depth_maps_rotten_club_reference_distances() {
        let space = DepthSpace2d {
            near_m: 1.0,
            far_m: 1500.0,
            curve: DepthCurve2d::Logarithmic,
        };

        assert!(approx_eq(distance_to_z_depth(1.0, space), 1.00));
        assert!(approx_eq(distance_to_z_depth(6.0, space), 0.75));
        assert!(approx_eq(distance_to_z_depth(10.0, space), 0.69));
        assert!(approx_eq(distance_to_z_depth(75.0, space), 0.41));
        assert!(approx_eq(distance_to_z_depth(150.0, space), 0.31));
        assert!(approx_eq(distance_to_z_depth(1500.0, space), 0.00));
    }

    #[test]
    fn resolve_distance_preserves_authoring_distance_and_computes_z_depth() {
        let resolved = resolve_depth_source(
            DepthSource2d::Distance { meters: 75.0 },
            DepthSpace2d::default(),
        );
        assert_eq!(resolved.distance_m, Some(75.0));
        assert!(approx_eq(resolved.z_depth, 0.41));
    }

    #[test]
    fn infinity_resolves_to_far_plane_z_depth() {
        let resolved = resolve_depth_source(DepthSource2d::Infinity, DepthSpace2d::default());
        assert_eq!(resolved.z_depth, 0.0);
        assert_eq!(resolved.distance_m, None);
    }

    #[test]
    fn z_depth_camera_motion_scale_uses_near_as_full_motion_and_far_as_zero_motion() {
        assert_eq!(z_depth_to_camera_motion_scale(1.0), 1.0);
        assert_eq!(z_depth_to_camera_motion_scale(0.0), 0.0);
        assert_eq!(z_depth_to_camera_motion_scale(0.42), 0.42);
    }
}
