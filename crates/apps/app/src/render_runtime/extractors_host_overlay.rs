use amigo_render_api::RenderFrameExtractor;
use amigo_session::{
    runtime_capabilities::{
        RenderExtractorContribution, RenderExtractorDescriptor, RenderExtractorProvider,
        RuntimeCapability, RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
    },
    RuntimeSession,
};

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_host_overlay_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedUiOverlayExtractor);
    registry.register(ResolvedDevConsoleOverlayExtractor);
    registry.register(ResolvedDebugOverlayExtractor);
}

pub(crate) struct HostAppRenderExtractorProvider;

impl RenderExtractorProvider for HostAppRenderExtractorProvider {
    fn register_render_extractors(&self, descriptors: &mut Vec<RenderExtractorDescriptor>) {
        descriptors.extend(
            [
                ("resolved_ui_overlay", "UI Overlay Extractor"),
                ("resolved_dev_console_overlay", "Dev Console Overlay Extractor"),
                ("resolved_debug_overlay", "Debug Overlay Extractor"),
            ]
            .into_iter()
            .map(|(id, label)| RenderExtractorDescriptor {
                descriptor: RuntimeCapabilityDescriptor {
                    domain_id: RuntimeDomainId::new("app.host"),
                    kind: RuntimeCapabilityKind::RenderExtractor,
                    id: id.to_string(),
                    label: label.to_string(),
                    description: "app host render overlay extractor".to_string(),
                    capabilities: Vec::new(),
                    tags: vec!["app".to_string(), "host".to_string()],
                    migration_seam: false,
                },
            }),
        );
    }
}

pub(crate) fn register_host_render_extractor_provider(
    session: &mut RuntimeSession,
) -> Vec<RenderExtractorContribution> {
    let mut descriptors = Vec::new();
    HostAppRenderExtractorProvider.register_render_extractors(&mut descriptors);
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

pub(crate) struct ResolvedUiOverlayExtractor;
pub(crate) struct ResolvedDevConsoleOverlayExtractor;
pub(crate) struct ResolvedDebugOverlayExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedUiOverlayExtractor
{
    fn name(&self) -> &'static str {
        "resolved_ui_overlay"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        let overlays = crate::ui_runtime::resolve_ui_overlay_documents(
            context.ui_scene_service,
            context.ui_state_service,
            context.ui_theme_service,
        )
        .into_iter()
        .map(|document| document.overlay);
        packet.extend_game_ui_overlay(overlays);
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedDevConsoleOverlayExtractor
{
    fn name(&self) -> &'static str {
        "resolved_dev_console_overlay"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        if let Some(overlay) = crate::dev_console::overlay::build_dev_console_overlay(
            context.dev_console_state,
            context.dev_console_completion.snapshot().as_ref(),
            context.ui_viewport_state.get(),
        ) {
            packet.extend_debug_overlay([overlay]);
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedDebugOverlayExtractor
{
    fn name(&self) -> &'static str {
        "resolved_debug_overlay"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        let snapshot = context.debug_overlay_service.snapshot();
        if let Some(overlay) = crate::debug_overlay::build_debug_overlay_document(
            &snapshot,
            context.ui_viewport_state.get(),
        ) {
            packet.extend_debug_overlay([overlay]);
        }
    }
}
