use crate::CameraOpticalResponse2d;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightEmitterKind2d {
    Beacon,
    GlobalLight,
    LightMapSource,
    LightMapChannel,
    LightGroup,
    EmissiveMaterial,
    EmissiveVisualSource,
    ParticleLight,
    Point,
    Directional,
    Spot,
    Area,
}

impl LightEmitterKind2d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Beacon => "beacon",
            Self::GlobalLight => "global_light",
            Self::LightMapSource => "lightmap_source",
            Self::LightMapChannel => "lightmap_channel",
            Self::LightGroup => "light_group",
            Self::EmissiveMaterial => "emissive_material",
            Self::EmissiveVisualSource => "emissive_visual_source",
            Self::ParticleLight => "particle_light",
            Self::Point => "point",
            Self::Directional => "directional",
            Self::Spot => "spot",
            Self::Area => "area",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightContributionKind2d {
    LightingEmit,
    RelightPlate,
    BloomSource,
    CameraFxSource,
    EmissiveBuffer,
}

impl LightContributionKind2d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LightingEmit => "lighting_emit",
            Self::RelightPlate => "relight_plate",
            Self::BloomSource => "bloom_source",
            Self::CameraFxSource => "camera_fx_source",
            Self::EmissiveBuffer => "emissive_buffer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightSourceStatus2d {
    Active,
    Skipped,
}

impl LightSourceStatus2d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightSource2dCommon {
    pub owner: String,
    pub component_kind: String,
    pub emitter_kind: LightEmitterKind2d,
    pub emitter_id: Option<String>,
    pub render_layer: Option<String>,
    pub color_rgba: Option<[f32; 4]>,
    pub intensity: Option<f32>,
    pub effective_intensity: Option<f32>,
    pub response: Option<f32>,
    pub camera_response: Option<CameraOpticalResponse2d>,
    pub bloom: Option<f32>,
    pub lens_influence: Option<f32>,
    pub radius_px: Option<f32>,
    pub falloff: Option<f32>,
    pub distance_m: Option<f32>,
    pub z_depth: Option<f32>,
    pub contributions: Vec<LightContributionKind2d>,
    pub status: LightSourceStatus2d,
    pub reason: String,
    pub position_px: Option<[f32; 2]>,
}

impl LightSource2dCommon {
    #[allow(clippy::too_many_arguments)]
    pub fn active(
        owner: impl Into<String>,
        component_kind: impl Into<String>,
        emitter_kind: LightEmitterKind2d,
        emitter_id: Option<String>,
        render_layer: Option<String>,
        color_rgba: Option<[f32; 4]>,
        intensity: Option<f32>,
        effective_intensity: Option<f32>,
        response: Option<f32>,
        camera_response: Option<CameraOpticalResponse2d>,
        bloom: Option<f32>,
        lens_influence: Option<f32>,
        radius_px: Option<f32>,
        falloff: Option<f32>,
        distance_m: Option<f32>,
        z_depth: Option<f32>,
        contributions: Vec<LightContributionKind2d>,
        reason: impl Into<String>,
        position_px: Option<[f32; 2]>,
    ) -> Self {
        Self {
            owner: owner.into(),
            component_kind: component_kind.into(),
            emitter_kind,
            emitter_id,
            render_layer,
            color_rgba,
            intensity,
            effective_intensity,
            response,
            camera_response,
            bloom,
            lens_influence,
            radius_px,
            falloff,
            distance_m,
            z_depth,
            contributions,
            status: LightSourceStatus2d::Active,
            reason: reason.into(),
            position_px,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn skipped(
        owner: impl Into<String>,
        component_kind: impl Into<String>,
        emitter_kind: LightEmitterKind2d,
        emitter_id: Option<String>,
        render_layer: Option<String>,
        color_rgba: Option<[f32; 4]>,
        intensity: Option<f32>,
        effective_intensity: Option<f32>,
        response: Option<f32>,
        camera_response: Option<CameraOpticalResponse2d>,
        bloom: Option<f32>,
        lens_influence: Option<f32>,
        radius_px: Option<f32>,
        falloff: Option<f32>,
        distance_m: Option<f32>,
        z_depth: Option<f32>,
        contributions: Vec<LightContributionKind2d>,
        reason: impl Into<String>,
        position_px: Option<[f32; 2]>,
    ) -> Self {
        Self {
            owner: owner.into(),
            component_kind: component_kind.into(),
            emitter_kind,
            emitter_id,
            render_layer,
            color_rgba,
            intensity,
            effective_intensity,
            response,
            camera_response,
            bloom,
            lens_influence,
            radius_px,
            falloff,
            distance_m,
            z_depth,
            contributions,
            status: LightSourceStatus2d::Skipped,
            reason: reason.into(),
            position_px,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LightContributionKind2d, LightEmitterKind2d};

    #[test]
    fn light_source_2d_kinds_report_stable_names() {
        assert_eq!(LightEmitterKind2d::Beacon.as_str(), "beacon");
        assert_eq!(
            LightEmitterKind2d::LightMapChannel.as_str(),
            "lightmap_channel"
        );
        assert_eq!(LightEmitterKind2d::LightGroup.as_str(), "light_group");
        assert_eq!(
            LightEmitterKind2d::EmissiveMaterial.as_str(),
            "emissive_material"
        );
        assert_eq!(LightEmitterKind2d::ParticleLight.as_str(), "particle_light");
        assert_eq!(LightContributionKind2d::CameraFxSource.as_str(), "camera_fx_source");
    }
}
