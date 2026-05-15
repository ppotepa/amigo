use amigo_math::ColorRgba;
use amigo_render_wgpu::{
    UiOverlayDocument, UiOverlayLayer, UiOverlayNode, UiOverlayNodeKind, UiOverlayStyle,
    UiOverlayViewport, UiOverlayViewportScaling, UiViewportSize,
};
use amigo_session::ResolvedFrameClockStrategy;

use crate::graph::build_frame_time_graph_nodes;
use crate::theme::DebugOverlayTheme;
use crate::{DebugOverlayCorner, DebugOverlayLayoutMode, DebugOverlayPanel, DebugOverlaySnapshot};

pub trait DebugOverlayRenderOutput {
    fn push_debug_overlay_document(&mut self, document: UiOverlayDocument);
}

impl DebugOverlayRenderOutput for amigo_render_wgpu::WgpuRenderFramePacket {
    fn push_debug_overlay_document(&mut self, document: UiOverlayDocument) {
        self.push_debug_overlay(document);
    }
}

pub struct DebugOverlayRenderExtractor;

impl DebugOverlayRenderExtractor {
    pub fn name(&self) -> &'static str {
        "debug_overlay"
    }

    pub fn extract(
        &self,
        snapshot: &DebugOverlaySnapshot,
        viewport: Option<UiViewportSize>,
        output: &mut impl DebugOverlayRenderOutput,
    ) {
        if let Some(document) = build_debug_overlay_document(snapshot, viewport) {
            output.push_debug_overlay_document(document);
        }
    }
}

#[derive(Debug)]
struct OverlayLine {
    id: String,
    content: String,
    color: ColorRgba,
}

