    use crate::{
        UiDrawPrimitive, UiOverlayDocument, UiOverlayLayer, UiOverlayNode,
        UiOverlayNodeKind, UiOverlayStyle, UiOverlayTab, UiRect, UiViewportSize, build_ui_layout_tree,
        build_ui_overlay_primitives, tab_view_tab_from_mouse,
    };
    

    #[test]
    fn tab_view_lays_out_and_renders_only_selected_panel() {
        let document = UiOverlayDocument {
            entity_name: "ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("tabs".to_owned()),
                kind: UiOverlayNodeKind::TabView {
                    selected: "settings".to_owned(),
                    tabs: vec![
                        UiOverlayTab {
                            id: "overview".to_owned(),
                            label: "Overview".to_owned(),
                        },
                        UiOverlayTab {
                            id: "settings".to_owned(),
                            label: "Settings".to_owned(),
                        },
                    ],
                    font: None,
                },
                style: UiOverlayStyle {
                    width: Some(320.0),
                    height: Some(180.0),
                    padding: 4.0,
                    ..UiOverlayStyle::default()
                },
                children: vec![
                    UiOverlayNode {
                        id: Some("overview".to_owned()),
                        kind: UiOverlayNodeKind::Text {
                            content: "Overview panel".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle::default(),
                        children: Vec::new(),
                    },
                    UiOverlayNode {
                        id: Some("settings".to_owned()),
                        kind: UiOverlayNodeKind::Text {
                            content: "Settings panel".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle::default(),
                        children: Vec::new(),
                    },
                ],
            },
        };

        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &document);
        assert_eq!(layout.children.len(), 1);
        assert_eq!(layout.children[0].path, "ui.tabs.settings");

        let labels = build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[document])
            .into_iter()
            .filter_map(|primitive| match primitive {
                UiDrawPrimitive::Text { content, .. } => Some(content),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(labels.iter().any(|label| label == "Settings"));
        assert!(labels.iter().any(|label| label == "Settings panel"));
        assert!(!labels.iter().any(|label| label == "Overview panel"));
    }

    #[test]
    fn tab_view_hit_helper_selects_header_tab() {
        let node = UiOverlayNode {
            id: Some("tabs".to_owned()),
            kind: UiOverlayNodeKind::TabView {
                selected: "overview".to_owned(),
                tabs: Vec::new(),
                font: None,
            },
            style: UiOverlayStyle {
                width: Some(200.0),
                height: Some(120.0),
                ..UiOverlayStyle::default()
            },
            children: Vec::new(),
        };
        let tabs = vec![
            UiOverlayTab {
                id: "overview".to_owned(),
                label: "Overview".to_owned(),
            },
            UiOverlayTab {
                id: "settings".to_owned(),
                label: "Settings".to_owned(),
            },
        ];

        assert_eq!(
            tab_view_tab_from_mouse(
                UiRect::new(0.0, 0.0, 200.0, 120.0),
                &node,
                &tabs,
                150.0,
                10.0
            )
            .as_deref(),
            Some("settings")
        );
        assert_eq!(
            tab_view_tab_from_mouse(
                UiRect::new(0.0, 0.0, 200.0, 120.0),
                &node,
                &tabs,
                150.0,
                80.0
            ),
            None
        );
    }
