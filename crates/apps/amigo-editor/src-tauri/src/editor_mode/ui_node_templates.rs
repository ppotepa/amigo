use serde_yaml::{Mapping, Value};

use crate::editor_mode::dto::EditorUiTemplateKindDto;

/// @codemap P1 editor_mode.ui_node_templates
pub fn make_template_subtree(
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
    action_kind: &str,
    target: &str,
) -> Value {
    let mut node = match button(id, text_value, width, height) {
        Value::Mapping(mapping) => mapping,
        _ => Mapping::new(),
    };
    let mut action = Mapping::new();
    action.insert(key("kind"), Value::String(action_kind.to_owned()));
    action.insert(key("target"), Value::String(target.to_owned()));
    node.insert(key("action"), Value::Mapping(action));
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
    let mut mapping = Mapping::new();
    for (name, value) in fields {
        mapping.insert(key(name), number(*value));
    }
    mapping
}

fn key(name: &str) -> Value {
    Value::String(name.to_owned())
}

fn number(value: f32) -> Value {
    serde_yaml::to_value(value).unwrap_or(Value::Null)
}
