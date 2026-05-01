use std::path::PathBuf;

use amigo_core::AmigoResult;

use crate::types::ScriptParams;

#[derive(Debug, Clone)]
pub struct ScriptRuntimeInfo {
    pub backend_name: &'static str,
    pub file_extension: &'static str,
}

pub trait ScriptRuntime: Send + Sync {
    fn backend_name(&self) -> &'static str;
    fn file_extension(&self) -> &'static str;
    fn validate(&self, source: &str) -> AmigoResult<()>;
    fn set_source_context(&self, _context: ScriptSourceContext) -> AmigoResult<()> {
        Ok(())
    }
    fn execute(&self, source_name: &str, source: &str) -> AmigoResult<()>;
    fn unload(&self, source_name: &str) -> AmigoResult<()>;
    fn call_update(&self, source_name: &str, delta_seconds: f32) -> AmigoResult<()>;
    fn call_on_enter(&self, source_name: &str) -> AmigoResult<()>;
    fn call_on_exit(&self, source_name: &str) -> AmigoResult<()>;
    fn call_on_event(&self, source_name: &str, topic: &str, payload: &[String]) -> AmigoResult<()>;
    fn call_event_function(
        &self,
        _source_name: &str,
        _function_name: &str,
        _topic: &str,
        _payload: &[String],
    ) -> AmigoResult<()> {
        Ok(())
    }
    fn call_component_on_attach(
        &self,
        _source_name: &str,
        _entity_name: &str,
        _params: &ScriptParams,
    ) -> AmigoResult<()> {
        Ok(())
    }
    fn call_component_update(
        &self,
        _source_name: &str,
        _entity_name: &str,
        _params: &ScriptParams,
        _delta_seconds: f32,
    ) -> AmigoResult<()> {
        Ok(())
    }
    fn call_component_on_detach(
        &self,
        _source_name: &str,
        _entity_name: &str,
        _params: &ScriptParams,
    ) -> AmigoResult<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptSourceContext {
    pub source_name: String,
    pub mod_root_path: PathBuf,
    pub script_dir_path: PathBuf,
}
