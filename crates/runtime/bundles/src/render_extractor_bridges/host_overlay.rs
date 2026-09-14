use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_session::{
    RuntimeLoadingService, RuntimeLoadingSnapshot, RuntimeLoadingState, RuntimeSession,
    runtime_capabilities::{
        RenderExtractorContribution, RenderExtractorDescriptor, RenderExtractorProvider,
        RuntimeCapability, RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
    },
};

use super::context::WgpuRenderExtractorRegistry;

pub fn register_host_overlay_render_extractors(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuUiOverlayRenderExtractorBridge);
    registry.register(WgpuEmbeddedPanelRenderExtractorBridge);
    register_surface_overlay_render_extractors(registry);
}

pub fn register_surface_overlay_render_extractors(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuDevConsoleOverlayRenderExtractorBridge);
    registry.register(WgpuDebugOverlayRenderExtractorBridge);
}

fn optional<T: Send + Sync + 'static>(runtime: &Runtime) -> Option<std::sync::Arc<T>> {
    runtime.resolve::<T>()
}

struct WgpuDebugOverlayOutput<'a>(&'a mut WgpuRenderFramePacket);

struct WgpuUiOverlayOutput<'a>(&'a mut WgpuRenderFramePacket);

impl amigo_ui::UiOverlayRenderOutput for WgpuUiOverlayOutput<'_> {
    fn push_ui_overlay_document(&mut self, document: amigo_overlay_api::UiOverlayDocument) {
        self.0.push_game_ui_overlay(document);
    }
}

impl amigo_devtools::DebugOverlayRenderOutput for WgpuDebugOverlayOutput<'_> {
    fn push_debug_overlay_document(&mut self, document: amigo_overlay_api::UiOverlayDocument) {
        self.0.push_debug_overlay(document);
    }
}

struct WgpuDevConsoleOverlayOutput<'a>(&'a mut WgpuRenderFramePacket);

impl amigo_devtools::DevConsoleOverlayRenderOutput for WgpuDevConsoleOverlayOutput<'_> {
    fn push_dev_console_overlay_document(
        &mut self,
        document: amigo_overlay_api::UiOverlayDocument,
    ) {
        self.0.push_debug_overlay(document);
    }
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
                    domain_id: RuntimeDomainId::new("amigo.panels"),
                    kind: RuntimeCapabilityKind::RenderExtractor,
                    id: "embedded_panel_overlay".to_owned(),
                    label: "Embedded Panel Overlay Extractor".to_owned(),
                    description: "panel snapshot to host overlay extractor".to_owned(),
                    capabilities: vec!["embedded-panel".to_owned()],
                    tags: vec!["ui".to_owned(), "panel".to_owned()],
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
pub struct WgpuEmbeddedPanelRenderExtractorBridge;
pub struct WgpuDevConsoleOverlayRenderExtractorBridge;
pub struct WgpuDebugOverlayRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuUiOverlayRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_ui::UiOverlayRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        if let Some(loading) = optional::<RuntimeLoadingService>(runtime) {
            let assets = optional::<amigo_assets::AssetCatalog>(runtime)
                .map(|catalog| catalog.loading_summary());
            if let Some(document) = runtime_loading_overlay(loading.snapshot(), assets) {
                packet.push_game_ui_overlay(document);
            }
        }
        let Some(ui_scene_service) = optional::<amigo_ui::UiSceneService>(runtime) else {
            return;
        };
        let Some(ui_state_service) = optional::<amigo_ui::UiStateService>(runtime) else {
            return;
        };
        let Some(ui_theme_service) = optional::<amigo_ui::UiThemeService>(runtime) else {
            return;
        };
        amigo_ui::UiOverlayRenderExtractor.extract(
            ui_scene_service.as_ref(),
            ui_state_service.as_ref(),
            ui_theme_service.as_ref(),
            &mut WgpuUiOverlayOutput(packet),
        );
    }
}

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket>
    for WgpuEmbeddedPanelRenderExtractorBridge
{
    fn name(&self) -> &'static str {
        "embedded_panel_overlay"
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(panels) = optional::<amigo_panels::PanelService>(runtime) else {
            return;
        };
        let Some(controls) = optional::<amigo_runtime_control::RuntimeControlService>(runtime)
        else {
            return;
        };
        let Some(presets) = optional::<amigo_panels::PresetService>(runtime) else {
            return;
        };
        if let Ok(overlays) = panels.embedded_overlays(controls.as_ref(), presets.as_ref()) {
            packet.extend_game_ui_overlay(overlays);
        }
    }
}

