use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Stdout};
use std::path::{Path, PathBuf};

use amigo_core::AmigoResult;
use amigo_modding::{ModCatalog, ModSceneManifest, requested_mods_for_root};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap,
};

use crate::config::{LauncherConfig, LauncherProfile};
use crate::diagnostics::{DiagnosticSeverity, ProfileDiagnostics, collect_profile_diagnostics};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    Headless,
    Hosted,
}

#[derive(Debug, Clone)]
pub enum TuiOutcome {
    Launch {
        config: LauncherConfig,
        mode: LaunchMode,
    },
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusPane {
    Profiles,
    Mods,
    Scenes,
}

#[derive(Debug, Clone)]
struct KnownMod {
    id: String,
    name: String,
    description: String,
    scenes: Vec<ModSceneManifest>,
    discovered: bool,
}

#[derive(Debug, Clone)]
struct LauncherTuiState {
    config_path: PathBuf,
    config: LauncherConfig,
    known_mods: Vec<KnownMod>,
    profile_diagnostics: BTreeMap<String, ProfileDiagnostics>,
    resolved_mod_ids: Vec<String>,
    focus: FocusPane,
    selected_profile_index: usize,
    selected_mod_index: usize,
    selected_scene_index: usize,
    dirty: bool,
    status: String,
}

impl LauncherTuiState {
    fn new(config_path: PathBuf, config: LauncherConfig) -> AmigoResult<Self> {
        let known_mods = discover_known_mods(&config)?;
        let profile_diagnostics = collect_profile_diagnostics(&config);
        let mut state = Self {
            config_path,
            config,
            known_mods,
            profile_diagnostics,
            resolved_mod_ids: Vec::new(),
            focus: FocusPane::Profiles,
            selected_profile_index: 0,
            selected_mod_index: 0,
            selected_scene_index: 0,
            dirty: false,
            status: "Profiles on top. Enter on a mod or scene opens the hosted app.".to_owned(),
        };
        state.sync_selection_from_active_profile();
        Ok(state)
    }

    fn active_profile(&self) -> &LauncherProfile {
        self.config
            .active_profile()
            .expect("launcher TUI state should always contain a valid active profile")
    }

    fn active_profile_mut(&mut self) -> &mut LauncherProfile {
        self.config
            .active_profile_mut()
            .expect("launcher TUI state should always contain a valid active profile")
    }

    fn active_profile_diagnostics(&self) -> Option<&ProfileDiagnostics> {
        self.profile_diagnostics.get(&self.active_profile().id)
    }

    fn selected_mod(&self) -> Option<&KnownMod> {
        self.known_mods.get(self.selected_mod_index)
    }

    fn selected_scene(&self) -> Option<ModSceneManifest> {
        self.current_scene_list()
            .get(self.selected_scene_index)
            .cloned()
    }

    fn current_scene_list(&self) -> Vec<ModSceneManifest> {
        self.selected_mod()
            .map(|known_mod| known_mod.scenes.clone())
            .unwrap_or_default()
    }

    fn sync_selection_from_active_profile(&mut self) {
        self.selected_profile_index = self
            .config
            .profiles
            .iter()
            .position(|profile| profile.id == self.config.active_profile)
            .unwrap_or(0);

        let active_profile = self.active_profile();
        self.selected_mod_index = active_profile
            .root_mod
            .as_deref()
            .and_then(|root_mod| {
                self.known_mods
                    .iter()
                    .position(|known_mod| known_mod.id == root_mod)
            })
            .unwrap_or(0);

        self.refresh_resolved_mods();
        self.sync_scene_selection_for_current_mod();
    }

    fn refresh_resolved_mods(&mut self) {
        let root_mod = self.active_profile().root_mod_or_core().to_owned();
        self.resolved_mod_ids = self
            .active_profile_diagnostics()
            .filter(|report| !report.resolved_mod_ids.is_empty())
            .map(|report| report.resolved_mod_ids.clone())
            .unwrap_or_else(|| requested_mods_for_root(&root_mod));
    }

    fn refresh_profile_diagnostics(&mut self) {
        self.profile_diagnostics = collect_profile_diagnostics(&self.config);
        self.refresh_resolved_mods();
    }

    fn sync_scene_selection_for_current_mod(&mut self) {
        let startup_scene = self.active_profile().startup_scene.clone();
        let scenes = self.current_scene_list();

        self.selected_scene_index = startup_scene
            .as_deref()
            .and_then(|scene_id| scenes.iter().position(|scene| scene.id == scene_id))
            .unwrap_or(0);
    }

