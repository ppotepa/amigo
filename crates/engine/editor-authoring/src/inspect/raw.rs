fn raw_debug_only_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    semantic_status_panel(
        node,
        format!("Raw Debug: {}", node.label),
        "Raw YAML is available only in Raw Debug",
    )
}

fn semantic_status_panel(
    node: &AuthoringNode,
    title: impl Into<String>,
    status: impl Into<String>,
) -> AuthoringPropertyPanel {
    AuthoringPropertyPanel {
        title: title.into(),
        groups: vec![AuthoringPropertyGroup {
            id: "status".to_owned(),
            title: "Status".to_owned(),
            properties: vec![
                status_text_primary(node, "status", status),
                readonly_text(node, "source", node.source_file.display().to_string()),
            ],
        }],
    }
}

fn prefab_ref_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let prefab = match &node.value {
        Value::String(value) => value.clone(),
        _ => short_yaml(&node.value),
    };
    AuthoringPropertyPanel {
        title: "Prefab Reference".to_owned(),
        groups: vec![AuthoringPropertyGroup {
            id: "prefab".to_owned(),
            title: "Prefab".to_owned(),
            properties: vec![
                status_text_primary(node, "prefab", prefab),
                readonly_text(node, "source", node.source_file.display().to_string()),
                readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
                readonly_text(node, "editable", "false"),
            ],
        }],
    }
}

fn prefab_overrides_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let mut properties = vec![
        status_text_primary(node, "status", "Readonly"),
        readonly_text(node, "source", node.source_file.display().to_string()),
        readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
        readonly_text(node, "editable", "false"),
    ];
    match node.value.as_mapping() {
        Some(mapping) => {
            properties.push(readonly_text(
                node,
                "override_count",
                mapping.len().to_string(),
            ));
            for (key, value) in mapping {
                let Some(key) = key.as_str() else { continue };
                properties.push(readonly_text(node, key, short_yaml(value)));
            }
        }
        None => properties.push(readonly_text(node, "value", short_yaml(&node.value))),
    }
    AuthoringPropertyPanel {
        title: "Prefab Overrides".to_owned(),
        groups: vec![AuthoringPropertyGroup {
            id: "prefab.overrides".to_owned(),
            title: "Prefab Overrides".to_owned(),
            properties,
        }],
    }
}

fn use_ref_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    AuthoringPropertyPanel {
        title: format!("Use Ref: {}", node.label),
        groups: vec![AuthoringPropertyGroup {
            id: "use".to_owned(),
            title: "Use Reference".to_owned(),
            properties: vec![
                status_text_primary(node, "status", "Readonly"),
                readonly_text(node, "source", node.source_file.display().to_string()),
                readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
                readonly_text(node, "origin", format!("{:?}", node.origin)),
                readonly_text(node, "resolved", "true"),
                readonly_text(node, "children", node.children.len().to_string()),
            ],
        }],
    }
}

fn light_group_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let subject = node
        .semantic
        .light_group_id
        .clone()
        .or_else(|| string_field(&node.value, "id"))
        .or_else(|| string_field(&node.value, "name"))
        .unwrap_or_else(|| "unknown-light-group".to_owned());
    semantic_status_panel(
        node,
        format!("Light Group: {subject}"),
        "No live runtime binding yet",
    )
}

fn light_route_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let subject = node
        .semantic
        .light_route_receiver_layer
        .clone()
        .or_else(|| string_field(&node.value, "receiver_layer"))
        .or_else(|| string_field(&node.value, "layer"))
        .unwrap_or_else(|| "unknown-light-route".to_owned());
    semantic_status_panel(
        node,
        format!("Light Route: {subject}"),
        "No live runtime binding yet",
    )
}

