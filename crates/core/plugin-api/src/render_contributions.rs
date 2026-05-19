use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderContributionRoleId(String);

impl RenderContributionRoleId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RenderContributionRoleId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderContributionRoleId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for RenderContributionRoleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenderContributionSet {
    roles: BTreeMap<RenderContributionRoleId, bool>,
}

impl RenderContributionSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pairs<I, K>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, bool)>,
        K: Into<RenderContributionRoleId>,
    {
        let mut set = Self::new();
        for (role, enabled) in pairs {
            set.set(role, enabled);
        }
        set
    }

    pub fn set(&mut self, role: impl Into<RenderContributionRoleId>, enabled: bool) {
        self.roles.insert(role.into(), enabled);
    }

    pub fn get(&self, role: impl Into<RenderContributionRoleId>) -> Option<bool> {
        self.roles.get(&role.into()).copied()
    }

    pub fn enabled_or(&self, role: impl Into<RenderContributionRoleId>, default: bool) -> bool {
        self.get(role).unwrap_or(default)
    }

    pub fn merge_defaults<I, K>(&mut self, defaults: I)
    where
        I: IntoIterator<Item = (K, bool)>,
        K: Into<RenderContributionRoleId>,
    {
        for (role, enabled) in defaults {
            self.roles.entry(role.into()).or_insert(enabled);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&RenderContributionRoleId, bool)> {
        self.roles.iter().map(|(role, enabled)| (role, *enabled))
    }

    pub fn is_empty(&self) -> bool {
        self.roles.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderContributionStatus {
    Active,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderContributionDecision {
    pub owner: String,
    pub component: String,
    pub role: RenderContributionRoleId,
    pub status: RenderContributionStatus,
    pub reason: String,
}

impl RenderContributionDecision {
    pub fn active(
        owner: impl Into<String>,
        component: impl Into<String>,
        role: impl Into<RenderContributionRoleId>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            owner: owner.into(),
            component: component.into(),
            role: role.into(),
            status: RenderContributionStatus::Active,
            reason: reason.into(),
        }
    }

    pub fn skipped(
        owner: impl Into<String>,
        component: impl Into<String>,
        role: impl Into<RenderContributionRoleId>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            owner: owner.into(),
            component: component.into(),
            role: role.into(),
            status: RenderContributionStatus::Skipped,
            reason: reason.into(),
        }
    }
}

pub mod roles {
    pub const WORLD_COLOR: &str = "world.color";
    pub const WORLD_DEPTH: &str = "world.depth";
    pub const WORLD_NORMAL: &str = "world.normal";
    pub const WORLD_WETNESS: &str = "world.wetness";

    pub const OVERLAY_VISIBLE: &str = "overlay.visible";

    pub const LIGHTING_EMIT: &str = "lighting.emit";
    pub const RELIGHT_PLATE: &str = "relight.plate";
    pub const BLOOM_SOURCE: &str = "bloom.source";
    pub const CAMERA_FX_SOURCE: &str = "camera.fx_source";
    pub const MATERIAL_MASK: &str = "material.mask";
    pub const OPTICS_REFRACT: &str = "optics.refract";
    pub const TRANSMISSION_SOURCE: &str = "transmission.source";

    pub const POSTFX_HOST: &str = "postfx.host";
    pub const DEBUG_VISIBLE: &str = "debug.visible";
    pub const DEBUG_ONLY: &str = "debug.only";

    pub const CAMERA_PROJECTION: &str = "camera.projection";
    pub const CAMERA_EXPOSURE: &str = "camera.exposure";
    pub const CAMERA_SHUTTER: &str = "camera.shutter";
    pub const CAMERA_OPTICS: &str = "camera.optics";
    pub const CAMERA_FOCUS_BLUR: &str = "camera.focus_blur";
    pub const CAMERA_LENS_SURFACE: &str = "camera.lens_surface";
    pub const CAMERA_FILM: &str = "camera.film";
    pub const CAMERA_LOOK: &str = "camera.look";
    pub const CAMERA_SCAN_OUTPUT: &str = "camera.scan_output";
}
