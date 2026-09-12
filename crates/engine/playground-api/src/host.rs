use crate::*;
use std::sync::{Arc, Mutex};

struct HostedProvider {
    provider: Arc<dyn PlaygroundProvider>,
    previous: Option<PlaygroundSnapshot>,
    last_request: u64,
}

/// Serializes revision checks and mutations for every registered domain.
#[derive(Default)]
pub struct PlaygroundHostService {
    providers: Mutex<BTreeMap<PlaygroundId, HostedProvider>>,
}

impl PlaygroundHostService {
    pub fn descriptors(&self) -> Vec<PlaygroundDescriptor> {
        self.providers
            .lock()
            .unwrap()
            .values()
            .map(|p| p.provider.descriptor())
            .collect()
    }
    pub fn snapshot(&self, id: &PlaygroundId) -> Result<PlaygroundSnapshot, String> {
        Ok(self
            .providers
            .lock()
            .unwrap()
            .get(id)
            .ok_or("unregistered playground")?
            .provider
            .snapshot())
    }
    /// Script calls share revision validation but do not consume a client's request sequence.
    pub fn dispatch_script(
        &self,
        id: &PlaygroundId,
        intent: Value,
    ) -> Result<PlaygroundSnapshot, PlaygroundActionError> {
        let providers = self.providers.lock().unwrap();
        let p = providers.get(id).ok_or_else(|| PlaygroundActionError {
            control: "script".into(),
            code: "unknown_playground".into(),
            message: "Playground is not registered".into(),
        })?;
        let action = PlaygroundActionEnvelope {
            request_id: 0,
            base_revision: p.provider.revision(),
            control: "script".into(),
            intent,
        };
        p.provider.dispatch(&action)?;
        Ok(p.provider.snapshot())
    }
    pub fn open_scene(
        &self,
        id: &PlaygroundId,
        root: &std::path::Path,
        scene: &std::path::Path,
    ) -> Result<(), String> {
        let providers = self.providers.lock().unwrap();
        providers
            .get(id)
            .ok_or("unregistered playground")?
            .provider
            .open_scene(root, scene)
    }
    pub fn register(&self, provider: Arc<dyn PlaygroundProvider>) -> Result<(), String> {
        let id = provider.descriptor().id;
        let mut providers = self.providers.lock().unwrap();
        if providers.contains_key(&id) {
            return Err(format!("duplicate playground: {}", id.0));
        }
        providers.insert(
            id,
            HostedProvider {
                provider,
                previous: None,
                last_request: 0,
            },
        );
        Ok(())
    }

    pub fn descriptor(&self, id: &PlaygroundId) -> Option<PlaygroundDescriptor> {
        self.providers
            .lock()
            .unwrap()
            .get(id)
            .map(|p| p.provider.descriptor())
    }

    pub fn connect(&self, id: &PlaygroundId) -> Result<PlaygroundEvent, String> {
        let mut providers = self.providers.lock().unwrap();
        let p = providers.get_mut(id).ok_or("unregistered playground")?;
        let snapshot = p.provider.snapshot();
        p.previous = Some(snapshot.clone());
        p.last_request = 0;
        Ok(PlaygroundEvent::Snapshot { snapshot })
    }

    /// Resynchronization preserves the authenticated client's request sequence.
    pub fn refresh(&self, id: &PlaygroundId) -> Result<PlaygroundEvent, String> {
        let mut providers = self.providers.lock().unwrap();
        let p = providers.get_mut(id).ok_or("unregistered playground")?;
        p.provider.cancel_interaction();
        let snapshot = p.provider.snapshot();
        p.previous = Some(snapshot.clone());
        Ok(PlaygroundEvent::Snapshot { snapshot })
    }

    pub fn dispatch(&self, id: &PlaygroundId, action: PlaygroundActionEnvelope) -> PlaygroundEvent {
        let reject = |code: &str, message: &str| PlaygroundEvent::Rejected {
            request_id: action.request_id,
            error: PlaygroundActionError {
                control: action.control.clone(),
                code: code.into(),
                message: message.into(),
            },
        };
        let mut providers = self.providers.lock().unwrap();
        let Some(p) = providers.get_mut(id) else {
            return reject("unknown_playground", "Playground is not registered");
        };
        if action.request_id <= p.last_request {
            return reject("request_id", "Request already processed or out of order");
        }
        p.last_request = action.request_id;
        if action.base_revision != p.provider.revision() {
            p.provider.cancel_interaction();
            return reject(
                "revision_conflict",
                "State changed; refresh before retrying",
            );
        }
        match p.provider.dispatch(&action) {
            Ok(()) => PlaygroundEvent::Accepted {
                request_id: action.request_id,
                revision: p.provider.revision(),
            },
            Err(error) => PlaygroundEvent::Rejected {
                request_id: action.request_id,
                error,
            },
        }
    }

    pub fn poll(&self, id: &PlaygroundId) -> Result<Vec<PlaygroundEvent>, String> {
        let mut providers = self.providers.lock().unwrap();
        let p = providers.get_mut(id).ok_or("unregistered playground")?;
        let snapshot = p.provider.snapshot();
        let mut events = Vec::new();
        if let Some(previous) = &p.previous {
            if snapshot.revision != previous.revision {
                events.push(PlaygroundEvent::Delta {
                    base_revision: previous.revision,
                    revision: snapshot.revision,
                    changed: snapshot
                        .values
                        .iter()
                        .filter(|(k, v)| previous.values.get(*k) != Some(*v))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                    removed: previous
                        .values
                        .keys()
                        .filter(|k| !snapshot.values.contains_key(*k))
                        .cloned()
                        .collect(),
                    metadata: (snapshot.metadata != previous.metadata)
                        .then(|| snapshot.metadata.clone()),
                });
            }
        } else {
            events.push(PlaygroundEvent::Snapshot {
                snapshot: snapshot.clone(),
            });
        }
        p.previous = Some(snapshot);
        events.extend(p.provider.drain_events());
        Ok(events)
    }
}
