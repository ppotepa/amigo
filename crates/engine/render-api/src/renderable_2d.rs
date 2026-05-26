use crate::{RenderPrimitive2d, RenderPrimitive2dKind};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderSourceId(pub String);

impl RenderSourceId {
    pub fn for_component(owner_entity: &str, component_kind: &str) -> Self {
        Self(format!("component:{owner_entity}/{component_kind}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderObjectId(pub String);

impl RenderObjectId {
    pub fn for_scene_object(scene_object_id: &str) -> Self {
        Self(format!("scene-object:{scene_object_id}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches_subtree(&self, root: &RenderObjectId) -> bool {
        self.0 == root.0 || self.0.starts_with(root.0.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSpace2d {
    World,
    ScreenOverlay,
    DebugOverlay,
}

impl RenderSpace2d {
    pub fn uses_camera_pipeline(self) -> bool {
        matches!(self, Self::World)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Renderable2dKind {
    TileMap,
    LayeredImage,
    Vector,
    Beacon,
    Sprite,
    Text,
    Particle,
}

impl Renderable2dKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TileMap => "TileMap",
            Self::LayeredImage => "LayeredImage",
            Self::Vector => "Vector",
            Self::Beacon => "Beacon",
            Self::Sprite => "Sprite",
            Self::Text => "Text",
            Self::Particle => "Particle",
        }
    }

    pub fn sort_priority(self) -> u8 {
        match self {
            Self::TileMap => 0,
            Self::LayeredImage => 1,
            Self::Vector => 2,
            Self::Beacon => 3,
            Self::Sprite => 4,
            Self::Text => 5,
            Self::Particle => 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Renderable2dCommon {
    pub source_id: RenderSourceId,
    pub object_id: RenderObjectId,
    pub owner_entity: String,
    pub component_kind: String,
    pub render_space: RenderSpace2d,
    pub render_layer: String,
    pub z_index: f32,
    pub kind: Renderable2dKind,
    pub overlay_visible: bool,
}

impl Renderable2dCommon {
    pub fn world(
        owner_entity: impl Into<String>,
        component_kind: impl Into<String>,
        render_layer: impl Into<String>,
        z_index: f32,
        kind: Renderable2dKind,
    ) -> Self {
        let owner_entity = owner_entity.into();
        let component_kind = component_kind.into();
        Self {
            source_id: RenderSourceId::for_component(&owner_entity, &component_kind),
            object_id: RenderObjectId::for_scene_object(&owner_entity),
            owner_entity,
            component_kind,
            render_space: RenderSpace2d::World,
            render_layer: render_layer.into(),
            z_index,
            kind,
            overlay_visible: false,
        }
    }

    pub fn uses_camera_pipeline(&self) -> bool {
        self.render_space.uses_camera_pipeline()
    }

    pub fn with_overlay_visible(mut self, overlay_visible: bool) -> Self {
        self.overlay_visible = overlay_visible;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Renderable2dItem {
    pub common: Renderable2dCommon,
    pub primitive: RenderPrimitive2d,
}

impl Renderable2dItem {
    pub fn new(common: Renderable2dCommon, primitive: RenderPrimitive2d) -> Self {
        Self { common, primitive }
    }

    pub fn render_layer(&self) -> &str {
        &self.common.render_layer
    }

    pub fn z_index(&self) -> f32 {
        self.common.z_index
    }

    pub fn owner_entity(&self) -> &str {
        &self.common.owner_entity
    }

    pub fn source_id(&self) -> &RenderSourceId {
        &self.common.source_id
    }

    pub fn object_id(&self) -> &RenderObjectId {
        &self.common.object_id
    }

    pub fn component_kind(&self) -> &str {
        &self.common.component_kind
    }

    pub fn render_space(&self) -> RenderSpace2d {
        self.common.render_space
    }

    pub fn primitive_kind(&self) -> RenderPrimitive2dKind {
        self.primitive.kind()
    }

    pub fn uses_camera_pipeline(&self) -> bool {
        self.common.uses_camera_pipeline()
    }

    pub fn overlay_visible(&self) -> bool {
        self.common.overlay_visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_space_uses_camera_pipeline() {
        assert!(RenderSpace2d::World.uses_camera_pipeline());
        assert!(!RenderSpace2d::ScreenOverlay.uses_camera_pipeline());
        assert!(!RenderSpace2d::DebugOverlay.uses_camera_pipeline());
    }

    #[test]
    fn renderable_kind_has_stable_sort_priority() {
        assert!(
            Renderable2dKind::Text.sort_priority() < Renderable2dKind::Particle.sort_priority()
        );
        assert!(
            Renderable2dKind::TileMap.sort_priority() < Renderable2dKind::Sprite.sort_priority()
        );
    }

    #[test]
    fn common_uses_camera_pipeline_from_space() {
        let common = Renderable2dCommon {
            source_id: RenderSourceId::for_component("title", "component"),
            object_id: RenderObjectId::for_scene_object("title"),
            owner_entity: "title".to_owned(),
            component_kind: "component".to_owned(),
            render_space: RenderSpace2d::World,
            render_layer: "title.depth2d".to_owned(),
            z_index: 0.0,
            kind: Renderable2dKind::Text,
            overlay_visible: false,
        };

        assert!(common.uses_camera_pipeline());
    }
}
