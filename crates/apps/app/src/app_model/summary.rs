#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptExecutionRole {
    ModBootstrap,
    ModPersistent,
    Scene,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedScript {
    pub source_name: String,
    pub mod_id: String,
    pub scene_id: Option<String>,
    pub relative_script_path: PathBuf,
    pub role: ScriptExecutionRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSceneDocumentSummary {
    pub source_mod: String,
    pub scene_id: String,
    pub relative_path: PathBuf,
    pub entity_names: Vec<String>,
    pub component_kinds: Vec<String>,
    pub transition_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct LoadedSceneDocument {
    summary: LoadedSceneDocumentSummary,
    hydration_plan: SceneHydrationPlan,
    transition_plan: Option<SceneTransitionPlan>,
}

#[derive(Debug, Clone)]
pub struct BootstrapSummary {
    pub window_backend: String,
    pub input_backend: String,
    pub render_backend: String,
    pub script_backend: String,
    pub file_watch_backend: String,
    pub loaded_mods: Vec<String>,
    pub executed_scripts: Vec<ExecutedScript>,
    pub startup_mod: Option<String>,
    pub startup_scene: Option<String>,
    pub active_scene: Option<String>,
    pub loaded_scene_document: Option<LoadedSceneDocumentSummary>,
    pub scene_entities: Vec<String>,
    pub registered_assets: Vec<String>,
    pub loaded_assets: Vec<String>,
    pub prepared_assets: Vec<String>,
    pub failed_assets: Vec<String>,
    pub pending_asset_loads: Vec<String>,
    pub watched_reload_targets: Vec<String>,
    pub sprite_entities_2d: Vec<String>,
    pub text_entities_2d: Vec<String>,
    pub vector_entities_2d: Vec<String>,
    pub mesh_entities_3d: Vec<String>,
    pub material_entities_3d: Vec<String>,
    pub text_entities_3d: Vec<String>,
    pub ui_entities: Vec<String>,
    pub audio_clips: Vec<String>,
    pub audio_sources: Vec<String>,
    pub pending_audio_runtime_commands: Vec<String>,
    pub audio_master_volume: f32,
    pub mixed_audio_frame_count: usize,
    pub active_realtime_audio_sources: Vec<String>,
    pub audio_output_started: bool,
    pub audio_output_device: Option<String>,
    pub audio_output_buffered_samples: usize,
    pub audio_output_last_error: Option<String>,
    pub processed_script_commands: Vec<String>,
    pub processed_audio_commands: Vec<String>,
    pub processed_scene_commands: Vec<String>,
    pub processed_script_events: Vec<String>,
    pub console_commands: Vec<String>,
    pub console_output: Vec<String>,
    pub capabilities: Vec<String>,
    pub plugins: Vec<String>,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct PlaceholderBridgeSummary {
    processed_script_commands: Vec<String>,
    processed_audio_commands: Vec<String>,
    processed_scene_commands: Vec<String>,
    processed_script_events: Vec<String>,
    console_commands: Vec<String>,
    console_output: Vec<String>,
}

