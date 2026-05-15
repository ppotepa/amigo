fn render_layer_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let id = string_field(&node.value, "id").unwrap_or_else(|| "unknown".to_owned());
    let properties = RENDER_LAYER_PROPERTIES
        .iter()
        .map(|descriptor| property_from_node_descriptor(node, descriptor, Some(id.as_str())))
        .collect();

    AuthoringPropertyPanel {
        title: format!("Draw Layer: {id}"),
        groups: vec![AuthoringPropertyGroup {
            id: "render".to_owned(),
            title: "Render".to_owned(),
            properties,
        }],
    }
}

