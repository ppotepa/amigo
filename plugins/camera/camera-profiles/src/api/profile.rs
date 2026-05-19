#[derive(Clone, Debug, PartialEq)]
pub struct CameraProfileRef {
    pub id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CameraProfile2d {
    pub id: String,
    pub label: String,
    pub lens_profile: Option<String>,
    pub film_profile: Option<String>,
    pub focus_distance_m: Option<f32>,
}

impl CameraProfile2d {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            lens_profile: None,
            film_profile: None,
            focus_distance_m: None,
        }
    }
}
