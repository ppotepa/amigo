use crate::document::SceneComponentDocument;
use crate::metadata_traits::MetadataTraitKind;

/// Static semantic capabilities for scene components.
///
/// Runtime can still hydrate components manually, but editor/diagnostics use
/// this table as the shared semantic contract.
pub fn component_2d_traits(component: &SceneComponentDocument) -> Vec<MetadataTraitKind> {
    use MetadataTraitKind::*;

    match component {
        SceneComponentDocument::Sprite2d { .. } => vec![
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
        SceneComponentDocument::LayeredImage2d { .. } => vec![
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
        SceneComponentDocument::TileMap2d { .. } => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            HasAssetRefs,
            PostFxHost2D,
            Selectable,
        ],
        SceneComponentDocument::Text2d { .. } => vec![
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
        SceneComponentDocument::VectorShape2d { .. } => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            HasBounds2D,
            PostFxHost2D,
            Selectable,
        ],
        SceneComponentDocument::ParticleEmitter2d { .. } => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            Simulatable,
            PostFxHost2D,
            RuntimeControllable,
            Patchable,
        ],
        SceneComponentDocument::BeaconLight2d { .. } => vec![
            Component2D,
            UsesTransform2D,
            Renderable2D,
            RenderLayered2D,
            LightReceiver2D,
            PostFxHost2D,
            Selectable,
        ],
        SceneComponentDocument::Camera2d { .. } | SceneComponentDocument::CameraFollow2d { .. } => {
            vec![
                Component2D,
                UsesTransform2D,
                Camera,
                RuntimeControllable,
                Patchable,
            ]
        }
        SceneComponentDocument::Velocity2d { .. }
        | SceneComponentDocument::FreeflightMotion2d { .. }
        | SceneComponentDocument::MotionController2d { .. } => {
            vec![
                Component2D,
                UsesTransform2D,
                Motion2D,
                RuntimeControllable,
                Patchable,
            ]
        }
        SceneComponentDocument::KinematicBody2d { .. }
        | SceneComponentDocument::AabbCollider2d { .. }
        | SceneComponentDocument::StaticCollider2d { .. }
        | SceneComponentDocument::CircleCollider2d { .. }
        | SceneComponentDocument::Trigger2d { .. } => {
            vec![
                Component2D,
                UsesTransform2D,
                Collidable2D,
                RuntimeControllable,
                Patchable,
            ]
        }
        SceneComponentDocument::ScriptComponent { .. } => {
            vec![Component2D, Scriptable, RuntimeControllable, Patchable]
        }
        _ => vec![Component2D],
    }
}

pub fn component_is_renderable_2d(component: &SceneComponentDocument) -> bool {
    component_2d_traits(component).contains(&MetadataTraitKind::Renderable2D)
}

pub fn builtin_renderable_2d_component_kinds() -> &'static [&'static str] {
    &[
        "TileMap2D",
        "LayeredImage2D",
        "VectorShape2D",
        "BeaconLight2D",
        "Sprite2D",
        "Text2D",
        "ParticleEmitter2D",
    ]
}

pub fn component_uses_transform_2d(component: &SceneComponentDocument) -> bool {
    component_2d_traits(component).contains(&MetadataTraitKind::UsesTransform2D)
}
