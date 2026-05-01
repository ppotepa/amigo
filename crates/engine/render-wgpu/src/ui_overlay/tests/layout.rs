    use crate::{
        UiOverlayDocument, UiOverlayLayer, UiOverlayNode,
        UiOverlayNodeKind, UiOverlayStyle, UiViewportSize, build_ui_layout_tree,
    };
    

    #[test]
    fn computes_column_layout() {
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
                    width: Some(200.0),
                    padding: 16.0,
                    gap: 12.0,
                    ..UiOverlayStyle::default()
                },
                children: vec![
                    UiOverlayNode {
                        id: Some("title".to_owned()),
                        kind: UiOverlayNodeKind::Text {
                            content: "AMIGO".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle {
                            font_size: 28.0,
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                    UiOverlayNode {
                        id: Some("bar".to_owned()),
                        kind: UiOverlayNodeKind::ProgressBar { value: 0.75 },
                        style: UiOverlayStyle {
                            width: Some(120.0),
                            height: Some(18.0),
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                ],
            },
        };

        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &document);
        assert_eq!(layout.path, "playground-2d-ui-preview.root");
        assert_eq!(layout.rect.x, 24.0);
        assert_eq!(layout.rect.y, 24.0);
        assert_eq!(layout.children[0].rect.x, 40.0);
        assert_eq!(layout.children[0].rect.y, 40.0);
        assert_eq!(layout.children[1].rect.x, 40.0);
        assert!(layout.children[1].rect.y > layout.children[0].rect.y);
    }

