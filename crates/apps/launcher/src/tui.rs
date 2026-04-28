use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Stdout};
use std::path::{Path, PathBuf};

use amigo_core::AmigoResult;
use amigo_modding::{ModCatalog, ModSceneManifest, requested_mods_for_root};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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
    Tree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TreeEntry {
    mod_index: usize,
    scene_index: Option<usize>,
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
    tree_cursor_on_scene: bool,
    expanded_mod_ids: BTreeSet<String>,
    scene_filter: String,
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
            tree_cursor_on_scene: false,
            expanded_mod_ids: BTreeSet::new(),
            scene_filter: String::new(),
            dirty: false,
            status: "Profiles on top. Type to filter the tree, Enter launches hosted.".to_owned(),
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
        if !self.tree_cursor_on_scene {
            return None;
        }

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
        self.scene_filter.clear();
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

        self.expanded_mod_ids.clear();
        if let Some(root_mod) = self.active_profile().root_mod.as_deref() {
            self.expanded_mod_ids.insert(root_mod.to_owned());
        }
        self.refresh_resolved_mods();
        self.sync_scene_selection_for_current_mod();
        self.sync_tree_selection_to_visible();
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
        self.tree_cursor_on_scene = startup_scene.is_some() && !scenes.is_empty();
    }

    fn move_focus_next(&mut self) {
        self.focus = match self.focus {
            FocusPane::Profiles => FocusPane::Tree,
            FocusPane::Tree => FocusPane::Profiles,
        };
        self.status = format!("focus: {}", self.focus_label());
    }

    fn move_focus_previous(&mut self) {
        self.move_focus_next();
    }

    fn focus_label(&self) -> &'static str {
        match self.focus {
            FocusPane::Profiles => "profiles",
            FocusPane::Tree => "tree",
        }
    }

    fn append_scene_filter(&mut self, character: char) {
        self.focus = FocusPane::Tree;
        self.scene_filter.push(character);
        self.sync_tree_selection_to_visible();
        self.status = format!("scene filter: `{}`", self.scene_filter);
    }

    fn pop_scene_filter(&mut self) {
        if self.scene_filter.pop().is_some() {
            self.sync_tree_selection_to_visible();
            self.status = if self.scene_filter.is_empty() {
                "scene filter cleared".to_owned()
            } else {
                format!("scene filter: `{}`", self.scene_filter)
            };
        }
    }

    fn clear_scene_filter(&mut self) {
        self.scene_filter.clear();
        self.sync_tree_selection_to_visible();
        self.status = "scene filter cleared".to_owned();
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
            FocusPane::Tree => self.move_tree_selection(delta),
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
            FocusPane::Tree => self.try_launch_focused(LaunchMode::Hosted),
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
        self.scene_filter.clear();
        self.expanded_mod_ids.insert(mod_id.to_owned());
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
        self.scene_filter.clear();
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
            FocusPane::Tree => {
                let selected_mod_id = self.selected_mod().map(|known_mod| known_mod.id.clone());
                let selected_scene_id = self.selected_scene().map(|scene| scene.id);

                if let Some(mod_id) = selected_mod_id {
                    if self.active_profile().root_mod.as_deref() != Some(mod_id.as_str()) {
                        self.set_root_mod(&mod_id);
                    }
                    if let Some(scene_id) = selected_scene_id {
                        self.set_startup_scene(&scene_id);
                    }
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

    fn selected_tree_entry(&self) -> Option<TreeEntry> {
        self.visible_tree_entries().into_iter().find(|entry| {
            entry.mod_index == self.selected_mod_index
                && entry.scene_index
                    == if self.tree_cursor_on_scene {
                        Some(self.selected_scene_index)
                    } else {
                        None
                    }
        })
    }

    fn visible_tree_entries(&self) -> Vec<TreeEntry> {
        let filter_active = !self.scene_filter.trim().is_empty();
        let mut entries = Vec::new();

        for (mod_index, known_mod) in self.known_mods.iter().enumerate() {
            let mod_matches = filter_active && mod_matches_filter(known_mod, &self.scene_filter);
            let matching_scene_indices = known_mod
                .scenes
                .iter()
                .enumerate()
                .filter_map(|(scene_index, scene)| {
                    scene_matches_filter(scene, &self.scene_filter).then_some(scene_index)
                })
                .collect::<Vec<_>>();

            if filter_active && !mod_matches && matching_scene_indices.is_empty() {
                continue;
            }

            entries.push(TreeEntry {
                mod_index,
                scene_index: None,
            });

            let expanded = filter_active || self.expanded_mod_ids.contains(&known_mod.id);
            if !expanded {
                continue;
            }

            let scene_indices = if filter_active {
                if mod_matches && matching_scene_indices.is_empty() {
                    (0..known_mod.scenes.len()).collect::<Vec<_>>()
                } else {
                    matching_scene_indices
                }
            } else {
                (0..known_mod.scenes.len()).collect::<Vec<_>>()
            };

            for scene_index in scene_indices {
                entries.push(TreeEntry {
                    mod_index,
                    scene_index: Some(scene_index),
                });
            }
        }

        entries
    }

    fn sync_tree_selection_to_visible(&mut self) {
        let entries = self.visible_tree_entries();
        if entries.is_empty() {
            self.selected_mod_index = 0;
            self.selected_scene_index = 0;
            self.tree_cursor_on_scene = false;
            return;
        }

        if let Some(selected) = self.selected_tree_entry() {
            if entries.iter().any(|entry| *entry == selected)
                && (!self.filter_prefers_scene_selection() || selected.scene_index.is_some())
            {
                return;
            }
        }

        if let Some(entry) = self.preferred_filtered_scene_entry(&entries) {
            self.apply_tree_entry(entry);
            return;
        }

        self.apply_tree_entry(entries[0]);
    }

    fn apply_tree_entry(&mut self, entry: TreeEntry) {
        self.selected_mod_index = entry.mod_index;
        match entry.scene_index {
            Some(scene_index) => {
                self.selected_scene_index = scene_index;
                self.tree_cursor_on_scene = true;
            }
            None => {
                self.tree_cursor_on_scene = false;
            }
        }
    }

    fn move_tree_selection(&mut self, delta: isize) {
        let entries = self.visible_tree_entries();
        if entries.is_empty() {
            return;
        }

        let current = self
            .selected_tree_entry()
            .and_then(|selected| entries.iter().position(|entry| *entry == selected))
            .unwrap_or(0);
        let next = wrapped_next_index(current, entries.len(), delta);
        self.apply_tree_entry(entries[next]);
    }

    fn filter_prefers_scene_selection(&self) -> bool {
        !self.scene_filter.trim().is_empty() && self.first_matching_scene_entry().is_some()
    }

    fn preferred_filtered_scene_entry(&self, entries: &[TreeEntry]) -> Option<TreeEntry> {
        if !self.filter_prefers_scene_selection() {
            return None;
        }

        self.first_matching_scene_entry()
            .filter(|entry| entries.contains(entry))
    }

    fn first_matching_scene_entry(&self) -> Option<TreeEntry> {
        if self.scene_filter.trim().is_empty() {
            return None;
        }

        for (mod_index, known_mod) in self.known_mods.iter().enumerate() {
            if !mod_matches_filter(known_mod, &self.scene_filter) {
                for (scene_index, scene) in known_mod.scenes.iter().enumerate() {
                    if scene_matches_filter(scene, &self.scene_filter) {
                        return Some(TreeEntry {
                            mod_index,
                            scene_index: Some(scene_index),
                        });
                    }
                }
                continue;
            }

            if let Some((scene_index, _)) = known_mod
                .scenes
                .iter()
                .enumerate()
                .find(|(_, scene)| scene_matches_filter(scene, &self.scene_filter))
            {
                return Some(TreeEntry {
                    mod_index,
                    scene_index: Some(scene_index),
                });
            }
        }

        None
    }

    fn toggle_selected_expansion(&mut self) {
        let Some(known_mod) = self.selected_mod() else {
            return;
        };
        let mod_id = known_mod.id.clone();
        if self.expanded_mod_ids.contains(&mod_id) {
            self.expanded_mod_ids.remove(&mod_id);
            if self.tree_cursor_on_scene {
                self.tree_cursor_on_scene = false;
            }
            self.status = format!("collapsed `{mod_id}`");
        } else {
            self.expanded_mod_ids.insert(mod_id.clone());
            self.status = format!("expanded `{mod_id}`");
        }
        self.sync_tree_selection_to_visible();
    }

    fn expand_selected_mod(&mut self) {
        let Some(known_mod) = self.selected_mod() else {
            return;
        };
        let mod_id = known_mod.id.clone();
        if self.tree_cursor_on_scene {
            return;
        }
        if self.expanded_mod_ids.insert(mod_id.clone()) {
            self.status = format!("expanded `{mod_id}`");
        }
        self.sync_tree_selection_to_visible();
    }

    fn collapse_selected_mod_or_parent(&mut self) {
        if self.tree_cursor_on_scene {
            self.tree_cursor_on_scene = false;
            self.status = "moved to parent mod".to_owned();
            return;
        }

        let Some(known_mod) = self.selected_mod() else {
            return;
        };
        let mod_id = known_mod.id.clone();
        if self.expanded_mod_ids.remove(&mod_id) {
            self.status = format!("collapsed `{mod_id}`");
        }
        self.sync_tree_selection_to_visible();
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
                FocusPane::Tree => state.collapse_selected_mod_or_parent(),
            },
            KeyCode::Right => match state.focus {
                FocusPane::Profiles => state.move_selection(1),
                FocusPane::Tree => state.expand_selected_mod(),
            },
            KeyCode::Up => state.move_selection(-1),
            KeyCode::Down => state.move_selection(1),
            KeyCode::Backspace if !state.scene_filter.is_empty() => state.pop_scene_filter(),
            KeyCode::Enter => {
                if let Some(outcome) = state.activate_focused() {
                    return Ok(outcome);
                }
            }
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
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
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
    render_tree(frame, state, body[0]);
    render_details(frame, state, body[1]);

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
                    "root mod={}  startup scene={}  cursor={}  scene-filter={}",
                    state.active_profile().root_mod_or_core(),
                    state
                        .active_profile()
                        .startup_scene
                        .as_deref()
                        .unwrap_or("none"),
                    selected_tree_label(state),
                    if state.scene_filter.is_empty() {
                        "none".to_owned()
                    } else {
                        state.scene_filter.clone()
                    }
                )),
            ]),
            Line::from(vec![
                Span::styled("details: ", Style::default().fg(Color::Gray)),
                Span::raw(selected_detail_text(state)),
            ]),
            active_profile_health_line(state),
            primary_diagnostic_line(state.active_profile_diagnostics()),
            Line::from(
            "Typing: fuzzy filter tree  Left/Right: profile or expand/collapse  Up/Down: move  Space: toggle  Enter: hosted  Ctrl+L: headless  Ctrl+S/R/O: save/reload/toggle default",
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

fn render_tree(
    frame: &mut ratatui::Frame<'_>,
    state: &LauncherTuiState,
    area: ratatui::layout::Rect,
) {
    let entries = state.visible_tree_entries();
    let items = entries
        .iter()
        .map(|entry| tree_item_for_entry(state, *entry))
        .collect::<Vec<_>>();
    let selected_index = state
        .selected_tree_entry()
        .and_then(|selected| entries.iter().position(|entry| *entry == selected))
        .unwrap_or(0);
    let mut list_state = ListState::default().with_selected(Some(selected_index));
    let title = if state.scene_filter.is_empty() {
        "Mod Tree".to_owned()
    } else {
        format!("Mod Tree  /  fuzzy `{}`", state.scene_filter)
    };
    let list = List::new(items)
        .block(commander_block(&title, state.focus == FocusPane::Tree))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_details(
    frame: &mut ratatui::Frame<'_>,
    state: &LauncherTuiState,
    area: ratatui::layout::Rect,
) {
    let selected_mod = state.selected_mod();
    let selected_scene = state.selected_scene();
    let lines = vec![
        Line::from(vec![
            Span::styled("cursor: ", Style::default().fg(Color::Gray)),
            Span::raw(selected_tree_label(state)),
        ]),
        Line::from(vec![
            Span::styled("mod: ", Style::default().fg(Color::Gray)),
            Span::raw(
                selected_mod
                    .map(|known_mod| known_mod.id.clone())
                    .unwrap_or_else(|| "none".to_owned()),
            ),
        ]),
        Line::from(vec![
            Span::styled("scene: ", Style::default().fg(Color::Gray)),
            Span::raw(
                selected_scene
                    .as_ref()
                    .map(|scene| scene.id.clone())
                    .unwrap_or_else(|| "none".to_owned()),
            ),
        ]),
        Line::from(vec![
            Span::styled("startup: ", Style::default().fg(Color::Gray)),
            Span::raw(
                state
                    .active_profile()
                    .startup_scene
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
            ),
        ]),
        Line::from(vec![
            Span::styled("filter: ", Style::default().fg(Color::Gray)),
            Span::raw(if state.scene_filter.is_empty() {
                "none".to_owned()
            } else {
                state.scene_filter.clone()
            }),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("details: ", Style::default().fg(Color::Gray)),
            Span::raw(selected_detail_text(state)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("resolved mods: ", Style::default().fg(Color::Gray)),
            Span::raw(display_string_list(&state.resolved_mod_ids)),
        ]),
    ];
    let details = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(commander_block("Details", false));
    frame.render_widget(details, area);
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

fn selected_tree_label(state: &LauncherTuiState) -> String {
    let Some(entry) = state.selected_tree_entry() else {
        return "none".to_owned();
    };

    if let Some(scene_index) = entry.scene_index {
        let Some(known_mod) = state.known_mods.get(entry.mod_index) else {
            return "none".to_owned();
        };
        let Some(scene) = known_mod.scenes.get(scene_index) else {
            return "none".to_owned();
        };
        return format!("{} / {}", known_mod.id, scene.id);
    }

    state
        .known_mods
        .get(entry.mod_index)
        .map(|known_mod| known_mod.id.clone())
        .unwrap_or_else(|| "none".to_owned())
}

fn tree_item_for_entry(state: &LauncherTuiState, entry: TreeEntry) -> ListItem<'static> {
    let known_mod = &state.known_mods[entry.mod_index];
    let mod_number = format_position_index(entry.mod_index, state.known_mods.len());
    let root_selected = state.active_profile().root_mod.as_deref() == Some(known_mod.id.as_str());

    match entry.scene_index {
        None => {
            let expanded =
                !state.scene_filter.is_empty() || state.expanded_mod_ids.contains(&known_mod.id);
            let mut spans = vec![
                Span::styled(format!("{mod_number} "), Style::default().fg(Color::Gray)),
                Span::styled(
                    if expanded { "[-] " } else { "[+] " },
                    Style::default().fg(Color::Yellow),
                ),
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
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    "MISSING",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ));
            }

            ListItem::new(Line::from(spans))
        }
        Some(scene_index) => {
            let scene = &known_mod.scenes[scene_index];
            let scene_number = format!(
                "{}.{}",
                mod_number,
                format_position_index(scene_index, known_mod.scenes.len())
            );
            let startup_selected = state.active_profile().startup_scene.as_deref()
                == Some(scene.id.as_str())
                && root_selected;

            let mut spans = vec![
                Span::styled(format!("{scene_number} "), Style::default().fg(Color::Gray)),
                Span::raw("    "),
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
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    scene.label.clone(),
                    Style::default().fg(Color::Cyan),
                ));
            }

            ListItem::new(Line::from(spans))
        }
    }
}

