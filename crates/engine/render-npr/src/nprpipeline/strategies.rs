use crate::{
    FeatureSegment, NprCamera, NprFillTriangle, NprLogicalMark, NprPipelineContext,
    NprMediumDefinition, TessellatedStroke, TopologyEdge,
};
use glam::Vec2;

pub trait NprSurfaceStrategy: Send + Sync {
    fn prepare(&self, context: &mut NprPipelineContext<'_>);
}

pub trait NprFeatureStrategy: Send + Sync {
    fn detect(
        &self,
        context: &NprPipelineContext<'_>,
        topology: &[TopologyEdge],
    ) -> Vec<FeatureSegment>;
}

/// Separates feature detection from drawing intent. A detector may find every
/// crease, while a salience strategy can intentionally leave most of them out.
pub trait NprSalienceStrategy: Send + Sync {
    fn select(
        &self,
        context: &NprPipelineContext<'_>,
        features: &[FeatureSegment],
    ) -> Vec<FeatureSegment>;
}

pub trait NprProjectionStrategy: Send + Sync {
    fn project_point(
        &self,
        camera: NprCamera,
        point: glam::Vec3,
        viewport: Vec2,
    ) -> Option<crate::ProjectedPoint>;

    fn project_segment(
        &self,
        camera: NprCamera,
        a: glam::Vec3,
        b: glam::Vec3,
        viewport: Vec2,
    ) -> Option<(crate::ProjectedPoint, crate::ProjectedPoint)>;
}

pub trait NprValueStrategy: Send + Sync {
    fn plan_fills(
        &self,
        context: &NprPipelineContext<'_>,
        projection: &dyn NprProjectionStrategy,
    ) -> Vec<NprFillTriangle>;
}

pub trait NprMarkStrategy: Send + Sync {
    fn plan_marks(
        &self,
        context: &NprPipelineContext<'_>,
        projection: &dyn NprProjectionStrategy,
    ) -> Vec<NprLogicalMark>;
}

/// Adds value-driven surface marks after contour marks have been planned.
/// Keeping this separate lets ink, pencil and charcoal use distinct hatching
/// without changing feature extraction or gesture realization.
pub trait NprHatchingStrategy: Send + Sync {
    fn plan_hatching(
        &self,
        context: &NprPipelineContext<'_>,
        projection: &dyn NprProjectionStrategy,
    ) -> Vec<NprLogicalMark>;
}

/// Joins projected candidates into the paths a draughtsperson would execute
/// as one gesture. It runs after hatching so a profile can chain contours
/// while deliberately leaving short hatch marks independent.
pub trait NprStrokeChainStrategy: Send + Sync {
    fn chain(
        &self,
        context: &NprPipelineContext<'_>,
        marks: Vec<NprLogicalMark>,
    ) -> Vec<NprLogicalMark>;
}

pub trait NprGestureStrategy: Send + Sync {
    fn realize(
        &self,
        context: &NprPipelineContext<'_>,
        marks: &[NprLogicalMark],
    ) -> Vec<TessellatedStroke>;
}

pub trait NprPaperStrategy: Send + Sync {
    fn paper(&self, context: &NprPipelineContext<'_>) -> crate::NprPaperDefinition;
}

/// Chooses how the backend should deposit planned marks. No WGPU type leaks
/// here: another backend can realize the same neutral medium identifier.
pub trait NprMediumStrategy: Send + Sync {
    fn medium(&self, context: &NprPipelineContext<'_>) -> NprMediumDefinition;
}
