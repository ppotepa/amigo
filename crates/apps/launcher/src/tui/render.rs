use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListState, Paragraph, Tabs, Wrap};

use super::filtering::display_string_list;
use super::profiles::{active_profile_health_line, primary_diagnostic_line, profile_tab_title};
use super::{
    FocusPane, LaunchMode, LauncherTuiState,
    details::{selected_detail_text, selected_tree_label, tree_item_for_entry},
};

const THEME_BACKGROUND: Color = Color::Rgb(5, 8, 18);
const THEME_PANEL: Color = Color::Rgb(16, 24, 39);
const THEME_PANEL_ALT: Color = Color::Rgb(23, 32, 51);
const THEME_TEXT: Color = Color::Rgb(234, 246, 255);
const THEME_MUTED: Color = Color::Rgb(125, 135, 150);
const THEME_BORDER: Color = Color::Rgb(42, 111, 158);
const THEME_ACCENT: Color = Color::Rgb(57, 215, 255);
const THEME_ACCENT_TEXT: Color = Color::Rgb(0, 16, 24);
const THEME_GOLD: Color = Color::Rgb(255, 176, 0);
const THEME_MAGENTA: Color = Color::Rgb(219, 73, 172);
const THEME_SUCCESS: Color = Color::Rgb(92, 255, 156);

pub fn render(frame: &mut Frame<'_>, state: &LauncherTuiState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(8),
        ])
        .split(frame.area());
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(root[2]);

    render_header(frame, state, root[0]);
    render_profiles(frame, state, root[1]);
    render_tree(frame, state, body[0]);
    render_details(frame, state, body[1]);
    render_footer(frame, state, root[3]);
}

fn render_header(frame: &mut Frame<'_>, state: &LauncherTuiState, area: Rect) {
    let dirty = if state.dirty { " [dirty]" } else { "" };
    let header = Paragraph::new(vec![
        Line::from(vec![
            label_span("Profile"),
            Span::raw(state.active_profile().display_label()),
            Span::raw(format!(
                " ({})",
                state.active_profile().cargo_profile.as_str()
            )),
            Span::raw("  "),
            label_span("Focus"),
            Span::styled(state.focus_label(), Style::default().fg(THEME_GOLD)),
            Span::raw("  "),
            label_span("Config"),
            Span::styled(
                format!("{}{}", state.config_path.display(), dirty),
                Style::default().fg(THEME_MUTED),
            ),
        ]),
        Line::from(vec![
            key_span("Enter"),
            Span::raw(format!(" {}   ", launch_mode_label(LaunchMode::Hosted))),
            key_span("E"),
            Span::raw(format!(" {}   ", launch_mode_label(LaunchMode::Editor))),
            key_span("Ctrl+L"),
            Span::raw(format!(" {}   ", launch_mode_label(LaunchMode::Headless))),
            key_span("Tab"),
            Span::raw(" switch pane   "),
            key_span("Type"),
            Span::raw(" filter"),
        ]),
    ])
    .wrap(Wrap { trim: true })
    .style(Style::default().fg(THEME_TEXT).bg(THEME_BACKGROUND))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(THEME_PANEL))
            .border_style(Style::default().fg(THEME_GOLD))
            .title(Span::styled(
                " Amigo Launcher - Night Mexico ",
                Style::default().fg(THEME_GOLD).add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(header, area);
}

fn render_footer(frame: &mut Frame<'_>, state: &LauncherTuiState, area: Rect) {
    let filter = if state.scene_filter.is_empty() {
        "none".to_owned()
    } else {
        state.scene_filter.clone()
    };
    let footer = Paragraph::new(vec![
        field_line(
            "Selection",
            format!(
                "mod={}  scene={}  cursor={}  filter={}",
                state.active_profile().root_mod_or_core(),
                state
                    .active_profile()
                    .startup_scene
                    .as_deref()
                    .unwrap_or("none"),
                selected_tree_label(state),
                filter
            ),
        ),
        field_line("Details", selected_detail_text(state)),
        active_profile_health_line(state),
        primary_diagnostic_line(state.active_profile_diagnostics()),
        field_line("Status", state.status.clone()),
        Line::from(
            "Navigation: Up/Down move  Left/Right profile or expand/collapse  Space toggle  Ctrl+S/R/O save/reload/toggle default",
        ),
    ])
    .wrap(Wrap { trim: true })
    .style(Style::default().fg(THEME_TEXT).bg(THEME_BACKGROUND))
    .block(commander_block("Status", false));
    frame.render_widget(footer, area);
}

fn label_span(label: impl Into<String>) -> Span<'static> {
    Span::styled(
        format!("{}: ", label.into()),
        Style::default().fg(THEME_MUTED),
    )
}

