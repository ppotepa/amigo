use std::sync::Arc;

use amigo_input_actions::InputActionService;
use amigo_input_api::InputState;

#[derive(Clone)]
pub struct ActionsApi {
    pub(crate) actions: Option<Arc<InputActionService>>,
    pub(crate) input_state: Option<Arc<InputState>>,
}

impl ActionsApi {
    pub fn axis(&mut self, action: &str) -> rhai::FLOAT {
        let (Some(actions), Some(input_state)) = (self.actions.as_ref(), self.input_state.as_ref())
        else {
            return 0.0;
        };
        actions.axis(input_state, action) as rhai::FLOAT
    }

    pub fn down(&mut self, action: &str) -> bool {
        let (Some(actions), Some(input_state)) = (self.actions.as_ref(), self.input_state.as_ref())
        else {
            return false;
        };
        actions.down(input_state, action)
    }

    pub fn pressed(&mut self, action: &str) -> bool {
        let (Some(actions), Some(input_state)) = (self.actions.as_ref(), self.input_state.as_ref())
        else {
            return false;
        };
        actions.pressed(input_state, action)
    }

    pub fn set_active_map(&mut self, map_id: &str) -> bool {
        self.actions
            .as_ref()
            .map(|actions| actions.set_active_map(map_id))
            .unwrap_or(false)
    }

    pub fn active_map(&mut self) -> String {
        self.actions
            .as_ref()
            .and_then(|actions| actions.active_map_id())
            .unwrap_or_default()
    }
}
