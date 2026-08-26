use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PerformanceMetric {
    StartupMillis,
    SceneHydrationMillis,
    FrameCpuMillis,
    RenderExtractionMillis,
    GpuPasses,
    DrawCalls,
    AllocationsPerFrame,
    SchedulerUtilizationPercent,
}

impl PerformanceMetric {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StartupMillis => "startup_ms",
            Self::SceneHydrationMillis => "scene_hydration_ms",
            Self::FrameCpuMillis => "frame_cpu_ms",
            Self::RenderExtractionMillis => "render_extraction_ms",
            Self::GpuPasses => "gpu_passes",
            Self::DrawCalls => "draw_calls",
            Self::AllocationsPerFrame => "allocations_per_frame",
            Self::SchedulerUtilizationPercent => "scheduler_utilization_percent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceDirection {
    LowerIsBetter,
    HigherIsBetter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerformanceBudget {
    pub direction: PerformanceDirection,
    pub max_regression_percent: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PerformanceSnapshot {
    values: BTreeMap<PerformanceMetric, f64>,
}

impl PerformanceSnapshot {
    pub fn record(&mut self, metric: PerformanceMetric, value: f64) {
        assert!(value.is_finite() && value >= 0.0, "performance samples must be finite and non-negative");
        self.values.insert(metric, value);
    }

    pub fn value(&self, metric: PerformanceMetric) -> Option<f64> {
        self.values.get(&metric).copied()
    }

    pub fn values(&self) -> impl Iterator<Item = (PerformanceMetric, f64)> + '_ {
        self.values.iter().map(|(metric, value)| (*metric, *value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceRegression {
    pub metric: PerformanceMetric,
    pub baseline: f64,
    pub observed: f64,
    pub regression_percent: f64,
    pub budget_percent: f64,
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceBaselineSet {
    baseline: PerformanceSnapshot,
    budgets: BTreeMap<PerformanceMetric, PerformanceBudget>,
}

impl PerformanceBaselineSet {
    pub fn new(baseline: PerformanceSnapshot) -> Self {
        Self {
            baseline,
            budgets: default_performance_budgets(),
        }
    }

    pub fn with_budget(mut self, metric: PerformanceMetric, budget: PerformanceBudget) -> Self {
        self.budgets.insert(metric, budget);
        self
    }

    pub fn baseline(&self) -> &PerformanceSnapshot { &self.baseline }

    pub fn evaluate(&self, observed: &PerformanceSnapshot) -> Vec<PerformanceRegression> {
        let mut regressions = Vec::new();
        for (metric, baseline) in self.baseline.values() {
            let Some(observed) = observed.value(metric) else { continue; };
            let budget = self.budgets.get(&metric).copied().unwrap_or(PerformanceBudget {
                direction: PerformanceDirection::LowerIsBetter,
                max_regression_percent: 10.0,
            });
            let regression_percent = regression_percent(baseline, observed, budget.direction);
            if regression_percent > budget.max_regression_percent {
                regressions.push(PerformanceRegression {
                    metric,
                    baseline,
                    observed,
                    regression_percent,
                    budget_percent: budget.max_regression_percent,
                });
            }
        }
        regressions
    }
}

pub fn default_performance_budgets() -> BTreeMap<PerformanceMetric, PerformanceBudget> {
    use PerformanceDirection::{HigherIsBetter, LowerIsBetter};
    use PerformanceMetric::*;
    [
        (StartupMillis, PerformanceBudget { direction: LowerIsBetter, max_regression_percent: 15.0 }),
        (SceneHydrationMillis, PerformanceBudget { direction: LowerIsBetter, max_regression_percent: 10.0 }),
        (FrameCpuMillis, PerformanceBudget { direction: LowerIsBetter, max_regression_percent: 5.0 }),
        (RenderExtractionMillis, PerformanceBudget { direction: LowerIsBetter, max_regression_percent: 5.0 }),
        (GpuPasses, PerformanceBudget { direction: LowerIsBetter, max_regression_percent: 10.0 }),
        (DrawCalls, PerformanceBudget { direction: LowerIsBetter, max_regression_percent: 10.0 }),
        (AllocationsPerFrame, PerformanceBudget { direction: LowerIsBetter, max_regression_percent: 5.0 }),
        (SchedulerUtilizationPercent, PerformanceBudget { direction: HigherIsBetter, max_regression_percent: 10.0 }),
    ]
    .into_iter()
    .collect()
}

fn regression_percent(baseline: f64, observed: f64, direction: PerformanceDirection) -> f64 {
    if baseline == 0.0 {
        return if observed == 0.0 { 0.0 } else { f64::INFINITY };
    }
    match direction {
        PerformanceDirection::LowerIsBetter => ((observed - baseline) / baseline * 100.0).max(0.0),
        PerformanceDirection::HigherIsBetter => ((baseline - observed) / baseline * 100.0).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_frame_cpu_regression_beyond_budget() {
        let mut baseline = PerformanceSnapshot::default();
        baseline.record(PerformanceMetric::FrameCpuMillis, 10.0);
        let baselines = PerformanceBaselineSet::new(baseline);
        let mut observed = PerformanceSnapshot::default();
        observed.record(PerformanceMetric::FrameCpuMillis, 11.0);
        let regressions = baselines.evaluate(&observed);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].metric, PerformanceMetric::FrameCpuMillis);
    }

    #[test]
    fn treats_scheduler_utilization_as_higher_is_better() {
        let mut baseline = PerformanceSnapshot::default();
        baseline.record(PerformanceMetric::SchedulerUtilizationPercent, 80.0);
        let baselines = PerformanceBaselineSet::new(baseline);
        let mut observed = PerformanceSnapshot::default();
        observed.record(PerformanceMetric::SchedulerUtilizationPercent, 60.0);
        assert_eq!(baselines.evaluate(&observed).len(), 1);
    }
}
