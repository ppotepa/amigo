use crate::ids::TargetId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetAccess {
    Read,
    Write,
    ReadWrite,
    Contribute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StandardTarget {
    SceneColor,
    SceneAlpha,
    SceneDepth,
    SceneNormal,
    SceneVelocity,
    SceneHighlight,
    SceneEmissive,
    SceneLighting,
    LightMap,
    CameraArtifactLayer,
    FinalComposite,
    DiagnosticsSnapshot,
}

impl StandardTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            StandardTarget::SceneColor => "SceneColor",
            StandardTarget::SceneAlpha => "SceneAlpha",
            StandardTarget::SceneDepth => "SceneDepth",
            StandardTarget::SceneNormal => "SceneNormal",
            StandardTarget::SceneVelocity => "SceneVelocity",
            StandardTarget::SceneHighlight => "SceneHighlight",
            StandardTarget::SceneEmissive => "SceneEmissive",
            StandardTarget::SceneLighting => "SceneLighting",
            StandardTarget::LightMap => "LightMap",
            StandardTarget::CameraArtifactLayer => "CameraArtifactLayer",
            StandardTarget::FinalComposite => "FinalComposite",
            StandardTarget::DiagnosticsSnapshot => "DiagnosticsSnapshot",
        }
    }

    pub fn id(self) -> TargetId {
        TargetId(self.as_str().to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetRef {
    pub id: TargetId,
    pub access: TargetAccess,
}

impl TargetRef {
    pub fn read(id: impl Into<String>) -> Self {
        Self {
            id: TargetId(id.into()),
            access: TargetAccess::Read,
        }
    }

    pub fn write(id: impl Into<String>) -> Self {
        Self {
            id: TargetId(id.into()),
            access: TargetAccess::Write,
        }
    }

    pub fn read_write(id: impl Into<String>) -> Self {
        Self {
            id: TargetId(id.into()),
            access: TargetAccess::ReadWrite,
        }
    }

    pub fn contribute(id: impl Into<String>) -> Self {
        Self {
            id: TargetId(id.into()),
            access: TargetAccess::Contribute,
        }
    }

    pub fn read_standard(target: StandardTarget) -> Self {
        Self {
            id: target.id(),
            access: TargetAccess::Read,
        }
    }

    pub fn write_standard(target: StandardTarget) -> Self {
        Self {
            id: target.id(),
            access: TargetAccess::Write,
        }
    }

    pub fn contribute_standard(target: StandardTarget) -> Self {
        Self {
            id: target.id(),
            access: TargetAccess::Contribute,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.id.0.trim().is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TargetManifest {
    pub reads: Vec<TargetId>,
    pub writes: Vec<TargetId>,
    pub contributes: Vec<TargetId>,
}

pub fn scene_color() -> TargetId {
    StandardTarget::SceneColor.id()
}

pub fn scene_alpha() -> TargetId {
    StandardTarget::SceneAlpha.id()
}

pub fn scene_depth() -> TargetId {
    StandardTarget::SceneDepth.id()
}

pub fn scene_normal() -> TargetId {
    StandardTarget::SceneNormal.id()
}

pub fn scene_velocity() -> TargetId {
    StandardTarget::SceneVelocity.id()
}

pub fn scene_highlight() -> TargetId {
    StandardTarget::SceneHighlight.id()
}

pub fn scene_emissive() -> TargetId {
    StandardTarget::SceneEmissive.id()
}

pub fn scene_lighting() -> TargetId {
    StandardTarget::SceneLighting.id()
}

pub fn light_map() -> TargetId {
    StandardTarget::LightMap.id()
}

pub fn camera_artifact_layer() -> TargetId {
    StandardTarget::CameraArtifactLayer.id()
}

pub fn final_composite() -> TargetId {
    StandardTarget::FinalComposite.id()
}

pub fn diagnostics_snapshot() -> TargetId {
    StandardTarget::DiagnosticsSnapshot.id()
}
