use std::sync::Mutex;

use crate::{LayeredImageBlendMode2d, LayeredImageDrawCommand, LayeredImageLayerOverride};

#[derive(Debug, Default)]
pub struct LayeredImageSceneService {
    commands: Mutex<Vec<LayeredImageDrawCommand>>,
}

impl LayeredImageSceneService {
    pub fn queue(&self, command: LayeredImageDrawCommand) {
        self.commands
            .lock()
            .expect("layered image scene service mutex should not be poisoned")
            .push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("layered image scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<LayeredImageDrawCommand> {
        self.commands
            .lock()
            .expect("layered image scene service mutex should not be poisoned")
            .clone()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }

    pub fn set_base_opacity(&self, entity_name: &str, opacity: f32) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("layered image scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };
        command.image.base_opacity = opacity.clamp(0.0, 1.0);
        true
    }

    pub fn set_layer_opacity(&self, entity_name: &str, layer_id: &str, opacity: f32) -> bool {
        self.update_override(entity_name, layer_id, |override_| {
            override_.opacity = Some(opacity.clamp(0.0, 4.0));
        })
    }

    pub fn set_layer_enabled(&self, entity_name: &str, layer_id: &str, enabled: bool) -> bool {
        self.update_override(entity_name, layer_id, |override_| {
            override_.enabled = Some(enabled);
        })
    }

    pub fn set_layer_blend_mode(
        &self,
        entity_name: &str,
        layer_id: &str,
        blend_mode: LayeredImageBlendMode2d,
    ) -> bool {
        self.update_override(entity_name, layer_id, |override_| {
            override_.blend_mode = Some(blend_mode);
        })
    }

    fn update_override(
        &self,
        entity_name: &str,
        layer_id: &str,
        update: impl FnOnce(&mut LayeredImageLayerOverride),
    ) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("layered image scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };
        let index = command
            .image
            .layer_overrides
            .iter()
            .position(|override_| override_.id == layer_id)
            .unwrap_or_else(|| {
                command
                    .image
                    .layer_overrides
                    .push(LayeredImageLayerOverride {
                        id: layer_id.to_owned(),
                        opacity: None,
                        enabled: None,
                        blend_mode: None,
                    });
                command.image.layer_overrides.len() - 1
            });
        update(&mut command.image.layer_overrides[index]);
        true
    }
}
