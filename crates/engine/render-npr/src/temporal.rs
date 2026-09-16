//! Temporal identity for hand-drawn rendering.
//!
//! Path redraw and material boil are intentionally separate. A static building
//! can keep its geometry forever while graphite deposition changes subtly; an
//! animated character can redraw its path without forcing every material mark
//! in the scene to reroll.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprPathMotionMode {
    Stable,
    RedrawOnMotion,
    RedrawContinuously,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprMaterialMotionMode {
    Stable,
    Boil,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprMotionPolicy {
    pub path_mode: NprPathMotionMode,
    pub path_redraw_hz: f32,
    pub material_mode: NprMaterialMotionMode,
    pub material_redraw_hz: f32,
}

impl Default for NprMotionPolicy {
    fn default() -> Self {
        Self {
            path_mode: NprPathMotionMode::Stable,
            path_redraw_hz: 0.0,
            material_mode: NprMaterialMotionMode::Stable,
            material_redraw_hz: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NprTemporalState {
    pub path_epoch: u64,
    pub material_epoch: u64,
}

impl NprTemporalState {
    pub fn at_time(policy: NprMotionPolicy, seconds: f32, surface_moved: bool) -> Self {
        let epoch = |hz: f32| (seconds.max(0.0) * hz.max(0.0)).floor() as u64;
        let path_epoch = match policy.path_mode {
            NprPathMotionMode::Stable => 0,
            NprPathMotionMode::RedrawOnMotion if !surface_moved => 0,
            NprPathMotionMode::RedrawOnMotion | NprPathMotionMode::RedrawContinuously => epoch(policy.path_redraw_hz),
        };
        let material_epoch = match policy.material_mode {
            NprMaterialMotionMode::Stable => 0,
            NprMaterialMotionMode::Boil => epoch(policy.material_redraw_hz),
        };
        Self { path_epoch, material_epoch }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_paths_do_not_reroll_when_only_material_boils() {
        let state = NprTemporalState::at_time(NprMotionPolicy {
            material_mode: NprMaterialMotionMode::Boil,
            material_redraw_hz: 8.0,
            ..Default::default()
        }, 1.25, false);
        assert_eq!(state.path_epoch, 0);
        assert_eq!(state.material_epoch, 10);
    }

    #[test]
    fn redraw_on_motion_waits_for_motion() {
        let policy = NprMotionPolicy { path_mode: NprPathMotionMode::RedrawOnMotion, path_redraw_hz: 8.0, ..Default::default() };
        assert_eq!(NprTemporalState::at_time(policy, 1.0, false).path_epoch, 0);
        assert_eq!(NprTemporalState::at_time(policy, 1.0, true).path_epoch, 8);
    }
}
