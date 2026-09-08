use std::sync::mpsc;
use std::time::Duration;

use amigo_runtime::{
    EngineJob, EngineLane, EngineSchedulerMode, EngineSchedulingConfig, EngineTaskSystem,
    JobContext,
};

fn worker_config(workers: usize) -> EngineSchedulingConfig {
    EngineSchedulingConfig {
        mode: EngineSchedulerMode::Hybrid,
        max_workers: workers,
        deterministic: true,
        allow_frame_latency: true,
    }
}

#[test]
fn persistent_pool_drains_one_thousand_detached_jobs() {
    let system = EngineTaskSystem::new(worker_config(4));
    let (tx, rx) = mpsc::channel();
    for value in 0usize..1_000 {
        let tx = tx.clone();
        assert!(system.spawn_detached(move || {
            tx.send(value).expect("stress receiver must remain alive");
        }));
    }
    drop(tx);

    let mut values = Vec::with_capacity(1_000);
    for _ in 0..1_000 {
        values.push(
            rx.recv_timeout(Duration::from_secs(5))
                .expect("worker pool should drain stress queue"),
        );
    }
    values.sort_unstable();
    assert_eq!(values, (0usize..1_000).collect::<Vec<_>>());
}

struct ContextJob;

impl EngineJob for ContextJob {
    type Output = JobContext;

    fn name(&self) -> &'static str {
        "context-stress"
    }
    fn lane(&self) -> EngineLane {
        EngineLane::Simulation
    }
    fn run(self, ctx: JobContext) -> Self::Output {
        ctx
    }
}

#[test]
fn blocking_jobs_preserve_frame_and_declared_lane_under_load() {
    let system = EngineTaskSystem::new(worker_config(4));
    for frame_index in 0..512u64 {
        let context = system.run(
            ContextJob,
            JobContext {
                frame_index,
                lane: EngineLane::Main,
            },
        );
        assert_eq!(context.frame_index, frame_index);
        assert_eq!(context.lane, EngineLane::Simulation);
    }
}

#[test]
fn worker_count_can_scale_without_losing_jobs() {
    let system = EngineTaskSystem::new(worker_config(1));
    system.ensure_workers(1);
    system.set_config(worker_config(8));
    system.ensure_workers(8);

    let (tx, rx) = mpsc::channel();
    for _ in 0..256 {
        let tx = tx.clone();
        assert!(system.spawn_detached(move || {
            tx.send(()).unwrap();
        }));
    }
    drop(tx);
    for _ in 0..256 {
        rx.recv_timeout(Duration::from_secs(5))
            .expect("scaled pool should complete all jobs");
    }
}