/// A neutral runtime overlay. It intentionally lives beside the existing game
/// UI bridge, so a mod cannot accidentally hide engine loading feedback by
/// omitting its own UI scene.
fn runtime_loading_overlay(
    snapshot: RuntimeLoadingSnapshot,
    assets: Option<amigo_assets::AssetCatalogLoadingSummary>,
) -> Option<amigo_overlay_api::UiOverlayDocument> {
    use amigo_math::ColorRgba;
    use amigo_overlay_api::{
        UiOverlayDocument, UiOverlayLayer, UiOverlayNode, UiOverlayNodeKind, UiOverlayStyle,
    };

    let assets = assets.unwrap_or_default();
    let assets_loading = assets.pending_assets + assets.external_loading_assets > 0;
    let assets_failed = assets.failed_assets > 0;
    if !matches!(
        snapshot.state,
        RuntimeLoadingState::Loading | RuntimeLoadingState::Failed
    ) && !assets_loading
        && !assets_failed
    {
        return None;
    }
    if !snapshot.presentation.enabled
        || snapshot.presentation.mode == amigo_session::RuntimeLoadingPresentationMode::Hidden
    {
        return None;
    }
    let asset_progress = if assets.total_assets > 0 {
        assets.completed_assets as f32 / assets.total_assets as f32
    } else {
        0.0
    };
    let stage = if assets_loading {
        "Loading assets"
    } else {
        snapshot.stage.as_str()
    };
    let current = if assets_loading {
        assets
            .current_asset
            .as_ref()
            .map(|asset| format!("Asset: {}", asset.as_str()))
    } else {
        snapshot
            .current_item
            .as_ref()
            .map(|item| format!("Preparing {item}"))
    };
    let progress = if assets_loading {
        asset_progress
    } else {
        snapshot.progress
    };
    let error = snapshot
        .error
        .as_ref()
        .map(|error| format!("FAILED: {error}"))
        .or_else(|| assets_failed.then(|| format!("FAILED: {} asset(s)", assets.failed_assets)));
    // `font: None` selects the engine's built-in bitmap font. The loading
    // screen deliberately has no mod asset dependency, so it can appear while
    // every authored font is still unavailable.
    let foreground = snapshot
        .presentation
        .text
        .as_deref()
        .and_then(parse_overlay_color)
        .unwrap_or(ColorRgba::new(0.92, 0.96, 0.90, 1.0));
    let accent = snapshot
        .presentation
        .accent
        .as_deref()
        .and_then(parse_overlay_color)
        .unwrap_or(ColorRgba::new(0.96, 0.70, 0.28, 1.0));
    let panel_background = snapshot
        .presentation
        .background
        .as_deref()
        .and_then(parse_overlay_color)
        .unwrap_or(ColorRgba::new(0.04, 0.06, 0.10, 0.96));
    let muted = ColorRgba::new(
        foreground.r * 0.62,
        foreground.g * 0.62,
        foreground.b * 0.62,
        foreground.a,
    );
    let mut children = vec![overlay_text(
        "loading-title",
        snapshot.presentation.title,
        24.0,
        foreground,
    )];
    if snapshot.presentation.show_stage {
        children.push(overlay_text("loading-stage", stage, 16.0, muted));
    }
    if snapshot.presentation.show_item
        && let Some(current) = current
    {
        children.push(overlay_text(
            "loading-item",
            current,
            14.0,
            ColorRgba::new(0.62, 0.70, 0.80, 1.0),
        ));
    }
    if let Some(error) = error {
        children.push(overlay_text(
            "loading-error",
            error,
            16.0,
            ColorRgba::new(1.0, 0.38, 0.40, 1.0),
        ));
    } else {
        children.push(UiOverlayNode {
            id: Some("loading-progress".into()),
            kind: UiOverlayNodeKind::ProgressBar { value: progress },
            style: UiOverlayStyle {
                width: Some(488.0),
                height: Some(18.0),
                background: Some(ColorRgba::new(0.10, 0.12, 0.16, 1.0)),
                color: Some(accent),
                border_color: Some(accent),
                border_width: 2.0,
                ..Default::default()
            },
            children: Vec::new(),
        });
        if snapshot.presentation.show_percentage {
            children.push(overlay_text(
                "loading-percent",
                format!("{:>3.0}%", progress * 100.0),
                16.0,
                foreground,
            ));
        }
    }
    Some(UiOverlayDocument {
        entity_name: "runtime-loading-overlay".into(),
        layer: UiOverlayLayer::Menu,
        viewport: None,
        root: UiOverlayNode {
            id: Some("runtime-loading-overlay".into()),
            kind: UiOverlayNodeKind::Panel,
            style: UiOverlayStyle {
                left: Some(0.0),
                top: Some(0.0),
                right: Some(0.0),
                bottom: Some(0.0),
                background: Some(ColorRgba::new(0.0, 0.0, 0.0, 1.0)),
                ..Default::default()
            },
            children: vec![UiOverlayNode {
                id: Some("loading-content".into()),
                kind: UiOverlayNodeKind::Column,
                style: UiOverlayStyle {
                    left: Some(32.0),
                    top: Some(32.0),
                    width: Some(520.0),
                    padding: 16.0,
                    gap: 8.0,
                    background: Some(panel_background),
                    border_color: Some(accent),
                    border_width: 2.0,
                    ..Default::default()
                },
                children,
            }],
        },
    })
}

