use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::{Sprite2dSceneCommand, Text2dSceneCommand, VectorShape2dSceneCommand};

pub trait PluginSceneCommandPayload: Send + Sync {
    fn command_type(&self) -> &'static str;
    fn command_as_any(&self) -> &dyn Any;
    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool;
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

    pub fn sprite_2d_command(&self) -> Option<&Sprite2dSceneCommand> {
        self.payload_as::<Sprite2dSceneCommand>()
    }

    pub fn text_2d_command(&self) -> Option<&Text2dSceneCommand> {
        self.payload_as::<Text2dSceneCommand>()
    }

    pub fn vector_shape_2d_command(&self) -> Option<&VectorShape2dSceneCommand> {
        self.payload_as::<VectorShape2dSceneCommand>()
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
