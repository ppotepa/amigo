    use crate::{
        UiDrawPrimitive, UiOverlayDocument, UiOverlayLayer, UiOverlayNode,
        UiOverlayNodeKind, UiOverlayStyle, UiViewportSize, build_ui_layout_tree,
        build_ui_overlay_primitives,
    };
    

    #[test]
    fn expanded_dropdown_does_not_push_sibling_layout() {
        let document = UiOverlayDocument {
            entity_name: "ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Column,
                style: UiOverlayStyle {
                    left: Some(0.0),
                    top: Some(0.0),
                    width: Some(260.0),
                    gap: 8.0,
                    ..UiOverlayStyle::default()
                },
                children: vec![
                    UiOverlayNode {
                        id: Some("dropdown".to_owned()),
                        kind: UiOverlayNodeKind::Dropdown {
                            selected: "A".to_owned(),
                            options: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
                            expanded: true,
                            scroll_offset: 0.0,
                            font: None,
                        },
                        style: UiOverlayStyle {
                            width: Some(220.0),
                            height: Some(38.0),
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                    UiOverlayNode {
                        id: Some("button".to_owned()),
                        kind: UiOverlayNodeKind::Button {
                            text: "Below".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle {
                            width: Some(220.0),
                            height: Some(40.0),
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                ],
            },
        };

        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &document);
        assert_eq!(layout.children[0].rect.height, 38.0);
        assert_eq!(layout.children[1].rect.y, 46.0);

        let primitives =
            build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[document]);
        let text_order = primitives
            .into_iter()
            .filter_map(|primitive| match primitive {
                UiDrawPrimitive::Text { content, .. } => Some(content),
                _ => None,
            })
            .collect::<Vec<_>>();
        let below = text_order
            .iter()
            .position(|content| content == "Below")
            .expect("button text should render");
        let popup_option = text_order
            .iter()
            .rposition(|content| content == "A")
            .expect("dropdown popup option should render");
        assert!(
            popup_option > below,
            "dropdown popup should render after normal sibling primitives"
        );
    }

    #[test]
    fn expanded_dropdown_limits_popup_rows_and_uses_scroll_offset() {
        let options = (0..14)
            .map(|index| format!("option-{index:02}"))
            .collect::<Vec<_>>();
        let document = UiOverlayDocument {
            entity_name: "ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Dropdown {
                    selected: "option-04".to_owned(),
                    options,
                    expanded: true,
                    scroll_offset: 4.5,
                    font: None,
                },
                style: UiOverlayStyle {
                    left: Some(0.0),
                    top: Some(0.0),
                    width: Some(220.0),
                    height: Some(38.0),
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            },
        };

        let primitives =
            build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[document]);
        let scrollbar_quads = primitives
            .iter()
            .filter(|primitive| match primitive {
                UiDrawPrimitive::Quad { rect, .. } => rect.x >= 210.0 && rect.width <= 10.0,
                _ => false,
            })
            .count();
        let labels = primitives
            .into_iter()
            .filter_map(|primitive| match primitive {
                UiDrawPrimitive::Text { content, .. } => Some(content),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(labels.iter().any(|label| label == "option-04"));
        assert!(labels.iter().any(|label| label == "option-13"));
        assert!(!labels.iter().any(|label| label == "option-03"));
        assert!(
            scrollbar_quads >= 2,
            "long dropdown popup should render scrollbar track and thumb"
        );
    }

