use serde_yaml::{Mapping, Value};

use super::dto::{
    EditorUiNodeCreateDto, EditorUiNodeCreateKindDto, EditorUiNodeMoveDirectionDto,
    EditorUiNodeSelectionDto, EditorUiTemplateKindDto,
};

#[derive(Debug, Clone)]
pub struct UiStructurePatchOutcome {
    pub changed_entity_id: String,
    pub selected_entity_id: Option<String>,
    pub selected_ui_node: Option<EditorUiNodeSelectionDto>,
    pub message: String,
}

pub fn patch_create_ui_document(
    document: &mut Value,
    entity_id: &str,
    label: &str,
    viewport_width: f32,
    viewport_height: f32,
    template: EditorUiTemplateKindDto,
) -> Result<UiStructurePatchOutcome, String> {
    validate_id(entity_id, "entity id")?;
    let entities = entities_mut_or_insert(document)?;
    if entities
        .iter()
        .any(|entity| mapping_string(entity, "id").as_deref() == Some(entity_id))
    {
        return Err(format!("entity `{entity_id}` already exists"));
    }

    let mut root = node_mapping("column", "root");
    root.insert(
        key("style"),
        Value::Mapping(style(&[
            ("width", viewport_width),
            ("height", viewport_height),
            ("padding", 0.0),
            ("gap", 12.0),
        ])),
    );
    let children = children_mut_or_insert(&mut root)?;
    if !matches!(template, EditorUiTemplateKindDto::EmptyDocument) {
        children.push(make_template_subtree(template, "menu")?);
    }

    let mut target = Mapping::new();
    target.insert(key("type"), Value::String("screen-space".to_owned()));
    target.insert(key("layer"), Value::String("menu".to_owned()));
    let mut viewport = Mapping::new();
    viewport.insert(key("width"), number(viewport_width));
    viewport.insert(key("height"), number(viewport_height));
    viewport.insert(key("scaling"), Value::String("fit".to_owned()));
    target.insert(key("viewport"), Value::Mapping(viewport));

    let mut component = Mapping::new();
    component.insert(key("type"), Value::String("UiDocument".to_owned()));
    component.insert(key("target"), Value::Mapping(target));
    component.insert(key("root"), Value::Mapping(root));

    let mut entity = Mapping::new();
    entity.insert(key("id"), Value::String(entity_id.to_owned()));
    entity.insert(key("name"), Value::String(label.to_owned()));
    entity.insert(key("visible"), Value::Bool(true));
    entity.insert(key("simulation_enabled"), Value::Bool(false));
    entity.insert(key("collision_enabled"), Value::Bool(false));
    entity.insert(
        key("components"),
        Value::Sequence(vec![Value::Mapping(component)]),
    );
    entities.push(Value::Mapping(entity));

    Ok(outcome(
        entity_id,
        Some(EditorUiNodeSelectionDto {
            entity_id: entity_id.to_owned(),
            component_index: 0,
            node_path: "root".to_owned(),
        }),
        "UI document was created.",
    ))
}

pub fn patch_add_ui_node(
    document: &mut Value,
    entity_id: &str,
    component_index: usize,
    parent_path: &str,
    node: EditorUiNodeCreateDto,
    insert_index: Option<usize>,
) -> Result<UiStructurePatchOutcome, String> {
    let node_id = node.id.clone();
    validate_id(&node_id, "node id")?;
    let root = find_ui_document_root_mut(document, entity_id, component_index)?;
    let parent = find_ui_node_mut(root, parent_path)?;
    assert_can_have_children(parent)?;
    let children = children_mut_or_insert(parent)?;
    ensure_child_id_available(children, &node_id)?;

    let index = insert_index.unwrap_or(children.len()).min(children.len());
    children.insert(index, make_node_from_create_dto(node)?);
    let selected_path = format!("{parent_path}.{node_id}");

    Ok(outcome(
        entity_id,
        Some(EditorUiNodeSelectionDto {
            entity_id: entity_id.to_owned(),
            component_index,
            node_path: selected_path,
        }),
        "UI node was added.",
    ))
}

