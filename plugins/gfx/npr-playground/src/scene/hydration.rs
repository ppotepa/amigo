use amigo_scene::{
    PluginComponentHydrationContext, PluginComponentHydrator, SceneCommand, SceneDocumentError,
    SceneDocumentResult,
};

use super::{
    document::{NprPlaygroundSceneDocument, NPR_SETTINGS_COMPONENT_TYPE},
    npr_playground_plugin_scene_command, NprPlaygroundSceneCommand,
};

#[derive(Default)]
pub struct NprPlaygroundPluginComponentHydrator;

impl PluginComponentHydrator for NprPlaygroundPluginComponentHydrator {
    fn provider_id(&self) -> &'static str { "amigo.gfx.npr-playground" }
    fn component_type(&self) -> &'static str { NPR_SETTINGS_COMPONENT_TYPE }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx.payload.as_any().downcast_ref::<NprPlaygroundSceneDocument>() else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "NPR settings hydrator received wrong payload".to_owned(),
            });
        };
        ctx.commands.push(SceneCommand::Plugin {
            command: npr_playground_plugin_scene_command(NprPlaygroundSceneCommand {
                settings: document.clone(),
            }),
        });
        Ok(())
    }
}
