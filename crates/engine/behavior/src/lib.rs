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