pub fn patch_add_ui_template(
    document: &mut Value,
    entity_id: &str,
    component_index: usize,
    parent_path: &str,
    template: EditorUiTemplateKindDto,
    id_prefix: &str,
    insert_index: Option<usize>,
) -> Result<UiStructurePatchOutcome, String> {
    validate_id(id_prefix, "template id prefix")?;
    let root = find_ui_document_root_mut(document, entity_id, component_index)?;
    let parent = find_ui_node_mut(root, parent_path)?;
    assert_can_have_children(parent)?;
    let children = children_mut_or_insert(parent)?;
    let mut subtree = make_template_subtree(template, id_prefix)?;
    let subtree_id =
        mapping_string(&subtree, "id").ok_or_else(|| "template root node has no id".to_owned())?;
    let unique_id = ensure_unique_child_id(children, &subtree_id);
    if unique_id != subtree_id {
        if let Some(mapping) = subtree.as_mapping_mut() {
            mapping.insert(key("id"), Value::String(unique_id.clone()));
        }
    }

    let index = insert_index.unwrap_or(children.len()).min(children.len());
    children.insert(index, subtree);
    let selected_path = format!("{parent_path}.{unique_id}");

    Ok(outcome(
        entity_id,
        Some(EditorUiNodeSelectionDto {
            entity_id: entity_id.to_owned(),
            component_index,
            node_path: selected_path,
        }),
        "UI template was added.",
    ))
}

pub fn patch_duplicate_ui_node(
    document: &mut Value,
    entity_id: &str,
    component_index: usize,
    node_path: &str,
    new_id: Option<String>,
    copy_actions: bool,
) -> Result<UiStructurePatchOutcome, String> {
    let parent_path = parent_path_of(node_path)?;
    let target_id = node_id_from_path(node_path)?;
    let root = find_ui_document_root_mut(document, entity_id, component_index)?;
    let parent = find_ui_node_mut(root, &parent_path)?;
    let children = children_mut_or_insert(parent)?;
    let index = child_index(children, &target_id)?;
    let mut clone = children[index].clone();

    let desired_id = new_id.unwrap_or_else(|| format!("{target_id}-copy"));
    validate_id(&desired_id, "duplicate node id")?;
    let unique_id = ensure_unique_child_id(children, &desired_id);
    if let Some(mapping) = clone.as_mapping_mut() {
        mapping.insert(key("id"), Value::String(unique_id.clone()));
    }
    if !copy_actions {
        strip_action_bindings_recursive(&mut clone);
    }
    children.insert(index + 1, clone);

    Ok(outcome(
        entity_id,
        Some(EditorUiNodeSelectionDto {
            entity_id: entity_id.to_owned(),
            component_index,
            node_path: format!("{parent_path}.{unique_id}"),
        }),
        "UI node was duplicated.",
    ))
}

pub fn patch_remove_ui_node(
    document: &mut Value,
    entity_id: &str,
    component_index: usize,
    node_path: &str,
) -> Result<UiStructurePatchOutcome, String> {
    let parent_path = parent_path_of(node_path)?;
    let target_id = node_id_from_path(node_path)?;
    let root = find_ui_document_root_mut(document, entity_id, component_index)?;
    let parent = find_ui_node_mut(root, &parent_path)?;
    let children = children_mut_or_insert(parent)?;
    let index = child_index(children, &target_id)?;
    children.remove(index);

    Ok(outcome(
        entity_id,
        Some(EditorUiNodeSelectionDto {
            entity_id: entity_id.to_owned(),
            component_index,
            node_path: parent_path,
        }),
        "UI node was removed.",
    ))
}

pub fn patch_move_ui_node(
    document: &mut Value,
    entity_id: &str,
    component_index: usize,
    node_path: &str,
    direction: EditorUiNodeMoveDirectionDto,
) -> Result<UiStructurePatchOutcome, String> {
    let parent_path = parent_path_of(node_path)?;
    let target_id = node_id_from_path(node_path)?;
    let root = find_ui_document_root_mut(document, entity_id, component_index)?;
    let parent = find_ui_node_mut(root, &parent_path)?;
    let children = children_mut_or_insert(parent)?;
    let index = child_index(children, &target_id)?;
    match direction {
        EditorUiNodeMoveDirectionDto::Up if index > 0 => children.swap(index, index - 1),
        EditorUiNodeMoveDirectionDto::Down if index + 1 < children.len() => {
            children.swap(index, index + 1)
        }
        EditorUiNodeMoveDirectionDto::Up => {
            return Ok(outcome(
                entity_id,
                Some(EditorUiNodeSelectionDto {
                    entity_id: entity_id.to_owned(),
                    component_index,
                    node_path: node_path.to_owned(),
                }),
                "UI node is already first.",
            ));
        }
        EditorUiNodeMoveDirectionDto::Down => {
            return Ok(outcome(
                entity_id,
                Some(EditorUiNodeSelectionDto {
                    entity_id: entity_id.to_owned(),
                    component_index,
                    node_path: node_path.to_owned(),
                }),
                "UI node is already last.",
            ));
        }
    }

    Ok(outcome(
        entity_id,
        Some(EditorUiNodeSelectionDto {
            entity_id: entity_id.to_owned(),
            component_index,
            node_path: node_path.to_owned(),
        }),
        "UI node was moved.",
    ))
}