    fn move_focus_next(&mut self) {
        self.focus = match self.focus {
            FocusPane::Profiles => FocusPane::Mods,
            FocusPane::Mods => FocusPane::Scenes,
            FocusPane::Scenes => FocusPane::Profiles,
        };
        self.status = format!("focus: {}", self.focus_label());
    }

    fn move_focus_previous(&mut self) {
        self.focus = match self.focus {
            FocusPane::Profiles => FocusPane::Scenes,
            FocusPane::Mods => FocusPane::Profiles,
            FocusPane::Scenes => FocusPane::Mods,
        };
        self.status = format!("focus: {}", self.focus_label());
    }

    fn focus_label(&self) -> &'static str {
        match self.focus {
            FocusPane::Profiles => "profiles",
            FocusPane::Mods => "mods",
            FocusPane::Scenes => "scenes",
        }
    }

    fn focus_left_panel(&mut self) {
        self.focus = FocusPane::Mods;
        self.status = "focus: left panel / mods".to_owned();
    }

    fn focus_right_panel(&mut self) {
        self.focus = FocusPane::Scenes;
        self.status = "focus: right panel / scenes".to_owned();
    }

    fn move_selection(&mut self, delta: isize) {
        match self.focus {
            FocusPane::Profiles => {
                self.selected_profile_index = wrapped_next_index(
                    self.selected_profile_index,
                    self.config.profiles.len(),
                    delta,
                );
            }
            FocusPane::Mods => {
                self.selected_mod_index =
                    wrapped_next_index(self.selected_mod_index, self.known_mods.len(), delta);
                self.sync_scene_selection_for_current_mod();
            }
            FocusPane::Scenes => {
                self.selected_scene_index = wrapped_next_index(
                    self.selected_scene_index,
                    self.current_scene_list().len(),
                    delta,
                );
            }
        }
    }

    fn activate_focused(&mut self) -> Option<TuiOutcome> {
        match self.focus {
            FocusPane::Profiles => {
                let Some(profile) = self.config.profiles.get(self.selected_profile_index) else {
                    return None;
                };
                let selected_profile_id = profile.id.clone();
                self.config
                    .set_active_profile(&selected_profile_id)
                    .expect("selected TUI profile should exist");
                self.sync_selection_from_active_profile();
                self.dirty = true;
                self.status = format!("active profile set to `{selected_profile_id}`");
                None
            }
            FocusPane::Mods | FocusPane::Scenes => self.try_launch_focused(LaunchMode::Hosted),
        }
    }

    fn toggle_hosted_default(&mut self) {
        let (profile_id, hosted_default) = {
            let profile = self.active_profile_mut();
            profile.hosted_default = !profile.hosted_default;
            (profile.id.clone(), profile.hosted_default)
        };
        self.refresh_profile_diagnostics();
        self.dirty = true;
        self.status = format!(
            "profile `{}` hosted_default set to {}",
            profile_id, hosted_default
        );
    }

    fn set_root_mod(&mut self, mod_id: &str) {
        let mod_scenes = self
            .find_mod(mod_id)
            .map(|known_mod| known_mod.scenes.clone())
            .unwrap_or_default();
        let profile = self.active_profile_mut();
        profile.root_mod = Some(mod_id.to_owned());
        profile.startup_scene = mod_scenes.first().map(|scene| scene.id.clone());

        self.refresh_profile_diagnostics();
        self.dirty = true;
        self.status = format!(
            "root mod set to `{mod_id}` with startup scene `{}`",
            self.active_profile()
                .startup_scene
                .as_deref()
                .unwrap_or("none")
        );
        self.sync_selection_from_active_profile();
    }

    fn set_startup_scene(&mut self, scene_id: &str) {
        let profile = self.active_profile_mut();
        profile.startup_scene = Some(scene_id.to_owned());
        self.refresh_profile_diagnostics();
        self.dirty = true;
        self.status = format!("startup scene set to `{scene_id}`");
        self.sync_scene_selection_for_current_mod();
    }

    fn save_config(&mut self) -> AmigoResult<()> {
        self.config.save(&self.config_path)?;
        self.dirty = false;
        self.status = format!("saved launcher config to `{}`", self.config_path.display());
        Ok(())
    }