fn overlay_text(
    id: impl Into<String>,
    content: impl Into<String>,
    font_size: f32,
    color: amigo_math::ColorRgba,
) -> amigo_overlay_api::UiOverlayNode {
    amigo_overlay_api::UiOverlayNode {
        id: Some(id.into()),
        kind: amigo_overlay_api::UiOverlayNodeKind::Text {
            content: content.into(),
            font: None,
        },
        style: amigo_overlay_api::UiOverlayStyle {
            width: Some(488.0),
            font_size,
            color: Some(color),
            word_wrap: true,
            fit_to_width: true,
            ..Default::default()
        },
        children: Vec::new(),
    }
}

fn parse_overlay_color(value: &str) -> Option<amigo_math::ColorRgba> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 && hex.len() != 8 {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&hex[range], 16)
            .ok()
            .map(|value| value as f32 / 255.0)
    };
    Some(amigo_math::ColorRgba::new(
        channel(0..2)?,
        channel(2..4)?,
        channel(4..6)?,
        if hex.len() == 8 { channel(6..8)? } else { 1.0 },
    ))
}

#[cfg(test)]
mod loading_overlay_tests {
    use super::*;
    use amigo_assets::{AssetCatalogLoadingSummary, AssetKey};
    use amigo_session::RuntimeLoadingPresentation;

    fn ready_snapshot() -> RuntimeLoadingSnapshot {
        RuntimeLoadingSnapshot {
            state: RuntimeLoadingState::Ready,
            presentation: RuntimeLoadingPresentation::default(),
            ..RuntimeLoadingSnapshot::default()
        }
    }

    #[test]
    fn ready_scene_has_no_loading_overlay() {
        assert!(runtime_loading_overlay(ready_snapshot(), None).is_none());
    }

    #[test]
    fn pending_asset_keeps_generic_overlay_visible_after_scene_ready() {
        let document = runtime_loading_overlay(
            ready_snapshot(),
            Some(AssetCatalogLoadingSummary {
                total_assets: 8,
                completed_assets: 3,
                pending_assets: 1,
                current_asset: Some(AssetKey::new("npr-city/meshes/tower.glb")),
                ..AssetCatalogLoadingSummary::default()
            }),
        )
        .expect("pending asset should keep the loading overlay visible");

        assert_eq!(
            document.root.style.background,
            Some(amigo_math::ColorRgba::new(0.0, 0.0, 0.0, 1.0))
        );
        let content = &document.root.children[0];
        assert!(content.children.iter().any(|node| matches!(
            &node.kind,
            amigo_overlay_api::UiOverlayNodeKind::Text { content, .. } if content == "Loading assets"
        )));
        assert!(content.children.iter().any(|node| matches!(
            &node.kind,
            amigo_overlay_api::UiOverlayNodeKind::ProgressBar { value } if (*value - 0.375).abs() < f32::EPSILON
        )));
    }
}

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket>
    for WgpuDevConsoleOverlayRenderExtractorBridge
{
    fn name(&self) -> &'static str {
        amigo_devtools::DevConsoleOverlayRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(dev_console_state) = runtime.resolve::<amigo_scripting_api::DevConsoleState>()
        else {
            return;
        };
        let Some(dev_console_completion) =
            runtime.resolve::<amigo_devtools::ConsoleCompletionState>()
        else {
            return;
        };
        let Some(ui_viewport_state) = runtime.resolve::<amigo_ui::UiInputViewportState>() else {
            return;
        };
        amigo_devtools::DevConsoleOverlayRenderExtractor.extract(
            dev_console_state.as_ref(),
            dev_console_completion.snapshot().as_ref(),
            ui_viewport_state.get(),
            &mut WgpuDevConsoleOverlayOutput(packet),
        );
    }
}

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket>
    for WgpuDebugOverlayRenderExtractorBridge
{
    fn name(&self) -> &'static str {
        amigo_devtools::DebugOverlayRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(debug_overlay_service) = optional::<amigo_devtools::DebugOverlayService>(runtime)
        else {
            return;
        };
        let Some(ui_viewport_state) = optional::<amigo_ui::UiInputViewportState>(runtime) else {
            return;
        };
        let snapshot = debug_overlay_service.snapshot();
        amigo_devtools::DebugOverlayRenderExtractor.extract(
            &snapshot,
            ui_viewport_state.get(),
            &mut WgpuDebugOverlayOutput(packet),
        );
    }
}
