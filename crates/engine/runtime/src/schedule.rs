use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use amigo_core::{AmigoError, AmigoResult};

use crate::Runtime;
use crate::{
    EngineLane, EngineTaskSystem, Parallelism, SchedulingDescriptor, SchedulingPriority,
    ThreadPolicy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SystemPhase {
    PreUpdate,
    FixedUpdate,
    Update,
    PostUpdate,
    RenderExtract,
}

pub trait RuntimeSystem: Send + Sync {
    fn name(&self) -> &'static str;
    fn phase(&self) -> SystemPhase;
    fn run(&self, runtime: &Runtime) -> AmigoResult<()>;

    fn scheduling_descriptor(&self) -> SchedulingDescriptor {
        SchedulingDescriptor::main_only(self.name())
    }
}

struct FnRuntimeSystem<F>
where
    F: Fn(&Runtime) -> AmigoResult<()> + Send + Sync + 'static,
{
    name: &'static str,
    phase: SystemPhase,
    run: F,
}

impl<F> RuntimeSystem for FnRuntimeSystem<F>
where
    F: Fn(&Runtime) -> AmigoResult<()> + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn phase(&self) -> SystemPhase {
        self.phase
    }

    fn run(&self, runtime: &Runtime) -> AmigoResult<()> {
        (self.run)(runtime)
    }
}

pub struct SystemRegistry {
    systems: Mutex<BTreeMap<SystemPhase, Vec<Arc<dyn RuntimeSystem>>>>,
}

impl Default for SystemRegistry {
    fn default() -> Self {
        Self {
            systems: Mutex::new(BTreeMap::new()),
        }
    }
}

impl SystemRegistry {
    pub fn register<S>(&self, system: S)
    where
        S: RuntimeSystem + 'static,
    {
        self.systems
            .lock()
            .unwrap()
            .entry(system.phase())
            .or_default()
            .push(Arc::new(system));
    }