    fn reload_config(&mut self) -> AmigoResult<()> {
        let config = LauncherConfig::load(&self.config_path)?;
        config.validate_phase1()?;
        self.known_mods = discover_known_mods(&config)?;
        self.config = config;
        self.refresh_profile_diagnostics();
        self.dirty = false;
        self.sync_selection_from_active_profile();
        self.status = format!(
            "reloaded launcher config from `{}`",
            self.config_path.display()
        );
        Ok(())
    }

    fn try_launch(&mut self, mode: LaunchMode) -> Option<TuiOutcome> {
        let Some(report) = self.active_profile_diagnostics().cloned() else {
            let profile_id = self.active_profile().id.clone();
            self.status = format!("profile `{profile_id}` has no diagnostics report");
            return None;
        };
        let profile_id = report.profile_id.clone();

        if !report.is_launchable() {
            let first_error = report
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
                .map(|diagnostic| diagnostic.message.clone())
                .unwrap_or_else(|| "unknown launch error".to_owned());
            self.status = format!("profile `{profile_id}` is blocked: {first_error}");
            return None;
        }

        if report.warning_count() > 0 {
            self.status = format!(
                "launching profile `{profile_id}` with {} warning(s)",
                report.warning_count()
            );
        } else {
            self.status = format!("launching profile `{profile_id}`");
        }

        Some(TuiOutcome::Launch {
            config: self.config.clone(),
            mode,
        })
    }

    fn sync_profile_to_focused_selection(&mut self) {
        match self.focus {
            FocusPane::Profiles => {}
            FocusPane::Mods => {
                if let Some(mod_id) = self.selected_mod().map(|known_mod| known_mod.id.clone()) {
                    self.set_root_mod(&mod_id);
                }
            }
            FocusPane::Scenes => {
                if let Some(root_mod) = self.selected_mod().map(|known_mod| known_mod.id.clone()) {
                    if self.active_profile().root_mod.as_deref() != Some(root_mod.as_str()) {
                        self.set_root_mod(&root_mod);
                    }
                }

                if let Some(scene) = self.selected_scene() {
                    self.set_startup_scene(&scene.id);
                }
            }
        }
    }

    fn try_launch_focused(&mut self, mode: LaunchMode) -> Option<TuiOutcome> {
        self.sync_profile_to_focused_selection();
        self.try_launch(mode)
    }

    fn find_mod(&self, mod_id: &str) -> Option<&KnownMod> {
        self.known_mods
            .iter()
            .find(|known_mod| known_mod.id == mod_id)
    }
}

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
                FocusPane::Mods | FocusPane::Scenes => state.focus_left_panel(),
            },
            KeyCode::Right => match state.focus {
                FocusPane::Profiles => state.move_selection(1),
                FocusPane::Mods | FocusPane::Scenes => state.focus_right_panel(),
            },
            KeyCode::Up => state.move_selection(-1),
            KeyCode::Down => state.move_selection(1),
            KeyCode::Enter => {
                if let Some(outcome) = state.activate_focused() {
                    return Ok(outcome);
                }
            }
            KeyCode::Char('o') | KeyCode::Char('O') => state.toggle_hosted_default(),
            KeyCode::Char('s') | KeyCode::Char('S') => state.save_config()?,
            KeyCode::Char('r') | KeyCode::Char('R') => state.reload_config()?,
            KeyCode::Char('l') | KeyCode::Char('L') => {
                if let Some(outcome) = state.try_launch_focused(LaunchMode::Headless) {
                    return Ok(outcome);
                }
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                if let Some(outcome) = state.try_launch_focused(LaunchMode::Hosted) {
                    return Ok(outcome);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                return Ok(TuiOutcome::Quit);
            }
            _ => {}
        }
    }
}

