use amigo_math::ColorRgba;
use amigo_render_wgpu::{
    UiOverlayDocument, UiOverlayLayer, UiOverlayNode, UiOverlayNodeKind, UiOverlayStyle,
    UiOverlayViewport, UiOverlayViewportScaling, UiViewportSize,
};

use super::graph::build_frame_time_graph_nodes;
use super::service::{DebugOverlayCorner, DebugOverlayLayoutMode, DebugOverlayPanel};
use super::snapshot::DebugOverlaySnapshot;
use super::theme::DebugOverlayTheme;

#[derive(Debug)]
struct OverlayLine {
    id: String,
    content: String,
    color: ColorRgba,
}

pub(crate) fn build_debug_overlay_document(
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
    let content_width = (layout.width * scale).min((viewport.width - layout.margin * scale * 2.0).max(180.0));

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
    if snapshot.settings.panels.contains(&DebugOverlayPanel::FpsGraph) {
        graph_enabled = true;
        lines.push(section_title("fps graph", theme.muted));
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Stats) {
        push_stats_lines(&mut lines, snapshot, &theme, snapshot.settings.layout_mode);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Render) {
        push_render_lines(&mut lines, snapshot, &theme, snapshot.settings.layout_mode);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Particles) {
        push_particles_lines(&mut lines, snapshot, &theme);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Scheduler) {
        push_scheduler_lines(&mut lines, snapshot, &theme);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Audio) {
        push_audio_lines(&mut lines, snapshot, &theme);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Input) {
        push_input_lines(&mut lines, snapshot, &theme);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Lights) {
        push_light_lines(&mut lines, snapshot, &theme);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Layers) {
        push_layer_lines(&mut lines, snapshot, &theme);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Timings) {
        push_timing_lines(&mut lines, snapshot, &theme);
    }
    if snapshot.settings.panels.contains(&DebugOverlayPanel::Memory) {
        lines.push(section_title("memory", theme.muted));
        lines.push(body_line("memory unavailable", theme.muted));
    }

    if lines.len() == 1 {
        lines.push(body_line("overlay enabled; no panels selected", theme.muted));
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
        let height = if index == 0 { header_height } else { line_height };
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
    let color = frame_time_color(sample.frame_ms, theme);
    lines.push(section_title("fps", theme.muted));
    match mode {
        DebugOverlayLayoutMode::Compact => lines.push(body_line(
            format!("FPS {:.1}  {:.1} ms", sample.fps, sample.frame_ms),
            color,
        )),
        DebugOverlayLayoutMode::Full => {
            lines.push(body_line(format!("FPS {:.1}", sample.fps), color));
            lines.push(body_line(
                format!("frame {:.1} ms  idx={}", sample.frame_ms, sample.frame_index),
                color,
            ));
        }
    }
}

fn push_stats_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
    mode: DebugOverlayLayoutMode,
) {
    let stats = &snapshot.render_stats;
    lines.push(section_title("stats", theme.muted));
    lines.push(body_line(
        format!(
            "frame={} window={}x{}",
            stats.frame_index, stats.window_width, stats.window_height
        ),
        theme.text,
    ));
    if matches!(mode, DebugOverlayLayoutMode::Full) {
        lines.push(body_line(
            format!(
                "particles={} ui={} emitters={}",
                stats.world_2d_particles, stats.ui_overlays, snapshot.particles.emitter_count
            ),
            theme.text,
        ));
    } else {
        lines.push(body_line(
            format!(
                "particles={} ui={}",
                stats.world_2d_particles, stats.ui_overlays
            ),
            theme.text,
        ));
    }
}

fn push_render_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
    mode: DebugOverlayLayoutMode,
) {
    let stats = &snapshot.render_stats;
    lines.push(section_title("render", theme.muted));
    lines.push(body_line(
        format!(
            "2d tile={} sprite={} layered={} particles={}",
            stats.world_2d_tilemaps,
            stats.world_2d_sprites,
            stats.world_2d_layered_images,
            stats.world_2d_particles
        ),
        theme.text,
    ));
    lines.push(body_line(
        format!(
            "2d vectors={} text={} layers={}",
            stats.world_2d_vectors, stats.world_2d_text, stats.world_2d_render_layers
        ),
        theme.text,
    ));
    if matches!(mode, DebugOverlayLayoutMode::Full) {
        lines.push(body_line(
            format!(
                "3d meshes={} materials={} text={}",
                stats.world_3d_meshes, stats.world_3d_materials, stats.world_3d_text
            ),
            theme.text,
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
            "draw={} emitters={} active={}",
            snapshot.render_stats.world_2d_particles,
            snapshot.particles.emitter_count,
            snapshot.particles.active_emitters
        ),
        theme.text,
    ));
}

fn push_scheduler_lines(
    lines: &mut Vec<OverlayLine>,
    snapshot: &DebugOverlaySnapshot,
    theme: &DebugOverlayTheme,
) {
    let stats = &snapshot.scheduling_stats;
    lines.push(section_title("scheduler", theme.muted));
    lines.push(body_line(
        format!(
            "mode={:?} particle_mode={}",
            stats.mode, stats.particle_mode
        ),
        theme.text,
    ));
    lines.push(body_line(
        format!(
            "particle={:.2} ms render={:.2} ms jobs={}/{}",
            stats.particle_update_ms,
            stats.render_prepare_ms,
            stats.worker_jobs_submitted,
            stats.worker_jobs_completed
        ),
        theme.text,
    ));
}

