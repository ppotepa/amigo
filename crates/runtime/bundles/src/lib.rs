mod audio;
mod core;
mod devtools;
mod event_pipeline;
mod full;
mod host_viewport;
mod offscreen_runtime_frame;
mod platform;
mod plugin_composition;
mod render_diagnostics;
pub mod render_extractor_bridges;
mod render_extractor_registry;
mod render_packet_services;
mod render_scene_view;
mod render_session;
mod runtime_service_types;
mod runtime_summary;
mod scripting;
mod three_d;
mod two_d;

pub use audio::*;
pub use core::*;
pub use devtools::*;
pub use event_pipeline::*;
pub use full::*;
pub use host_viewport::*;
pub use offscreen_runtime_frame::*;
pub use platform::*;
pub use plugin_composition::*;
pub use render_diagnostics::*;
pub use render_extractor_bridges::{
    WgpuFrameCompositionBuilder, WgpuFrameCompositionOptions, WgpuRenderExtractorRegistry,
    default_wgpu_render_extractor_registry, register_host_render_extractor_provider,
};
pub use render_extractor_registry::*;
pub use render_packet_services::*;
pub use render_scene_view::*;
pub use render_session::*;
pub use runtime_service_types::*;
pub use runtime_summary::*;
pub use scripting::*;
pub use three_d::*;
pub use two_d::*;

use amigo_session::{
    RuntimeCapability, RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
    RuntimeSession,
};

pub fn register_runtime_bundle_capabilities(session: &mut RuntimeSession) {
    register_core_runtime_capabilities(session);
    register_platform_runtime_capabilities(session);
    register_two_d_runtime_capabilities(session);
    register_audio_runtime_capabilities(session);
    register_three_d_runtime_capabilities(session);
    register_modding_and_scripting_runtime_capabilities(session);
    register_devtools_runtime_capabilities(session);
}

pub fn register_full_runtime_capabilities(session: &mut RuntimeSession) {
    register_runtime_bundle_capabilities(session);
    register_host_runtime_capabilities(session);
    amigo_devtools::register_devtools_capabilities(session);
    register_host_render_extractor_provider(session);
}

fn register_host_runtime_capabilities(session: &mut RuntimeSession) {
    for descriptor in [
        RuntimeCapabilityDescriptor {
            domain_id: RuntimeDomainId::new("app.host"),
            kind: RuntimeCapabilityKind::DiagnosticsProvider,
            id: "runtime.diagnostics.overview".to_owned(),
            label: "Runtime diagnostics overview".to_owned(),
            description: "Host runtime diagnostics and runtime-service summary".to_owned(),
            capabilities: vec!["runtime-diagnostics".to_owned()],
            tags: vec!["app".to_owned(), "host".to_owned()],
            migration_seam: false,
        },
        RuntimeCapabilityDescriptor {
            domain_id: RuntimeDomainId::new("app.host"),
            kind: RuntimeCapabilityKind::MetadataProvider,
            id: "runtime.metadata.overview".to_owned(),
            label: "Runtime metadata overview".to_owned(),
            description: "Host runtime and scene metadata descriptor snapshot".to_owned(),
            capabilities: vec!["runtime-metadata".to_owned()],
            tags: vec!["app".to_owned(), "host".to_owned()],
            migration_seam: false,
        },
    ] {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability { descriptor });
    }
}
