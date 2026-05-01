    use crate::{
        UiDrawPrimitive, UiOverlayCurvePoint, UiOverlayDocument, UiOverlayLayer, UiOverlayNode,
        UiOverlayNodeKind, UiOverlayStyle, UiViewportSize, build_ui_layout_tree,
        build_ui_overlay_primitives,
    };
    use amigo_math::ColorRgba;

    #[test]
    fn color_picker_rgb_builds_channel_primitives() {
        let document = UiOverlayDocument {
            entity_name: "ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::ColorPickerRgb {
                    color: ColorRgba::new(0.25, 0.5, 0.75, 1.0),
                },
                style: UiOverlayStyle {
                    left: Some(0.0),
                    top: Some(0.0),
                    width: Some(260.0),
                    height: Some(118.0),
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            },
        };

        let primitives =
            build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[document]);
        for label in ["R", "G", "B"] {
            assert!(primitives.iter().any(|primitive| matches!(
                primitive,
                UiDrawPrimitive::Text { content, .. } if content == label
            )));
        }
    }

    #[test]
    fn curve_editor_builds_fallback_primitives() {
        let document = UiOverlayDocument {
            entity_name: "ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("curve".to_owned()),
                kind: UiOverlayNodeKind::CurveEditor {
                    points: vec![
                        UiOverlayCurvePoint { t: 0.0, value: 0.0 },
                        UiOverlayCurvePoint { t: 0.5, value: 1.0 },
                        UiOverlayCurvePoint {
                            t: 1.0,
                            value: 0.25,
                        },
                    ],
                },
                style: UiOverlayStyle {
                    width: Some(260.0),
                    height: Some(118.0),
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            },
        };

        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &document);
        assert_eq!(layout.rect.height, 118.0);

        let primitives =
            build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[document]);
        assert!(
            primitives
                .iter()
                .filter(|primitive| matches!(primitive, UiDrawPrimitive::Quad { .. }))
                .count()
                >= 8
        );
    }

