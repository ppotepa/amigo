use amigo_assets::AssetKey;
use amigo_camera::{CameraOpticalCandidate2d, CameraOpticalResponse2d};
use amigo_math::{Transform2, Vec2};

use crate::{LightSource2dCommon, RenderContributionSet};

#[derive(Debug, Clone, PartialEq)]
pub enum RenderContribution2d {
    LightSource2d(LightSource2dCommon),
    CameraOpticalCandidate2d(CameraOpticalCandidate2d),
    DepthMap2d(RenderDepthMap2d),
    DepthAuxMap2d(RenderDepthAuxMap2d),
    LightMap2d(RenderLightMap2dSource),
    LightGroup2d(RenderLightGroup2d),
    Layer2d(RenderLayerContribution2d),
    Target2d(RenderTargetContribution2d),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderDepthMapViewportFit2d {
    Fixed,
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderDepthMap2d {
    pub owner_entity: String,
    pub id: String,
    pub asset: AssetKey,
    pub size: Vec2,
    pub viewport_fit: RenderDepthMapViewportFit2d,
    pub white_is_near: bool,
    pub z_index: f32,
    pub transform: Transform2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderDepthAuxMap2dChannels {
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderDepthAuxMap2d {
    pub owner_entity: String,
    pub id: String,
    pub asset: AssetKey,
    pub surface_asset: Option<AssetKey>,
    pub size: Vec2,
    pub viewport_fit: RenderDepthMapViewportFit2d,
    pub channels: RenderDepthAuxMap2dChannels,
    pub z_index: f32,
    pub transform: Transform2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderLightMap2dSource {
    pub source_mod: String,
    pub owner_entity: String,
    pub source_id: String,
    pub source: RenderLightMap2dSourceRef,
    pub channels: Vec<RenderLightMap2dChannel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderLightMap2dChannel {
    pub id: String,
    pub layers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderLightMap2dSourceRef {
    pub kind: RenderLightMap2dSourceKind,
    pub entity_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderLightMap2dSourceKind {
    LayeredImage2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderLightGroup2d {
    pub source_mod: String,
    pub id: String,
    pub label: Option<String>,
    pub color_rgba: [f32; 4],
    pub intensity: f32,
    pub contributions: RenderContributionSet,
    pub camera_response: CameraOpticalResponse2d,
    pub sources: Vec<RenderLightGroupSource2d>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderLightGroupSource2d {
    pub kind: RenderLightGroupSourceKind2d,
    pub response: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderLightGroupSourceKind2d {
    LightMapChannel { source: String, channel: String },
    GlobalLight { id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderLayerContribution2d {
    pub id: String,
    pub z_depth: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTargetContribution2d {
    pub target_id: String,
    pub source_id: String,
}

impl RenderContribution2d {
    pub fn light_source_2d(source: LightSource2dCommon) -> Self {
        Self::LightSource2d(source)
    }

    pub fn camera_optical_candidate_2d(candidate: CameraOpticalCandidate2d) -> Self {
        Self::CameraOpticalCandidate2d(candidate)
    }

    pub fn depth_map_2d(depth_map: RenderDepthMap2d) -> Self {
        Self::DepthMap2d(depth_map)
    }

    pub fn depth_aux_map_2d(depth_aux_map: RenderDepthAuxMap2d) -> Self {
        Self::DepthAuxMap2d(depth_aux_map)
    }

    pub fn lightmap_2d(source: RenderLightMap2dSource) -> Self {
        Self::LightMap2d(source)
    }

    pub fn light_group_2d(group: RenderLightGroup2d) -> Self {
        Self::LightGroup2d(group)
    }

    pub fn layer_2d(id: impl Into<String>, z_depth: f32) -> Self {
        Self::Layer2d(RenderLayerContribution2d {
            id: id.into(),
            z_depth,
        })
    }

    pub fn target_2d(target_id: impl Into<String>, source_id: impl Into<String>) -> Self {
        Self::Target2d(RenderTargetContribution2d {
            target_id: target_id.into(),
            source_id: source_id.into(),
        })
    }

    pub fn as_light_source_2d(&self) -> Option<&LightSource2dCommon> {
        match self {
            Self::LightSource2d(source) => Some(source),
            _ => None,
        }
    }

    pub fn as_camera_optical_candidate_2d(&self) -> Option<&CameraOpticalCandidate2d> {
        match self {
            Self::CameraOpticalCandidate2d(candidate) => Some(candidate),
            _ => None,
        }
    }

    pub fn as_depth_map_2d(&self) -> Option<&RenderDepthMap2d> {
        match self {
            Self::DepthMap2d(depth_map) => Some(depth_map),
            _ => None,
        }
    }

    pub fn as_depth_aux_map_2d(&self) -> Option<&RenderDepthAuxMap2d> {
        match self {
            Self::DepthAuxMap2d(depth_aux_map) => Some(depth_aux_map),
            _ => None,
        }
    }

    pub fn as_lightmap_2d(&self) -> Option<&RenderLightMap2dSource> {
        match self {
            Self::LightMap2d(source) => Some(source),
            _ => None,
        }
    }

    pub fn as_light_group_2d(&self) -> Option<&RenderLightGroup2d> {
        match self {
            Self::LightGroup2d(group) => Some(group),
            _ => None,
        }
    }
}
