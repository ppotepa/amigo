# Performance baselines

Amigo treats performance measurements as versioned data rather than informal observations. `amigo-session::PerformanceBaselineSet` defines the comparison contract and default regression budgets for:

- startup time;
- scene hydration time;
- frame CPU time;
- render extraction time;
- GPU pass count;
- draw-call count;
- allocations per frame;
- scheduler utilization.

Do not commit fabricated machine-specific baseline values. Capture a `PerformanceSnapshot` on controlled hardware/build settings, persist it with the benchmark artifact, and compare later captures using the same scene, resolution, build profile, GPU/driver and scheduler configuration. The default budgets are intentionally stricter for per-frame CPU/render/allocation metrics than for startup/pass-count metrics.

A baseline is only comparable when its environment metadata matches the observed capture. CI should publish captures even when it does not enforce them on heterogeneous hosted runners.
