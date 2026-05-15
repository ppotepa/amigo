use super::SceneGraphNodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneReferenceTargetKind {
    SceneObject,
    DrawLayer,
    Asset,
    Curve2d,
    PostFxHost,
    ImagePart,
    Component,
    LightGroup,
    Camera,
    Font,
    Script,
    Mesh,
    Material,
    UiDocument,
    Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneReferenceKind {
    RendersIntoDrawLayer,
    UsesAsset,
    UsesTileset,
    UsesRuleset,
    UsesFont,
    UsesScript,
    UsesMesh,
    UsesMaterial,
    FollowsSceneObject,
    AttachedToSceneObject,
    UsesCameraObject,
    UsesTileMapObject,
    UsesLightGroup,
    UsesImagePart,
    UsesPostFxHost,
    UsesCurve2d,
    LightRouteReceiver,
    LightMapSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneReferenceEdge {
    pub from: SceneGraphNodeId,
    pub port: String,
    pub kind: SceneReferenceKind,
    pub target_kind: SceneReferenceTargetKind,
    pub raw_target: String,
    pub required: bool,
    pub resolved: Option<SceneGraphNodeId>,
}

impl SceneReferenceEdge {
    pub fn new(
        from: SceneGraphNodeId,
        port: impl Into<String>,
        kind: SceneReferenceKind,
        target_kind: SceneReferenceTargetKind,
        raw_target: impl Into<String>,
        required: bool,
        resolved: Option<SceneGraphNodeId>,
    ) -> Self {
        Self {
            from,
            port: port.into(),
            kind,
            target_kind,
            raw_target: raw_target.into(),
            required,
            resolved,
        }
    }
}
