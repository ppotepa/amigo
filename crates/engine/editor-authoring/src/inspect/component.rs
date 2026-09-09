fn group_label(group: &str) -> &'static str {
    if group.starts_with("render2d") {
        "Render"
    } else if group.starts_with("transform") {
        "Transform"
    } else if group.starts_with("particles") {
        "Particles"
    } else if group.starts_with("asset") {
        "Assets"
    } else if group.starts_with("metadata") {
        "Metadata"
    } else {
        "General"
    }
}

fn component_panel(
    node: &AuthoringNode,
    registry: &ComponentRegistry,
) -> AuthoringPropertyPanel {
    let component_type = node
        .semantic
        .component_type
        .clone()
        .or_else(|| string_field(&node.value, "type"))
        .unwrap_or_else(|| "Component".to_owned());

    let Some(descriptor) = registry.descriptor_by_type_name(&component_type) else {
        return semantic_status_panel(
            node,
            format!("Component: {component_type}"),
            "No component descriptor yet",
        );
    };

    component_panel_from_descriptor(node, descriptor)
}

fn component_panel_from_descriptor(
    node: &AuthoringNode,
    descriptor: &ComponentTypeDescriptor,
) -> AuthoringPropertyPanel {
    let mut groups = Vec::new();

    push_grouped(
        &mut groups,
        "metadata",
        "Metadata",
        readonly_text(node, "type", descriptor.type_name),
    );

    if let Some(owner) = node.semantic.owner_entity_name.clone() {
        push_grouped(
            &mut groups,
            "metadata",
            "Metadata",
            readonly_text(node, "entity", owner),
        );
    }

    for property_descriptor in descriptor.properties {
        let property = descriptor_property(node, property_descriptor);
        push_grouped(
            &mut groups,
            property_descriptor.group,
            group_label(property_descriptor.group),
            property,
        );
    }

    if descriptor.type_name == "LayeredImage2D" {
        append_layered_image_dynamic_properties(node, &mut groups);
    }

    AuthoringPropertyPanel {
        title: format!("Component: {}", descriptor.label),
        groups,
    }
}

fn descriptor_property(
    node: &AuthoringNode,
    descriptor: &EditorPropertyDescriptor,
) -> AuthoringProperty {
    let yaml_value = value_at_path(&node.value, descriptor.path);
    let binding = resolve_property_binding(node, descriptor, yaml_value);
    let read_only = matches!(descriptor.access, EditorPropertyAccess::ReadOnly);
    let trait_kind = descriptor.trait_kind.map(|kind| kind.id().to_owned());
    let visibility = authoring_visibility(descriptor.visibility);
    let mut tags = descriptor
        .tags
        .iter()
        .map(|tag| (*tag).to_owned())
        .collect::<Vec<_>>();
    if !read_only && binding.is_none() {
        tags.push("NoBinding".to_owned());
    }

    AuthoringProperty {
        id: property_id(node, descriptor.path),
        label: descriptor.label.to_owned(),
        value: value_from_descriptor(descriptor.value_kind, yaml_value),
        editor: editor_from_descriptor(descriptor),
        hints: hints_from_descriptor(descriptor),
        read_only,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: property_yaml_pointer(node, descriptor.path),
        source_value: yaml_value.cloned(),
        group: descriptor.group.to_owned(),
        trait_kind,
        binding: binding.clone(),
        display: display_for_binding(&binding, read_only, visibility, tags),
    }
}

fn push_grouped(
    groups: &mut Vec<AuthoringPropertyGroup>,
    id: &str,
    title: &str,
    mut property: AuthoringProperty,
) {
    property.group = id.to_owned();
    if let Some(group) = groups.iter_mut().find(|group| group.id == id) {
        group.properties.push(property);
        return;
    }
    groups.push(AuthoringPropertyGroup {
        id: id.to_owned(),
        title: title.to_owned(),
        properties: vec![property],
    });
}

fn append_layered_image_dynamic_properties(
    node: &AuthoringNode,
    groups: &mut Vec<AuthoringPropertyGroup>,
) {
    for property in collect_image_part_properties(node) {
        push_grouped(
            groups,
            "render2d.image_parts.dynamic",
            "Image Parts",
            property,
        );
    }
}

