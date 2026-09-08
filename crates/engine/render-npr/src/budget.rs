//! Deterministic pre-allocation quality selection for NPR candidates.

/// A candidate's priority and stable tie-breaker. The renderer never needs to
/// infer why it was accepted; the domain has already made that decision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RankedCandidate {
    pub priority: f32,
    pub stable_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BudgetReport {
    pub accepted: usize,
    pub rejected: usize,
}

/// Selects the highest-priority candidates before any backend allocation.
/// Equal priorities use a stable ID rather than face iteration order.
pub fn select_ranked<T>(
    limit: usize,
    mut candidates: Vec<(RankedCandidate, T)>,
) -> (Vec<T>, BudgetReport) {
    candidates.sort_by(|(left, _), (right, _)| {
        let left_priority = if left.priority.is_finite() {
            left.priority
        } else {
            f32::NEG_INFINITY
        };
        let right_priority = if right.priority.is_finite() {
            right.priority
        } else {
            f32::NEG_INFINITY
        };
        right_priority
            .total_cmp(&left_priority)
            .then_with(|| left.stable_id.cmp(&right.stable_id))
    });
    let accepted = candidates.len().min(limit);
    let rejected = candidates.len() - accepted;
    (
        candidates
            .into_iter()
            .take(accepted)
            .map(|(_, candidate)| candidate)
            .collect(),
        BudgetReport { accepted, rejected },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_by_priority_then_stable_id() {
        let (selected, report) = select_ranked(
            2,
            vec![
                (
                    RankedCandidate {
                        priority: 0.5,
                        stable_id: 8,
                    },
                    "later",
                ),
                (
                    RankedCandidate {
                        priority: 0.8,
                        stable_id: 9,
                    },
                    "high",
                ),
                (
                    RankedCandidate {
                        priority: 0.5,
                        stable_id: 2,
                    },
                    "early",
                ),
            ],
        );
        assert_eq!(selected, ["high", "early"]);
        assert_eq!(
            report,
            BudgetReport {
                accepted: 2,
                rejected: 1
            }
        );
    }

    #[test]
    fn zero_limit_rejects_without_losing_accounting() {
        let (selected, report) = select_ranked(
            0,
            vec![(
                RankedCandidate {
                    priority: 1.0,
                    stable_id: 1,
                },
                (),
            )],
        );
        assert!(selected.is_empty());
        assert_eq!(
            report,
            BudgetReport {
                accepted: 0,
                rejected: 1
            }
        );
    }
}
