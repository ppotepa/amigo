use crate::style::ComicInk;

/// A rendering-independent estimate of how much graphite a visible surface
/// needs. `mass` is deliberately not an opacity: it is additive material
/// budget, which lets a caller choose a first hatch family and an optional
/// crossing family without treating every dark area as a flat alpha fill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphiteTonePlan {
    pub mass: f32,
    pub primary_density: f32,
    pub primary_coverage: f32,
    pub cross_density: f32,
    pub cross_coverage: f32,
}

impl GraphiteTonePlan {
    pub const PAPER: Self = Self {
        mass: 0.0,
        primary_density: 0.0,
        primary_coverage: 0.0,
        cross_density: 0.0,
        cross_coverage: 0.0,
    };

    pub fn is_visible(self) -> bool {
        self.primary_density > 0.001
    }
}

/// Converts a Lambert form response into a target material mass. The mapping
/// follows the useful saturation law `reflectance = exp(-sigma * mass)`: each
/// additional pass darkens the paper less than the previous one. It avoids the
/// mechanical look created when a single "density" value simultaneously means
/// spacing, opacity and the number of hatch families.
pub fn plan_graphite_tone(shade: f32, style: ComicInk) -> GraphiteTonePlan {
    let requested_darkness =
        style.tone_density.clamp(0.0, 1.0) * ((0.5 - shade) / 1.25).clamp(0.0, 1.0);
    if requested_darkness <= 0.001 {
        return GraphiteTonePlan::PAPER;
    }

    // Graphite cannot turn a paper pixel into mathematically perfect black;
    // preserving a small paper response keeps repeated marks legible.
    let target_reflectance = (1.0 - requested_darkness * 0.86).clamp(0.14, 1.0);
    let sigma = 1.15;
    let mass = (-target_reflectance.ln() / sigma).max(0.0);
    let primary_density = (mass / 0.82).clamp(0.0, 1.0);
    let cross_density = ((mass - 0.72) / 0.82).clamp(0.0, 1.0);
    let primary_coverage = (0.30 + 0.70 * (1.0 - (-mass).exp())).clamp(0.0, 1.0);
    let cross_coverage = (0.18 + 0.52 * (1.0 - (-cross_density).exp())).clamp(0.0, 0.70);
    GraphiteTonePlan {
        mass,
        primary_density,
        primary_coverage,
        cross_density,
        cross_coverage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_receives_no_graphite() {
        assert_eq!(
            plan_graphite_tone(1.0, ComicInk::default()),
            GraphiteTonePlan::PAPER
        );
    }

    #[test]
    fn darker_form_receives_more_material() {
        let style = ComicInk {
            tone_density: 1.0,
            ..Default::default()
        };
        let light = plan_graphite_tone(0.35, style);
        let dark = plan_graphite_tone(-0.65, style);
        assert!(dark.mass > light.mass);
        assert!(dark.primary_density > light.primary_density);
        assert!(dark.cross_density > light.cross_density);
    }

    #[test]
    fn plan_never_exceeds_its_normalized_ranges() {
        let plan = plan_graphite_tone(
            -1.0,
            ComicInk {
                tone_density: 1.0,
                ..Default::default()
            },
        );
        assert!((0.0..=1.0).contains(&plan.primary_density));
        assert!((0.0..=1.0).contains(&plan.cross_density));
        assert!((0.0..=1.0).contains(&plan.primary_coverage));
        assert!((0.0..=0.70).contains(&plan.cross_coverage));
    }
}