    pub fn register_fn<F>(&self, phase: SystemPhase, name: &'static str, run: F)
    where
        F: Fn(&Runtime) -> AmigoResult<()> + Send + Sync + 'static,
    {
        self.register(FnRuntimeSystem { name, phase, run });
    }

    pub fn clear(&self) {
        self.systems.lock().unwrap().clear();
    }

    pub fn phase_systems(&self, phase: SystemPhase) -> Vec<Arc<dyn RuntimeSystem>> {
        let mut systems = self
            .systems
            .lock()
            .unwrap()
            .get(&phase)
            .cloned()
            .unwrap_or_default();
        // Stable sorting preserves registration order among systems with the same
        // scheduling priority while making the descriptor materially affect dispatch.
        systems.sort_by_key(|system| {
            std::cmp::Reverse(priority_rank(system.scheduling_descriptor().priority))
        });
        systems
    }

    pub fn execution_plan(&self, phase: SystemPhase) -> Vec<SchedulingDescriptor> {
        self.phase_systems(phase)
            .into_iter()
            .map(|system| system.scheduling_descriptor())
            .collect()
    }

    pub fn run_phase(&self, phase: SystemPhase, runtime: &Runtime) -> AmigoResult<()> {
        for system in self.phase_systems(phase) {
            let descriptor = system.scheduling_descriptor();
            validate_scheduling_descriptor(&descriptor, runtime)?;
            system.run(runtime)?;
        }

        Ok(())
    }

    pub fn run_all(&self, runtime: &Runtime) -> AmigoResult<()> {
        for phase in [
            SystemPhase::PreUpdate,
            SystemPhase::FixedUpdate,
            SystemPhase::Update,
            SystemPhase::PostUpdate,
            SystemPhase::RenderExtract,
        ] {
            self.run_phase(phase, runtime)?;
        }

        Ok(())
    }
}

fn priority_rank(priority: SchedulingPriority) -> u8 {
    match priority {
        SchedulingPriority::Background => 0,
        SchedulingPriority::Low => 1,
        SchedulingPriority::Normal => 2,
        SchedulingPriority::Foreground => 3,
        SchedulingPriority::Critical => 4,
    }
}

fn validate_scheduling_descriptor(
    descriptor: &SchedulingDescriptor,
    runtime: &Runtime,
) -> AmigoResult<()> {
    if descriptor.thread_policy == ThreadPolicy::BackgroundOnly {
        let workers = runtime
            .resolve::<EngineTaskSystem>()
            .map(|tasks| tasks.config().max_workers)
            .unwrap_or_default();
        if workers == 0 {
            return Err(AmigoError::Message(format!(
                "system `{}` requires a background worker, but workers are disabled",
                descriptor.id
            )));
        }
    }

    if descriptor.thread_policy == ThreadPolicy::MainOnly && descriptor.lane != EngineLane::Main {
        return Err(AmigoError::Message(format!(
            "system `{}` declares MainOnly but uses lane {:?}",
            descriptor.id, descriptor.lane
        )));
    }

    if descriptor.parallelism != Parallelism::None
        && descriptor.thread_policy == ThreadPolicy::MainOnly
    {
        return Err(AmigoError::Message(format!(
            "system `{}` requests parallelism while restricted to the main thread",
            descriptor.id
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use amigo_core::{AmigoError, AmigoResult};

    use super::{RuntimeSystem, SystemPhase, SystemRegistry};
    use crate::{
        EngineLane, Parallelism, Runtime, RuntimeBuilder, SchedulingDescriptor, SchedulingPriority,
        ThreadPolicy,
    };

    struct TestSystem {
        name: &'static str,
        phase: SystemPhase,
        sink: Arc<Mutex<Vec<&'static str>>>,
        fail: bool,
    }

    impl RuntimeSystem for TestSystem {
        fn name(&self) -> &'static str {
            self.name
        }

        fn phase(&self) -> SystemPhase {
            self.phase
        }

        fn run(&self, _runtime: &Runtime) -> AmigoResult<()> {
            self.sink.lock().unwrap().push(self.name);

            if self.fail {
                return Err(AmigoError::Message(format!(
                    "system failed: {}",
                    self.name()
                )));
            }

            Ok(())
        }
    }

    struct PrioritySystem {
        name: &'static str,
        priority: SchedulingPriority,
        sink: Arc<Mutex<Vec<&'static str>>>,
    }

    impl RuntimeSystem for PrioritySystem {
        fn name(&self) -> &'static str {
            self.name
        }

        fn phase(&self) -> SystemPhase {
            SystemPhase::Update
        }

        fn run(&self, _runtime: &Runtime) -> AmigoResult<()> {
            self.sink.lock().unwrap().push(self.name);
            Ok(())
        }

        fn scheduling_descriptor(&self) -> SchedulingDescriptor {
            SchedulingDescriptor {
                id: self.name,
                lane: EngineLane::Main,
                thread_policy: ThreadPolicy::MainOnly,
                parallelism: Parallelism::None,
                priority: self.priority,
                allow_frame_latency: false,
            }
        }
    }

    #[test]
    fn runs_systems_in_phase_order_and_registration_order() {
        let registry = SystemRegistry::default();
        let sink = Arc::new(Mutex::new(Vec::new()));

        registry.register(TestSystem {
            name: "update-a",
            phase: SystemPhase::Update,
            sink: sink.clone(),
            fail: false,
        });
        registry.register(TestSystem {
            name: "pre",
            phase: SystemPhase::PreUpdate,
            sink: sink.clone(),
            fail: false,
        });
        registry.register(TestSystem {
            name: "update-b",
            phase: SystemPhase::Update,
            sink: sink.clone(),
            fail: false,
        });
        registry.register(TestSystem {
            name: "post",
            phase: SystemPhase::PostUpdate,
            sink: sink.clone(),
            fail: false,
        });

        let runtime = RuntimeBuilder::default().build();
        registry.run_all(&runtime).unwrap();

        assert_eq!(
            sink.lock().unwrap().as_slice(),
            ["pre", "update-a", "update-b", "post"]
        );
    }

    #[test]
    fn higher_priority_systems_run_first() {
        let registry = SystemRegistry::default();
        let sink = Arc::new(Mutex::new(Vec::new()));
        registry.register(PrioritySystem {
            name: "normal",
            priority: SchedulingPriority::Normal,
            sink: sink.clone(),
        });
        registry.register(PrioritySystem {
            name: "critical",
            priority: SchedulingPriority::Critical,
            sink: sink.clone(),
        });

        registry
            .run_phase(SystemPhase::Update, &RuntimeBuilder::default().build())
            .unwrap();
        assert_eq!(sink.lock().unwrap().as_slice(), ["critical", "normal"]);
    }

    #[test]
    fn run_phase_limits_execution_to_requested_phase() {
        let registry = SystemRegistry::default();
        let sink = Arc::new(Mutex::new(Vec::new()));

        registry.register(TestSystem {
            name: "pre",
            phase: SystemPhase::PreUpdate,
            sink: sink.clone(),
            fail: false,
        });
        registry.register(TestSystem {
            name: "update",
            phase: SystemPhase::Update,
            sink: sink.clone(),
            fail: false,
        });

        let runtime = RuntimeBuilder::default().build();
        registry.run_phase(SystemPhase::Update, &runtime).unwrap();

        assert_eq!(sink.lock().unwrap().as_slice(), ["update"]);
    }

    #[test]
    fn stops_execution_on_first_error() {
        let registry = SystemRegistry::default();
        let sink = Arc::new(Mutex::new(Vec::new()));

        registry.register(TestSystem {
            name: "first",
            phase: SystemPhase::Update,
            sink: sink.clone(),
            fail: true,
        });
        registry.register(TestSystem {
            name: "second",
            phase: SystemPhase::Update,
            sink: sink.clone(),
            fail: false,
        });

        let runtime = RuntimeBuilder::default().build();
        let error = registry
            .run_phase(SystemPhase::Update, &runtime)
            .unwrap_err();

        assert!(matches!(error, AmigoError::Message(_)));
        assert_eq!(sink.lock().unwrap().as_slice(), ["first"]);
    }

    #[test]
    fn register_fn_registers_closure_backed_systems() {
        let registry = SystemRegistry::default();
        let sink = Arc::new(Mutex::new(Vec::new()));
        let sink_clone = Arc::clone(&sink);

        registry.register_fn(SystemPhase::Update, "closure-update", move |_| {
            sink_clone.lock().unwrap().push("closure-update");
            Ok(())
        });

        let runtime = RuntimeBuilder::default().build();
        registry.run_phase(SystemPhase::Update, &runtime).unwrap();

        assert_eq!(sink.lock().unwrap().as_slice(), ["closure-update"]);
    }

    #[test]
    fn clear_removes_registered_systems() {
        let registry = SystemRegistry::default();
        let sink = Arc::new(Mutex::new(Vec::new()));
        let sink_clone = Arc::clone(&sink);

        registry.register_fn(SystemPhase::Update, "closure-update", move |_| {
            sink_clone.lock().unwrap().push("closure-update");
            Ok(())
        });
        registry.clear();

        let runtime = RuntimeBuilder::default().build();
        registry.run_phase(SystemPhase::Update, &runtime).unwrap();

        assert!(sink.lock().unwrap().is_empty());
    }
}