fn key_span(label: impl Into<String>) -> Span<'static> {
    Span::styled(
        label.into(),
        Style::default()
            .fg(THEME_SUCCESS)
            .add_modifier(Modifier::BOLD),
    )
}

fn field_line(label: impl Into<String>, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![label_span(label), Span::raw(value.into())])
}

fn section_line(label: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(
        label.into(),
        Style::default()
            .fg(THEME_ACCENT)
            .add_modifier(Modifier::BOLD),
    ))
}

fn launch_mode_label(mode: LaunchMode) -> &'static str {
    match mode {
        LaunchMode::Headless => "headless check",
        LaunchMode::Hosted => "play",
        LaunchMode::Editor => "editor",
    }
}

fn render_profiles(frame: &mut Frame<'_>, state: &LauncherTuiState, area: Rect) {
    let titles = state
        .config
        .profiles
        .iter()
        .map(|profile| profile_tab_title(profile, state))
        .collect::<Vec<_>>();
    let tabs = Tabs::new(titles)
        .select(state.selected_profile_index)
        .style(Style::default().fg(THEME_TEXT).bg(THEME_BACKGROUND))
        .divider(Span::styled(" | ", Style::default().fg(THEME_MAGENTA)))
        .highlight_style(
            Style::default()
                .bg(THEME_GOLD)
                .fg(THEME_ACCENT_TEXT)
                .add_modifier(Modifier::BOLD),
        )
        .block(commander_block(
            "Profiles - Left/Right",
            state.focus == FocusPane::Profiles,
        ));
    frame.render_widget(tabs, area);
}

fn render_tree(frame: &mut Frame<'_>, state: &LauncherTuiState, area: Rect) {
    let entries = state.visible_tree_entries();
    let items = entries
        .iter()
        .map(|entry| tree_item_for_entry(state, entry))
        .collect::<Vec<_>>();
    let selected_index = state
        .selected_tree_entry()
        .and_then(|selected| entries.iter().position(|entry| *entry == selected))
        .unwrap_or(0);
    let mut list_state = ListState::default().with_selected(Some(selected_index));
    let title = if state.scene_filter.is_empty() {
        "Library - type to filter".to_owned()
    } else {
        format!("Library - filter `{}`", state.scene_filter)
    };
    let list = List::new(items)
        .block(commander_block(&title, state.focus == FocusPane::Tree))
        .style(Style::default().fg(THEME_TEXT).bg(THEME_BACKGROUND))
        .highlight_style(
            Style::default()
                .bg(THEME_GOLD)
                .fg(THEME_ACCENT_TEXT)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_details(frame: &mut Frame<'_>, state: &LauncherTuiState, area: Rect) {
    let selected_mod = state.selected_mod();
    let selected_scene = state.selected_scene();
    let filter = if state.scene_filter.is_empty() {
        "none".to_owned()
    } else {
        state.scene_filter.clone()
    };
    let lines = vec![
        section_line("Selection"),
        field_line("Cursor", selected_tree_label(state)),
        field_line(
            "Mod",
            selected_mod
                .map(|known_mod| known_mod.id.clone())
                .unwrap_or_else(|| "none".to_owned()),
        ),
        field_line(
            "Scene",
            selected_scene
                .as_ref()
                .map(|scene| scene.id.clone())
                .unwrap_or_else(|| "none".to_owned()),
        ),
        field_line("Filter", filter),
        Line::from(""),
        section_line("Profile"),
        field_line("Active", state.active_profile().display_label()),
        field_line("Cargo", state.active_profile().cargo_profile.as_str()),
        field_line("Root Mod", state.active_profile().root_mod_or_core()),
        field_line(
            "Startup Scene",
            state
                .active_profile()
                .startup_scene
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
        ),
        field_line(
            "Default Hosted",
            state.active_profile().hosted_default.to_string(),
        ),
        Line::from(""),
        section_line("Details"),
        Line::from(selected_detail_text(state)),
        field_line(
            "Resolved Mods",
            display_string_list(&state.resolved_mod_ids),
        ),
    ];
    let details = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(THEME_TEXT).bg(THEME_BACKGROUND))
        .block(commander_block("Details", false));
    frame.render_widget(details, area);
}

fn commander_block(title: &str, focused: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(if focused {
            THEME_PANEL_ALT
        } else {
            THEME_PANEL
        }))
        .border_type(if focused {
            BorderType::Double
        } else {
            BorderType::Plain
        })
        .border_style(if focused {
            Style::default().fg(THEME_GOLD)
        } else {
            Style::default().fg(THEME_BORDER)
        })
        .title(Span::styled(
            title.to_owned(),
            if focused {
                Style::default().fg(THEME_GOLD).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(THEME_ACCENT)
                    .add_modifier(Modifier::BOLD)
            },
        ))
}