fn outcome(
    entity_id: &str,
    selected_ui_node: Option<EditorUiNodeSelectionDto>,
    message: &str,
) -> UiStructurePatchOutcome {
    UiStructurePatchOutcome {
        changed_entity_id: entity_id.to_owned(),
        selected_entity_id: Some(entity_id.to_owned()),
        selected_ui_node,
        message: message.to_owned(),
    }
}

fn find_ui_document_root_mut<'a>(
    document: &'a mut Value,
    entity_id: &str,
    component_index: usize,
) -> Result<&'a mut Mapping, String> {
    let entities = entities_mut(document)?;
    let entity = entities
        .iter_mut()
        .find(|entity| mapping_string(entity, "id").as_deref() == Some(entity_id))
        .ok_or_else(|| format!("entity `{entity_id}` not found"))?;
    let entity = entity
        .as_mapping_mut()
        .ok_or_else(|| format!("entity `{entity_id}` is not a mapping"))?;
    let components = entity
        .get_mut(key("components"))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| format!("entity `{entity_id}` has no components"))?;
    let component = components.get_mut(component_index).ok_or_else(|| {
        format!("component index `{component_index}` out of range for entity `{entity_id}`")
    })?;
    let component = component.as_mapping_mut().ok_or_else(|| {
        format!("component `{component_index}` on entity `{entity_id}` is not a mapping")
    })?;
    if component.get(key("type")).and_then(Value::as_str) != Some("UiDocument") {
        return Err(format!(
            "component `{component_index}` on entity `{entity_id}` is not a UiDocument"
        ));
    }
    component
        .get_mut(key("root"))
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| format!("UiDocument `{component_index}` on `{entity_id}` has no root"))
}

fn find_ui_node_mut<'a>(root: &'a mut Mapping, node_path: &str) -> Result<&'a mut Mapping, String> {
    let parts = split_node_path(node_path)?;
    let root_id = string_field(root, "id").unwrap_or_else(|| "root".to_owned());
    if parts.first().map(String::as_str) != Some(root_id.as_str()) {
        return Err(format!(
            "UI node path `{node_path}` does not start at `{root_id}`"
        ));
    }
    find_ui_node_parts_mut(root, &parts[1..], node_path)
}

fn find_ui_node_parts_mut<'a>(
    current: &'a mut Mapping,
    remaining: &[String],
    full_path: &str,
) -> Result<&'a mut Mapping, String> {
    if remaining.is_empty() {
        return Ok(current);
    }
    let target_id = &remaining[0];
    let children = current
        .get_mut(key("children"))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| format!("UI node `{full_path}` cannot be found; missing children"))?;
    let child = children
        .iter_mut()
        .find(|child| mapping_string(child, "id").as_deref() == Some(target_id.as_str()))
        .ok_or_else(|| format!("UI node path segment `{target_id}` not found in `{full_path}`"))?;
    let child = child
        .as_mapping_mut()
        .ok_or_else(|| format!("UI node segment `{target_id}` is not a mapping"))?;
    find_ui_node_parts_mut(child, &remaining[1..], full_path)
}

fn entities_mut(document: &mut Value) -> Result<&mut Vec<Value>, String> {
    document
        .as_mapping_mut()
        .ok_or_else(|| "scene document root is not a mapping".to_owned())?
        .get_mut(key("entities"))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| "scene document has no entities sequence".to_owned())
}

