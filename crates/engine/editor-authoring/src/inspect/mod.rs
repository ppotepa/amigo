use amigo_editor_api::{
    AuthoringNumberConstraints, AuthoringOption, AuthoringProperty, AuthoringPropertyApplyMode,
    AuthoringPropertyDisplay, AuthoringPropertyEditor, AuthoringPropertyGroup,
    AuthoringPropertyHints, AuthoringPropertyPanel, AuthoringPropertyValue,
    AuthoringPropertyVisibility, AuthoringRuntimeBinding,
};
use amigo_scene::{
    ComponentTypeDescriptor, EditorPropertyAccess, EditorPropertyDescriptor,
    EditorPropertyEditorKind, EditorPropertyValueKind,
    EditorPropertyVisibility as ScenePropertyVisibility, default_component_registry,
};
use serde_yaml::Value;

use crate::bindings::resolve_property_binding;
use crate::ids::child_pointer;
use crate::image_parts::collect_image_part_properties;
use crate::node_descriptors::{RENDER_LAYER_PROPERTIES, property_from_node_descriptor};
use crate::{AuthoringNode, AuthoringNodeKind};

pub fn build_property_panel_for_node(node: &AuthoringNode) -> AuthoringPropertyPanel {
    match node.kind {
        AuthoringNodeKind::RenderLayer => render_layer_panel(node),
        AuthoringNodeKind::Component => component_panel(node),
        AuthoringNodeKind::PostFxItem => postfx_panel(node),
        AuthoringNodeKind::Entity => entity_panel(node),
        AuthoringNodeKind::PrefabRef => prefab_ref_panel(node),
        AuthoringNodeKind::PrefabOverrides => prefab_overrides_panel(node),
        AuthoringNodeKind::Use => use_ref_panel(node),
        AuthoringNodeKind::LightGroup => light_group_panel(node),
        AuthoringNodeKind::LightRoute => light_route_panel(node),
        AuthoringNodeKind::Scalar | AuthoringNodeKind::Mapping | AuthoringNodeKind::Sequence => {
            raw_debug_only_panel(node)
        }
        _ => semantic_status_panel(node, node.label.clone(), "No descriptor-backed properties"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorViewMode {
    Primary,
    Advanced,
    RawDebug,
}

pub fn filter_property_panel_for_view(
    mut panel: AuthoringPropertyPanel,
    mode: InspectorViewMode,
) -> AuthoringPropertyPanel {
    for group in &mut panel.groups {
        group
            .properties
            .retain(|row| property_visible_for_view(row, mode));
    }
    panel.groups.retain(|group| !group.properties.is_empty());
    panel
}

include!("visibility.rs");
include!("properties.rs");
include!("draw_layer.rs");
include!("component.rs");
include!("post_fx.rs");
include!("scene_object.rs");
include!("raw.rs");
