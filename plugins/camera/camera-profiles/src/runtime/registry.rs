use std::collections::BTreeMap;

use crate::api::CameraProfile2d;

#[derive(Clone, Debug, Default)]
pub struct CameraProfileRegistry2d {
    profiles: BTreeMap<String, CameraProfile2d>,
}

impl CameraProfileRegistry2d {
    pub fn insert(&mut self, profile: CameraProfile2d) {
        self.profiles.insert(profile.id.clone(), profile);
    }

    pub fn get(&self, id: &str) -> Option<&CameraProfile2d> {
        self.profiles.get(id)
    }

    pub fn profiles(&self) -> impl Iterator<Item = &CameraProfile2d> {
        self.profiles.values()
    }
}