pub fn build_debug_overlay_document(
    snapshot: &DebugOverlaySnapshot,
    viewport: Option<UiViewportSize>,
) -> Option<UiOverlayDocument> {
    if !snapshot.settings.enabled {
        return None;
    }

    let theme = DebugOverlayTheme::default();
    let viewport = viewport.unwrap_or_else(|| {
        UiViewportSize::new(
            theme.viewport.fallback_width,
            theme.viewport.fallback_height,
        )
    });
    let layout = theme.layout(snapshot.settings.layout_mode);
    let scale = snapshot.settings.scale.clamp(0.5, 3.0);
    let line_height = layout.line_height * scale;
    let header_height = layout.header_height * scale;
    let padding = layout.padding * scale;
    let section_gap = layout.section_gap * scale;
    let body_font_size = layout.body_font_size * scale;
    let header_font_size = layout.header_font_size * scale;
    let content_width =
        (layout.width * scale).min((viewport.width - layout.margin * scale * 2.0).max(180.0));

    let mut lines = vec![OverlayLine {
        id: "debug-overlay-header-line".to_owned(),
        content: format!(
            "DEBUG OVERLAY  {}",
            match snapshot.settings.layout_mode {
                DebugOverlayLayoutMode::Compact => "compact",
                DebugOverlayLayoutMode::Full => "full",
            }
        ),
        color: theme.text,
    }];
    let mut graph_enabled = false;

    if snapshot.settings.panels.contains(&DebugOverlayPanel::Fps) {
        push_fps_lines(&mut lines, snapshot, &theme, snapshot.settings.layout_mode);
    }
    if snapshot
        .settings
        .panels
        .contains(&DebugOverlayPanel::FpsGraph)
    {
        graph_enabled = true;
        lines.push(section_title("fps graph", theme.muted));
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Stats) {
        push_stats_lines(&mut lines, snapshot, &theme, snapshot.settings.layout_mode);
    }
    if snapshot
        .settings
        .panels
        .contains(&DebugOverlayPanel::Render)
    {
        push_render_lines(&mut lines, snapshot, &theme, snapshot.settings.layout_mode);
    }
    if snapshot
        .settings
        .panels
        .contains(&DebugOverlayPanel::Particles)
    {
        push_particles_lines(&mut lines, snapshot, &theme);
    }
    if snapshot
        .settings
        .panels
        .contains(&DebugOverlayPanel::Scheduler)
    {
        push_scheduler_lines(&mut lines, snapshot, &theme);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Audio) {
        push_audio_lines(&mut lines, snapshot, &theme);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Input) {
        push_input_lines(&mut lines, snapshot, &theme);
    }
    if snapshot
        .settings
        .panels
        .contains(&DebugOverlayPanel::Lights)
    {
        push_light_lines(&mut lines, snapshot, &theme);
    }
    if snapshot
        .settings
        .panels
        .contains(&DebugOverlayPanel::Layers)
    {
        push_layer_lines(&mut lines, snapshot, &theme);
    }
    if snapshot
        .settings
        .panels
        .contains(&DebugOverlayPanel::Timings)
    {
        push_timing_lines(&mut lines, snapshot, &theme);
    }
    if snapshot
        .settings
        .panels
        .contains(&DebugOverlayPanel::Memory)
    {
        lines.push(section_title("memory", theme.muted));
        lines.push(body_line("memory unavailable", theme.muted));
    }

    if lines.len() == 1 {
        lines.push(body_line(
            "overlay enabled; no panels selected",
            theme.muted,
        ));
    }

    let graph_height = if graph_enabled {
        layout.graph_height * scale + section_gap
    } else {
        0.0
    };
    let panel_height = (padding * 2.0
        + header_height
        + (lines.len().saturating_sub(1) as f32 * line_height)
        + graph_height
        + section_gap)
        .min((viewport.height - layout.margin * scale * 2.0).max(120.0));

    let (panel_left, panel_top) = anchor_panel(
        snapshot.settings.corner,
        viewport,
        content_width,
        panel_height,
        layout.margin * scale,
    );

    let mut children = Vec::new();
    let mut current_top = 0.0;
    for (index, line) in lines.into_iter().enumerate() {
        let font_size = if index == 0 {
            header_font_size
        } else {
            body_font_size
        };
        let height = if index == 0 {
            header_height
        } else {
            line_height
        };
        children.push(text_node(
            line.id,
            line.content,
            0.0,
            current_top,
            content_width - padding * 2.0,
            height,
            font_size,
            line.color,
            theme.font.clone(),
        ));
        current_top += height;
    }

    if graph_enabled {
        let graph_top = current_top + section_gap * 0.5;
        let graph_width = (content_width - padding * 2.0).max(80.0);
        let graph_height = layout.graph_height * scale;
        children.push(panel_node(
            "debug-overlay-graph-track",
            0.0,
            graph_top,
            graph_width,
            graph_height,
            ColorRgba::new(0.20, 0.80, 1.00, 0.12),
        ));
        children.extend(build_frame_time_graph_nodes(
            &snapshot.frame_history,
            0.0,
            graph_top,
            graph_width,
            graph_height,
            theme.good,
            theme.warning,
            theme.danger,
        ));
    }

    Some(UiOverlayDocument {
        entity_name: "debug-overlay".to_owned(),
        layer: UiOverlayLayer::Debug,
        viewport: Some(UiOverlayViewport {
            width: viewport.width,
            height: viewport.height,
            scaling: UiOverlayViewportScaling::Expand,
        }),
        root: UiOverlayNode {
            id: Some("debug-overlay-root".to_owned()),
            kind: UiOverlayNodeKind::Stack,
            style: UiOverlayStyle {
                width: Some(viewport.width),
                height: Some(viewport.height),
                ..UiOverlayStyle::default()
            },
            children: vec![UiOverlayNode {
                id: Some("debug-overlay-panel".to_owned()),
                kind: UiOverlayNodeKind::Stack,
                style: UiOverlayStyle {
                    left: Some(panel_left),
                    top: Some(panel_top),
                    width: Some(content_width),
                    height: Some(panel_height),
                    padding,
                    background: Some(theme.panel_background),
                    border_color: Some(theme.panel_border),
                    border_width: layout.border_width,
                    border_radius: layout.border_radius,
                    ..UiOverlayStyle::default()
                },
                children,
            }],
        },
    })
}

fn push_fps_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
    mode: DebugOverlayLayoutMode,
) {
    let sample = snapshot.frame_history.last().cloned().unwrap_or_default();
    lines.push(section_title("frame", theme.muted));
    lines.push(body_line(
        format!("{:.1} fps  {:.1} ms", sample.fps, sample.frame_ms),
        frame_time_color(sample.frame_ms, theme),
    ));
    match mode {
        DebugOverlayLayoutMode::Compact => lines.push(body_line(
            format!("frame {}", snapshot.render_stats.frame_index),
            theme.muted,
        )),
        DebugOverlayLayoutMode::Full => {
            lines.push(body_line(
                format!("frame {}", snapshot.render_stats.frame_index),
                theme.muted,
            ));
            if let Some(clock) = &snapshot.frame_clock {
                lines.push(body_line(
                    format!(
                        "clock {} host {:.1} game {:.1}/{:.1}",
                        frame_clock_strategy_label(clock.strategy),
                        clock.actual_host_fps,
                        clock.actual_game_render_fps,
                        clock.target_render_fps,
                    ),
                    theme.muted,
                ));
                lines.push(body_line(
                    format!(
                        "sim target {:.1} dt {:.4} pending {} cached={}",
                        clock.target_simulation_fps,
                        clock.simulation_delta_seconds,
                        clock.pending_simulation_ticks,
                        clock.holding_cached_game_frame,
                    ),
                    theme.muted,
                ));
            }
            lines.push(body_line(
                format!(
                    "window {}x{}",
                    snapshot.render_stats.window_width, snapshot.render_stats.window_height
                ),
                theme.muted,
            ));
        }
    }
}

