use std::io::{self, Stdout};
use std::path::PathBuf;

use amigo_core::AmigoResult;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::config::LauncherConfig;

use super::{FocusPane, LaunchMode, LauncherTuiState, TuiOutcome};
use super::filtering::is_scene_filter_character;
use super::render::render;

pub fn run_launcher_tui(
    config_path: impl Into<PathBuf>,
    config: LauncherConfig,
) -> AmigoResult<TuiOutcome> {
    let mut state = LauncherTuiState::new(config_path.into(), config)?;
    let mut stdout = io::stdout();

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_event_loop(&mut terminal, &mut state);
    let cleanup = restore_terminal(&mut terminal);

    cleanup?;
    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut LauncherTuiState,
) -> AmigoResult<TuiOutcome> {
    loop {
        terminal.draw(|frame| render(frame, state))?;

        let Event::Key(key_event) = event::read()? else {
            continue;
        };

        if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            continue;
        }

        match key_event.code {
            KeyCode::Tab => state.move_focus_next(),
            KeyCode::BackTab => state.move_focus_previous(),
            KeyCode::Left => match state.focus {
                FocusPane::Profiles => state.move_selection(-1),
                FocusPane::Tree => state.collapse_selected_mod_or_parent(),
            },
            KeyCode::Right => match state.focus {
                FocusPane::Profiles => state.move_selection(1),
                FocusPane::Tree => state.expand_selected_mod(),
            },
            KeyCode::Up => state.move_selection(-1),
            KeyCode::Down => state.move_selection(1),
            KeyCode::Backspace if !state.scene_filter.is_empty() => state.pop_scene_filter(),
            KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                state.save_config()?
            }
            KeyCode::Char('r') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                state.reload_config()?
            }
            KeyCode::Char('o') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                state.toggle_hosted_default()
            }
            KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(outcome) = state.try_launch_focused(LaunchMode::Headless) {
                    return Ok(outcome);
                }
            }
            KeyCode::Char('h') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(outcome) = state.try_launch_focused(LaunchMode::Hosted) {
                    return Ok(outcome);
                }
            }
            KeyCode::Esc if !state.scene_filter.is_empty() => {
                state.clear_scene_filter();
            }
            KeyCode::Char(character) if is_scene_filter_character(character) => {
                state.append_scene_filter(character);
            }
            KeyCode::Char(' ') if state.focus == FocusPane::Tree => {
                state.toggle_selected_expansion();
            }
            KeyCode::Esc => {
                return Ok(TuiOutcome::Quit);
            }
            KeyCode::Enter => {
                if let Some(outcome) = state.activate_focused() {
                    return Ok(outcome);
                }
            }
            _ => {}
        }
    }
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> AmigoResult<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
