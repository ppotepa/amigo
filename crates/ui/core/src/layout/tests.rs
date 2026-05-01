mod tests {
    use super::{UiLayoutService, compute_layout, hit_test};
    use crate::model::{
        UiCurvePoint, UiDocument, UiEventBinding, UiEvents, UiLayer, UiNode, UiNodeKind, UiRect,
        UiStyle, UiTab, curve_editor_edit_from_mouse,
    };

    #[test]
    fn computes_column_layout() {
        let document = UiDocument::screen_space(
            UiLayer::Hud,
            UiNode::new(UiNodeKind::Column)
                .with_style(UiStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(200.0),
                    padding: 16.0,
                    gap: 8.0,
                    ..UiStyle::default()
                })
                .with_children(vec![
                    UiNode::new(UiNodeKind::Text {
                        content: "Title".to_owned(),
                        font: None,
                    })
                    .with_id("title")
                    .with_style(UiStyle {
                        height: Some(20.0),
                        ..UiStyle::default()
                    }),
                    UiNode::new(UiNodeKind::Button {
                        text: "Click".to_owned(),
                        font: None,
                    })
                    .with_id("button")
                    .with_style(UiStyle {
                        height: Some(40.0),
                        ..UiStyle::default()
                    }),
                ]),
        );

        let layout = compute_layout(&document, UiRect::new(0.0, 0.0, 1280.0, 720.0));

        assert_eq!(layout.rect, UiRect::new(24.0, 24.0, 200.0, 100.0));
        assert_eq!(layout.children.len(), 2);
        assert_eq!(layout.children[0].path, "root.title");
        assert_eq!(
            layout.children[0].rect,
            UiRect::new(40.0, 40.0, 168.0, 20.0)
        );
        assert_eq!(
            layout.children[1].rect,
            UiRect::new(40.0, 68.0, 168.0, 40.0)
        );
    }

    #[test]
    fn hit_tests_button() {
        let document = UiDocument::screen_space(
            UiLayer::Hud,
            UiNode::new(UiNodeKind::Column).with_children(vec![
                UiNode::new(UiNodeKind::Button {
                    text: "Emit".to_owned(),
                    font: None,
                })
                .with_id("action-button")
                .with_style(UiStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(220.0),
                    height: Some(42.0),
                    ..UiStyle::default()
                })
                .with_events(UiEvents {
                    on_click: Some(UiEventBinding::new(
                        "playground-2d.ui-preview.button-clicked",
                        Vec::new(),
                    )),
                    on_change: None,
                }),
            ]),
        );

        let service = UiLayoutService;
        let layout = service.compute(&document, UiRect::new(0.0, 0.0, 1280.0, 720.0));

        assert_eq!(
            hit_test(&layout, 40.0, 40.0).as_deref(),
            Some("root.action-button")
        );
        assert_eq!(hit_test(&layout, 400.0, 400.0), None);
    }

    #[test]
    fn wrapped_text_increases_column_layout_height() {
        let document = UiDocument::screen_space(
            UiLayer::Hud,
            UiNode::new(UiNodeKind::Column)
                .with_style(UiStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(200.0),
                    padding: 16.0,
                    gap: 8.0,
                    ..UiStyle::default()
                })
                .with_children(vec![
                    UiNode::new(UiNodeKind::Text {
                        content: "grounded=false vx=120 vy=-10 anim=run".to_owned(),
                        font: None,
                    })
                    .with_id("debug")
                    .with_style(UiStyle {
                        width: Some(168.0),
                        font_size: 14.0,
                        word_wrap: true,
                        ..UiStyle::default()
                    }),
                    UiNode::new(UiNodeKind::Text {
                        content: "READY".to_owned(),
                        font: None,
                    })
                    .with_id("message"),
                ]),
        );

        let layout = compute_layout(&document, UiRect::new(0.0, 0.0, 1280.0, 720.0));

        assert!(layout.children[0].rect.height > 14.0 * 1.4);
        assert!(
            layout.children[1].rect.y >= layout.children[0].rect.y + layout.children[0].rect.height
        );
    }

    #[test]
    fn tab_view_lays_out_only_selected_panel() {
        let document = UiDocument::screen_space(
            UiLayer::Hud,
            UiNode::new(UiNodeKind::TabView {
                selected: "settings".to_owned(),
                tabs: vec![
                    UiTab {
                        id: "overview".to_owned(),
                        label: "Overview".to_owned(),
                    },
                    UiTab {
                        id: "settings".to_owned(),
                        label: "Settings".to_owned(),
                    },
                ],
                font: None,
            })
            .with_id("tabs")
            .with_style(UiStyle {
                width: Some(300.0),
                height: Some(180.0),
                padding: 4.0,
                ..UiStyle::default()
            })
            .with_children(vec![
                UiNode::new(UiNodeKind::Text {
                    content: "Overview panel".to_owned(),
                    font: None,
                })
                .with_id("overview"),
                UiNode::new(UiNodeKind::Text {
                    content: "Settings panel".to_owned(),
                    font: None,
                })
                .with_id("settings"),
            ]),
        );

        let layout = compute_layout(&document, UiRect::new(0.0, 0.0, 1280.0, 720.0));

        assert_eq!(layout.children.len(), 1);
        assert_eq!(layout.children[0].path, "tabs.settings");
        assert!(layout.children[0].rect.y >= 38.0);
    }

    #[test]
    fn curve_editor_lays_out_and_edits_normalized_points() {
        let document = UiDocument::screen_space(
            UiLayer::Hud,
            UiNode::new(UiNodeKind::Column).with_children(vec![
                UiNode::new(UiNodeKind::CurveEditor {
                    points: vec![UiCurvePoint::new(0.0, 0.0), UiCurvePoint::new(0.5, 1.0)],
                })
                .with_id("curve")
                .with_style(UiStyle {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..UiStyle::default()
                }),
            ]),
        );

        let layout = compute_layout(&document, UiRect::new(0.0, 0.0, 1280.0, 720.0));
        let curve = &layout.children[0];

        assert_eq!(curve.path, "root.curve");
        assert_eq!(
            hit_test(&layout, 100.0, 50.0).as_deref(),
            Some("root.curve")
        );

        let edit = curve_editor_edit_from_mouse(curve.rect, &curve_points(curve), 100.0, 25.0)
            .expect("curve edit should be produced");
        assert_eq!(edit.points.len(), 4);
        assert!((edit.point.t - 0.5).abs() <= 0.01);
        assert!((edit.point.value - 0.75).abs() <= 0.01);
        assert_eq!(edit.payload().len(), 8);
    }

    fn curve_points(layout: &crate::model::UiLayoutNode) -> Vec<UiCurvePoint> {
        match &layout.node.kind {
            UiNodeKind::CurveEditor { points } => points.clone(),
            _ => Vec::new(),
        }
    }
}