fn entities_mut_or_insert(document: &mut Value) -> Result<&mut Vec<Value>, String> {
    let root = document
        .as_mapping_mut()
        .ok_or_else(|| "scene document root is not a mapping".to_owned())?;
    if !root.contains_key(key("entities")) {
        root.insert(key("entities"), Value::Sequence(Vec::new()));
    }
    root.get_mut(key("entities"))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| "scene document entities field is not a sequence".to_owned())
}

fn children_mut_or_insert(node: &mut Mapping) -> Result<&mut Vec<Value>, String> {
    if !node.contains_key(key("children")) {
        node.insert(key("children"), Value::Sequence(Vec::new()));
    }
    node.get_mut(key("children"))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| "UI node children field is not a sequence".to_owned())
}

fn assert_can_have_children(node: &Mapping) -> Result<(), String> {
    let kind = string_field(node, "type").unwrap_or_else(|| "unknown".to_owned());
    if matches!(
        kind.as_str(),
        "column" | "row" | "panel" | "stack" | "grid" | "scroll-area" | "button" | "group-box"
    ) {
        Ok(())
    } else {
        Err(format!("UI node type `{kind}` cannot have children"))
    }
}

fn make_node_from_create_dto(node: EditorUiNodeCreateDto) -> Result<Value, String> {
    let kind = kind_string(&node.kind);
    let mut mapping = node_mapping(kind, &node.id);
    if let Some(label) = node.label.filter(|value| !value.trim().is_empty()) {
        mapping.insert(key("label"), Value::String(label));
    }
    if let Some(text) = node.text.filter(|value| !value.trim().is_empty()) {
        mapping.insert(key("text"), Value::String(text));
    } else if let Some(default_text) = default_text(&node.kind) {
        mapping.insert(key("text"), Value::String(default_text.to_owned()));
    }
    let style_fields = match node.kind {
        EditorUiNodeCreateKindDto::Button => {
            vec![("width", 320.0), ("height", 48.0), ("font_size", 18.0)]
        }
        EditorUiNodeCreateKindDto::Text => {
            vec![("width", 320.0), ("height", 32.0), ("font_size", 18.0)]
        }
        EditorUiNodeCreateKindDto::Panel => vec![
            ("width", 420.0),
            ("height", 240.0),
            ("padding", 16.0),
            ("gap", 12.0),
        ],
        EditorUiNodeCreateKindDto::Column
        | EditorUiNodeCreateKindDto::Row
        | EditorUiNodeCreateKindDto::Stack => vec![("gap", 12.0)],
        EditorUiNodeCreateKindDto::Spacer => vec![("width", 16.0), ("height", 16.0)],
        EditorUiNodeCreateKindDto::Image => vec![("width", 128.0), ("height", 128.0)],
        EditorUiNodeCreateKindDto::ProgressBar => vec![("width", 320.0), ("height", 24.0)],
    };
    if !style_fields.is_empty() {
        mapping.insert(key("style"), Value::Mapping(style(&style_fields)));
    }
    if matches!(
        node.kind,
        EditorUiNodeCreateKindDto::Column
            | EditorUiNodeCreateKindDto::Row
            | EditorUiNodeCreateKindDto::Panel
            | EditorUiNodeCreateKindDto::Stack
    ) {
        mapping.insert(key("children"), Value::Sequence(Vec::new()));
    }
    Ok(Value::Mapping(mapping))
}

