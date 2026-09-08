use crate::{ControlValue, RuntimeControlError, RuntimeControlProperty, RuntimeControlRegistry};

pub trait RuntimeControlProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn rebuild_registry(
        &self,
        registry: &mut RuntimeControlRegistry,
    ) -> Result<(), RuntimeControlError>;
    fn get(&self, path: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError>;
    /// Providers may capture one coherent domain snapshot for a whole panel.
    fn get_many(
        &self,
        paths: &[RuntimeControlProperty],
    ) -> Result<Vec<ControlValue>, RuntimeControlError> {
        paths.iter().map(|p| self.get(p)).collect()
    }
    fn set(
        &self,
        path: &RuntimeControlProperty,
        value: ControlValue,
    ) -> Result<(), RuntimeControlError>;
    fn reset(&self, path: &RuntimeControlProperty) -> Result<(), RuntimeControlError> {
        Err(RuntimeControlError::Unsupported {
            path: path.console_path.clone(),
            reason: "reset not implemented".to_owned(),
        })
    }
    fn commit(&self, path: &RuntimeControlProperty) -> Result<(), RuntimeControlError> {
        Err(RuntimeControlError::Unsupported {
            path: path.console_path.clone(),
            reason: "source commit not implemented".to_owned(),
        })
    }
}
