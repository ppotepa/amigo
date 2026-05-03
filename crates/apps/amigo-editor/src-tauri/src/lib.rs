mod cache;
mod commands;
mod dto;
mod mods;
mod preview;
mod session;
mod settings;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;

            let app_cache_root = app.path().app_cache_dir().ok();
            let cache_root = cache::root::resolve_cache_root_with_app_cache(app_cache_root);
            app.manage(cache::root::EditorPaths {
                cache_root: cache_root.path,
                cache_root_mode: cache::root::cache_root_mode_name(&cache_root.mode),
            });
            app.manage(session::EditorSessionRegistry::default());
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
            commands::validate_mod,
            commands::regenerate_all_scene_previews,
            commands::reveal_mod_folder,
            commands::reveal_scene_document,
            commands::get_scene_hierarchy,
            commands::get_project_tree,
            commands::read_project_file,
            commands::reveal_project_file,
            commands::get_theme_settings,
            commands::set_theme_settings,
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