fn make_template_subtree(
    template: EditorUiTemplateKindDto,
    id_prefix: &str,
) -> Result<Value, String> {
    match template {
        EditorUiTemplateKindDto::EmptyDocument => Ok(Value::Mapping(node_mapping(
            "column",
            &format!("{id_prefix}-root"),
        ))),
        EditorUiTemplateKindDto::VerticalMenu => {
            let mut screen = node_mapping("column", &format!("{id_prefix}-main-menu-screen"));
            screen.insert(
                key("style"),
                Value::Mapping(style(&[
                    ("width", 1280.0),
                    ("height", 720.0),
                    ("padding", 48.0),
                    ("gap", 18.0),
                ])),
            );

            let mut header = node_mapping("panel", &format!("{id_prefix}-header"));
            header.insert(
                key("style"),
                Value::Mapping(style(&[("width", 720.0), ("height", 120.0), ("gap", 10.0)])),
            );
            header.insert(
                key("children"),
                Value::Sequence(vec![text(&format!("{id_prefix}-title"), "MAIN MENU")]),
            );

            let mut menu = node_mapping("column", &format!("{id_prefix}-menu-container"));
            menu.insert(
                key("style"),
                Value::Mapping(style(&[("width", 420.0), ("gap", 12.0)])),
            );
            menu.insert(
                key("children"),
                Value::Sequence(vec![
                    action_button(
                        &format!("{id_prefix}-new-game"),
                        "NEW GAME",
                        420.0,
                        48.0,
                        "navigate",
                        &format!("{id_prefix}-new-game-flow"),
                    ),
                    action_button(
                        &format!("{id_prefix}-options"),
                        "OPTIONS",
                        420.0,
                        48.0,
                        "navigate",
                        &format!("{id_prefix}-options-screen"),
                    ),
                    button(&format!("{id_prefix}-credits"), "CREDITS", 420.0, 48.0),
                    button(&format!("{id_prefix}-quit"), "QUIT", 420.0, 48.0),
                ]),
            );

            let mut footer = node_mapping("panel", &format!("{id_prefix}-footer"));
            footer.insert(
                key("children"),
                Value::Sequence(vec![text(&format!("{id_prefix}-version"), "v0.1.0")]),
            );

            let mut options_screen = node_mapping("column", &format!("{id_prefix}-options-screen"));
            options_screen.insert(
                key("style"),
                Value::Mapping(style(&[("width", 520.0), ("gap", 10.0)])),
            );
            options_screen.insert(
                key("children"),
                Value::Sequence(vec![
                    text(&format!("{id_prefix}-audio-row"), "Audio"),
                    text(&format!("{id_prefix}-video-row"), "Video"),
                    text(&format!("{id_prefix}-controls-row"), "Controls"),
                    button(&format!("{id_prefix}-back"), "BACK", 220.0, 42.0),
                ]),
            );

            screen.insert(
                key("children"),
                Value::Sequence(vec![
                    Value::Mapping(header),
                    Value::Mapping(menu),
                    Value::Mapping(footer),
                    Value::Mapping(options_screen),
                ]),
            );

            Ok(Value::Mapping(screen))
        }
        EditorUiTemplateKindDto::ButtonRow => {
            let mut node = node_mapping("row", &format!("{id_prefix}-buttons"));
            node.insert(
                key("style"),
                Value::Mapping(style(&[("width", 520.0), ("gap", 10.0)])),
            );
            node.insert(
                key("children"),
                Value::Sequence(vec![
                    button(&format!("{id_prefix}-primary"), "PRIMARY", 240.0, 48.0),
                    button(&format!("{id_prefix}-secondary"), "SECONDARY", 240.0, 48.0),
                ]),
            );
            Ok(Value::Mapping(node))
        }
        EditorUiTemplateKindDto::HealthBar => {
            let mut node = node_mapping("row", &format!("{id_prefix}-health"));
            node.insert(
                key("style"),
                Value::Mapping(style(&[("width", 360.0), ("height", 36.0), ("gap", 8.0)])),
            );
            node.insert(
                key("children"),
                Value::Sequence(vec![
                    text(&format!("{id_prefix}-health-label"), "HP"),
                    progress(&format!("{id_prefix}-health-bar")),
                ]),
            );
            Ok(Value::Mapping(node))
        }
        EditorUiTemplateKindDto::AmmoCounter => {
            let mut node = node_mapping("row", &format!("{id_prefix}-ammo"));
            node.insert(
                key("style"),
                Value::Mapping(style(&[("width", 240.0), ("height", 32.0), ("gap", 8.0)])),
            );
            node.insert(
                key("children"),
                Value::Sequence(vec![
                    text(&format!("{id_prefix}-ammo-label"), "AMMO"),
                    text(&format!("{id_prefix}-ammo-value"), "30 / 120"),
                ]),
            );
            Ok(Value::Mapping(node))
        }
        EditorUiTemplateKindDto::DialogueBox => {
            let mut node = node_mapping("panel", &format!("{id_prefix}-dialogue-box"));
            node.insert(
                key("style"),
                Value::Mapping(style(&[
                    ("width", 760.0),
                    ("height", 180.0),
                    ("padding", 20.0),
                    ("gap", 10.0),
                ])),
            );
            node.insert(
                key("children"),
                Value::Sequence(vec![
                    text(&format!("{id_prefix}-dialogue-speaker"), "Speaker"),
                    text(&format!("{id_prefix}-dialogue-text"), "Dialogue text..."),
                    text(
                        &format!("{id_prefix}-dialogue-hint"),
                        "Press Space to continue",
                    ),
                ]),
            );
            Ok(Value::Mapping(node))
        }
    }
}

