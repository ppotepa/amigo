impl Display for BootstrapSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Amigo bootstrap")?;
        writeln!(f, "window backend: {}", self.window_backend)?;
        writeln!(f, "input backend: {}", self.input_backend)?;
        writeln!(f, "render backend: {}", self.render_backend)?;
        writeln!(f, "script backend: {}", self.script_backend)?;
        writeln!(f, "file watch backend: {}", self.file_watch_backend)?;
        writeln!(
            f,
            "mods: {}",
            app_helpers::display_string_list(&self.loaded_mods)
        )?;
        writeln!(
            f,
            "scripts: {}",
            app_helpers::display_executed_scripts(&self.executed_scripts)
        )?;
        writeln!(
            f,
            "root mod: {}",
            self.startup_mod.as_deref().unwrap_or("none")
        )?;
        writeln!(
            f,
            "startup scene: {}",
            self.startup_scene.as_deref().unwrap_or("none")
        )?;
        writeln!(
            f,
            "active scene: {}",
            self.active_scene.as_deref().unwrap_or("none")
        )?;
        writeln!(
            f,
            "scene document: {}",
            self.loaded_scene_document
                .as_ref()
                .map(|document| format!(
                    "{}:{}",
                    document.source_mod,
                    document.relative_path.display()
                ))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene document entities: {}",
            self.loaded_scene_document
                .as_ref()
                .map(|document| app_helpers::display_string_list(&document.entity_names))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene document components: {}",
            self.loaded_scene_document
                .as_ref()
                .map(|document| app_helpers::display_string_list(&document.component_kinds))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene document transitions: {}",
            self.loaded_scene_document
                .as_ref()
                .map(|document| app_helpers::display_string_list(&document.transition_ids))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene entities: {}",
            app_helpers::display_string_list(&self.scene_entities)
        )?;
        writeln!(
            f,
            "registered assets: {}",
            app_helpers::display_string_list(&self.registered_assets)
        )?;
        writeln!(
            f,
            "loaded assets: {}",
            app_helpers::display_string_list(&self.loaded_assets)
        )?;
        writeln!(
            f,
            "prepared assets: {}",
            app_helpers::display_string_list(&self.prepared_assets)
        )?;
        writeln!(
            f,
            "failed assets: {}",
            app_helpers::display_string_list(&self.failed_assets)
        )?;
        writeln!(
            f,
            "pending asset loads: {}",
            app_helpers::display_string_list(&self.pending_asset_loads)
        )?;
        writeln!(
            f,
            "watched reload targets: {}",
            app_helpers::display_string_list(&self.watched_reload_targets)
        )?;
        writeln!(
            f,
            "2d sprite entities: {}",
            app_helpers::display_string_list(&self.sprite_entities_2d)
        )?;
        writeln!(
            f,
            "2d text entities: {}",
            app_helpers::display_string_list(&self.text_entities_2d)
        )?;
        writeln!(
            f,
            "2d vector entities: {}",
            app_helpers::display_string_list(&self.vector_entities_2d)
        )?;
        writeln!(
            f,
            "3d mesh entities: {}",
            app_helpers::display_string_list(&self.mesh_entities_3d)
        )?;
        writeln!(
            f,
            "3d material entities: {}",
            app_helpers::display_string_list(&self.material_entities_3d)
        )?;
        writeln!(
            f,
            "3d text entities: {}",
            app_helpers::display_string_list(&self.text_entities_3d)
        )?;
        writeln!(
            f,
            "ui entities: {}",
            app_helpers::display_string_list(&self.ui_entities)
        )?;
        writeln!(
            f,
            "audio clips: {}",
            app_helpers::display_string_list(&self.audio_clips)
        )?;
        writeln!(
            f,
            "audio sources: {}",
            app_helpers::display_string_list(&self.audio_sources)
        )?;
        writeln!(
            f,
            "audio runtime commands: {}",
            app_helpers::display_string_list(&self.pending_audio_runtime_commands)
        )?;
        writeln!(f, "audio master volume: {}", self.audio_master_volume)?;
        writeln!(f, "audio mix frames: {}", self.mixed_audio_frame_count)?;
        writeln!(
            f,
            "audio realtime sources: {}",
            app_helpers::display_string_list(&self.active_realtime_audio_sources)
        )?;
        writeln!(
            f,
            "audio output: started={} device={} buffered_samples={} last_error={}",
            self.audio_output_started,
            self.audio_output_device.as_deref().unwrap_or("none"),
            self.audio_output_buffered_samples,
            self.audio_output_last_error.as_deref().unwrap_or("none")
        )?;
        writeln!(
            f,
            "script commands: {}",
            app_helpers::display_string_list(&self.processed_script_commands)
        )?;
        writeln!(
            f,
            "audio commands: {}",
            app_helpers::display_string_list(&self.processed_audio_commands)
        )?;
        writeln!(
            f,
            "scene commands: {}",
            app_helpers::display_string_list(&self.processed_scene_commands)
        )?;
        writeln!(
            f,
            "script events: {}",
            app_helpers::display_string_list(&self.processed_script_events)
        )?;
        writeln!(
            f,
            "console commands: {}",
            app_helpers::display_string_list(&self.console_commands)
        )?;
        writeln!(
            f,
            "console output: {}",
            app_helpers::display_string_list(&self.console_output)
        )?;
        writeln!(
            f,
            "capabilities: {}",
            app_helpers::display_string_list(&self.capabilities)
        )?;
        writeln!(
            f,
            "plugins: {}",
            app_helpers::display_string_list(&self.plugins)
        )?;
        write!(
            f,
            "services: {}",
            app_helpers::display_string_list(&self.services)
        )
    }
}
