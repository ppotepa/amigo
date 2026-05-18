use std::collections::BTreeMap;

use amigo_math::ColorRgba;
use amigo_render_api::{
    CameraCaptureInput2d, VisualSourceAvailability2d, VisualSourceKind2d, VisualSourceRef2d,
};

// WGPU-side runtime interpretation of CameraCaptureInput2d.
// Keep wgpu handles out of render-api. This module is allowed to know about debug/fallback behavior.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WgpuVisualSourceRuntimeKind2d {
    WorldColor,
    WorldDepth,
    ProducedLayerMask,
    ProducedSceneNormal,
    ProducedSceneWetness,
    ProducedSceneEmissive,
    ProducedSceneHighlight,
    ProducedSceneMotion,
    DerivedDebug,
    AssetTexture,
    MissingFallback,
}

impl WgpuVisualSourceRuntimeKind2d {
    pub fn is_real_target(self) -> bool {
        matches!(
            self,
            Self::ProducedLayerMask
                | Self::ProducedSceneNormal
                | Self::ProducedSceneWetness
                | Self::ProducedSceneEmissive
                | Self::ProducedSceneHighlight
                | Self::ProducedSceneMotion
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WgpuVisualSourceRuntime2d {
    pub source: VisualSourceRef2d,
    pub runtime_kind: WgpuVisualSourceRuntimeKind2d,
    pub fallback_color: ColorRgba,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct WgpuCameraVisualSources2d {
    pub sources: BTreeMap<VisualSourceKind2d, WgpuVisualSourceRuntime2d>,
}

impl WgpuCameraVisualSources2d {
    pub fn from_capture_input(input: &CameraCaptureInput2d) -> Self {
        let mut sources = BTreeMap::new();
        sources.insert(
            VisualSourceKind2d::SceneColor,
            WgpuVisualSourceRuntime2d {
                source: input.color.clone(),
                runtime_kind: WgpuVisualSourceRuntimeKind2d::WorldColor,
                fallback_color: fallback_color_for(VisualSourceKind2d::SceneColor),
            },
        );
        if let Some(source) = &input.depth {
            sources.insert(
                VisualSourceKind2d::SceneDepth,
                WgpuVisualSourceRuntime2d {
                    source: source.clone(),
                    runtime_kind: WgpuVisualSourceRuntimeKind2d::WorldDepth,
                    fallback_color: fallback_color_for(VisualSourceKind2d::SceneDepth),
                },
            );
        }
        if let Some(source) = &input.layer_mask {
            sources.insert(
                VisualSourceKind2d::LayerMask,
                WgpuVisualSourceRuntime2d {
                    source: source.clone(),
                    runtime_kind: WgpuVisualSourceRuntimeKind2d::ProducedLayerMask,
                    fallback_color: fallback_color_for(VisualSourceKind2d::LayerMask),
                },
            );
        }
        for kind in [
            VisualSourceKind2d::SceneNormal,
            VisualSourceKind2d::SceneWetness,
            VisualSourceKind2d::SceneEmissive,
            VisualSourceKind2d::SceneHighlight,
            VisualSourceKind2d::SceneMotion,
        ] {
            let Some(source) = input.source(kind).cloned() else {
                continue;
            };
            let runtime_kind = match source.availability {
                VisualSourceAvailability2d::Produced => match kind {
                    VisualSourceKind2d::SceneNormal => {
                        WgpuVisualSourceRuntimeKind2d::ProducedSceneNormal
                    }
                    VisualSourceKind2d::SceneWetness => {
                        WgpuVisualSourceRuntimeKind2d::ProducedSceneWetness
                    }
                    VisualSourceKind2d::SceneEmissive => {
                        WgpuVisualSourceRuntimeKind2d::ProducedSceneEmissive
                    }
                    VisualSourceKind2d::SceneHighlight => {
                        WgpuVisualSourceRuntimeKind2d::ProducedSceneHighlight
                    }
                    VisualSourceKind2d::SceneMotion => {
                        WgpuVisualSourceRuntimeKind2d::ProducedSceneMotion
                    }
                    _ => WgpuVisualSourceRuntimeKind2d::MissingFallback,
                },
                VisualSourceAvailability2d::Derived => WgpuVisualSourceRuntimeKind2d::DerivedDebug,
                VisualSourceAvailability2d::Asset => WgpuVisualSourceRuntimeKind2d::AssetTexture,
                VisualSourceAvailability2d::Fallback | VisualSourceAvailability2d::Missing => {
                    WgpuVisualSourceRuntimeKind2d::MissingFallback
                }
            };
            sources.insert(
                kind,
                WgpuVisualSourceRuntime2d {
                    source,
                    runtime_kind,
                    fallback_color: fallback_color_for(kind),
                },
            );
        }
        Self { sources }
    }

    pub fn get(&self, kind: VisualSourceKind2d) -> Option<&WgpuVisualSourceRuntime2d> {
        self.sources.get(&kind)
    }
}

pub(crate) fn fallback_color_for(kind: VisualSourceKind2d) -> ColorRgba {
    match kind {
        VisualSourceKind2d::SceneNormal => ColorRgba::new(0.5, 0.5, 1.0, 1.0),
        VisualSourceKind2d::SceneWetness => ColorRgba::new(0.0, 0.22, 0.28, 1.0),
        VisualSourceKind2d::SceneHighlight => ColorRgba::new(0.32, 0.24, 0.0, 1.0),
        VisualSourceKind2d::SceneEmissive => ColorRgba::new(0.3, 0.18, 0.0, 1.0),
        VisualSourceKind2d::SceneMotion => ColorRgba::new(0.12, 0.0, 0.2, 1.0),
        VisualSourceKind2d::LayerMask => ColorRgba::new(0.16, 0.16, 0.16, 1.0),
        VisualSourceKind2d::SceneColor
        | VisualSourceKind2d::SceneDepth
        | VisualSourceKind2d::Debug => ColorRgba::new(0.0, 0.0, 0.0, 1.0),
    }
}
