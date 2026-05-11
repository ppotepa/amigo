use amigo_math::ColorRgba;
use amigo_render_wgpu::{UiOverlayNode, UiOverlayNodeKind, UiOverlayStyle};

use super::snapshot::DebugOverlayFrameSample;

pub(crate) fn build_frame_time_graph_nodes(
    samples: &[DebugOverlayFrameSample],
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    good: ColorRgba,
    warning: ColorRgba,
    danger: ColorRgba,
) -> Vec<UiOverlayNode> {
    let recent = if samples.len() > 90 {
        &samples[samples.len() - 90..]
    } else {
        samples
    };

    if recent.is_empty() || width <= 0.0 || height <= 0.0 {
        return Vec::new();
    }

    let count = recent.len() as f32;
    let step = (width / count).max(1.0);
    let bar_width = step.max(1.0);
    let mut nodes = Vec::with_capacity(recent.len());

    for (index, sample) in recent.iter().enumerate() {
        let normalized = (sample.frame_ms / 40.0).clamp(0.0, 1.0);
        let bar_height = (height * normalized).max(1.0);
        let bar_left = left + index as f32 * step;
        let bar_top = top + height - bar_height;
        let color = if sample.frame_ms <= 16.7 {
            good
        } else if sample.frame_ms <= 25.0 {
            warning
        } else {
            danger
        };

        nodes.push(UiOverlayNode {
            id: Some(format!("debug-overlay-graph-bar-{index}")),
            kind: UiOverlayNodeKind::Panel,
            style: UiOverlayStyle {
                left: Some(bar_left),
                top: Some(bar_top),
                width: Some(bar_width),
                height: Some(bar_height),
                background: Some(color),
                ..UiOverlayStyle::default()
            },
            children: Vec::new(),
        });
    }

    nodes
}
