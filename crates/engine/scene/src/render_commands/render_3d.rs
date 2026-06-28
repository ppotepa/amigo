pub const MESH_3D_PLUGIN_SCENE_COMMAND_TYPE: &str = "amigo.rendering.3d.scene-command.Mesh3d";
pub const MATERIAL_3D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.rendering.3d.scene-command.Material3d";
pub const TEXT_3D_PLUGIN_SCENE_COMMAND_TYPE: &str = "amigo.rendering.3d.scene-command.Text3d";
pub const CAMERA_CONTROLLER_3D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.camera.camera-core.scene-command.CameraController3d";
pub const NPR_PRESET_3D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.rendering.3d.scene-command.NprPreset3d";

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub mesh_asset: AssetKey,
    pub transform: Transform3,
    pub npr: Option<amigo_render_api::NprLineSettings3d>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh3dPluginSceneCommandPayload(pub Mesh3dSceneCommand);

impl crate::PluginSceneCommandPayload for Mesh3dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        MESH_3D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Mesh3dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<crate::SceneAssetDependency> {
        let command = &self.0;
        vec![crate::SceneAssetDependency::new(
            command.source_mod.clone(),
            command.mesh_asset.clone(),
            "meshes",
            "mesh-3d",
        )]
    }
}

pub fn mesh_3d_plugin_scene_command(command: Mesh3dSceneCommand) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(Mesh3dPluginSceneCommandPayload(
        command,
    )))
}

impl Mesh3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        mesh_asset: AssetKey,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            mesh_asset,
            transform: Transform3::default(),
            npr: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprPreset3dSceneCommand {
    pub source_mod: String,
    pub id: String,
    pub label: String,
    pub settings: amigo_render_api::NprLineSettings3d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprPreset3dPluginSceneCommandPayload(pub NprPreset3dSceneCommand);

impl crate::PluginSceneCommandPayload for NprPreset3dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        NPR_PRESET_3D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<NprPreset3dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn npr_preset_3d_plugin_scene_command(
    command: NprPreset3dSceneCommand,
) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(
        NprPreset3dPluginSceneCommandPayload(command),
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub label: String,
    pub albedo: ColorRgba,
    pub source: Option<AssetKey>,
    pub render_order: i32,
    pub shading: amigo_render_api::Material3dShadingMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material3dPluginSceneCommandPayload(pub Material3dSceneCommand);

impl crate::PluginSceneCommandPayload for Material3dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        MATERIAL_3D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Material3dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<crate::SceneAssetDependency> {
        let command = &self.0;
        command
            .source
            .as_ref()
            .map(|source| {
                vec![crate::SceneAssetDependency::new(
                    command.source_mod.clone(),
                    source.clone(),
                    "materials",
                    "material-3d",
                )]
            })
            .unwrap_or_default()
    }
}

pub fn material_3d_plugin_scene_command(
    command: Material3dSceneCommand,
) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(Material3dPluginSceneCommandPayload(
        command,
    )))
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraController3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub camera: String,
    pub mode: CameraController3dModeSceneCommand,
    pub switch_action: Option<String>,
    pub orbit_target: Option<String>,
    pub orbit_distance: f32,
    pub orbit_min_distance: f32,
    pub orbit_max_distance: f32,
    pub orbit_yaw: f32,
    pub orbit_pitch: f32,
    pub orbit_sensitivity: f32,
    pub orbit_zoom_speed: f32,
    pub freelook_speed: f32,
    pub freelook_sensitivity: f32,
    pub move_forward_action: String,
    pub move_strafe_action: String,
    pub move_lift_action: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraController3dModeSceneCommand {
    Orbit,
    Freelook,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraController3dPluginSceneCommandPayload(pub CameraController3dSceneCommand);

impl crate::PluginSceneCommandPayload for CameraController3dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        CAMERA_CONTROLLER_3D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<CameraController3dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn camera_controller_3d_plugin_scene_command(
    command: CameraController3dSceneCommand,
) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(
        CameraController3dPluginSceneCommandPayload(command),
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub content: String,
    pub font: AssetKey,
    pub size: f32,
    pub transform: Transform3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text3dPluginSceneCommandPayload(pub Text3dSceneCommand);

impl crate::PluginSceneCommandPayload for Text3dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        TEXT_3D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Text3dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<crate::SceneAssetDependency> {
        let command = &self.0;
        vec![crate::SceneAssetDependency::new(
            command.source_mod.clone(),
            command.font.clone(),
            "fonts",
            "font-3d",
        )]
    }
}

pub fn text_3d_plugin_scene_command(command: Text3dSceneCommand) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(Text3dPluginSceneCommandPayload(
        command,
    )))
}

impl Text3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        content: impl Into<String>,
        font: AssetKey,
        size: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            content: content.into(),
            font,
            size,
            transform: Transform3::default(),
        }
    }
}