fn node_mapping(kind: &str, id: &str) -> Mapping {
    let mut mapping = Mapping::new();
    mapping.insert(key("type"), Value::String(kind.to_owned()));
    mapping.insert(key("id"), Value::String(id.to_owned()));
    mapping
}

fn button(id: &str, text_value: &str, width: f32, height: f32) -> Value {
    let mut node = node_mapping("button", id);
    node.insert(key("text"), Value::String(text_value.to_owned()));
    node.insert(
        key("style"),
        Value::Mapping(style(&[("width", width), ("height", height)])),
    );
    Value::Mapping(node)
}

fn action_button(
    id: &str,
    text_value: &str,
    width: f32,
    height: f32,
    event: &str,
    target: &str,
) -> Value {
    let mut node = match button(id, text_value, width, height) {
        Value::Mapping(node) => node,
        value => return value,
    };

    let mut on_click = Mapping::new();
    on_click.insert(key("event"), Value::String(event.to_owned()));
    on_click.insert(key("target"), Value::String(target.to_owned()));
    node.insert(key("on_click"), Value::Mapping(on_click));

    Value::Mapping(node)
}

fn text(id: &str, text_value: &str) -> Value {
    let mut node = node_mapping("text", id);
    node.insert(key("text"), Value::String(text_value.to_owned()));
    Value::Mapping(node)
}

fn progress(id: &str) -> Value {
    let mut node = node_mapping("progress-bar", id);
    node.insert(
        key("style"),
        Value::Mapping(style(&[("width", 260.0), ("height", 24.0)])),
    );
    Value::Mapping(node)
}

fn style(fields: &[(&str, f32)]) -> Mapping {
    fields
        .iter()
        .map(|(name, value)| (key(name), number(*value)))
        .collect()
}

fn strip_action_bindings_recursive(value: &mut Value) {
    if let Some(mapping) = value.as_mapping_mut() {
        for key_name in [
            "on_click",
            "on_change",
            "on_hover",
            "on_focus",
            "action",
            "action_event",
            "action_target",
        ] {
            mapping.remove(key(key_name));
        }
        if let Some(children) = mapping
            .get_mut(key("children"))
            .and_then(Value::as_sequence_mut)
        {
            for child in children {
                strip_action_bindings_recursive(child);
            }
        }
    }
}

fn ensure_child_id_available(children: &[Value], desired_id: &str) -> Result<(), String> {
    if children
        .iter()
        .any(|child| mapping_string(child, "id").as_deref() == Some(desired_id))
    {
        Err(format!(
            "UI node id `{desired_id}` already exists under this parent"
        ))
    } else {
        Ok(())
    }
}

fn ensure_unique_child_id(children: &[Value], desired_id: &str) -> String {
    if !children
        .iter()
        .any(|child| mapping_string(child, "id").as_deref() == Some(desired_id))
    {
        return desired_id.to_owned();
    }
    for index in 2.. {
        let candidate = format!("{desired_id}-{index}");
        if !children
            .iter()
            .any(|child| mapping_string(child, "id").as_deref() == Some(candidate.as_str()))
        {
            return candidate;
        }
    }
    unreachable!()
}

fn child_index(children: &[Value], target_id: &str) -> Result<usize, String> {
    children
        .iter()
        .position(|child| mapping_string(child, "id").as_deref() == Some(target_id))
        .ok_or_else(|| format!("UI node `{target_id}` not found under parent"))
}

