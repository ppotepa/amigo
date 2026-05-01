use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_core::AmigoResult;
use amigo_math::ColorRgba;
use amigo_scene::SceneEntityId;

use crate::layout::UiLayoutService;
use crate::model::{UiCurvePoint, UiDocument, UiTheme, normalize_curve_points};

#[derive(Debug, Clone, PartialEq)]
pub struct UiDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub document: UiDocument,
}

#[derive(Debug, Default)]
pub struct UiSceneService {
    commands: Mutex<Vec<UiDrawCommand>>,
}

impl UiSceneService {
    pub fn queue(&self, command: UiDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("ui scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("ui scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<UiDrawCommand> {
        self.commands
            .lock()
            .expect("ui scene service mutex should not be poisoned")
            .clone()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UiStateSnapshot {
    pub text_overrides: BTreeMap<String, String>,
    pub value_overrides: BTreeMap<String, f32>,
    pub curve_overrides: BTreeMap<String, Vec<UiCurvePoint>>,
    pub selected_overrides: BTreeMap<String, String>,
    pub options_overrides: BTreeMap<String, Vec<String>>,
    pub expanded_overrides: BTreeMap<String, bool>,
    pub dropdown_scroll_offsets: BTreeMap<String, f32>,
    pub color_overrides: BTreeMap<String, ColorRgba>,
    pub background_overrides: BTreeMap<String, ColorRgba>,
    pub visibility_overrides: BTreeMap<String, bool>,
    pub enabled_overrides: BTreeMap<String, bool>,
}

#[derive(Debug, Default)]
pub struct UiStateService {
    state: Mutex<UiStateSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiModelBindingKind {
    Text,
    Value,
    Visible,
    Enabled,
    Selected,
    Options,
    Color,
    Background,
    Theme,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiModelBinding {
    pub path: String,
    pub state_key: String,
    pub kind: UiModelBindingKind,
    pub format: Option<String>,
}

#[derive(Debug, Default)]
pub struct UiModelBindingService {
    bindings: Mutex<Vec<UiModelBinding>>,
}

#[derive(Debug, Default)]
pub struct UiThemeService {
    themes: Mutex<BTreeMap<String, UiTheme>>,
    active_theme_id: Mutex<Option<String>>,
}

impl UiThemeService {
    pub fn register_theme(&self, theme: UiTheme) -> bool {
        let mut themes = self
            .themes
            .lock()
            .expect("ui theme service mutex should not be poisoned");
        let changed = themes.get(&theme.id) != Some(&theme);
        themes.insert(theme.id.clone(), theme);
        changed
    }

    pub fn set_active_theme(&self, theme_id: &str) -> bool {
        if !self
            .themes
            .lock()
            .expect("ui theme service mutex should not be poisoned")
            .contains_key(theme_id)
        {
            return false;
        }
        let mut active = self
            .active_theme_id
            .lock()
            .expect("ui theme active mutex should not be poisoned");
        if active.as_deref() == Some(theme_id) {
            return false;
        }
        *active = Some(theme_id.to_owned());
        true
    }

    pub fn active_theme_id(&self) -> Option<String> {
        self.active_theme_id
            .lock()
            .expect("ui theme active mutex should not be poisoned")
            .clone()
    }

    pub fn active_theme(&self) -> Option<UiTheme> {
        let active = self.active_theme_id()?;
        self.themes
            .lock()
            .expect("ui theme service mutex should not be poisoned")
            .get(&active)
            .cloned()
    }

    pub fn themes(&self) -> Vec<UiTheme> {
        self.themes
            .lock()
            .expect("ui theme service mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.themes
            .lock()
            .expect("ui theme service mutex should not be poisoned")
            .clear();
        *self
            .active_theme_id
            .lock()
            .expect("ui theme active mutex should not be poisoned") = None;
    }
}

impl UiStateService {
    pub fn set_text(&self, path: impl Into<String>, value: impl Into<String>) -> bool {
        let path = path.into();
        let value = value.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.text_overrides.get(&path) == Some(&value) {
            return false;
        }
        state.text_overrides.insert(path, value);
        true
    }

    pub fn set_value(&self, path: impl Into<String>, value: f32) -> bool {
        let path = path.into();
        let value = value.clamp(0.0, 1.0);
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.value_overrides.get(&path).copied() == Some(value) {
            return false;
        }
        state.value_overrides.insert(path, value);
        true
    }

    pub fn set_curve_points(&self, path: impl Into<String>, points: Vec<UiCurvePoint>) -> bool {
        let path = path.into();
        let points = normalize_curve_points(&points);
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.curve_overrides.get(&path) == Some(&points) {
            return false;
        }
        state.curve_overrides.insert(path, points);
        true
    }

    pub fn set_selected(&self, path: impl Into<String>, value: impl Into<String>) -> bool {
        let path = path.into();
        let value = value.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.selected_overrides.get(&path) == Some(&value) {
            return false;
        }
        state.selected_overrides.insert(path, value);
        true
    }

    pub fn set_options(&self, path: impl Into<String>, options: Vec<String>) -> bool {
        let path = path.into();
        let options = options
            .into_iter()
            .filter(|option| !option.is_empty())
            .collect::<Vec<_>>();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.options_overrides.get(&path) == Some(&options) {
            return false;
        }
        if let Some(selected) = state.selected_overrides.get(&path) {
            if !options.contains(selected) {
                if let Some(first) = options.first() {
                    state.selected_overrides.insert(path.clone(), first.clone());
                }
            }
        }
        state.options_overrides.insert(path, options);
        true
    }

    pub fn set_curve(&self, path: impl Into<String>, values: Vec<f32>) -> bool {
        self.set_curve_points(path, crate::model::curve_points_from_values(&values))
    }

    pub fn set_expanded(&self, path: impl Into<String>, value: bool) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.expanded_overrides.get(&path).copied() == Some(value) {
            return false;
        }
        state.expanded_overrides.insert(path, value);
        true
    }

    pub fn set_dropdown_scroll_offset(
        &self,
        path: impl Into<String>,
        offset: f32,
        option_count: usize,
        visible_count: usize,
    ) -> bool {
        let path = path.into();
        let max_offset = option_count.saturating_sub(visible_count.max(1)) as f32;
        let offset = if offset.is_finite() {
            offset.clamp(0.0, max_offset)
        } else {
            0.0
        };
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.dropdown_scroll_offsets.get(&path).copied() == Some(offset) {
            return false;
        }
        state.dropdown_scroll_offsets.insert(path, offset);
        true
    }

    pub fn set_color(&self, path: impl Into<String>, color: ColorRgba) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.color_overrides.get(&path).copied() == Some(color) {
            return false;
        }
        state.color_overrides.insert(path, color);
        true
    }

    pub fn set_background(&self, path: impl Into<String>, color: ColorRgba) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.background_overrides.get(&path).copied() == Some(color) {
            return false;
        }
        state.background_overrides.insert(path, color);
        true
    }

    pub fn clear_background(&self, path: &str) -> bool {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .background_overrides
            .remove(path)
            .is_some()
    }

    pub fn show(&self, path: impl Into<String>) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.visibility_overrides.get(&path).copied() == Some(true) {
            return false;
        }
        state.visibility_overrides.insert(path, true);
        true
    }

    pub fn hide(&self, path: impl Into<String>) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.visibility_overrides.get(&path).copied() == Some(false) {
            return false;
        }
        state.visibility_overrides.insert(path, false);
        true
    }

    pub fn enable(&self, path: impl Into<String>) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.enabled_overrides.get(&path).copied() == Some(true) {
            return false;
        }
        state.enabled_overrides.insert(path, true);
        true
    }

    pub fn disable(&self, path: impl Into<String>) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.enabled_overrides.get(&path).copied() == Some(false) {
            return false;
        }
        state.enabled_overrides.insert(path, false);
        true
    }

