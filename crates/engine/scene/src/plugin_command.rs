use std::any::Any;
use std::fmt;
use std::sync::Arc;

use amigo_assets::AssetKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneAssetDependency {
    pub source_mod: String,
    pub key: AssetKey,
    pub domain_scope: &'static str,
    pub domain_tag: &'static str,
}

impl SceneAssetDependency {
    pub fn new(
        source_mod: impl Into<String>,
        key: AssetKey,
        domain_scope: &'static str,
        domain_tag: &'static str,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            key,
            domain_scope,
            domain_tag,
        }
    }
}

pub trait PluginSceneCommandPayload: Send + Sync {
    fn command_type(&self) -> &'static str;
    fn command_as_any(&self) -> &dyn Any;
    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool;

    fn asset_dependencies(&self) -> Vec<SceneAssetDependency> {
        Vec::new()
    }
}

#[derive(Clone)]
pub struct PluginSceneCommand {
    pub command_type: String,
    pub payload: Arc<dyn PluginSceneCommandPayload>,
}

impl PluginSceneCommand {
    pub fn new(payload: Arc<dyn PluginSceneCommandPayload>) -> Self {
        let command_type = payload.command_type().to_owned();
        Self {
            command_type,
            payload,
        }
    }

    pub fn payload_as<T: 'static>(&self) -> Option<&T> {
        self.payload.command_as_any().downcast_ref::<T>()
    }

    pub fn asset_dependencies(&self) -> Vec<SceneAssetDependency> {
        self.payload.asset_dependencies()
    }
}

impl fmt::Debug for PluginSceneCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginSceneCommand")
            .field("command_type", &self.command_type)
            .finish()
    }
}

impl PartialEq for PluginSceneCommand {
    fn eq(&self, other: &Self) -> bool {
        self.command_type == other.command_type && self.payload.eq_payload(other.payload.as_ref())
    }
}
