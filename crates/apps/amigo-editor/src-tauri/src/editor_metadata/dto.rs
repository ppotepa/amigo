use amigo_scene::{
    ComponentAssetRefDescriptor, ComponentKind, ComponentTypeDescriptor, EditorPropertyDescriptor,
    default_component_registry,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorMetadataCatalogDto {
    pub components: Vec<EditorComponentDescriptorDto>,
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
    pub editor_controls: Vec<String>,
    pub patch_ops: Vec<String>,
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
    EditorMetadataCatalogDto { components }
}

fn component_descriptor_dto(descriptor: &ComponentTypeDescriptor) -> EditorComponentDescriptorDto {
    EditorComponentDescriptorDto {
        kind: format!("{:?}", descriptor.kind),
        type_name: descriptor.type_name.to_string(),
        label: descriptor.label.to_string(),
        domains: descriptor.domains.iter().map(format_debug).collect(),
        capabilities: descriptor.capabilities.iter().map(format_debug).collect(),
        asset_refs: descriptor.asset_refs.iter().map(asset_ref_dto).collect(),
        properties: descriptor.properties.iter().map(property_dto).collect(),
        editor_controls: descriptor
            .editor_controls
            .iter()
            .map(format_debug)
            .collect(),
        patch_ops: descriptor.patch_ops.iter().map(format_debug).collect(),
    }
}

fn asset_ref_dto(value: &ComponentAssetRefDescriptor) -> EditorAssetRefDescriptorDto {
    EditorAssetRefDescriptorDto {
        field_path: value.field_path.to_string(),
        domain: format!("{:?}", value.domain),
        required: value.required,
    }
}

fn property_dto(value: &EditorPropertyDescriptor) -> EditorPropertyDescriptorDto {
    EditorPropertyDescriptorDto {
        path: value.path.to_string(),
        label: value.label.to_string(),
        value_kind: format!("{:?}", value.value_kind),
        access: format!("{:?}", value.access),
        editor: format!("{:?}", value.editor),
        asset_domain: value.asset_domain.map(|domain| format!("{:?}", domain)),
        patch_op: value.patch_op.map(|op| format!("{:?}", op)),
    }
}

fn format_debug<T: std::fmt::Debug>(value: &T) -> String {
    format!("{:?}", value)
}
