use std::sync::Arc;

use amigo_render_api::RenderFrameExtractor;
use amigo_runtime::Runtime;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_session::{
    RuntimeSession,
    runtime_capabilities::{
        RenderExtractorContribution, RenderExtractorDescriptor, RenderExtractorProvider,
        RuntimeCapability, RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
    },
};

use super::context::WgpuRenderExtractorRegistry;

pub fn register_host_overlay_render_extractors(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuUiOverlayRenderExtractorBridge);
    registry.register(WgpuDevConsoleOverlayRenderExtractorBridge);
    registry.register(WgpuDebugOverlayRenderExtractorBridge);
}

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> Arc<T> {
    runtime
        .required::<T>()
        .expect("render extractor required service should be registered")
}

pub struct WgpuHostOverlayRenderExtractorProvider;

impl RenderExtractorProvider for WgpuHostOverlayRenderExtractorProvider {
    fn register_render_extractors(&self, descriptors: &mut Vec<RenderExtractorDescriptor>) {
        descriptors.extend([
            RenderExtractorDescriptor {
                descriptor: RuntimeCapabilityDescriptor {
                    domain_id: RuntimeDomainId::new("amigo.ui.core"),
                    kind: RuntimeCapabilityKind::RenderExtractor,
                    id: "ui_overlay".to_owned(),
                    label: "UI Overlay Extractor".to_owned(),
                    description: "ui domain render overlay extractor".to_owned(),
                    capabilities: vec!["ui".to_owned()],
                    tags: vec!["ui".to_owned()],
                    migration_seam: false,
                },
            },
            RenderExtractorDescriptor {
                descriptor: RuntimeCapabilityDescriptor {
                    domain_id: RuntimeDomainId::new("app.host"),
                    kind: RuntimeCapabilityKind::RenderExtractor,
                    id: "app_dev_console_overlay".to_owned(),
                    label: "Dev Console Overlay Extractor".to_owned(),
                    description: "app host render overlay extractor".to_owned(),
                    capabilities: Vec::new(),
                    tags: vec!["app".to_string(), "host".to_string()],
                    migration_seam: false,
                },
            },
            RenderExtractorDescriptor {
                descriptor: RuntimeCapabilityDescriptor {
                    domain_id: RuntimeDomainId::new("amigo.devtools"),
                    kind: RuntimeCapabilityKind::RenderExtractor,
                    id: "debug_overlay".to_owned(),
                    label: "Debug Overlay Extractor".to_owned(),
                    description: "devtools debug overlay render extractor".to_owned(),
                    capabilities: Vec::new(),
                    tags: vec!["devtools".to_string(), "debug".to_string()],
                    migration_seam: false,
                },
            },
        ]);
    }
}

pub fn register_host_render_extractor_provider(
    session: &mut RuntimeSession,
) -> Vec<RenderExtractorContribution> {
    let mut descriptors = Vec::new();
    WgpuHostOverlayRenderExtractorProvider.register_render_extractors(&mut descriptors);
    let contributions = descriptors
        .into_iter()
        .map(|descriptor| RenderExtractorContribution {
            descriptor: descriptor.clone(),
        })
        .collect::<Vec<_>>();

    for contribution in &contributions {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    contributions
}

pub struct WgpuUiOverlayRenderExtractorBridge;
pub struct WgpuDevConsoleOverlayRenderExtractorBridge;
pub struct WgpuDebugOverlayRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuUiOverlayRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_ui::UiOverlayRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let ui_scene_service = required::<amigo_ui::UiSceneService>(runtime);
        let ui_state_service = required::<amigo_ui::UiStateService>(runtime);
        let ui_theme_service = required::<amigo_ui::UiThemeService>(runtime);
        amigo_ui::UiOverlayRenderExtractor.extract(
            ui_scene_service.as_ref(),
            ui_state_service.as_ref(),
            ui_theme_service.as_ref(),
            packet,
        );
    }
}

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket>
    for WgpuDevConsoleOverlayRenderExtractorBridge
{
    fn name(&self) -> &'static str {
        amigo_devtools::DevConsoleOverlayRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(dev_console_state) = runtime.resolve::<amigo_scripting_api::DevConsoleState>() else {
            return;
        };
        let Some(dev_console_completion) = runtime.resolve::<amigo_devtools::ConsoleCompletionState>() else {
            return;
        };
        let Some(ui_viewport_state) = runtime.resolve::<amigo_ui::UiInputViewportState>() else {
            return;
        };
        amigo_devtools::DevConsoleOverlayRenderExtractor.extract(
            dev_console_state.as_ref(),
            dev_console_completion.snapshot().as_ref(),
            ui_viewport_state.get(),
            packet,
        );
    }
}

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuDebugOverlayRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_devtools::DebugOverlayRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let debug_overlay_service = required::<amigo_devtools::DebugOverlayService>(runtime);
        let ui_viewport_state = required::<amigo_ui::UiInputViewportState>(runtime);
        let snapshot = debug_overlay_service.snapshot();
        amigo_devtools::DebugOverlayRenderExtractor.extract(
            &snapshot,
            ui_viewport_state.get(),
            packet,
        );
    }
}


