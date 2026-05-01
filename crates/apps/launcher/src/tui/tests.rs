use std::path::PathBuf;

use super::{FocusPane, LauncherTuiState, LaunchMode, TreeEntry, TuiOutcome};
use super::filtering::mod_node_id;
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
fn launcher_tree_groups_scenes_by_engine_categories() {
    let mut state = state();
    state
        .expanded_mod_ids
        .insert(mod_node_id("UI/HUD", "playground-2d"));
    state.expanded_mod_ids.insert(mod_node_id(
        "2D/FX/Particles",
        "playground-2d-particles",
    ));
    state.expanded_mod_ids.insert(mod_node_id(
        "2D/Games/Asteroids",
        "playground-2d-asteroids",
    ));
    let entries = state.visible_tree_entries();

    assert!(entries.iter().any(|entry| matches!(
        entry,
        TreeEntry::Category { category_id } if category_id == "2D"
    )));
    assert!(entries.iter().any(|entry| matches!(
        entry,
        TreeEntry::Category { category_id } if category_id == "2D/FX/Particles"
    )));
    assert!(entries.iter().any(|entry| matches!(
        entry,
        TreeEntry::Category { category_id } if category_id == "UI/HUD"
    )));
    assert!(entries.iter().any(|entry| matches!(
        entry,
        TreeEntry::Category { category_id } if category_id == "2D/Games/Asteroids"
    )));
    assert!(entries.iter().any(|entry| matches!(
        entry,
        TreeEntry::Scene {
            category_id,
            mod_index,
            scene_index
        } if category_id == "UI/HUD"
            && state.known_mods[*mod_index].id == "playground-2d"
            && state.known_mods[*mod_index].scenes[*scene_index].id == "screen-space-preview"
    )));
    assert!(entries.iter().any(|entry| matches!(
        entry,
        TreeEntry::Scene {
            category_id,
            mod_index,
            scene_index
        } if category_id == "2D/FX/Particles"
            && state.known_mods[*mod_index].id == "playground-2d-particles"
            && state.known_mods[*mod_index].scenes[*scene_index].id == "showcase"
    )));
    assert!(entries.iter().any(|entry| matches!(
        entry,
        TreeEntry::Scene {
            category_id,
            mod_index,
            scene_index
        } if category_id == "2D/Games/Asteroids"
            && state.known_mods[*mod_index].id == "playground-2d-asteroids"
            && state.known_mods[*mod_index].scenes[*scene_index].id == "main-menu"
    )));
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

    let outcome = state.try_launch(LaunchMode::Headless);

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
        .find(|entry| {
            matches!(
                entry,
                TreeEntry::Scene {
                    mod_index,
                    scene_index,
                    ..
                } if *mod_index == state.selected_mod_index
                    && state
                        .known_mods
                        .get(*mod_index)
                        .and_then(|known_mod| known_mod.scenes.get(*scene_index))
                        .map(|scene| scene.id.as_str() == "screen-space-preview")
                        .unwrap_or(false)
            )
        })
        .expect("screen-space-preview should remain visible after fuzzy filter");
    state.apply_tree_entry(target.clone());

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
