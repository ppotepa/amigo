use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_runtime::{RuntimePlugin, ServiceRegistry};

include!("state/model.rs");
include!("state/scene_service.rs");
include!("state/session_service.rs");
include!("state/timers.rs");
include!("state/plugin.rs");

#[cfg(test)]
include!("state/tests.rs");
