use std::collections::BTreeSet;

use amigo_scene::{
    ComponentAssetRefDescriptor, ComponentKind, ComponentRegistry, default_component_registry,
};
use amigo_scene::{SceneComponentDocument, SceneDocument};

pub fn scene_asset_refs(scene: &SceneDocument) -> BTreeSet<String> {
    let registry = default_component_registry();
    scene_asset_refs_with_registry(scene, &registry)
}

fn scene_asset_refs_with_registry(
    scene: &SceneDocument,
    registry: &ComponentRegistry,
) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();

    for entity in &scene.entities {
        for component in &entity.components {
            let kind = component.component_kind();
            let Some(descriptor) = registry.descriptor(kind) else {
                continue;
            };

            for asset_ref in descriptor.asset_refs {
                if let Some(value) = component_asset_ref_value(component, kind, asset_ref) {
                    refs.insert(value);
                }
            }
        }
    }

    refs
}

fn component_asset_ref_value(
    component: &SceneComponentDocument,
    kind: ComponentKind,
    asset_ref: &ComponentAssetRefDescriptor,
) -> Option<String> {
    match (kind, component, asset_ref.field_path) {
        (ComponentKind::Sprite2D, SceneComponentDocument::Sprite2d { texture, .. }, "texture") => {
            Some(texture.clone())
        }
        (ComponentKind::TileMap2D, SceneComponentDocument::TileMap2d { tileset, .. }, "tileset") => {
            Some(tileset.clone())
        }
        (ComponentKind::TileMap2D, SceneComponentDocument::TileMap2d { ruleset, .. }, "ruleset") => {
            ruleset.clone()
        }
        (ComponentKind::Text2D, SceneComponentDocument::Text2d { font, .. }, "font") => {
            Some(font.clone())
        }
        (
            ComponentKind::ScriptComponent,
            SceneComponentDocument::ScriptComponent { script, .. },
            "script",
        ) => Some(script.clone()),
        (ComponentKind::Mesh3D, SceneComponentDocument::Mesh3d { mesh }, "mesh") => {
            Some(mesh.clone())
        }
        (
            ComponentKind::Material3D,
            SceneComponentDocument::Material3d { source, .. },
            "source",
        ) => source.clone(),
        (
            ComponentKind::Material3D,
            SceneComponentDocument::Material3d { albedo, .. },
            "albedo",
        ) => albedo.clone(),
        (ComponentKind::Text3D, SceneComponentDocument::Text3d { font, .. }, "font") => {
            Some(font.clone())
        }
        _ => None,
    }
}