fn frame_clock_strategy_label(strategy: ResolvedFrameClockStrategy) -> &'static str {
    match strategy {
        ResolvedFrameClockStrategy::HostRealtime => "host_realtime",
        ResolvedFrameClockStrategy::FixedUpdateAndRender => "fixed_update_and_render",
        ResolvedFrameClockStrategy::FixedSimulationSampledRender => {
            "fixed_simulation_sampled_render"
        }
        ResolvedFrameClockStrategy::RealtimeUpdateSampledRender => "realtime_update_sampled_render",
    }
}

fn push_stats_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
    mode: DebugOverlayLayoutMode,
) {
    lines.push(section_title("stats", theme.muted));
    lines.push(body_line(
        format!(
            "ui={} debug={} graph={} postfx={}",
            snapshot.render_stats.ui_overlays,
            snapshot.render_stats.debug_overlays,
            snapshot.render_stats.render_graph_nodes,
            snapshot.render_stats.post_fx_effects,
        ),
        theme.text,
    ));
    if matches!(mode, DebugOverlayLayoutMode::Full) {
        lines.push(body_line(
            format!(
                "scheduler {:?} jobs={}/{} waited={}",
                snapshot.scheduling_stats.mode,
                snapshot.scheduling_stats.worker_jobs_submitted,
                snapshot.scheduling_stats.worker_jobs_completed,
                snapshot.scheduling_stats.worker_waited_this_frame,
            ),
            theme.muted,
        ));
    }
}

fn push_render_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
    mode: DebugOverlayLayoutMode,
) {
    lines.push(section_title("render", theme.muted));
    lines.push(body_line(
        format!(
            "2d sprites={} text={} particles={}",
            snapshot.render_stats.world_2d_sprites,
            snapshot.render_stats.world_2d_text,
            snapshot.render_stats.world_2d_particles,
        ),
        theme.text,
    ));
    lines.push(body_line(
        format!(
            "3d meshes={} materials={} text={}",
            snapshot.render_stats.world_3d_meshes,
            snapshot.render_stats.world_3d_materials,
            snapshot.render_stats.world_3d_text,
        ),
        theme.text,
    ));
    if matches!(mode, DebugOverlayLayoutMode::Full) {
        lines.push(body_line(
            format!(
                "tilemaps={} layered={} vectors={}",
                snapshot.render_stats.world_2d_tilemaps,
                snapshot.render_stats.world_2d_layered_images,
                snapshot.render_stats.world_2d_vectors,
            ),
            theme.muted,
        ));
    }
}

fn push_particles_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
) {
    lines.push(section_title("particles", theme.muted));
    lines.push(body_line(
        format!(
            "emitters={} active={}",
            snapshot.particles.emitter_count, snapshot.particles.active_emitters
        ),
        theme.text,
    ));
}

fn push_scheduler_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
) {
    lines.push(section_title("scheduler", theme.muted));
    lines.push(body_line(
        format!(
            "mode={:?} jobs={}/{}",
            snapshot.scheduling_stats.mode,
            snapshot.scheduling_stats.worker_jobs_submitted,
            snapshot.scheduling_stats.worker_jobs_completed,
        ),
        theme.text,
    ));
    lines.push(body_line(
        format!(
            "in_flight={} reused={} waited={}",
            snapshot.scheduling_stats.particle_job_in_flight,
            snapshot.scheduling_stats.reused_previous_particle_frame,
            snapshot.scheduling_stats.worker_waited_this_frame,
        ),
        theme.muted,
    ));
}

