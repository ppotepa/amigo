//! GPU backend setup and surface management for the WGPU renderer.
//! This module isolates adapter/device creation, surface configuration, and low-level backend helpers.

mod helpers;
mod readback;
#[cfg(windows)]
mod shared_texture;
pub use readback::{WgpuReadbackFrame, WgpuReadbackPool};
#[cfg(windows)]
pub use shared_texture::WgpuSharedTextures;
mod surface;
mod types;

pub use types::{
    WgpuHeadlessContext, WgpuOffscreenTarget, WgpuRenderBackend, WgpuRenderPlugin, WgpuSurfaceState,
};
