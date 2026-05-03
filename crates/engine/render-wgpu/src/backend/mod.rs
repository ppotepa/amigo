//! GPU backend setup and surface management for the WGPU renderer.
//! This module isolates adapter/device creation, surface configuration, and low-level backend helpers.

mod helpers;
mod surface;
mod types;

pub use types::{
    WgpuHeadlessContext, WgpuOffscreenTarget, WgpuRenderBackend, WgpuRenderPlugin,
    WgpuSurfaceState,
};
