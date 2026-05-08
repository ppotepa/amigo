use amigo_scene::{
    AssetDomain, BoundsPolicy, ComponentAssetRefDescriptor, ComponentKind, ComponentTypeDescriptor,
    EditorControlKind, EditorPatchOpKind, EditorPropertyDescriptor, default_component_registry,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorMetadataCatalogDto {
    pub target_kinds: Vec<EditorTargetKindDescriptorDto>,
    pub components: Vec<EditorComponentDescriptorDto>,
    pub asset_kinds: Vec<EditorAssetKindDescriptorDto>,
    pub document_kinds: Vec<EditorDocumentKindDescriptorDto>,
    pub controls: Vec<EditorControlDescriptorDto>,
    pub patch_ops: Vec<EditorPatchOpDescriptorDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorTargetKindDescriptorDto {
    pub kind: String,
    pub label: String,
    pub source_surfaces: Vec<String>,
    pub allowed_intents: Vec<String>,
    pub capabilities: Vec<String>,
    pub primary_context_role: String,
    pub related_context_role: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorComponentDescriptorDto {
    pub kind: String,
    pub type_name: String,
    pub label: String,
    pub domains: Vec<String>,
    pub capabilities: Vec<String>,
    pub asset_refs: Vec<EditorAssetRefDescriptorDto>,
    pub properties: Vec<EditorPropertyDescriptorDto>,
    pub transform_policy: String,
    pub bounds_policy: EditorBoundsPolicyDto,
    pub editor_controls: Vec<EditorControlRefDto>,
    pub patch_ops: Vec<EditorPatchOpRefDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorAssetKindDescriptorDto {
    pub kind: String,
    pub label: String,
    pub domain: String,
    pub preview_kind: String,
    pub document_policy: String,
    pub usages_supported: bool,
    pub can_create: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorDocumentKindDescriptorDto {
    pub kind: String,
    pub label: String,
    pub extensions: Vec<String>,
    pub target_kind: String,
    pub patch_sink: String,
    pub supports_reload: bool,
    pub supports_validation: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorControlDescriptorDto {
    pub kind: String,
    pub label: String,
    pub target_scope: String,
    pub handles: Vec<String>,
    pub default_patch_op: Option<String>,
    pub requires_bounds: bool,
    pub viewport_visible: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorPatchOpDescriptorDto {
    pub kind: String,
    pub label: String,
    pub target_scope: String,
    pub value_kind: String,
    pub persistence: String,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorControlRefDto {
    pub kind: String,
    pub target_scope: String,
    pub handles: Vec<String>,
    pub patch_op: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorPatchOpRefDto {
    pub kind: String,
    pub target_scope: String,
    pub persistence: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorBoundsPolicyDto {
    pub kind: String,
    pub field: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorAssetRefDescriptorDto {
    pub field_path: String,
    pub domain: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorPropertyDescriptorDto {
    pub path: String,
    pub label: String,
    pub value_kind: String,
    pub access: String,
    pub editor: String,
    pub asset_domain: Option<String>,
    pub patch_op: Option<String>,
}

pub fn editor_metadata_catalog_dto() -> EditorMetadataCatalogDto {
    let registry = default_component_registry();
    let mut components = ComponentKind::all()
        .iter()
        .filter_map(|kind| registry.descriptor(*kind))
        .map(component_descriptor_dto)
        .collect::<Vec<_>>();
    components.sort_by(|a, b| a.type_name.cmp(&b.type_name));

    EditorMetadataCatalogDto {
        target_kinds: target_kind_descriptors(),
        components,
        asset_kinds: asset_kind_descriptors(),
        document_kinds: document_kind_descriptors(),
        controls: control_descriptors(),
        patch_ops: patch_op_descriptors(),
    }
}

fn component_descriptor_dto(descriptor: &ComponentTypeDescriptor) -> EditorComponentDescriptorDto {
    EditorComponentDescriptorDto {
        kind: format_debug(&descriptor.kind),
        type_name: descriptor.type_name.to_string(),
        label: descriptor.label.to_string(),
        domains: descriptor.domains.iter().map(format_debug).collect(),
        capabilities: descriptor.capabilities.iter().map(format_debug).collect(),
        asset_refs: descriptor.asset_refs.iter().map(asset_ref_dto).collect(),
        properties: descriptor.properties.iter().map(property_dto).collect(),
        transform_policy: format_debug(&descriptor.transform_policy),
        bounds_policy: bounds_policy_dto(&descriptor.bounds_policy),
        editor_controls: descriptor.editor_controls.iter().map(control_ref_dto).collect(),
        patch_ops: descriptor.patch_ops.iter().map(patch_op_ref_dto).collect(),
    }
}

fn target_kind_descriptors() -> Vec<EditorTargetKindDescriptorDto> {
    vec![
        target_kind(
            "mod",
            "Mod",
            &["Launcher", "ProjectTree", "Workspace"],
            &["select", "open", "inspect", "reveal"],
            &["openable", "inspectable", "hasDiagnostics"],
            "project-summary",
            "project-related",
        ),
        target_kind(
            "projectNode",
            "Project Node",
            &["ProjectTree"],
            &["select", "open", "inspect", "reveal", "contextMenu"],
            &["navigable", "inspectable"],
            "project-node-summary",
            "project-node-related",
        ),
        target_kind(
            "projectFile",
            "Project File",
            &["ProjectTree", "Files", "Search", "Diagnostics"],
            &["select", "open", "inspect", "reveal", "contextMenu"],
            &["openable", "sourcePreview", "hasDiagnostics"],
            "file-summary",
            "file-related",
        ),
        target_kind(
            "script",
            "Script File",
            &["ProjectTree", "SceneTree", "Files", "Diagnostics"],
            &["select", "open", "inspect", "reveal", "contextMenu"],
            &["openable", "sourcePreview", "scriptable", "hasDiagnostics"],
            "script-summary",
            "script-related",
        ),
        target_kind(
            "asset",
            "Asset Definition",
            &["AssetTree", "ProjectTree", "SceneUsage", "Inspector"],
            &["select", "open", "inspect", "reveal", "contextMenu"],
            &["openable", "previewable", "hasUsages", "hasDiagnostics"],
            "asset-summary",
            "asset-related",
        ),
        target_kind(
            "scene",
            "Scene Document",
            &["ProjectTree", "SceneTree", "Launcher"],
            &["select", "open", "inspect", "reveal", "contextMenu"],
            &["openable", "previewable", "hasEntities", "hasDiagnostics"],
            "scene-summary",
            "scene-related",
        ),
        target_kind(
            "sceneEntity",
            "Scene Entity",
            &["SceneTree", "Viewport", "Search", "Diagnostics"],
            &["select", "open", "inspect", "reveal", "contextMenu"],
            &["selectable", "focusable", "hasComponents", "hasDiagnostics"],
            "entity-summary",
            "entity-related",
        ),
        target_kind(
            "uiDocument",
            "UI Document",
            &["SceneTree", "UiTree", "Workspace"],
            &["select", "open", "inspect", "reveal", "contextMenu"],
            &["openable", "previewable", "hasUiNodes", "hasDiagnostics"],
            "ui-document-summary",
            "ui-document-related",
        ),
        target_kind(
            "uiNode",
            "UI Node",
            &["UiTree", "UiPreview", "Search", "Diagnostics"],
            &["select", "open", "inspect", "reveal", "contextMenu"],
            &["selectable", "inspectable", "hasProperties", "hasDiagnostics"],
            "ui-node-summary",
            "ui-node-related",
        ),
        target_kind(
            "diagnostic",
            "Diagnostic",
            &["Diagnostics", "Problems", "Search"],
            &["select", "open", "inspect", "reveal"],
            &["revealable", "sourcePreview"],
            "diagnostic-summary",
            "diagnostic-related",
        ),
        target_kind(
            "capability",
            "Capability",
            &["ProjectTree", "Capabilities"],
            &["select", "open", "inspect"],
            &["inspectable", "projectMetadata"],
            "capability-summary",
            "capability-related",
        ),
        target_kind(
            "dependency",
            "Dependency",
            &["ProjectTree", "Dependencies"],
            &["select", "open", "inspect"],
            &["inspectable", "projectMetadata"],
            "dependency-summary",
            "dependency-related",
        ),
    ]
}

fn asset_kind_descriptors() -> Vec<EditorAssetKindDescriptorDto> {
    vec![
        asset_kind(AssetDomain::Image, "Image", "image", "asset-manifest", true, true),
        asset_kind(AssetDomain::Sprite, "Sprite", "image", "asset-manifest", true, true),
        asset_kind(
            AssetDomain::Spritesheet,
            "Spritesheet",
            "image-grid",
            "asset-manifest",
            true,
            true,
        ),
        asset_kind(AssetDomain::TileSet, "Tile Set", "tiles", "asset-manifest", true, true),
        asset_kind(
            AssetDomain::TileRuleSet,
            "Tile Rule Set",
            "rules",
            "asset-manifest",
            true,
            true,
        ),
        asset_kind(AssetDomain::TileMap, "Tile Map", "tilemap", "scene-or-asset", true, true),
        asset_kind(AssetDomain::Audio, "Audio", "waveform", "asset-manifest", true, true),
        asset_kind(AssetDomain::Font, "Font", "font", "asset-manifest", true, true),
        asset_kind(AssetDomain::Scene, "Scene", "scene", "scene-yaml", true, true),
        asset_kind(AssetDomain::Prefab, "Prefab", "prefab", "prefab-yaml", true, true),
        asset_kind(AssetDomain::Script, "Script", "source", "script-file", true, true),
        asset_kind(AssetDomain::Material, "Material", "material", "asset-manifest", true, true),
        asset_kind(AssetDomain::Mesh, "Mesh", "mesh", "asset-manifest", true, true),
        asset_kind(
            AssetDomain::ParticlePreset,
            "Particle Preset",
            "particles",
            "asset-manifest",
            true,
            true,
        ),
        asset_kind(
            AssetDomain::CursorPack,
            "Cursor Pack",
            "cursor-pack",
            "asset-manifest",
            true,
            true,
        ),
        asset_kind(AssetDomain::UiTheme, "UI Theme", "theme", "ui-theme-yaml", true, true),
        asset_kind(AssetDomain::Raw, "Raw File", "raw", "file", false, false),
    ]
}

fn document_kind_descriptors() -> Vec<EditorDocumentKindDescriptorDto> {
    vec![
        document_kind(
            "sceneYaml",
            "Scene YAML",
            &[".scene.yaml", ".scene.yml"],
            "scene",
            "scene-yaml",
            true,
            true,
        ),
        document_kind(
            "assetManifestYaml",
            "Asset Manifest YAML",
            &[".assets.yaml", ".assets.yml", "assets.yaml", "assets.yml"],
            "asset",
            "asset-manifest-yaml",
            true,
            true,
        ),
        document_kind(
            "prefabYaml",
            "Prefab YAML",
            &[".prefab.yaml", ".prefab.yml"],
            "asset",
            "prefab-yaml",
            true,
            true,
        ),
        document_kind(
            "uiDocumentYaml",
            "UI Document YAML",
            &[".ui.yaml", ".ui.yml"],
            "uiDocument",
            "ui-document-yaml",
            true,
            true,
        ),
        document_kind("scriptRhai", "Rhai Script", &[".rhai"], "script", "script-file", true, true),
        document_kind("projectFile", "Project File", &["*"], "projectFile", "file", true, false),
    ]
}

fn control_descriptors() -> Vec<EditorControlDescriptorDto> {
    vec![
        control_descriptor(
            EditorControlKind::Transform2D,
            "Transform 2D",
            "scene.entity.transform2",
            &[
                "move.xy",
                "move.x",
                "move.y",
                "rotate.z",
                "scale.uniform",
                "scale.x",
                "scale.y",
            ],
            Some(EditorPatchOpKind::SetTransform2),
            false,
            true,
        ),
        control_descriptor(
            EditorControlKind::Transform3D,
            "Transform 3D",
            "scene.entity.transform3",
            &["move.xyz", "rotate.xyz", "scale.xyz"],
            Some(EditorPatchOpKind::SetTransform3),
            false,
            true,
        ),
        control_descriptor(
            EditorControlKind::Rect2D,
            "Rect 2D",
            "component.bounds2",
            &["resize.n", "resize.e", "resize.s", "resize.w", "resize.corner", "move.center"],
            None,
            true,
            true,
        ),
        control_descriptor(
            EditorControlKind::TextBounds2D,
            "Text Bounds 2D",
            "component.text.bounds",
            &["resize.n", "resize.e", "resize.s", "resize.w", "resize.corner"],
            Some(EditorPatchOpKind::SetTextBounds),
            true,
            true,
        ),
        control_descriptor(
            EditorControlKind::VectorVertex2D,
            "Vector Vertex 2D",
            "component.vector.points",
            &["vertex.move", "vertex.insert", "vertex.delete"],
            Some(EditorPatchOpKind::SetVectorPoints),
            true,
            true,
        ),
        control_descriptor(
            EditorControlKind::TileMapBrush2D,
            "Tile Map Brush 2D",
            "component.tilemap.cells",
            &["paint.cell", "erase.cell", "pick.cell", "resize.tilemap"],
            Some(EditorPatchOpKind::SetTileCell),
            true,
            true,
        ),
        control_descriptor(
            EditorControlKind::Collider2D,
            "Collider 2D",
            "component.collider.shape",
            &["resize.shape", "move.center"],
            Some(EditorPatchOpKind::SetColliderShape),
            true,
            true,
        ),
        control_descriptor(
            EditorControlKind::Trigger2D,
            "Trigger 2D",
            "component.trigger.shape",
            &["resize.shape", "move.center"],
            Some(EditorPatchOpKind::SetColliderShape),
            true,
            true,
        ),
        control_descriptor(
            EditorControlKind::Camera2D,
            "Camera 2D",
            "component.camera2d",
            &["viewport.rect", "zoom.handle"],
            Some(EditorPatchOpKind::SetCamera2D),
            true,
            true,
        ),
        control_descriptor(
            EditorControlKind::AudioRadius2D,
            "Audio Radius 2D",
            "component.audio.radius",
            &["radius.handle"],
            None,
            true,
            true,
        ),
        control_descriptor(
            EditorControlKind::InspectorOnly,
            "Inspector Only",
            "inspector",
            &[],
            None,
            false,
            false,
        ),
    ]
}

fn patch_op_descriptors() -> Vec<EditorPatchOpDescriptorDto> {
    vec![
        patch_op(EditorPatchOpKind::SetTransform2, "Set Transform 2D", "scene.entity", "Transform2", "scene-yaml", "medium"),
        patch_op(EditorPatchOpKind::SetTransform3, "Set Transform 3D", "scene.entity", "Transform3", "scene-yaml", "medium"),
        patch_op(EditorPatchOpKind::SetTextContent, "Set Text Content", "component.text", "String", "scene-yaml", "low"),
        patch_op(EditorPatchOpKind::SetTextBounds, "Set Text Bounds", "component.text", "Vec2", "scene-yaml", "medium"),
        patch_op(EditorPatchOpKind::SetVectorPoints, "Set Vector Points", "component.vector", "Vec2[]", "scene-yaml", "medium"),
        patch_op(EditorPatchOpKind::SetTileCell, "Set Tile Cell", "component.tilemap", "TileCell", "scene-yaml", "medium"),
        patch_op(EditorPatchOpKind::ResizeTileMap, "Resize Tile Map", "component.tilemap", "TileMapSize", "scene-yaml", "high"),
        patch_op(EditorPatchOpKind::SetColliderShape, "Set Collider Shape", "component.collider", "ColliderShape", "scene-yaml", "medium"),
        patch_op(EditorPatchOpKind::SetCamera2D, "Set Camera 2D", "component.camera2d", "Camera2D", "scene-yaml", "medium"),
        patch_op(EditorPatchOpKind::SetPrefabOverride, "Set Prefab Override", "prefab.instance", "PrefabOverride", "scene-or-prefab-yaml", "high"),
    ]
}

fn target_kind(
    kind: &str,
    label: &str,
    source_surfaces: &[&str],
    allowed_intents: &[&str],
    capabilities: &[&str],
    primary_context_role: &str,
    related_context_role: &str,
) -> EditorTargetKindDescriptorDto {
    EditorTargetKindDescriptorDto {
        kind: kind.to_string(),
        label: label.to_string(),
        source_surfaces: strings(source_surfaces),
        allowed_intents: strings(allowed_intents),
        capabilities: strings(capabilities),
        primary_context_role: primary_context_role.to_string(),
        related_context_role: related_context_role.to_string(),
    }
}

fn asset_kind(
    domain: AssetDomain,
    label: &str,
    preview_kind: &str,
    document_policy: &str,
    usages_supported: bool,
    can_create: bool,
) -> EditorAssetKindDescriptorDto {
    EditorAssetKindDescriptorDto {
        kind: format_debug(&domain),
        label: label.to_string(),
        domain: format_debug(&domain),
        preview_kind: preview_kind.to_string(),
        document_policy: document_policy.to_string(),
        usages_supported,
        can_create,
    }
}

fn document_kind(
    kind: &str,
    label: &str,
    extensions: &[&str],
    target_kind: &str,
    patch_sink: &str,
    supports_reload: bool,
    supports_validation: bool,
) -> EditorDocumentKindDescriptorDto {
    EditorDocumentKindDescriptorDto {
        kind: kind.to_string(),
        label: label.to_string(),
        extensions: strings(extensions),
        target_kind: target_kind.to_string(),
        patch_sink: patch_sink.to_string(),
        supports_reload,
        supports_validation,
    }
}

fn control_descriptor(
    kind: EditorControlKind,
    label: &str,
    target_scope: &str,
    handles: &[&str],
    default_patch_op: Option<EditorPatchOpKind>,
    requires_bounds: bool,
    viewport_visible: bool,
) -> EditorControlDescriptorDto {
    EditorControlDescriptorDto {
        kind: format_debug(&kind),
        label: label.to_string(),
        target_scope: target_scope.to_string(),
        handles: strings(handles),
        default_patch_op: default_patch_op.map(|op| format_debug(&op)),
        requires_bounds,
        viewport_visible,
    }
}

fn patch_op(
    kind: EditorPatchOpKind,
    label: &str,
    target_scope: &str,
    value_kind: &str,
    persistence: &str,
    risk: &str,
) -> EditorPatchOpDescriptorDto {
    EditorPatchOpDescriptorDto {
        kind: format_debug(&kind),
        label: label.to_string(),
        target_scope: target_scope.to_string(),
        value_kind: value_kind.to_string(),
        persistence: persistence.to_string(),
        risk: risk.to_string(),
    }
}

fn control_ref_dto(value: &EditorControlKind) -> EditorControlRefDto {
    let descriptor = control_descriptor_for(*value);
    EditorControlRefDto {
        kind: descriptor.kind,
        target_scope: descriptor.target_scope,
        handles: descriptor.handles,
        patch_op: descriptor.default_patch_op,
    }
}

fn patch_op_ref_dto(value: &EditorPatchOpKind) -> EditorPatchOpRefDto {
    let descriptor = patch_op_descriptor_for(*value);
    EditorPatchOpRefDto {
        kind: descriptor.kind,
        target_scope: descriptor.target_scope,
        persistence: descriptor.persistence,
    }
}

fn control_descriptor_for(kind: EditorControlKind) -> EditorControlDescriptorDto {
    control_descriptors()
        .into_iter()
        .find(|descriptor| descriptor.kind == format_debug(&kind))
        .unwrap_or_else(|| {
            control_descriptor(kind, &format_debug(&kind), "unknown", &[], None, false, false)
        })
}

fn patch_op_descriptor_for(kind: EditorPatchOpKind) -> EditorPatchOpDescriptorDto {
    patch_op_descriptors()
        .into_iter()
        .find(|descriptor| descriptor.kind == format_debug(&kind))
        .unwrap_or_else(|| patch_op(kind, &format_debug(&kind), "unknown", "unknown", "unknown", "medium"))
}

fn asset_ref_dto(value: &ComponentAssetRefDescriptor) -> EditorAssetRefDescriptorDto {
    EditorAssetRefDescriptorDto {
        field_path: value.field_path.to_string(),
        domain: format_debug(&value.domain),
        required: value.required,
    }
}

fn property_dto(value: &EditorPropertyDescriptor) -> EditorPropertyDescriptorDto {
    EditorPropertyDescriptorDto {
        path: value.path.to_string(),
        label: value.label.to_string(),
        value_kind: format_debug(&value.value_kind),
        access: format_debug(&value.access),
        editor: format_debug(&value.editor),
        asset_domain: value.asset_domain.map(|domain| format_debug(&domain)),
        patch_op: value.patch_op.map(|op| format_debug(&op)),
    }
}

fn bounds_policy_dto(value: &BoundsPolicy) -> EditorBoundsPolicyDto {
    match value {
        BoundsPolicy::ComponentBounds2D { field } => EditorBoundsPolicyDto {
            kind: "ComponentBounds2D".to_string(),
            field: Some((*field).to_string()),
        },
        _ => EditorBoundsPolicyDto {
            kind: format_debug(value),
            field: None,
        },
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn format_debug<T: std::fmt::Debug>(value: &T) -> String {
    format!("{:?}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_exposes_editor_target_metadata() {
        let catalog = editor_metadata_catalog_dto();

        assert!(catalog.target_kinds.iter().any(|target| target.kind == "sceneEntity"));
        assert!(catalog.document_kinds.iter().any(|document| document.kind == "sceneYaml"));
        assert!(catalog.asset_kinds.iter().any(|asset| asset.kind == "Image"));
    }

    #[test]
    fn component_descriptors_expose_structured_controls_and_patch_ops() {
        let catalog = editor_metadata_catalog_dto();
        let sprite = catalog
            .components
            .iter()
            .find(|component| component.type_name == "Sprite2D")
            .expect("Sprite2D descriptor should be present");

        assert!(sprite.editor_controls.iter().any(|control| control.kind == "Transform2D"));
        assert!(sprite.patch_ops.iter().any(|op| op.kind == "SetTransform2"));
        assert_eq!(sprite.bounds_policy.kind, "ComponentBounds2D");
    }
}