fn push_audio_lines(lines: &mut Vec<OverlayLine>, snapshot: &DebugOverlaySnapshot, theme: &DebugOverlayTheme) {
    let audio = &snapshot.audio;
    lines.push(section_title("audio", theme.muted));
    lines.push(body_line(
        format!(
            "{} started={} device={}",
            if audio.backend_name.is_empty() {
                "audio".to_owned()
            } else {
                audio.backend_name.clone()
            },
            audio.started,
            audio.device_name.as_deref().unwrap_or("none")
        ),
        theme.text,
    ));
    lines.push(body_line(
        format!(
            "buffered={} active={} pending={} buses={} volume={:.2}",
            audio.buffered_samples,
            audio.active_sources,
            audio.pending_commands,
            audio.bus_count,
            audio.master_volume
        ),
        if audio.started { theme.text } else { theme.warning },
    ));
    lines.push(body_line(
        format!(
            "rate={} channels={} error={}",
            audio.sample_rate.unwrap_or_default(),
            audio.channels.unwrap_or_default(),
            audio.last_error.as_deref().unwrap_or("none")
        ),
        if audio.last_error.is_some() {
            theme.warning
        } else {
            theme.muted
        },
    ));
}

fn push_input_lines(lines: &mut Vec<OverlayLine>, snapshot: &DebugOverlaySnapshot, theme: &DebugOverlayTheme) {
    let input = &snapshot.input;
    lines.push(section_title("input", theme.muted));
    lines.push(body_line(
        format!(
            "{} keys={}",
            input.backend_name.as_deref().unwrap_or("input"),
            if input.pressed_keys.is_empty() {
                "none".to_owned()
            } else {
                input.pressed_keys.join(",")
            }
        ),
        theme.text,
    ));
    lines.push(body_line(
        format!(
            "map={} actions={}",
            input.active_map.as_deref().unwrap_or("none"),
            if input.active_actions.is_empty() {
                "none".to_owned()
            } else {
                input.active_actions.join(",")
            }
        ),
        theme.text,
    ));
}

fn push_light_lines(lines: &mut Vec<OverlayLine>, snapshot: &DebugOverlaySnapshot, theme: &DebugOverlayTheme) {
    let stats = &snapshot.render_stats;
    lines.push(section_title("lights", theme.muted));
    lines.push(body_line(
        format!(
            "routes={} global={} maps={} groups={}",
            stats.world_2d_light_routes,
            stats.world_2d_global_lights,
            stats.world_2d_lightmaps,
            stats.world_2d_light_groups
        ),
        theme.text,
    ));
}

fn push_layer_lines(lines: &mut Vec<OverlayLine>, snapshot: &DebugOverlaySnapshot, theme: &DebugOverlayTheme) {
    let stats = &snapshot.render_stats;
    lines.push(section_title("layers", theme.muted));
    lines.push(body_line(
        format!(
            "render_layers={} ui_overlays={}",
            stats.world_2d_render_layers, stats.ui_overlays
        ),
        theme.text,
    ));
}

fn push_timing_lines(lines: &mut Vec<OverlayLine>, snapshot: &DebugOverlaySnapshot, theme: &DebugOverlayTheme) {
    let sample = snapshot.frame_history.last().cloned().unwrap_or_default();
    let scheduling = &snapshot.scheduling_stats;
    lines.push(section_title("timings", theme.muted));
    lines.push(body_line(
        format!(
            "frame={:.2} ms particle={:.2} ms render={:.2} ms",
            sample.frame_ms, scheduling.particle_update_ms, scheduling.render_prepare_ms
        ),
        frame_time_color(sample.frame_ms, theme),
    ));
}

fn section_title(title: impl Into<String>, color: ColorRgba) -> OverlayLine {
    let title = title.into();
    OverlayLine {
        id: format!("debug-overlay-section-{title}"),
        content: format!("[{title}]"),
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
            color: Some(color),
            font_size,
            fit_to_width: true,
            word_wrap: false,
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

#[cfg(test)]
mod tests {
    use amigo_render_wgpu::UiViewportSize;

    use crate::debug_overlay::{
        build_debug_overlay_document, DebugOverlayCorner, DebugOverlayLayoutMode,
        DebugOverlayService,
    };

    #[test]
    fn disabled_overlay_returns_none() {
        let overlay = DebugOverlayService::default();
        let snapshot = overlay.snapshot();

        assert!(build_debug_overlay_document(&snapshot, None).is_none());
    }

    #[test]
    fn enabled_overlay_uses_debug_layer() {
        let overlay = DebugOverlayService::default();
        overlay.set_enabled(true);
        let snapshot = overlay.snapshot();
        let document =
            build_debug_overlay_document(&snapshot, Some(UiViewportSize::new(1280.0, 720.0)))
                .expect("overlay should render");

        assert_eq!(document.entity_name, "debug-overlay");
        assert_eq!(document.layer, amigo_render_wgpu::UiOverlayLayer::Debug);
        assert_eq!(document.root.children.len(), 1);
    }

    #[test]
    fn bottom_right_corner_anchors_panel() {
        let overlay = DebugOverlayService::default();
        overlay.set_enabled(true);
        overlay.set_corner(DebugOverlayCorner::BottomRight);
        overlay.set_layout_mode(DebugOverlayLayoutMode::Full);
        let snapshot = overlay.snapshot();
        let document =
            build_debug_overlay_document(&snapshot, Some(UiViewportSize::new(1280.0, 720.0)))
                .expect("overlay should render");
        let panel = &document.root.children[0];

        assert!(panel.style.left.expect("left") > 600.0);
        assert!(panel.style.top.expect("top") > 500.0);
    }
}
