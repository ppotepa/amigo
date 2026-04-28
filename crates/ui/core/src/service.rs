use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_core::AmigoResult;
use amigo_scene::SceneEntityId;

use crate::layout::UiLayoutService;
use crate::model::UiDocument;

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
    pub visibility_overrides: BTreeMap<String, bool>,
    pub enabled_overrides: BTreeMap<String, bool>,
}

#[derive(Debug, Default)]
pub struct UiStateService {
    state: Mutex<UiStateSnapshot>,
}

impl UiStateService {
    pub fn set_text(&self, path: impl Into<String>, value: impl Into<String>) {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .text_overrides
            .insert(path.into(), value.into());
    }

    pub fn set_value(&self, path: impl Into<String>, value: f32) {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .value_overrides
            .insert(path.into(), value.clamp(0.0, 1.0));
    }

    pub fn show(&self, path: impl Into<String>) {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .visibility_overrides
            .insert(path.into(), true);
    }

    pub fn hide(&self, path: impl Into<String>) {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .visibility_overrides
            .insert(path.into(), false);
    }

    pub fn enable(&self, path: impl Into<String>) {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .enabled_overrides
            .insert(path.into(), true);
    }

    pub fn disable(&self, path: impl Into<String>) {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .enabled_overrides
            .insert(path.into(), false);
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

#[derive(Debug, Clone)]
pub struct UiDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub fn register_ui_services(registry: &mut amigo_runtime::ServiceRegistry) -> AmigoResult<()> {
    registry.register(UiSceneService::default())?;
    registry.register(UiStateService::default())?;
    registry.register(crate::input::UiInputService::default())?;
    registry.register(UiLayoutService)?;
    registry.register(UiDomainInfo {
        crate_name: "amigo-ui",
        capability: "screen_space_ui",
    })
}

#[cfg(test)]
mod tests {
    use super::{UiDrawCommand, UiSceneService, UiStateService};
    use crate::model::{UiDocument, UiLayer, UiNode, UiNodeKind};
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
}
