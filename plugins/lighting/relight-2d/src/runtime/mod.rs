use crate::api::{Relight2dCandidate, Relight2dContribution};

pub fn resolve_relight_2d_candidate(
    contributions: Vec<Relight2dContribution>,
) -> Relight2dCandidate {
    Relight2dCandidate::from_contributions(contributions)
}
