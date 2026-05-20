use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CameraDebugViewId(String);

impl CameraDebugViewId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "final" | "final_output" | "camera.final" => Self::final_output(),
            "raw_scene_color" | "raw" | "source" | "camera.raw_scene_color" => {
                Self::raw_scene_color()
            }
            "scene_depth" | "depth" | "camera.scene_depth" => Self::scene_depth(),
            "scene_normals" | "scene_normal" | "normals" | "camera.scene_normal" => {
                Self::scene_normal()
            }
            "scene_emissive" | "emissive" | "camera.scene_emissive" => Self::scene_emissive(),
            "scene_highlights" | "scene_highlight" | "highlights" | "camera.scene_highlight" => {
                Self::scene_highlight()
            }
            "scene_motion" | "motion" | "camera.scene_motion" => Self::scene_motion(),
            "camera_artifacts" | "artifacts" | "camera.artifacts" => Self::camera_artifacts(),
            "computed_z_depth" | "z_depth" | "computed_depth" => {
                Self::new("camera.computed_z_depth")
            }
            "layer_optical_roles" | "roles" => Self::new("camera.layer_optical_roles"),
            "layer_mask" | "mask" => Self::new("camera.layer_mask"),
            "scene_wetness" | "wetness" => Self::new("camera.scene_wetness"),
            "camera_after_exposure" | "after_exposure" => Self::new("camera.after_exposure"),
            "camera_after_optics" | "after_optics" => Self::new("camera.after_optics"),
            "camera_after_dof" | "after_dof" | "dof" => Self::new("camera.after_dof"),
            "camera_after_lens_surface" | "after_lens_surface" => {
                Self::new("camera.after_lens_surface")
            }
            "camera_after_film" | "after_film" => Self::new("camera.after_film"),
            "camera_after_look" | "after_look" => Self::new("camera.after_look"),
            other => Self::new(other),
        }
    }

    pub fn final_output() -> Self {
        Self::new("camera.final")
    }

    pub fn raw_scene_color() -> Self {
        Self::new("camera.raw_scene_color")
    }

    pub fn scene_depth() -> Self {
        Self::new("camera.scene_depth")
    }

    pub fn scene_normal() -> Self {
        Self::new("camera.scene_normal")
    }

    pub fn scene_emissive() -> Self {
        Self::new("camera.scene_emissive")
    }

    pub fn scene_highlight() -> Self {
        Self::new("camera.scene_highlight")
    }

    pub fn scene_motion() -> Self {
        Self::new("camera.scene_motion")
    }

    pub fn camera_artifacts() -> Self {
        Self::new("camera.artifacts")
    }
}

impl Default for CameraDebugViewId {
    fn default() -> Self {
        Self::final_output()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraDebugViewDescriptor {
    pub id: CameraDebugViewId,
    pub label: String,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub stop_after_feature: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CameraDebugViewRegistry {
    views: std::collections::BTreeMap<CameraDebugViewId, CameraDebugViewDescriptor>,
    aliases: std::collections::BTreeMap<String, CameraDebugViewId>,
}

impl CameraDebugViewRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, descriptor: CameraDebugViewDescriptor) {
        for alias in &descriptor.aliases {
            self.aliases.insert(alias.clone(), descriptor.id.clone());
        }
        self.views.insert(descriptor.id.clone(), descriptor);
    }

    pub fn get(&self, id: &CameraDebugViewId) -> Option<&CameraDebugViewDescriptor> {
        self.views.get(id)
    }

    pub fn resolve(&self, value: &str) -> Option<CameraDebugViewId> {
        let direct = CameraDebugViewId::new(value);
        if self.views.contains_key(&direct) {
            return Some(direct);
        }
        self.aliases.get(value).cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CameraDebugViewDescriptor> {
        self.views.values()
    }
}
