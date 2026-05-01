use ratatui::text::Line;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::ListItem;

use super::filtering::mod_node_id;
use super::{LauncherTuiState, TreeEntry};

pub(super) fn selected_detail_text(state: &LauncherTuiState) -> String {
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
                .as_ref()
                .filter(|description| !description.trim().is_empty())
                .map(std::string::ToString::to_string)
                .or(scene
                    .document
                    .map(|document| format!("document: {document}")))
                .unwrap_or_else(|| scene.label)
        })
        .unwrap_or_else(|| "no scene selected".to_owned());

    format!("{mod_detail}  |  {scene_detail}")
}

pub(super) fn selected_tree_label(state: &LauncherTuiState) -> String {
    let Some(entry) = state.selected_tree_entry() else {
        return "none".to_owned();
    };

    match entry {
        TreeEntry::Category { category_id } => category_id,
        TreeEntry::Mod { mod_index, .. } => state
            .known_mods
            .get(mod_index)
            .map(|known_mod| known_mod.id.clone())
            .unwrap_or_else(|| "none".to_owned()),
        TreeEntry::Scene {
            mod_index,
            scene_index,
            ..
        } => {
            let Some(known_mod) = state.known_mods.get(mod_index) else {
                return "none".to_owned();
            };
            let Some(scene) = known_mod.scenes.get(scene_index) else {
                return "none".to_owned();
            };
            format!("{} / {}", known_mod.id, scene.id)
        }
    }
}

pub(super) fn tree_item_for_entry(state: &LauncherTuiState, entry: &TreeEntry) -> ListItem<'static> {
    match entry {
        TreeEntry::Category { category_id } => {
            let expanded =
                !state.scene_filter.is_empty() || state.expanded_category_ids.contains(category_id);
            let depth = category_id.split('/').count().saturating_sub(1);
            let label = category_id
                .rsplit('/')
                .next()
                .unwrap_or(category_id.as_str())
                .to_owned();
            ListItem::new(Line::from(vec![
                Span::raw("  ".repeat(depth)),
                Span::styled(
                    if expanded { "[-] " } else { "[+] " },
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(label, Style::default().add_modifier(Modifier::BOLD)),
            ]))
        }
        TreeEntry::Mod {
            category_id,
            mod_index,
        } => {
            let known_mod = &state.known_mods[*mod_index];
            let mod_number = format_position_index(*mod_index, state.known_mods.len());
            let root_selected =
                state.active_profile().root_mod.as_deref() == Some(known_mod.id.as_str());
            let expanded = !state.scene_filter.is_empty()
                || state
                    .expanded_mod_ids
                    .contains(&mod_node_id(category_id, known_mod.id.as_str()))
                || state.expanded_mod_ids.contains(&known_mod.id);
            let depth = category_id.split('/').count();
            let mut spans = vec![
                Span::raw("  ".repeat(depth)),
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
        TreeEntry::Scene {
            category_id,
            mod_index,
            scene_index,
        } => {
            let known_mod = &state.known_mods[*mod_index];
            let mod_number = format_position_index(*mod_index, state.known_mods.len());
            let root_selected =
                state.active_profile().root_mod.as_deref() == Some(known_mod.id.as_str());
            let scene = &known_mod.scenes[*scene_index];
            let scene_number = format!(
                "{}.{}",
                mod_number,
                format_position_index(*scene_index, known_mod.scenes.len())
            );
            let startup_selected = state.active_profile().startup_scene.as_deref()
                == Some(scene.id.as_str())
                && root_selected;
            let depth = category_id.split('/').count() + 1;

            let mut spans = vec![
                Span::raw("  ".repeat(depth)),
                Span::styled(format!("{scene_number} "), Style::default().fg(Color::Gray)),
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

