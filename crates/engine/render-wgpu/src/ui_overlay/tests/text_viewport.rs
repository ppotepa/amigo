    use crate::{
        UiOverlayDocument, UiOverlayLayer, UiOverlayNode,
        UiOverlayNodeKind, UiOverlayStyle, UiOverlayViewport,
        UiOverlayViewportScaling, UiViewportSize, build_ui_layout_tree,
    };
    

    #[test]
    fn wrapped_text_increases_layout_height() {
        let document = UiOverlayDocument {
            entity_name: "debug-ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Column,
                style: UiOverlayStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(200.0),
                    padding: 12.0,
                    gap: 8.0,
                    ..UiOverlayStyle::default()
                },
                children: vec![
                    UiOverlayNode {
                        id: Some("debug".to_owned()),
                        kind: UiOverlayNodeKind::Text {
                            content: "grounded=false vx=120 vy=-10 anim=run".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle {
                            width: Some(176.0),
                            font_size: 14.0,
                            word_wrap: true,
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                    UiOverlayNode {
                        id: Some("message".to_owned()),
                        kind: UiOverlayNodeKind::Text {
                            content: "READY".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle::default(),
                        children: Vec::new(),
                    },
                ],
            },
        };

        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &document);
        assert!(layout.children[0].rect.height > 14.0 * 1.2);
        assert!(
            layout.children[1].rect.y >= layout.children[0].rect.y + layout.children[0].rect.height
        );
    }

    #[test]
    fn fixed_fit_viewport_centers_and_scales_design_layout() {
        let document = UiOverlayDocument {
            entity_name: "fixed-ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: Some(UiOverlayViewport {
                width: 1440.0,
                height: 900.0,
                scaling: UiOverlayViewportScaling::Fit,
            }),
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Panel,
                style: UiOverlayStyle {
                    left: Some(24.0),
                    top: Some(18.0),
                    width: Some(1392.0),
                    height: Some(72.0),
                    font_size: 20.0,
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            },
        };

        let layout = build_ui_layout_tree(UiViewportSize::new(1920.0, 1080.0), &document);

        assert!((layout.rect.x - 124.8).abs() < 0.001);
        assert!((layout.rect.y - 21.6).abs() < 0.001);
        assert!((layout.rect.width - 1670.4).abs() < 0.001);
        assert!((layout.rect.height - 86.4).abs() < 0.001);
    }

