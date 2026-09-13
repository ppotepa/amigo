use amigo_plugin_api::PluginId;

/// Exposes registered domains without importing domain crates into the Rhai backend.
pub(crate) fn register_playground_bindings(
    engine: &mut rhai::Engine,
    host: std::sync::Arc<amigo_playground_api::PlaygroundHostService>,
) {
    for descriptor in host.descriptors() {
        let namespace = descriptor.id.0.replace('-', "_");
        let id = descriptor.id.clone();
        let metadata_host = host.clone();
        engine.register_fn(
            format!("{namespace}_metadata"),
            move || -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
                let snapshot = metadata_host
                    .snapshot(&id)
                    .map_err(|e| Box::<rhai::EvalAltResult>::from(e))?;
                rhai::serde::to_dynamic(snapshot)
                    .map_err(|e| Box::<rhai::EvalAltResult>::from(e.to_string()))
            },
        );
        let id = descriptor.id;
        let dispatch_host = host.clone();
        engine.register_fn(
            format!("{namespace}_dispatch"),
            move |intent: rhai::Dynamic| -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
                let intent: serde_json::Value = rhai::serde::from_dynamic(&intent)
                    .map_err(|e| Box::<rhai::EvalAltResult>::from(e.to_string()))?;
                let result = dispatch_host.dispatch_script(&id, intent).map_err(|e| {
                    Box::<rhai::EvalAltResult>::from(format!("{}: {}", e.control, e.message))
                })?;
                rhai::serde::to_dynamic(result)
                    .map_err(|e| Box::<rhai::EvalAltResult>::from(e.to_string()))
            },
        );
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RhaiPluginBindingProviderDescriptor {
    pub owner: PluginId,
    pub namespace: String,
    pub bindings: Vec<String>,
}

impl RhaiPluginBindingProviderDescriptor {
    pub fn new(owner: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self {
            owner: PluginId(owner.into()),
            namespace: namespace.into(),
            bindings: Vec::new(),
        }
    }

    pub fn with_binding(mut self, binding: impl Into<String>) -> Self {
        self.bindings.push(binding.into());
        self
    }

    pub fn is_valid(&self) -> bool {
        !self.owner.0.trim().is_empty()
            && !self.namespace.trim().is_empty()
            && !self.bindings.is_empty()
            && self
                .bindings
                .iter()
                .all(|binding| !binding.trim().is_empty())
    }
}

#[cfg(test)]
mod playground_tests {
    use super::*;
    use amigo_playground_api::*;
    use std::{
        collections::BTreeMap,
        sync::{Arc, Mutex},
    };

    #[derive(Default)]
    struct Domain(Mutex<(u64, bool)>);
    impl PlaygroundProvider for Domain {
        fn descriptor(&self) -> PlaygroundDescriptor {
            PlaygroundDescriptor {
                id: PlaygroundId("npr-playground".into()),
                label: "Fixture".into(),
                client: "fixture".into(),
            }
        }
        fn snapshot(&self) -> PlaygroundSnapshot {
            let state = self.0.lock().unwrap();
            PlaygroundSnapshot {
                revision: state.0,
                values: BTreeMap::from([("paused".into(), serde_json::json!(state.1))]),
                metadata: serde_json::json!({"controls":["paused"]}),
            }
        }
        fn dispatch(&self, action: &PlaygroundActionEnvelope) -> Result<(), PlaygroundActionError> {
            let value = action
                .intent
                .get("paused")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| PlaygroundActionError {
                    control: "paused".into(),
                    code: "invalid".into(),
                    message: "expected boolean".into(),
                })?;
            let mut state = self.0.lock().unwrap();
            state.0 += 1;
            state.1 = value;
            Ok(())
        }
    }
    #[test]
    fn playground_rhai_metadata_and_actions_share_the_registered_provider() {
        let host = Arc::new(PlaygroundHostService::default());
        host.register(Arc::new(Domain::default())).unwrap();
        let mut engine = rhai::Engine::new();
        register_playground_bindings(&mut engine, host.clone());
        assert!(
            !engine
                .eval::<bool>("npr_playground_metadata().values.paused")
                .unwrap()
        );
        assert!(
            engine
                .eval::<bool>("npr_playground_dispatch(#{paused: true}).values.paused")
                .unwrap()
        );
        assert_eq!(
            host.snapshot(&PlaygroundId("npr-playground".into()))
                .unwrap()
                .revision,
            1
        );
        let error = engine
            .eval::<rhai::Dynamic>("npr_playground_dispatch(#{paused: 7})")
            .unwrap_err()
            .to_string();
        assert!(error.contains("paused: expected boolean"));
    }
}
