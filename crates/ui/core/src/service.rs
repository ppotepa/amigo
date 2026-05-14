use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_core::AmigoResult;
use amigo_math::ColorRgba;
use amigo_scene::SceneEntityId;

use crate::layout::UiLayoutService;
use crate::model::{UiCurvePoint, UiDocument, UiTheme, normalize_curve_points};

include!("service/scene.rs");
include!("service/bindings.rs");
include!("service/state.rs");
include!("service/theme.rs");
include!("service/model_bindings.rs");
include!("service/plugin.rs");

#[cfg(test)]
include!("service/tests.rs");
