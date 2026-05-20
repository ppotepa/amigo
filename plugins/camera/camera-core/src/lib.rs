pub mod api;
pub mod diagnostics;
pub mod manifest;
pub mod participation;
pub mod plugin;
pub mod runtime;
pub mod scene;
pub mod scripting;

pub use api::*;
pub use diagnostics::*;
pub use runtime::*;
pub use scene::*;
pub use amigo_camera_optics_plugin::runtime::*;
pub use amigo_camera_profiles_plugin::api::*;
pub use amigo_camera_profiles_plugin::runtime::*;
pub use amigo_camera::{CameraDebugViewDescriptor, CameraDebugViewId, CameraDebugViewRegistry};

// Transitional compatibility alias. Prefer CameraDebugViewId for new neutral contracts.
pub use amigo_render_api::CameraDebugView2d;
