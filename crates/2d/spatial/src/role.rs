#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OpticalLayerRole2d {
    #[default]
    WorldSurface,
    SceneMedium,
    ForegroundMedium,
    LensSurface,
    Overlay,
    Debug,
}
