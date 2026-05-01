use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListState, Paragraph, Tabs, Wrap,
};
use ratatui::Frame;

use super::filtering::display_string_list;
use super::profiles::{
    active_profile_health_line, primary_diagnostic_line, profile_tab_title,
};
use super::{
    details::{
        selected_detail_text,
        selected_tree_label,
        tree_item_for_entry,
    },
    FocusPane,
    LauncherTuiState,
};

pub fn render(frame: &mut Frame<'_>, state: &LauncherTuiState) {
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
    frame: &mut Frame<'_>,
    state: &LauncherTuiState,
    area: Rect,
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

fn render_details(frame: &mut Frame<'_>, state: &LauncherTuiState, area: Rect) {
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
