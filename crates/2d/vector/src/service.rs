use std::sync::Mutex;

use amigo_math::Vec2;
use crate::model::{
    RadialJitterPolygon, VectorShapeKind2d, VectorShape2dDrawCommand,
};

#[derive(Debug, Default)]
pub struct VectorSceneService {
    commands: Mutex<Vec<VectorShape2dDrawCommand>>,
}

impl VectorSceneService {
    pub fn queue(&self, command: VectorShape2dDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("vector scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("vector scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<VectorShape2dDrawCommand> {
        self.commands
            .lock()
            .expect("vector scene service mutex should not be poisoned")
            .clone()
    }

    pub fn set_polygon_points(&self, entity_name: &str, points: Vec<Vec2>) -> bool {
        if points.len() < 3 {
            return false;
        }

        let mut commands = self
            .commands
            .lock()
            .expect("vector scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };

        command.shape.kind = VectorShapeKind2d::Polygon { points };
        true
    }

    pub fn set_polyline_points(&self, entity_name: &str, points: Vec<Vec2>, closed: bool) -> bool {
        if points.len() < 2 {
            return false;
        }

        let mut commands = self
            .commands
            .lock()
            .expect("vector scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };

        command.shape.kind = VectorShapeKind2d::Polyline { points, closed };
        true
    }

    pub fn set_radial_jitter_polygon(
        &self,
        entity_name: &str,
        config: RadialJitterPolygon,
    ) -> bool {
        let Ok(points) = crate::model::radial_jitter_polygon_points(config) else {
            return false;
        };
        let mut commands = self
            .commands
            .lock()
            .expect("vector scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };

        command.shape.kind = VectorShapeKind2d::Polygon { points };
        true
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}