fn format_position_index(index: usize, len: usize) -> String {
    let width = len.max(1).to_string().len().max(2);
    format!("{:0width$}", index + 1, width = width)
}

fn mod_matches_filter(known_mod: &KnownMod, filter: &str) -> bool {
    let filter = normalize_filter_text(filter);
    if filter.is_empty() {
        return true;
    }

    let primary = normalize_filter_text(&format!("{} {}", known_mod.id, known_mod.name));
    let description = normalize_filter_text(&known_mod.description);

    primary.contains(&filter)
        || tokenize_filter_text(&primary)
            .into_iter()
            .any(|token| is_fuzzy_subsequence(&filter, &token))
        || description.contains(&filter)
}

fn scene_matches_filter(scene: &ModSceneManifest, filter: &str) -> bool {
    let filter = normalize_filter_text(filter);
    if filter.is_empty() {
        return true;
    }

    let primary = normalize_filter_text(&format!("{} {}", scene.id, scene.label));
    let description = normalize_filter_text(&scene.description.clone().unwrap_or_default());

    primary.contains(&filter)
        || tokenize_filter_text(&primary)
            .into_iter()
            .any(|token| is_fuzzy_subsequence(&filter, &token))
        || description.contains(&filter)
}

fn normalize_filter_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| character.to_lowercase())
        .collect()
}

