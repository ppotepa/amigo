use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

use super::shared::overlay_service;

pub(crate) struct DebugDumpCommandHandler;

impl ConsoleCommandHandler for DebugDumpCommandHandler {
    fn name(&self) -> &'static str {
        "debug-dump"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.dump",
            aliases: &["debug.snapshot"],
            category: "debug",
            help: "Dump the current debug overlay snapshot to the console.",
            usage: "debug.dump",
            examples: &["debug.dump"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        matches!(command.name.as_str(), "debug.dump" | "debug.snapshot")
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        _command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let overlay = match overlay_service(ctx) {
            Ok(service) => service,
            Err(result) => return result,
        };
        let snapshot = overlay.snapshot();
        let sample = snapshot.frame_history.last().cloned().unwrap_or_default();
        let panels = snapshot
            .settings
            .panels
            .iter()
            .map(|panel| panel.as_str())
            .collect::<Vec<_>>();

        ConsoleCommandResult::ok(format!(
            "debug overlay:\n  enabled={} layout={:?} corner={:?} scale={:.2}\n  panels=[{}]\n  fps={:.1} frame_ms={:.1}\n  frame={} window={}x{} particles={} ui_overlays={}\n  scheduler={:?} jobs={}/{}\n  audio={} started={} active={} buffered={}\n  input map={} keys={} actions={}",
            snapshot.settings.enabled,
            snapshot.settings.layout_mode,
            snapshot.settings.corner,
            snapshot.settings.scale,
            panels.join(","),
            sample.fps,
            sample.frame_ms,
            snapshot.render_stats.frame_index,
            snapshot.render_stats.window_width,
            snapshot.render_stats.window_height,
            snapshot.render_stats.world_2d_particles,
            snapshot.render_stats.ui_overlays,
            snapshot.scheduling_stats.mode,
            snapshot.scheduling_stats.worker_jobs_submitted,
            snapshot.scheduling_stats.worker_jobs_completed,
            if snapshot.audio.backend_name.is_empty() {
                "audio"
            } else {
                snapshot.audio.backend_name.as_str()
            },
            snapshot.audio.started,
            snapshot.audio.active_sources,
            snapshot.audio.buffered_samples,
            snapshot.input.active_map.as_deref().unwrap_or("none"),
            if snapshot.input.pressed_keys.is_empty() {
                "none".to_owned()
            } else {
                snapshot.input.pressed_keys.join(",")
            },
            if snapshot.input.active_actions.is_empty() {
                "none".to_owned()
            } else {
                snapshot.input.active_actions.join(",")
            },
        ))
    }
}
