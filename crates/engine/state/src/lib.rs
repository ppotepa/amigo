//! Per-scene and session state services for gameplay logic.
//! It stores scalar state and timers that scripts and systems mutate during runtime.

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
