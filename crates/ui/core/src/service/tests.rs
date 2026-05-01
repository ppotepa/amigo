mod tests {
    use super::{UiDrawCommand, UiSceneService, UiStateService, UiThemeService};
    use crate::model::{UiDocument, UiLayer, UiNode, UiNodeKind, UiTheme, UiThemePalette};
    use amigo_math::ColorRgba;
    use amigo_scene::SceneEntityId;

    #[test]
    fn stores_ui_draw_commands() {
        let service = UiSceneService::default();
        service.queue(UiDrawCommand {
            entity_id: SceneEntityId::new(3),
            entity_name: "playground-2d-ui-preview".to_owned(),
            document: UiDocument::screen_space(UiLayer::Hud, UiNode::new(UiNodeKind::Panel)),
        });

        assert_eq!(service.commands().len(), 1);
        assert_eq!(
            service.entity_names(),
            vec!["playground-2d-ui-preview".to_owned()]
        );

        service.clear();
        assert!(service.commands().is_empty());
    }

    #[test]
    fn updates_ui_state() {
        let service = UiStateService::default();
        let subtitle = "playground-2d-ui-preview.subtitle";
        let bar = "playground-2d-ui-preview.hp-bar";

        service.set_text(subtitle, "Updated from Rhai");
        service.set_value(bar, 0.5);
        service.hide("playground-2d-ui-preview.root");
        service.disable("playground-2d-ui-preview.action-button");

        assert_eq!(
            service.text_override(subtitle).as_deref(),
            Some("Updated from Rhai")
        );
        assert_eq!(service.value_override(bar), Some(0.5));
        assert!(!service.is_visible("playground-2d-ui-preview.root"));
        assert!(!service.is_enabled("playground-2d-ui-preview.action-button"));

        service.show("playground-2d-ui-preview.root");
        service.enable("playground-2d-ui-preview.action-button");
        assert!(service.is_visible("playground-2d-ui-preview.root"));
        assert!(service.is_enabled("playground-2d-ui-preview.action-button"));
    }

    #[test]
    fn updates_ui_options_and_repairs_invalid_selection() {
        let service = UiStateService::default();
        let dropdown = "playground-2d-ui-preview.preset-dropdown";

        service.set_selected(dropdown, "missing");
        assert!(service.set_options(
            dropdown,
            vec!["fire".to_owned(), "smoke".to_owned(), "rain".to_owned()]
        ));

        assert_eq!(
            service.options_override(dropdown),
            Some(vec![
                "fire".to_owned(),
                "smoke".to_owned(),
                "rain".to_owned()
            ])
        );
        assert_eq!(service.selected_override(dropdown).as_deref(), Some("fire"));
    }

    #[test]
    fn clamps_dropdown_scroll_offset_to_visible_range() {
        let service = UiStateService::default();
        let dropdown = "playground-2d-ui-preview.preset-dropdown";

        assert!(service.set_dropdown_scroll_offset(dropdown, 99.0, 14, 10));
        assert_eq!(service.dropdown_scroll_offset(dropdown), 4.0);

        assert!(service.set_dropdown_scroll_offset(dropdown, 2.5, 14, 10));
        assert_eq!(service.dropdown_scroll_offset(dropdown), 2.5);
    }

    #[test]
    fn registers_themes_and_switches_active_theme() {
        let service = UiThemeService::default();
        let theme = UiTheme::from_palette(
            "space_dark",
            UiThemePalette {
                background: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
                surface: ColorRgba::new(0.1, 0.1, 0.15, 1.0),
                surface_alt: ColorRgba::new(0.15, 0.15, 0.2, 1.0),
                text: ColorRgba::WHITE,
                text_muted: ColorRgba::new(0.6, 0.7, 0.8, 1.0),
                border: ColorRgba::new(0.2, 0.4, 0.6, 1.0),
                accent: ColorRgba::new(0.0, 0.8, 1.0, 1.0),
                accent_text: ColorRgba::new(0.0, 0.05, 0.08, 1.0),
                danger: ColorRgba::new(1.0, 0.1, 0.2, 1.0),
                warning: ColorRgba::new(1.0, 0.7, 0.0, 1.0),
                success: ColorRgba::new(0.2, 1.0, 0.5, 1.0),
            },
        );

        assert!(service.register_theme(theme));
        assert!(service.set_active_theme("space_dark"));
        assert_eq!(service.active_theme_id().as_deref(), Some("space_dark"));
        assert!(service.active_theme().is_some());
        assert!(!service.set_active_theme("missing"));
    }
}
