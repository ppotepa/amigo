fn postfx_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let effect_type = string_field(&node.value, "type").unwrap_or_else(|| "effect".to_owned());
    let effect_id = string_field(&node.value, "id").unwrap_or_else(|| effect_type.clone());
    semantic_status_panel(
        node,
        format!("Frame Post FX: {effect_id}"),
        format!("{effect_type}: Frame scope. No live runtime binding yet"),
    )
}

