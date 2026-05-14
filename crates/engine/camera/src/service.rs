use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_math::Vec2;
use amigo_scene::{CameraFollow2dSceneCommand, Parallax2dSceneCommand};

use crate::{Camera, CameraId};

#[derive(Debug, Default)]
pub struct CameraService {
    cameras: Mutex<BTreeMap<CameraId, Camera>>,
}

impl CameraService {
    pub fn upsert(&self, camera: Camera) {
        self.cameras
            .lock()
            .expect("camera service mutex should not be poisoned")
            .insert(camera.id.clone(), camera);
    }

    pub fn get(&self, id: &CameraId) -> Option<Camera> {
        self.cameras
            .lock()
            .expect("camera service mutex should not be poisoned")
            .get(id)
            .cloned()
    }

    pub fn main_camera_id(&self) -> Option<CameraId> {
        let id = CameraId::new("main");
        self.get(&id).map(|camera| camera.id)
    }

    pub fn camera(&self, id: &CameraId) -> Option<Camera> {
        self.get(id)
    }

    pub fn camera_by_binding(&self, binding: &amigo_render_api::CameraBinding) -> Option<Camera> {
        let id = CameraId::new(binding.camera_id.clone());
        self.camera(&id).or_else(|| match binding.fallback {
            amigo_render_api::CameraFallback::Main => {
                self.main_camera_id().and_then(|main| self.camera(&main))
            }
            amigo_render_api::CameraFallback::None => None,
        })
    }

    pub fn cameras(&self) -> Vec<Camera> {
        self.cameras
            .lock()
            .expect("camera service mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct CameraFollow2dSceneService {
    commands: Mutex<Vec<CameraFollow2dSceneCommand>>,
}

impl CameraFollow2dSceneService {
    pub fn queue(&self, command: CameraFollow2dSceneCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<CameraFollow2dSceneCommand> {
        self.commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned")
            .clone()
    }

    pub fn follow(&self, entity_name: &str) -> Option<CameraFollow2dSceneCommand> {
        self.commands()
            .into_iter()
            .find(|command| command.entity_name == entity_name)
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct Parallax2dSceneService {
    commands: Mutex<Vec<Parallax2dSceneCommand>>,
}

impl Parallax2dSceneService {
    pub fn queue(&self, command: Parallax2dSceneCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<Parallax2dSceneCommand> {
        self.commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned")
            .clone()
    }

    pub fn set_camera_origin(&self, entity_name: &str, camera_origin: Vec2) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };
        command.camera_origin = Some(camera_origin);
        true
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}
