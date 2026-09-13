pub mod api;
pub mod asset_browser;
pub mod authoring;
pub mod playback;
pub mod diagnostics;
pub mod documents;
mod editor_provider;
pub mod playground;
pub mod plugin;
pub mod render;
pub mod runtime;
pub mod scene;
pub mod scripting;
pub mod sources;
pub mod state;
mod zoom;

#[cfg(feature = "playground-client")]
include!(concat!(env!("OUT_DIR"), "/playground_client.rs"));

pub use editor_provider::NprPlaygroundEditorRuntimeApplyProvider;
pub use plugin::NprPlaygroundPlugin;
pub use render::{NprPlaygroundRenderService, NprSurfacePick};
pub use state::NprPlaygroundState;