fn tokenize_filter_text(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_owned())
        .collect()
}

fn is_fuzzy_subsequence(needle: &str, haystack: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let mut needle = needle.chars();
    let mut expected = needle.next();

    for character in haystack.chars() {
        if Some(character) == expected {
            expected = needle.next();
            if expected.is_none() {
                return true;
            }
        }
    }

    false
}

fn is_scene_filter_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ' ' | '.')
}

fn display_string_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_owned()
    } else {
        values.join(", ")
    }
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
        state.focus = FocusPane::Tree;
        state.selected_mod_index = state
            .known_mods
            .iter()
            .position(|known_mod| known_mod.id == "playground-2d")
            .expect("playground-2d mod should exist");
        state.tree_cursor_on_scene = false;
        state.expanded_mod_ids.insert("playground-2d".to_owned());
        state.sync_tree_selection_to_visible();

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
        state.focus = FocusPane::Tree;
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
        state.focus = FocusPane::Tree;
        state.selected_mod_index = state
            .known_mods
            .iter()
            .position(|known_mod| known_mod.id == "playground-3d")
            .expect("playground-3d mod should exist");
        state.tree_cursor_on_scene = false;
        state.expanded_mod_ids.insert("playground-3d".to_owned());
        state.sync_tree_selection_to_visible();

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
        state.focus = FocusPane::Tree;
        state.selected_mod_index = state
            .known_mods
            .iter()
            .position(|known_mod| known_mod.id == "playground-2d")
            .expect("playground-2d mod should exist");
        state.tree_cursor_on_scene = false;
        state.expanded_mod_ids.insert("playground-2d".to_owned());
        state.sync_tree_selection_to_visible();
        state.activate_focused();
        state.active_profile_mut().startup_scene = Some("missing-scene".to_owned());
        state.refresh_profile_diagnostics();

        let outcome = state.try_launch(super::LaunchMode::Headless);

        assert!(outcome.is_none());
        assert!(state.status.contains("blocked"));
    }

    #[test]
    fn activating_scene_from_different_mod_preserves_selected_scene() {
        let mut state = state();
        state.focus = FocusPane::Tree;
        state.selected_mod_index = state
            .known_mods
            .iter()
            .position(|known_mod| known_mod.id == "playground-2d")
            .expect("playground-2d mod should exist");
        state.expanded_mod_ids.insert("playground-2d".to_owned());
        state.selected_scene_index = state
            .current_scene_list()
            .iter()
            .position(|scene| scene.id == "screen-space-preview")
            .expect("screen-space-preview should exist");
        state.tree_cursor_on_scene = true;
        state.sync_tree_selection_to_visible();

        let outcome = state.activate_focused();

        assert_eq!(
            state.active_profile().root_mod.as_deref(),
            Some("playground-2d")
        );
        assert_eq!(
            state.active_profile().startup_scene.as_deref(),
            Some("screen-space-preview")
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
    fn scene_filter_fuzzy_matches_screen_space_preview() {
        let mut state = state();
        state.focus = FocusPane::Tree;
        state.selected_mod_index = state
            .known_mods
            .iter()
            .position(|known_mod| known_mod.id == "playground-2d")
            .expect("playground-2d mod should exist");
        state.tree_cursor_on_scene = false;
        state.expanded_mod_ids.insert("playground-2d".to_owned());
        state.sync_scene_selection_for_current_mod();
        for character in ['s', 'c', 'r', 'e', 'e', 'n'] {
            state.append_scene_filter(character);
        }

        let entries = state.visible_tree_entries();
        let target = entries
            .iter()
            .copied()
            .find(|entry| {
                entry.mod_index == state.selected_mod_index
                    && entry
                        .scene_index
                        .and_then(|scene_index| {
                            state
                                .known_mods
                                .get(entry.mod_index)
                                .and_then(|known_mod| known_mod.scenes.get(scene_index))
                        })
                        .map(|scene| scene.id.as_str() == "screen-space-preview")
                        .unwrap_or(false)
            })
            .expect("screen-space-preview should remain visible after fuzzy filter");
        state.apply_tree_entry(target);

        assert_eq!(
            state.selected_scene().map(|scene| scene.id),
            Some("screen-space-preview".to_owned())
        );
    }

    #[test]
    fn scene_filter_prefers_matching_scene_over_parent_mod() {
        let mut state = state();
        state.focus = FocusPane::Tree;
        state.selected_mod_index = state
            .known_mods
            .iter()
            .position(|known_mod| known_mod.id == "playground-2d")
            .expect("playground-2d mod should exist");
        state.tree_cursor_on_scene = false;
        state.expanded_mod_ids.insert("playground-2d".to_owned());
        state.sync_tree_selection_to_visible();

        for character in "screen".chars() {
            state.append_scene_filter(character);
        }

        assert!(state.tree_cursor_on_scene);
        assert_eq!(
            state.selected_scene().map(|scene| scene.id),
            Some("screen-space-preview".to_owned())
        );
    }
}
