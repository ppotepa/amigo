mod asset_registry;
mod cache;
mod commands;
mod dto;
mod events;
mod mods;
mod preview;
mod session;
mod settings;
mod sheet;
mod windows;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .on_window_event(|window, event| {
            if !matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                return;
            }

            let label = window.label().to_owned();
            let app = window.app_handle();
            let registry = app.state::<windows::registry::EditorWindowRegistry>();
            let _ = registry.remove_window(&label);

            if let Some(session_id) = label.strip_prefix("workspace-") {
                let sessions = app.state::<session::EditorSessionRegistry>();
                let _ = sessions.close_session(session_id);
                let _ = events::bus::emit_session_closed(app, session_id.to_owned());
            }
        })
        .setup(|app| {
            use tauri::Manager;

            let app_cache_root = app.path().app_cache_dir().ok();
            let cache_root = cache::root::resolve_cache_root_with_app_cache(app_cache_root);
            app.manage(cache::root::EditorPaths {
                cache_root: cache_root.path,
                cache_root_mode: cache::root::cache_root_mode_name(&cache_root.mode),
            });
            app.manage(session::EditorSessionRegistry::default());
            app.manage(windows::registry::EditorWindowRegistry::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_known_mods,
            commands::get_mod_details,
            commands::request_scene_preview,
            commands::open_mod,
            commands::open_mod_workspace,
            commands::get_editor_session,
            commands::close_editor_session,
            commands::open_theme_window,
            commands::open_settings_window,
            commands::open_mod_settings_window,
            commands::register_editor_window,
            commands::mark_editor_window_focused,
            commands::unregister_editor_window,
            commands::get_window_registry,
            commands::focus_workspace_window,
            commands::close_workspace_window,
            commands::validate_mod,
            commands::regenerate_all_scene_previews,
            commands::reveal_mod_folder,
            commands::reveal_scene_document,
            commands::get_scene_hierarchy,
            commands::get_project_tree,
            commands::get_project_structure_tree,
            commands::read_project_file,
            commands::write_project_file,
            commands::reveal_project_file,
            commands::create_expected_project_folder,
            commands::get_asset_registry,
            commands::create_asset_descriptor,
            commands::scan_asset_migration_plan,
            commands::apply_asset_migration_plan,
            commands::load_sheet_resource,
            commands::load_tilemap_resource,
            commands::save_sheet_resource,
            commands::save_tilemap_resource,
            commands::get_theme_settings,
            commands::set_theme_settings,
            commands::set_font_settings,
            commands::get_editor_settings,
            commands::set_editor_mods_root,
            commands::reset_editor_mods_root,
            commands::pick_mods_root,
            commands::get_cache_info,
            commands::get_cache_policy,
            commands::set_cache_policy,
            commands::run_cache_maintenance,
            commands::clear_orphaned_project_caches,
            commands::clear_project_cache,
            commands::clear_preview_cache,
            commands::clear_all_preview_cache,
            commands::reveal_cache_folder
        ])
        .run(tauri::generate_context!())
        .expect("failed to run amigo-editor");
}