    pub fn text_override(&self, path: &str) -> Option<String> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .text_overrides
            .get(path)
            .cloned()
    }

    pub fn value_override(&self, path: &str) -> Option<f32> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .value_overrides
            .get(path)
            .copied()
    }

    pub fn curve_override(&self, path: &str) -> Option<Vec<UiCurvePoint>> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .curve_overrides
            .get(path)
            .cloned()
    }

    pub fn selected_override(&self, path: &str) -> Option<String> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .selected_overrides
            .get(path)
            .cloned()
    }

    pub fn options_override(&self, path: &str) -> Option<Vec<String>> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .options_overrides
            .get(path)
            .cloned()
    }

    pub fn expanded_override(&self, path: &str) -> Option<bool> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .expanded_overrides
            .get(path)
            .copied()
    }

    pub fn dropdown_scroll_offset(&self, path: &str) -> f32 {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .dropdown_scroll_offsets
            .get(path)
            .copied()
            .unwrap_or(0.0)
    }

    pub fn color_override(&self, path: &str) -> Option<ColorRgba> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .color_overrides
            .get(path)
            .copied()
    }

    pub fn background_override(&self, path: &str) -> Option<ColorRgba> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .background_overrides
            .get(path)
            .copied()
    }

    pub fn is_visible(&self, path: &str) -> bool {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .visibility_overrides
            .get(path)
            .copied()
            .unwrap_or(true)
    }

    pub fn is_enabled(&self, path: &str) -> bool {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .enabled_overrides
            .get(path)
            .copied()
            .unwrap_or(true)
    }

    pub fn snapshot(&self) -> UiStateSnapshot {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .clone()
    }

    pub fn clear(&self) {
        *self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned") = UiStateSnapshot::default();
    }
}

impl UiModelBindingService {
    pub fn queue(&self, binding: UiModelBinding) {
        let mut bindings = self
            .bindings
            .lock()
            .expect("ui model binding service mutex should not be poisoned");
        if let Some(existing) = bindings
            .iter_mut()
            .find(|existing| existing.path == binding.path && existing.kind == binding.kind)
        {
            *existing = binding;
        } else {
            bindings.push(binding);
        }
    }

    pub fn bindings(&self) -> Vec<UiModelBinding> {
        self.bindings
            .lock()
            .expect("ui model binding service mutex should not be poisoned")
            .clone()
    }

    pub fn clear(&self) {
        self.bindings
            .lock()
            .expect("ui model binding service mutex should not be poisoned")
            .clear();
    }
}

#[derive(Debug, Clone)]
pub struct UiDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub fn register_ui_services(registry: &mut amigo_runtime::ServiceRegistry) -> AmigoResult<()> {
    registry.register(UiSceneService::default())?;
    registry.register(UiStateService::default())?;
    registry.register(UiModelBindingService::default())?;
    registry.register(UiThemeService::default())?;
    registry.register(crate::input::UiInputService::default())?;
    registry.register(UiLayoutService)?;
    registry.register(UiDomainInfo {
        crate_name: "amigo-ui",
        capability: "screen_space_ui",
    })
}

#[cfg(test)]
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
