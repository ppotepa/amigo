use std::sync::Mutex;

use crate::{LightRoute2dCommand, RenderLayer2dCommand};

#[derive(Debug, Default)]
pub struct RenderLayer2dSceneService {
    commands: Mutex<Vec<RenderLayer2dCommand>>,
}

impl RenderLayer2dSceneService {
    pub fn queue(&self, command: RenderLayer2dCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        commands.retain(|existing| existing.id != command.id);
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<RenderLayer2dCommand> {
        self.commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned")
            .clone()
    }

    pub fn set_opacity(&self, id: &str, opacity: f32) -> bool {
        if !opacity.is_finite() {
            return false;
        }
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        let Some(layer) = commands.iter_mut().find(|layer| layer.id == id) else {
            return false;
        };
        layer.opacity = opacity.clamp(0.0, 1.0);
        true
    }

    pub fn set_visible(&self, id: &str, visible: bool) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        let Some(layer) = commands.iter_mut().find(|layer| layer.id == id) else {
            return false;
        };
        layer.visible = visible;
        true
    }

    pub fn set_order(&self, id: &str, order: f32) -> bool {
        if !order.is_finite() {
            return false;
        }
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        let Some(layer) = commands.iter_mut().find(|layer| layer.id == id) else {
            return false;
        };
        layer.order = order;
        true
    }
}

#[derive(Debug, Default)]
pub struct LightRoute2dSceneService {
    commands: Mutex<Vec<LightRoute2dCommand>>,
}

impl LightRoute2dSceneService {
    pub fn queue(&self, command: LightRoute2dCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("light route 2d scene service mutex should not be poisoned");
        commands.retain(|existing| existing.receiver_layer != command.receiver_layer);
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("light route 2d scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<LightRoute2dCommand> {
        self.commands
            .lock()
            .expect("light route 2d scene service mutex should not be poisoned")
            .clone()
    }
}