fn push_audio_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
) {
    lines.push(section_title("audio", theme.muted));
    lines.push(body_line(
        format!(
            "{} active={} buffered={}",
            if snapshot.audio.backend_name.is_empty() {
                "audio"
            } else {
                snapshot.audio.backend_name.as_str()
            },
            snapshot.audio.active_sources,
            snapshot.audio.buffered_samples,
        ),
        theme.text,
    ));
    lines.push(body_line(
        format!(
            "started={} volume={:.2} pending={} buses={}",
            snapshot.audio.started,
            snapshot.audio.master_volume,
            snapshot.audio.pending_commands,
            snapshot.audio.bus_count,
        ),
        theme.muted,
    ));
    if let Some(device) = &snapshot.audio.device_name {
        lines.push(body_line(format!("device {device}"), theme.muted));
    }
}

fn push_input_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
) {
    lines.push(section_title("input", theme.muted));
    lines.push(body_line(
        format!(
            "map={} keys={}",
            snapshot.input.active_map.as_deref().unwrap_or("none"),
            if snapshot.input.pressed_keys.is_empty() {
                "none".to_owned()
            } else {
                snapshot.input.pressed_keys.join(",")
            }
        ),
        theme.text,
    ));
    lines.push(body_line(
        format!(
            "actions={}",
            if snapshot.input.active_actions.is_empty() {
                "none".to_owned()
            } else {
                snapshot.input.active_actions.join(",")
            }
        ),
        theme.muted,
    ));
}

fn push_light_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
) {
    lines.push(section_title("lights", theme.muted));
    lines.push(body_line(
        format!(
            "global={} maps={} groups={}",
            snapshot.render_stats.world_2d_global_lights,
            snapshot.render_stats.world_2d_lightmaps,
            snapshot.render_stats.world_2d_light_groups,
        ),
        theme.text,
    ));
}

fn push_layer_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
) {
    lines.push(section_title("layers", theme.muted));
    lines.push(body_line(
        format!(
            "render={} routes={}",
            snapshot.render_stats.world_2d_render_layers,
            snapshot.render_stats.world_2d_light_routes,
        ),
        theme.text,
    ));
}

fn push_timing_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
) {
    let sample = snapshot.frame_history.last().cloned().unwrap_or_default();
    lines.push(section_title("timings", theme.muted));
    lines.push(body_line(
        format!("{:.2} ms / {:.1} fps", sample.frame_ms, sample.fps),
        frame_time_color(sample.frame_ms, theme),
    ));
}

fn section_title(title: impl Into<String>, color: ColorRgba) -> OverlayLine {
    let content = title.into();
    OverlayLine {
        id: format!("debug-overlay-title-{}", randless_id_seed(&content)),
        content,
        color,
    }
}

fn body_line(content: impl Into<String>, color: ColorRgba) -> OverlayLine {
    let content = content.into();
    OverlayLine {
        id: format!("debug-overlay-line-{}", randless_id_seed(&content)),
        content,
        color,
    }
}

fn text_node(
    id: impl Into<String>,
    content: String,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    font_size: f32,
    color: ColorRgba,
    font: Option<amigo_assets::AssetKey>,
) -> UiOverlayNode {
    UiOverlayNode {
        id: Some(id.into()),
        kind: UiOverlayNodeKind::Text { content, font },
        style: UiOverlayStyle {
            left: Some(left),
            top: Some(top),
            width: Some(width),
            height: Some(height),
            font_size,
            color: Some(color),
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

fn panel_node(
    id: impl Into<String>,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    background: ColorRgba,
) -> UiOverlayNode {
    UiOverlayNode {
        id: Some(id.into()),
        kind: UiOverlayNodeKind::Panel,
        style: UiOverlayStyle {
            left: Some(left),
            top: Some(top),
            width: Some(width),
            height: Some(height),
            background: Some(background),
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

fn anchor_panel(
    corner: DebugOverlayCorner,
    viewport: UiViewportSize,
    width: f32,
    height: f32,
    margin: f32,
) -> (f32, f32) {
    match corner {
        DebugOverlayCorner::TopLeft => (margin, margin),
        DebugOverlayCorner::TopRight => ((viewport.width - width - margin).max(margin), margin),
        DebugOverlayCorner::BottomLeft => (margin, (viewport.height - height - margin).max(margin)),
        DebugOverlayCorner::BottomRight => (
            (viewport.width - width - margin).max(margin),
            (viewport.height - height - margin).max(margin),
        ),
    }
}

fn frame_time_color(frame_ms: f32, theme: &DebugOverlayTheme) -> ColorRgba {
    if frame_ms <= 16.7 {
        theme.good
    } else if frame_ms <= 25.0 {
        theme.warning
    } else {
        theme.danger
    }
}

fn randless_id_seed(text: &str) -> u64 {
    text.bytes()
        .fold(1469598103934665603_u64, |hash, byte| hash ^ byte as u64)
}
