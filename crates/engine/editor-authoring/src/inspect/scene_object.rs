fn entity_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let name = string_field(&node.value, "name")
        .or_else(|| string_field(&node.value, "id"))
        .unwrap_or_else(|| "unknown".to_owned());
    let mut components = Vec::new();
    collect_component_nodes(node, &mut components);
    let component_count = components.len();
    let groups = vec![
        AuthoringPropertyGroup {
            id: "summary".to_owned(),
            title: "Summary".to_owned(),
            properties: vec![
                status_text_primary(node, "scene object", name.as_str()),
                status_text_primary(node, "components", component_count.to_string()),
                readonly_text(node, "source", node.source_file.display().to_string()),
            ],
        },
        AuthoringPropertyGroup {
            id: "components".to_owned(),
            title: "Components".to_owned(),
            properties: entity_component_rows(&components, node),
        },
    ];
    AuthoringPropertyPanel {
        title: format!("Scene Object: {name}"),
        groups,
    }
}

fn collect_component_nodes<'a>(node: &'a AuthoringNode, out: &mut Vec<&'a AuthoringNode>) {
    for child in &node.children {
        if matches!(child.kind, AuthoringNodeKind::Component) {
            out.push(child);
        }
        collect_component_nodes(child, out);
    }
}

fn entity_component_rows(components: &[&AuthoringNode], fallback_node: &AuthoringNode) -> Vec<AuthoringProperty> {
    let mut rows = Vec::new();
    for child in components {
        let component_type = child
            .semantic
            .component_type
            .clone()
            .or_else(|| string_field(&child.value, "type"))
            .unwrap_or_else(|| "Component".to_owned());
        rows.push(readonly_text(
            child,
            component_type.as_str(),
            "descriptor-backed",
        ));
    }
    if rows.is_empty() {
        rows.push(readonly_text(fallback_node, "components", "none"));
    }
    rows
}