fn render(frame: &mut ratatui::Frame<'_>, state: &LauncherTuiState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(6),
        ])
        .split(frame.area());
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[2]);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "AMIGO LAUNCHER",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("focus: {}", state.focus_label()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  "),
        Span::styled(
            format!(
                "{}{}",
                state.config_path.display(),
                if state.dirty { " [dirty]" } else { "" }
            ),
            Style::default().fg(Color::Gray),
        ),
    ]));
    frame.render_widget(header, root[0]);

    render_profiles(frame, state, root[1]);
    render_mods(frame, state, body[0]);
    render_scenes(frame, state, body[1]);

    let footer = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("profile: ", Style::default().fg(Color::Gray)),
                Span::raw(format!(
                    "{} ({})  enter={}  active-profile-default={}",
                    state.active_profile().display_label(),
                    state.active_profile().cargo_profile.as_str(),
                    "hosted",
                    state.active_profile().hosted_default
                )),
            ]),
            Line::from(vec![
                Span::styled("selection: ", Style::default().fg(Color::Gray)),
                Span::raw(format!(
                    "root mod={}  startup scene={}  cursor mod={}  cursor scene={}",
                    state.active_profile().root_mod_or_core(),
                    state
                        .active_profile()
                        .startup_scene
                        .as_deref()
                        .unwrap_or("none"),
                    state
                        .selected_mod()
                        .map(|known_mod| known_mod.id.as_str())
                        .unwrap_or("none"),
                    state
                        .selected_scene()
                        .map(|scene| scene.id)
                        .unwrap_or_else(|| "none".to_owned())
                )),
            ]),
            Line::from(vec![
                Span::styled("details: ", Style::default().fg(Color::Gray)),
                Span::raw(selected_detail_text(state)),
            ]),
            active_profile_health_line(state),
            primary_diagnostic_line(state.active_profile_diagnostics()),
            Line::from(
            "Left/Right: profile or pane  Up/Down: move  Enter: hosted run  H: hosted  L: headless  S: save  R: reload  O: toggle profile default",
        ),
            Line::from(Span::styled(
                format!("status: {}", state.status),
                Style::default().fg(Color::Green),
            )),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(footer, root[3]);
}

fn render_profiles(
    frame: &mut ratatui::Frame<'_>,
    state: &LauncherTuiState,
    area: ratatui::layout::Rect,
) {
    let titles = state
        .config
        .profiles
        .iter()
        .map(|profile| profile_tab_title(profile, state))
        .collect::<Vec<_>>();
    let tabs = Tabs::new(titles)
        .select(state.selected_profile_index)
        .divider(Span::styled(" | ", Style::default().fg(Color::Blue)))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .block(commander_block(
            "Profiles",
            state.focus == FocusPane::Profiles,
        ));
    frame.render_widget(tabs, area);
}