fn split_node_path(node_path: &str) -> Result<Vec<String>, String> {
    let parts = node_path
        .split('.')
        .filter(|part| !part.trim().is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if parts.is_empty() {
        Err("UI node path is empty".to_owned())
    } else {
        Ok(parts)
    }
}

fn parent_path_of(node_path: &str) -> Result<String, String> {
    let parts = split_node_path(node_path)?;
    if parts.len() <= 1 {
        return Err("root UI node cannot be removed, duplicated, or moved".to_owned());
    }
    Ok(parts[..parts.len() - 1].join("."))
}

fn node_id_from_path(node_path: &str) -> Result<String, String> {
    split_node_path(node_path)?
        .last()
        .cloned()
        .ok_or_else(|| "UI node path is empty".to_owned())
}

fn validate_id(id: &str, label: &str) -> Result<(), String> {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return Err(format!("{label} is required"));
    };
    if !first.is_ascii_lowercase() {
        return Err(format!("{label} must start with a lowercase letter"));
    }
    let mut prev_dash = false;
    for ch in chars {
        if ch == '-' {
            if prev_dash {
                return Err(format!("{label} cannot contain repeated dashes"));
            }
            prev_dash = true;
            continue;
        }
        prev_dash = false;
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() {
            return Err(format!(
                "{label} must use lowercase letters, numbers and dashes"
            ));
        }
    }
    if id.ends_with('-') {
        return Err(format!("{label} cannot end with a dash"));
    }
    Ok(())
}

fn kind_string(kind: &EditorUiNodeCreateKindDto) -> &'static str {
    match kind {
        EditorUiNodeCreateKindDto::Column => "column",
        EditorUiNodeCreateKindDto::Row => "row",
        EditorUiNodeCreateKindDto::Panel => "panel",
        EditorUiNodeCreateKindDto::Stack => "stack",
        EditorUiNodeCreateKindDto::Spacer => "spacer",
        EditorUiNodeCreateKindDto::Text => "text",
        EditorUiNodeCreateKindDto::Button => "button",
        EditorUiNodeCreateKindDto::Image => "image",
        EditorUiNodeCreateKindDto::ProgressBar => "progress-bar",
    }
}

fn default_text(kind: &EditorUiNodeCreateKindDto) -> Option<&'static str> {
    match kind {
        EditorUiNodeCreateKindDto::Text => Some("New Text"),
        EditorUiNodeCreateKindDto::Button => Some("New Button"),
        EditorUiNodeCreateKindDto::ProgressBar => Some("100 / 100"),
        _ => None,
    }
}

fn string_field(mapping: &Mapping, key_name: &str) -> Option<String> {
    mapping
        .get(key(key_name))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn mapping_string(value: &Value, key_name: &str) -> Option<String> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(key(key_name)))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn key(name: &str) -> Value {
    Value::String(name.to_owned())
}

