use std::collections::BTreeMap;

use amigo_math::{Transform2, Transform3, Vec2};

use crate::{Camera3dRenderSettings, Light3dRenderSettings};

#[derive(Debug, Clone, Default)]
pub struct RenderSceneView {
    camera_2d: Transform2,
    camera_3d: Transform3,
    camera_3d_settings: Camera3dRenderSettings,
    light_3d_settings: Light3dRenderSettings,
    entity_transforms: BTreeMap<String, Transform3>,
}

impl RenderSceneView {
    pub fn new(camera_2d: Transform2, camera_3d: Transform3) -> Self {
        Self {
            camera_2d,
            camera_3d,
            camera_3d_settings: Camera3dRenderSettings::default(),
            light_3d_settings: Light3dRenderSettings::default(),
            entity_transforms: BTreeMap::new(),
        }
    }

    pub fn camera_2d(&self) -> Transform2 {
        self.camera_2d
    }

    pub fn camera_3d(&self) -> Transform3 {
        self.camera_3d
    }

    pub fn camera_3d_settings(&self) -> Camera3dRenderSettings {
        self.camera_3d_settings
    }

    pub fn light_3d_settings(&self) -> Light3dRenderSettings {
        self.light_3d_settings
    }

    pub fn set_camera_3d_settings(&mut self, settings: Camera3dRenderSettings) {
        self.camera_3d_settings = settings;
    }

    pub fn set_light_3d_settings(&mut self, settings: Light3dRenderSettings) {
        self.light_3d_settings = settings;
    }

    pub fn insert_entity_transform(
        &mut self,
        entity_name: impl Into<String>,
        transform: Transform3,
    ) -> Option<Transform3> {
        self.entity_transforms.insert(entity_name.into(), transform)
    }

    pub fn transform3_of(&self, entity_name: &str) -> Option<Transform3> {
        self.entity_transforms.get(entity_name).copied()
    }

    pub fn transform2_of(&self, entity_name: &str) -> Option<Transform2> {
        self.transform3_of(entity_name)
            .map(transform2_from_transform3)
    }
}

fn transform2_from_transform3(transform: Transform3) -> Transform2 {
    Transform2 {
        translation: Vec2::new(transform.translation.x, transform.translation.y),
        rotation_radians: transform.rotation_euler.z,
        scale: Vec2::new(transform.scale.x, transform.scale.y),
    }
}