fn render_mods(
    frame: &mut ratatui::Frame<'_>,
    state: &LauncherTuiState,
    area: ratatui::layout::Rect,
) {
    let items = state
        .known_mods
        .iter()
        .map(|known_mod| {
            let root_selected =
                state.active_profile().root_mod.as_deref() == Some(known_mod.id.as_str());
            let mut header = vec![
                Span::styled(
                    if root_selected { "ROOT " } else { "     " },
                    if root_selected {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::styled(
                    known_mod.id.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{} scene(s)", known_mod.scenes.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ];

            if !known_mod.discovered {
                header.push(Span::raw("  "));
                header.push(Span::styled(
                    "MISSING",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ));
            }

            ListItem::new(Line::from(header))
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default().with_selected(Some(state.selected_mod_index));
    let title = if let Some(known_mod) = state.selected_mod() {
        format!("Left Panel :: Root Mods ({})", known_mod.name)
    } else {
        "Left Panel :: Root Mods".to_owned()
    };
    let list = List::new(items)
        .block(commander_block(&title, state.focus == FocusPane::Mods))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_scenes(
    frame: &mut ratatui::Frame<'_>,
    state: &LauncherTuiState,
    area: ratatui::layout::Rect,
) {
    let selected_mod = state.selected_mod();
    let scenes = state.current_scene_list();
    let active_profile = state.active_profile();
    let title = selected_mod
        .map(|known_mod| format!("Right Panel :: Scenes ({})", known_mod.id))
        .unwrap_or_else(|| "Right Panel :: Scenes".to_owned());

    if scenes.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("mod: ", Style::default().fg(Color::Gray)),
                Span::raw(
                    selected_mod
                        .map(|known_mod| known_mod.id.clone())
                        .unwrap_or_else(|| "none".to_owned()),
                ),
            ]),
            Line::from(vec![
                Span::styled("startup: ", Style::default().fg(Color::Gray)),
                Span::raw(
                    active_profile
                        .startup_scene
                        .clone()
                        .unwrap_or_else(|| "none".to_owned()),
                ),
            ]),
            Line::from(""),
            Line::from("no scenes declared by this mod"),
        ])
        .wrap(Wrap { trim: true })
        .block(commander_block(&title, state.focus == FocusPane::Scenes));
        frame.render_widget(empty, area);
        return;
    }

    let items = scenes
        .iter()
        .map(|scene| {
            let startup_selected = active_profile.startup_scene.as_deref()
                == Some(scene.id.as_str())
                && active_profile.root_mod.as_deref()
                    == selected_mod.map(|known_mod| known_mod.id.as_str());
            let mut header = vec![
                Span::styled(
                    if startup_selected { "START " } else { "      " },
                    if startup_selected {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::styled(
                    scene.id.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ];
            if !scene.label.trim().is_empty() && scene.label != scene.id {
                header.push(Span::raw("  "));
                header.push(Span::styled(
                    scene.label.clone(),
                    Style::default().fg(Color::Cyan),
                ));
            }

            ListItem::new(Line::from(header))
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default().with_selected(Some(state.selected_scene_index));
    let list = List::new(items)
        .block(commander_block(&title, state.focus == FocusPane::Scenes))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn commander_block(title: &str, focused: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Double
        } else {
            BorderType::Plain
        })
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Blue)
        })
        .title(Span::styled(
            title.to_owned(),
            if focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            },
        ))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> AmigoResult<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn discover_known_mods(config: &LauncherConfig) -> AmigoResult<Vec<KnownMod>> {
    let discovered = ModCatalog::discover_unresolved(Path::new(&config.mods_root))?;
    let mut known_ids = BTreeSet::new();
    let mut known_mods = Vec::new();

    for discovered_mod in discovered {
        known_ids.insert(discovered_mod.manifest.id.clone());
        known_mods.push(KnownMod {
            id: discovered_mod.manifest.id.clone(),
            name: discovered_mod.manifest.name.clone(),
            description: discovered_mod
                .manifest
                .description
                .clone()
                .unwrap_or_default(),
            scenes: discovered_mod
                .manifest
                .scenes
                .iter()
                .filter(|scene| scene.is_launcher_visible())
                .cloned()
                .collect(),
            discovered: true,
        });
    }

    let mut configured_only_ids = BTreeSet::new();

    for profile in &config.profiles {
        if let Some(root_mod) = profile.root_mod.as_deref() {
            if !known_ids.contains(root_mod) {
                configured_only_ids.insert(root_mod.to_owned());
            }
        }
    }

    for mod_id in configured_only_ids {
        known_mods.push(KnownMod {
            id: mod_id.clone(),
            name: mod_id.clone(),
            description: "Configured in launcher profile but not discovered on disk.".to_owned(),
            scenes: Vec::new(),
            discovered: false,
        });
    }

    known_mods.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(known_mods)
}

fn selected_detail_text(state: &LauncherTuiState) -> String {
    let mod_detail = state
        .selected_mod()
        .map(|known_mod| {
            if known_mod.description.trim().is_empty() {
                known_mod.name.clone()
            } else {
                known_mod.description.clone()
            }
        })
        .unwrap_or_else(|| "no mod selected".to_owned());

    let scene_detail = state
        .selected_scene()
        .map(|scene| {
            scene
                .description
                .filter(|description| !description.trim().is_empty())
                .or(scene
                    .document
                    .map(|document| format!("document: {document}")))
                .unwrap_or_else(|| scene.label)
        })
        .unwrap_or_else(|| "no scene selected".to_owned());

    format!("{mod_detail}  |  {scene_detail}")
}

fn wrapped_next_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }

    (current as isize + delta).rem_euclid(len as isize) as usize
}

fn active_profile_health_line(state: &LauncherTuiState) -> Line<'static> {
    profile_health_line(state.active_profile_diagnostics())
}

