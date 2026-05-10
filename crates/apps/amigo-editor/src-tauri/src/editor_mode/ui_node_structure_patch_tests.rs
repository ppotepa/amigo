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
        patch_remove_ui_node(&mut document, "main-menu-ui", 0, "root.menu-card.options").unwrap();
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