fn number(value: f32) -> Value {
    serde_yaml::to_value(value).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> Value {
        serde_yaml::from_str(
            r#"
version: 1
scene: { id: main-menu }
entities:
  - id: main-menu-ui
    components:
      - type: UiDocument
        root:
          type: column
          id: root
          children:
            - type: panel
              id: menu-card
              children:
                - type: button
                  id: start
                  text: START
                  on_click: start_game
                - type: button
                  id: options
                  text: OPTIONS
            - type: text
              id: title
              text: TITLE
"#,
        )
        .unwrap()
    }

    #[test]
    fn adds_button_under_panel() {
        let mut document = document();
        let outcome = patch_add_ui_node(
            &mut document,
            "main-menu-ui",
            0,
            "root.menu-card",
            EditorUiNodeCreateDto {
                kind: EditorUiNodeCreateKindDto::Button,
                id: "new-button".to_owned(),
                label: None,
                text: Some("BEGIN".to_owned()),
            },
            None,
        )
        .unwrap();

        let text = serde_yaml::to_string(&document).unwrap();
        assert!(text.contains("new-button"));
        assert_eq!(
            outcome.selected_ui_node.unwrap().node_path,
            "root.menu-card.new-button"
        );
    }

    #[test]
    fn rejects_duplicate_child_id() {
        let mut document = document();
        let error = patch_add_ui_node(
            &mut document,
            "main-menu-ui",
            0,
            "root.menu-card",
            EditorUiNodeCreateDto {
                kind: EditorUiNodeCreateKindDto::Button,
                id: "start".to_owned(),
                label: None,
                text: None,
            },
            None,
        )
        .unwrap_err();
        assert!(error.contains("already exists"));
    }

    #[test]
    fn rejects_child_under_text() {
        let mut document = document();
        let error = patch_add_ui_node(
            &mut document,
            "main-menu-ui",
            0,
            "root.title",
            EditorUiNodeCreateDto {
                kind: EditorUiNodeCreateKindDto::Button,
                id: "bad".to_owned(),
                label: None,
                text: None,
            },
            None,
        )
        .unwrap_err();
        assert!(error.contains("cannot have children"));
    }

    #[test]
    fn adds_vertical_menu_template() {
        let mut document = document();
        let outcome = patch_add_ui_template(
            &mut document,
            "main-menu-ui",
            0,
            "root",
            EditorUiTemplateKindDto::VerticalMenu,
            "menu",
            None,
        )
        .unwrap();
        let text = serde_yaml::to_string(&document).unwrap();
        assert!(text.contains("menu-main-menu-screen"));
        assert!(text.contains("menu-menu-container"));
        assert!(text.contains("event: navigate"));
        assert!(text.contains("target: menu-options-screen"));
        assert_eq!(
            outcome.selected_ui_node.unwrap().node_path,
            "root.menu-main-menu-screen"
        );
    }

    #[test]
    fn duplicates_node_without_on_click() {
        let mut document = document();
        let outcome = patch_duplicate_ui_node(
            &mut document,
            "main-menu-ui",
            0,
            "root.menu-card.start",
            None,
            false,
        )
        .unwrap();
        let text = serde_yaml::to_string(&document).unwrap();
        assert!(text.contains("start-copy"));
        assert_eq!(text.matches("on_click").count(), 1);
        assert_eq!(
            outcome.selected_ui_node.unwrap().node_path,
            "root.menu-card.start-copy"
        );
    }

    #[test]
    fn removes_node_and_selects_parent() {
        let mut document = document();
        let outcome =
            patch_remove_ui_node(&mut document, "main-menu-ui", 0, "root.menu-card.options")
                .unwrap();
        let text = serde_yaml::to_string(&document).unwrap();
        assert!(!text.contains("OPTIONS"));
        assert_eq!(
            outcome.selected_ui_node.unwrap().node_path,
            "root.menu-card"
        );
    }

    #[test]
    fn does_not_remove_root() {
        let mut document = document();
        assert!(patch_remove_ui_node(&mut document, "main-menu-ui", 0, "root").is_err());
    }

    #[test]
    fn moves_node_up_and_down() {
        let mut document = document();
        patch_move_ui_node(
            &mut document,
            "main-menu-ui",
            0,
            "root.menu-card.options",
            EditorUiNodeMoveDirectionDto::Up,
        )
        .unwrap();
        let text = serde_yaml::to_string(&document).unwrap();
        assert!(text.find("options").unwrap() < text.find("start").unwrap());

        patch_move_ui_node(
            &mut document,
            "main-menu-ui",
            0,
            "root.menu-card.options",
            EditorUiNodeMoveDirectionDto::Down,
        )
        .unwrap();
        let text = serde_yaml::to_string(&document).unwrap();
        assert!(text.find("start").unwrap() < text.find("options").unwrap());
    }

    #[test]
    fn creates_ui_document_empty() {
        let mut document: Value =
            serde_yaml::from_str("version: 1\nscene: { id: main-menu }\n").unwrap();
        let outcome = patch_create_ui_document(
            &mut document,
            "main-ui",
            "Main UI",
            1280.0,
            720.0,
            EditorUiTemplateKindDto::EmptyDocument,
        )
        .unwrap();
        let text = serde_yaml::to_string(&document).unwrap();
        assert!(text.contains("UiDocument"));
        assert!(text.contains("main-ui"));
        assert!(text.contains("layer: menu"));
        assert_eq!(outcome.selected_ui_node.unwrap().node_path, "root");
    }

    #[test]
    fn creates_ui_document_vertical_menu() {
        let mut document: Value =
            serde_yaml::from_str("version: 1\nscene: { id: main-menu }\n").unwrap();
        patch_create_ui_document(
            &mut document,
            "main-ui",
            "Main UI",
            1280.0,
            720.0,
            EditorUiTemplateKindDto::VerticalMenu,
        )
        .unwrap();
        let text = serde_yaml::to_string(&document).unwrap();
        assert!(text.contains("menu-main-menu-screen"));
        assert!(text.contains("menu-menu-container"));
        assert!(text.contains("event: navigate"));
        assert!(text.contains("target: menu-options-screen"));
    }
}