fn profile_tab_title(profile: &LauncherProfile, state: &LauncherTuiState) -> Line<'static> {
    let diagnostics = state.profile_diagnostics.get(&profile.id);
    let mut spans = vec![
        Span::styled(
            if profile.id == state.config.active_profile {
                "* "
            } else {
                "  "
            },
            if profile.id == state.config.active_profile {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::styled(
            profile.display_label().to_owned(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];

    if let Some(report) = diagnostics {
        if report.error_count() > 0 {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("E{}", report.error_count()),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ));
        }

        if report.warning_count() > 0 {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("W{}", report.warning_count()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
    }

    Line::from(spans)
}

fn profile_health_line(diagnostics: Option<&ProfileDiagnostics>) -> Line<'static> {
    let (label, style) = match diagnostics {
        Some(report) if report.has_errors() => (
            format!("health: blocked ({} error(s))", report.error_count()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Some(report) if report.warning_count() > 0 => (
            format!("health: warnings ({} warning(s))", report.warning_count()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Some(_) => (
            "health: ready".to_owned(),
            Style::default().fg(Color::Green),
        ),
        None => (
            "health: diagnostics unavailable".to_owned(),
            Style::default().fg(Color::Yellow),
        ),
    };

    Line::from(Span::styled(label, style))
}

fn primary_diagnostic_line(diagnostics: Option<&ProfileDiagnostics>) -> Line<'static> {
    let Some(report) = diagnostics else {
        return Line::from(Span::styled(
            "diagnostics: unavailable",
            Style::default().fg(Color::Yellow),
        ));
    };

    let Some(diagnostic) = report.diagnostics.first() else {
        return Line::from(Span::styled(
            "diagnostics: none",
            Style::default().fg(Color::Green),
        ));
    };

    let label = match diagnostic.severity {
        DiagnosticSeverity::Warning => "diagnostics: warning",
        DiagnosticSeverity::Error => "diagnostics: error",
    };
    let style = match diagnostic.severity {
        DiagnosticSeverity::Warning => Style::default().fg(Color::Yellow),
        DiagnosticSeverity::Error => Style::default().fg(Color::Red),
    };

    Line::from(vec![
        Span::styled(format!("{label} "), style.add_modifier(Modifier::BOLD)),
        Span::raw(diagnostic.message.clone()),
    ])
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{FocusPane, LaunchMode, LauncherTuiState, TuiOutcome};
    use crate::config::LauncherConfig;

    fn state() -> LauncherTuiState {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();
        let mut config = LauncherConfig::default();
        config.mods_root = workspace_root.join("mods").display().to_string();

        LauncherTuiState::new("config/launcher.toml".into(), config).expect("state should build")
    }

    #[test]
    fn selecting_release_profile_syncs_startup_selection() {
        let mut state = state();
        state.selected_profile_index = state
            .config
            .profiles
            .iter()
            .position(|profile| profile.id == "release")
            .expect("release profile should exist");

        state.activate_focused();

        assert_eq!(state.config.active_profile, "release");
        assert_eq!(state.active_profile().root_mod.as_deref(), Some("core"));
        assert_eq!(
            state.active_profile().startup_scene.as_deref(),
            Some("bootstrap")
        );
    }

    #[test]
    fn activating_mod_sets_root_mod_and_scene() {
        let mut state = state();
        state.focus = FocusPane::Mods;
        state.selected_mod_index = state
            .known_mods
            .iter()
            .position(|known_mod| known_mod.id == "playground-2d")
            .expect("playground-2d mod should exist");

        let outcome = state.activate_focused();

        assert_eq!(
            state.active_profile().root_mod.as_deref(),
            Some("playground-2d")
        );
        assert_eq!(
            state.active_profile().startup_scene.as_deref(),
            Some("basic-scripting-demo")
        );
        assert_eq!(
            state.resolved_mod_ids,
            vec!["core".to_owned(), "playground-2d".to_owned()]
        );
        assert!(matches!(
            outcome,
            Some(TuiOutcome::Launch {
                mode: LaunchMode::Hosted,
                ..
            })
        ));
    }

    #[test]
    fn launcher_hides_legacy_fixture_scenes_for_playgrounds() {
        let mut state = state();
        state.focus = FocusPane::Mods;
        state.selected_mod_index = state
            .known_mods
            .iter()
            .position(|known_mod| known_mod.id == "playground-3d")
            .expect("playground-3d mod should exist");

        let scenes = state.current_scene_list();

        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].id, "hello-world-cube");
    }

    #[test]
    fn activating_mod_uses_declared_first_scene() {
        let mut state = state();
        state.focus = FocusPane::Mods;
        state.selected_mod_index = state
            .known_mods
            .iter()
            .position(|known_mod| known_mod.id == "playground-3d")
            .expect("playground-3d mod should exist");

        let outcome = state.activate_focused();

        assert_eq!(
            state.active_profile().root_mod.as_deref(),
            Some("playground-3d")
        );
        assert_eq!(
            state.active_profile().startup_scene.as_deref(),
            Some("hello-world-cube")
        );
        assert!(matches!(
            outcome,
            Some(TuiOutcome::Launch {
                mode: LaunchMode::Hosted,
                ..
            })
        ));
    }

    #[test]
    fn blocked_profile_does_not_launch() {
        let mut state = state();
        state.focus = FocusPane::Mods;
        state.selected_mod_index = state
            .known_mods
            .iter()
            .position(|known_mod| known_mod.id == "playground-2d")
            .expect("playground-2d mod should exist");
        state.activate_focused();
        state.active_profile_mut().startup_scene = Some("missing-scene".to_owned());
        state.refresh_profile_diagnostics();

        let outcome = state.try_launch(super::LaunchMode::Headless);

        assert!(outcome.is_none());
        assert!(state.status.contains("blocked"));
    }
}
