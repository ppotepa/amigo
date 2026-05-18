use std::sync::Mutex;

use crate::{LightRoute2dCommand, RenderDepth2d, RenderDepthMode2d, RenderLayer2dCommand};

#[derive(Debug, Default)]
pub struct RenderLayer2dSceneService {
    commands: Mutex<Vec<RenderLayer2dCommand>>,
    depth_space: Mutex<amigo_2d_spatial::DepthSpace2d>,
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
        *self
            .depth_space
            .lock()
            .expect("render layer 2d depth space mutex should not be poisoned") =
            amigo_2d_spatial::DepthSpace2d::default();
    }

    pub fn commands(&self) -> Vec<RenderLayer2dCommand> {
        self.commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned")
            .clone()
    }

    pub fn set_depth_space(&self, depth_space: amigo_2d_spatial::DepthSpace2d) {
        *self
            .depth_space
            .lock()
            .expect("render layer 2d depth space mutex should not be poisoned") =
            depth_space.normalized();
    }

    pub fn depth_space(&self) -> amigo_2d_spatial::DepthSpace2d {
        *self
            .depth_space
            .lock()
            .expect("render layer 2d depth space mutex should not be poisoned")
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

    pub fn set_depth(&self, id: &str, depth: RenderDepth2d) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        let Some(layer) = commands.iter_mut().find(|layer| layer.id == id) else {
            return false;
        };
        layer.depth = depth.normalized();
        true
    }

    pub fn set_depth_mode(&self, id: &str, mode: RenderDepthMode2d) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        let Some(layer) = commands.iter_mut().find(|layer| layer.id == id) else {
            return false;
        };
        layer.depth.mode = mode;
        true
    }

    pub fn set_z_depth(&self, id: &str, z_depth: f32) -> bool {
        if !z_depth.is_finite() {
            return false;
        }
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        let Some(layer) = commands.iter_mut().find(|layer| layer.id == id) else {
            return false;
        };
        layer.depth.z_depth = z_depth.clamp(0.0, 1.0);
        true
    }

    pub fn set_distance_m(&self, id: &str, distance_m: f32) -> bool {
        if !distance_m.is_finite() {
            return false;
        }
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        let Some(layer) = commands.iter_mut().find(|layer| layer.id == id) else {
            return false;
        };
        layer.depth.distance_m = Some(distance_m.max(0.0));
        true
    }

    pub fn resolve_distance_with_space(
        &self,
        id: &str,
        depth_space: amigo_2d_spatial::DepthSpace2d,
    ) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        let Some(layer) = commands.iter_mut().find(|layer| layer.id == id) else {
            return false;
        };
        let Some(distance_m) = layer.depth.distance_m else {
            return false;
        };
        let resolved = amigo_2d_spatial::resolve_depth_source(
            amigo_2d_spatial::DepthSource2d::Distance { meters: distance_m },
            depth_space,
        );
        layer.depth.z_depth = resolved.z_depth;
        true
    }

    pub fn set_distance_m_with_default_space(&self, id: &str, distance_m: f32) -> bool {
        if !self.set_distance_m(id, distance_m) {
            return false;
        }
        self.resolve_distance_with_space(id, amigo_2d_spatial::DepthSpace2d::default())
    }

    pub fn set_depth_blur_scale(&self, id: &str, blur_scale: f32) -> bool {
        if !blur_scale.is_finite() {
            return false;
        }
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        let Some(layer) = commands.iter_mut().find(|layer| layer.id == id) else {
            return false;
        };
        layer.depth.blur_scale = blur_scale.clamp(0.0, 4.0);
        true
    }

    pub fn set_optical_role(
        &self,
        id: &str,
        optical_role: amigo_2d_spatial::OpticalLayerRole2d,
    ) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("render layer 2d scene service mutex should not be poisoned");
        let Some(layer) = commands.iter_mut().find(|layer| layer.id == id) else {
            return false;
        };
        layer.optical_role = optical_role;
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
