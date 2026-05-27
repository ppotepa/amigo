use crate::document::{SceneComponentDocument, SceneComponentSemanticClass};
use crate::metadata_traits::MetadataTraitKind;

/// Static semantic capabilities for scene components.
///
/// Runtime can still hydrate components manually, but editor/diagnostics use
/// this table as the shared semantic contract.
pub fn component_2d_traits(component: &SceneComponentDocument) -> Vec<MetadataTraitKind> {
    use MetadataTraitKind::*;

    match component.semantic_class() {
        SceneComponentSemanticClass::Sprite2d => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            HasAssetRefs,
            PostFxHost2D,
            Selectable,
            RuntimeControllable,
            Patchable,
        ],
        SceneComponentSemanticClass::LayeredImage2d => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            HasAssetRefs,
            PostFxHost2D,
            Selectable,
            RuntimeControllable,
            Patchable,
        ],
        SceneComponentSemanticClass::TileMap2d => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            HasAssetRefs,
            PostFxHost2D,
            Selectable,
        ],
        SceneComponentSemanticClass::Text2d => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            PostFxHost2D,
            Selectable,
            RuntimeControllable,
            Patchable,
        ],
        SceneComponentSemanticClass::VectorShape2d => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            PostFxHost2D,
            Selectable,
        ],
        SceneComponentSemanticClass::ParticleEmitter2d => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            Simulatable,
            PostFxHost2D,
            RuntimeControllable,
            Patchable,
        ],
        SceneComponentSemanticClass::BeaconLight2d => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            LightReceiver2D,
            PostFxHost2D,
            Selectable,
        ],
        SceneComponentSemanticClass::Camera2d => vec![
            Component2D,
            UsesTransform2D,
            Camera,
            RuntimeControllable,
            Patchable,
        ],
        SceneComponentSemanticClass::Motion2d => vec![
            Component2D,
            UsesTransform2D,
            Motion2D,
            RuntimeControllable,
            Patchable,
        ],
        SceneComponentSemanticClass::Physics2d => vec![
            Component2D,
            UsesTransform2D,
            Collidable2D,
            RuntimeControllable,
            Patchable,
        ],
        SceneComponentSemanticClass::Physics3d => {
            vec![UsesTransform3D, RuntimeControllable, Patchable]
        }
        SceneComponentSemanticClass::Script => {
            vec![Component2D, Scriptable, RuntimeControllable, Patchable]
        }
        SceneComponentSemanticClass::Plugin | SceneComponentSemanticClass::Generic2d => {
            vec![Component2D]
        }
    }
}

pub fn component_is_renderable_2d(component: &SceneComponentDocument) -> bool {
    component_2d_traits(component).contains(&MetadataTraitKind::Renderable2D)
}

pub fn component_uses_transform_2d(component: &SceneComponentDocument) -> bool {
    component_2d_traits(component).contains(&MetadataTraitKind::UsesTransform2D)
}
