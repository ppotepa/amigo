#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraOpticalResponse2dSceneCommand {
    pub enabled: bool,
    pub intensity: f32,
    pub bloom: f32,
    pub glare: f32,
    pub ghosting: f32,
    pub streaks: f32,
    pub chromatic_smear: f32,
    pub dirt_response: f32,
    pub halation: f32,
    pub threshold: f32,
}
