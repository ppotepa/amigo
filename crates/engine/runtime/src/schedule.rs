use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use amigo_core::AmigoResult;

use crate::Runtime;

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
        self.systems
            .lock()
            .unwrap()
            .get(&phase)
            .cloned()
            .unwrap_or_default()
    }

    pub fn run_phase(&self, phase: SystemPhase, runtime: &Runtime) -> AmigoResult<()> {
        for system in self.phase_systems(phase) {
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use amigo_core::{AmigoError, AmigoResult};

    use super::{RuntimeSystem, SystemPhase, SystemRegistry};
    use crate::{Runtime, RuntimeBuilder};

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
