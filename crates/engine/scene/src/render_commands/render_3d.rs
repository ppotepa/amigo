pub const MESH_3D_PLUGIN_SCENE_COMMAND_TYPE: &str = "amigo.rendering.3d.scene-command.Mesh3d";
pub const MATERIAL_3D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.rendering.3d.scene-command.Material3d";
pub const TEXT_3D_PLUGIN_SCENE_COMMAND_TYPE: &str = "amigo.rendering.3d.scene-command.Text3d";

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub mesh_asset: AssetKey,
    pub transform: Transform3,
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub label: String,
    pub albedo: ColorRgba,
    pub source: Option<AssetKey>,
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
