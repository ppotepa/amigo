use std::sync::Mutex;

use amigo_math::ColorRgba;

use crate::{GlobalLight2dCommand, LightGroup2dCommand, LightMap2dSourceCommand};

#[derive(Debug, Default)]
pub struct GlobalLight2dSceneService {
    commands: Mutex<Vec<GlobalLight2dCommand>>,
}

impl GlobalLight2dSceneService {
    pub fn queue(&self, command: GlobalLight2dCommand) {
        self.commands
            .lock()
            .expect("global light 2d scene service mutex should not be poisoned")
            .push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("global light 2d scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<GlobalLight2dCommand> {
        self.commands
            .lock()
            .expect("global light 2d scene service mutex should not be poisoned")
            .clone()
    }

    pub fn set_intensity(&self, id: &str, intensity: f32) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("global light 2d scene service mutex should not be poisoned");
        let Some(command) = commands.iter_mut().find(|command| command.id == id) else {
            return false;
        };
        command.intensity = intensity.max(0.0);
        true
    }

    pub fn set_color(&self, id: &str, color: ColorRgba) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("global light 2d scene service mutex should not be poisoned");
        let Some(command) = commands.iter_mut().find(|command| command.id == id) else {
            return false;
        };
        command.color = color;
        true
    }
}

#[derive(Debug, Default)]
pub struct LightGroup2dSceneService {
    commands: Mutex<Vec<LightGroup2dCommand>>,
}

impl LightGroup2dSceneService {
    pub fn queue(&self, command: LightGroup2dCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("light group 2d scene service mutex should not be poisoned");
        commands.retain(|existing| existing.id != command.id);
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("light group 2d scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<LightGroup2dCommand> {
        self.commands
            .lock()
            .expect("light group 2d scene service mutex should not be poisoned")
            .clone()
    }

    pub fn set_intensity(&self, id: &str, intensity: f32) -> bool {
        if !intensity.is_finite() {
            return false;
        }
        let mut commands = self
            .commands
            .lock()
            .expect("light group 2d scene service mutex should not be poisoned");
        let Some(command) = commands.iter_mut().find(|command| command.id == id) else {
            return false;
        };
        command.intensity = intensity.max(0.0);
        true
    }

    pub fn set_color(&self, id: &str, color: ColorRgba) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("light group 2d scene service mutex should not be poisoned");
        let Some(command) = commands.iter_mut().find(|command| command.id == id) else {
            return false;
        };
        command.color = color;
        true
    }
}

#[derive(Debug, Default)]
pub struct LightMap2dSceneService {
    commands: Mutex<Vec<LightMap2dSourceCommand>>,
}

impl LightMap2dSceneService {
    pub fn queue(&self, command: LightMap2dSourceCommand) {
        self.commands
            .lock()
            .expect("lightmap 2d scene service mutex should not be poisoned")
            .push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("lightmap 2d scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<LightMap2dSourceCommand> {
        self.commands
            .lock()
            .expect("lightmap 2d scene service mutex should not be poisoned")
            .clone()
    }
}
