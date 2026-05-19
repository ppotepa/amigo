#[derive(Clone, Debug, PartialEq)]
pub struct FilmLookResponse2dDocument {
    pub enabled: bool,
    pub grain: f32,
    pub halation: f32,
    pub sensor_response: f32,
    pub film_response: f32,
    pub tone_curve: f32,
}

impl Default for FilmLookResponse2dDocument {
    fn default() -> Self {
        Self {
            enabled: false,
            grain: 0.0,
            halation: 0.0,
            sensor_response: 1.0,
            film_response: 1.0,
            tone_curve: 1.0,
        }
    }
}
