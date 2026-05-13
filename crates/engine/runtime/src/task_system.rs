use crate::{EngineLane, EngineSchedulingConfig};
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

type BoxedEngineJob = Box<dyn FnOnce() + Send + 'static>;

pub struct EngineTaskSystem {
    config: Mutex<EngineSchedulingConfig>,
    sender: Mutex<Option<Sender<BoxedEngineJob>>>,
    receiver: Mutex<Option<SharedReceiver>>,
    workers: Mutex<Vec<JoinHandle<()>>>,
}

impl EngineTaskSystem {
    pub fn new(config: EngineSchedulingConfig) -> Self {
        Self {
            config: Mutex::new(config),
            sender: Mutex::new(None),
            receiver: Mutex::new(None),
            workers: Mutex::new(Vec::new()),
        }
    }

    pub fn set_config(&self, config: EngineSchedulingConfig) {
        *self
            .config
            .lock()
            .expect("engine task config mutex should not be poisoned") = config;
    }

    pub fn config(&self) -> EngineSchedulingConfig {
        self.config
            .lock()
            .expect("engine task config mutex should not be poisoned")
            .clone()
    }

    pub fn run_inline<J>(&self, job: J, ctx: JobContext) -> J::Output
    where
        J: EngineJob,
    {
        let _ = self;
        job.run(ctx)
    }

    pub fn run<J>(&self, job: J, ctx: JobContext) -> J::Output
    where
        J: EngineJob,
    {
        let config = self.config();
        if config.max_workers == 0 {
            return self.run_inline(job, ctx);
        }
        let (tx, rx) = mpsc::sync_channel(1);
        let _ = thread::Builder::new()
            .name(format!("amigo-engine-blocking-{}", job.name()))
            .spawn(move || {
                let output = job.run(ctx);
                let _ = tx.send(output);
            })
            .expect("engine task blocking worker thread should spawn");
        rx.recv()
            .expect("engine task blocking worker should send job output")
    }

    pub fn ensure_workers(&self, workers: usize) {
        if workers == 0 {
            return;
        }

        let mut workers_guard = self
            .workers
            .lock()
            .expect("engine task workers mutex should not be poisoned");
        if workers_guard.len() >= workers {
            return;
        }

        let mut sender_guard = self
            .sender
            .lock()
            .expect("engine task sender mutex should not be poisoned");
        let mut receiver_guard = self
            .receiver
            .lock()
            .expect("engine task receiver mutex should not be poisoned");
        if sender_guard.is_none() {
            let (tx, rx) = mpsc::channel::<BoxedEngineJob>();
            let shared_rx = std::sync::Arc::new(Mutex::new(rx));
            *sender_guard = Some(tx);
            *receiver_guard = Some(shared_rx.clone());
            for index in workers_guard.len()..workers {
                workers_guard.push(spawn_worker(index, shared_rx.clone()));
            }
            return;
        }

        let receiver = receiver_guard
            .as_ref()
            .expect("engine task receiver should exist")
            .clone();
        for index in workers_guard.len()..workers {
            workers_guard.push(spawn_worker(index, receiver.clone()));
        }
    }

    pub fn spawn_detached(&self, job: impl FnOnce() + Send + 'static) -> bool {
        let config = self.config();
        if config.max_workers == 0 {
            return false;
        }

        self.ensure_workers(config.max_workers);

        let Some(sender) = self
            .sender
            .lock()
            .expect("engine task sender mutex should not be poisoned")
            .as_ref()
            .cloned()
        else {
            return false;
        };

        sender.send(Box::new(job)).is_ok()
    }
}

impl Default for EngineTaskSystem {
    fn default() -> Self {
        Self::new(EngineSchedulingConfig::default())
    }
}

impl Drop for EngineTaskSystem {
    fn drop(&mut self) {
        self.sender
            .lock()
            .expect("engine task sender mutex should not be poisoned")
            .take();
        self.receiver
            .lock()
            .expect("engine task receiver mutex should not be poisoned")
            .take();
        let mut workers = self
            .workers
            .lock()
            .expect("engine task workers mutex should not be poisoned");
        for worker in workers.drain(..) {
            let _ = worker.join();
        }
    }
}

type SharedReceiver = std::sync::Arc<Mutex<Receiver<BoxedEngineJob>>>;

fn spawn_worker(index: usize, receiver: SharedReceiver) -> JoinHandle<()> {
    thread::Builder::new()
        .name(format!("amigo-engine-worker-{index}"))
        .spawn(move || {
            loop {
                let message = receiver
                    .lock()
                    .expect("engine task receiver mutex should not be poisoned")
                    .recv();
                match message {
                    Ok(job) => job(),
                    Err(_) => break,
                }
            }
        })
        .expect("engine task worker thread should spawn")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobContext {
    pub frame_index: u64,
    pub lane: EngineLane,
}

pub trait EngineJob: Send + 'static {
    type Output: Send + 'static;

    fn name(&self) -> &'static str;
    fn lane(&self) -> EngineLane;
    fn run(self, ctx: JobContext) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EngineSchedulerMode;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct AddJob(i32, i32);

    impl EngineJob for AddJob {
        type Output = i32;

        fn name(&self) -> &'static str {
            "add"
        }

        fn lane(&self) -> EngineLane {
            EngineLane::Simulation
        }

        fn run(self, _ctx: JobContext) -> Self::Output {
            self.0 + self.1
        }
    }

    #[test]
    fn runs_inline_when_workers_disabled() {
        let system = EngineTaskSystem::new(EngineSchedulingConfig {
            mode: EngineSchedulerMode::SingleThread,
            max_workers: 0,
            deterministic: true,
            allow_frame_latency: false,
        });
        let result = system.run(
            AddJob(2, 3),
            JobContext {
                frame_index: 1,
                lane: EngineLane::Simulation,
            },
        );
        assert_eq!(result, 5);
    }

    #[test]
    fn runs_with_worker_when_enabled() {
        let system = EngineTaskSystem::new(EngineSchedulingConfig {
            mode: EngineSchedulerMode::Hybrid,
            max_workers: 1,
            deterministic: true,
            allow_frame_latency: false,
        });
        let result = system.run(
            AddJob(7, 5),
            JobContext {
                frame_index: 2,
                lane: EngineLane::Simulation,
            },
        );
        assert_eq!(result, 12);
    }

    #[test]
    fn spawn_detached_runs_job_on_persistent_worker() {
        let system = EngineTaskSystem::new(EngineSchedulingConfig {
            mode: EngineSchedulerMode::Hybrid,
            max_workers: 1,
            deterministic: true,
            allow_frame_latency: true,
        });
        let hits = Arc::new(AtomicUsize::new(0));
        let job_hits = hits.clone();
        assert!(system.spawn_detached(move || {
            job_hits.fetch_add(1, Ordering::AcqRel);
        }));
        std::thread::sleep(std::time::Duration::from_millis(30));
        assert_eq!(hits.load(Ordering::Acquire), 1);
    }
}

