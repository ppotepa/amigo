use std::sync::Arc;

use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{ConsoleCommandResult, ParsedConsoleCommand};
use crate::{DebugOverlayPanel, DebugOverlayService};

pub(crate) enum ToggleAction {
    On,
    Off,
    Toggle,
}

pub(crate) fn overlay_service(
    ctx: &ConsoleCommandContext<'_>,
) -> Result<Arc<DebugOverlayService>, ConsoleCommandResult> {
    ctx.required::<DebugOverlayService>()
        .map_err(|error| ConsoleCommandResult::error(error.to_string()))
}

pub(crate) fn parse_toggle_action(
    command: &ParsedConsoleCommand,
) -> Result<ToggleAction, ConsoleCommandResult> {
    match command.args.first().map(String::as_str) {
        Some("on") => Ok(ToggleAction::On),
        Some("off") => Ok(ToggleAction::Off),
        Some("toggle") | None => Ok(ToggleAction::Toggle),
        Some(value) => Err(ConsoleCommandResult::error(format!(
            "invalid value `{value}`; expected on, off, or toggle"
        ))),
    }
}

pub(crate) fn apply_panel_toggle(
    ctx: &ConsoleCommandContext<'_>,
    command: &ParsedConsoleCommand,
    panel: DebugOverlayPanel,
    label: &str,
) -> ConsoleCommandResult {
    let overlay = match overlay_service(ctx) {
        Ok(service) => service,
        Err(result) => return result,
    };

    let enabled = match parse_toggle_action(command) {
        Ok(ToggleAction::On) => {
            overlay.set_panel_visible(panel, true);
            true
        }
        Ok(ToggleAction::Off) => {
            overlay.set_panel_visible(panel, false);
            false
        }
        Ok(ToggleAction::Toggle) => overlay.toggle_panel(panel),
        Err(result) => return result,
    };

    ConsoleCommandResult::ok(format!("{label} {}", state_label(enabled)))
}

pub(crate) fn apply_panel_group_toggle(
    ctx: &ConsoleCommandContext<'_>,
    command: &ParsedConsoleCommand,
    panels: &[DebugOverlayPanel],
    label: &str,
) -> ConsoleCommandResult {
    let overlay = match overlay_service(ctx) {
        Ok(service) => service,
        Err(result) => return result,
    };

    let enabled = match parse_toggle_action(command) {
        Ok(ToggleAction::On) => {
            for panel in panels {
                overlay.set_panel_visible(*panel, true);
            }
            true
        }
        Ok(ToggleAction::Off) => {
            for panel in panels {
                overlay.set_panel_visible(*panel, false);
            }
            false
        }
        Ok(ToggleAction::Toggle) => {
            let snapshot = overlay.snapshot();
            let next_state = panels
                .iter()
                .any(|panel| !snapshot.settings.panels.contains(panel));
            for panel in panels {
                overlay.set_panel_visible(*panel, next_state);
            }
            next_state
        }
        Err(result) => return result,
    };

    ConsoleCommandResult::ok(format!("{label} {}", state_label(enabled)))
}

pub(crate) fn state_label(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}
