//! Behavior graph and state orchestration for authored gameplay logic.
//! It stores behavior definitions and runtime state consumed by app systems and scripting.

use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_core::AmigoResult;
use amigo_fx::ColorRamp;
use amigo_math::Curve1d;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

include!("behavior/model.rs");
include!("behavior/service.rs");
include!("behavior/plugin.rs");

#[cfg(test)]
include!("behavior/tests.rs");
