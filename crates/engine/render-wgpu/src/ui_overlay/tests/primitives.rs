    use crate::{
        UiDrawPrimitive, UiOverlayDocument, UiOverlayLayer, UiOverlayNode,
        UiOverlayNodeKind, UiOverlayStyle, UiTextAnchor, UiViewportSize,
        build_ui_overlay_primitives,
    };
    use amigo_math::ColorRgba;

    #[test]
    fn builds_ui_primitives_for_button_and_progress_bar() {
        let document = UiOverlayDocument {
            entity_name: "playground-2d-ui-preview".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Column,
                style: UiOverlayStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(240.0),
                    padding: 12.0,
                    gap: 10.0,
                    background: Some(ColorRgba::new(0.1, 0.12, 0.18, 0.9)),
                    ..UiOverlayStyle::default()
                },
                children: vec![
                    UiOverlayNode {
                        id: Some("button".to_owned()),
                        kind: UiOverlayNodeKind::Button {
                            text: "Emit".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle {
                            width: Some(160.0),
                            height: Some(36.0),
                            background: Some(ColorRgba::new(0.2, 0.33, 0.66, 1.0)),
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                    UiOverlayNode {
                        id: Some("hp".to_owned()),
                        kind: UiOverlayNodeKind::ProgressBar { value: 0.5 },
                        style: UiOverlayStyle {
                            width: Some(180.0),
                            height: Some(18.0),
                            background: Some(ColorRgba::new(0.18, 0.2, 0.27, 1.0)),
                            color: Some(ColorRgba::new(0.4, 0.8, 0.53, 1.0)),
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                ],
            },
        };

        let primitives =
            build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[document]);
        assert!(primitives.iter().any(|primitive| matches!(
            primitive,
            UiDrawPrimitive::Text {
                content,
                anchor: UiTextAnchor::Center,
                ..
            } if content == "Emit"
        )));
        assert!(primitives.iter().any(|primitive| matches!(
            primitive,
            UiDrawPrimitive::ProgressBar { value, .. } if (*value - 0.5).abs() < f32::EPSILON
        )));
    }

    #[test]
    fn respects_layer_order_for_overlay_documents() {
        let background = UiOverlayDocument {
            entity_name: "background-ui".to_owned(),
            layer: UiOverlayLayer::Background,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Text {
                    content: "BACKGROUND".to_owned(),
                    font: None,
                },
                style: UiOverlayStyle {
                    left: Some(0.0),
                    top: Some(0.0),
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            },
        };
        let debug = UiOverlayDocument {
            entity_name: "debug-ui".to_owned(),
            layer: UiOverlayLayer::Debug,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Text {
                    content: "DEBUG".to_owned(),
                    font: None,
                },
                style: UiOverlayStyle {
                    left: Some(0.0),
                    top: Some(0.0),
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            },
        };

        let primitives =
            build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[debug, background]);
        let first_text = primitives
            .into_iter()
            .find_map(|primitive| match primitive {
                UiDrawPrimitive::Text { content, .. } => Some(content),
                _ => None,
            });

        assert_eq!(first_text.as_deref(), Some("BACKGROUND"));
    }

