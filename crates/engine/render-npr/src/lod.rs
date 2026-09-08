//! Stateful, hysteretic level-of-detail for tonal paths.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HatchLodPolicy {
    /// Pixel extents at which progressively finer tiers enter.
    pub enter_thresholds: [f32; 3],
    /// Fractional gap between entering and leaving a tier.
    pub hysteresis: f32,
    /// Multipliers for authored pixel spacing, from smallest to largest tier.
    pub spacing_multipliers: [f32; 4],
}

impl Default for HatchLodPolicy {
    fn default() -> Self {
        Self {
            enter_thresholds: [48.0, 96.0, 180.0],
            hysteresis: 0.15,
            // Normal-sized drawings retain their authored spacing. Only small
            // projected objects shed tonal detail.
            spacing_multipliers: [2.0, 1.5, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HatchLodDecision {
    pub tier: u8,
    pub spacing_multiplier: f32,
}

#[derive(Debug, Default)]
pub struct HatchLodState {
    tier: Option<u8>,
}

impl HatchLodState {
    pub fn reset(&mut self) {
        self.tier = None;
    }

    /// Advances using projected extent in pixels. The first observation picks
    /// the natural tier; subsequent movement must cross a separate leave/enter
    /// boundary, preventing zoom jitter around a threshold.
    pub fn advance(&mut self, projected_extent: f32, policy: HatchLodPolicy) -> HatchLodDecision {
        let extent = projected_extent.max(0.0);
        let hysteresis = policy.hysteresis.clamp(0.0, 0.9);
        let mut tier = self.tier.unwrap_or_else(|| desired_tier(extent, policy));
        if self.tier.is_some() {
            while tier < 3 && extent >= policy.enter_thresholds[tier as usize] * (1.0 + hysteresis)
            {
                tier += 1;
            }
            while tier > 0
                && extent < policy.enter_thresholds[tier as usize - 1] * (1.0 - hysteresis)
            {
                tier -= 1;
            }
        }
        self.tier = Some(tier);
        HatchLodDecision {
            tier,
            spacing_multiplier: policy.spacing_multipliers[tier as usize].max(0.01),
        }
    }
}

fn desired_tier(extent: f32, policy: HatchLodPolicy) -> u8 {
    policy
        .enter_thresholds
        .iter()
        .take_while(|threshold| extent >= **threshold)
        .count() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_have_a_stable_hysteresis_band() {
        let policy = HatchLodPolicy {
            enter_thresholds: [100.0, 200.0, 300.0],
            hysteresis: 0.1,
            spacing_multipliers: [2.0, 1.5, 1.0, 0.8],
        };
        let mut state = HatchLodState::default();
        assert_eq!(state.advance(150.0, policy).tier, 1);
        assert_eq!(state.advance(95.0, policy).tier, 1);
        assert_eq!(state.advance(89.0, policy).tier, 0);
        assert_eq!(state.advance(109.0, policy).tier, 0);
        assert_eq!(state.advance(110.0, policy).tier, 1);
    }

    #[test]
    fn first_observation_uses_the_natural_tier() {
        let mut state = HatchLodState::default();
        let decision = state.advance(190.0, HatchLodPolicy::default());
        assert_eq!(decision.tier, 3);
        assert_eq!(decision.spacing_multiplier, 1.0);
    }
}
