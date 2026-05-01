use std::sync::Arc;

use amigo_scripting_api::ScriptTraceService;

#[derive(Clone)]
pub struct TraceApi {
    pub(crate) trace: Option<Arc<ScriptTraceService>>,
}

impl TraceApi {
    pub fn begin(&mut self, label: &str) -> bool {
        self.trace.as_ref().is_some_and(|trace| {
            trace.begin(label);
            true
        })
    }

    pub fn value(&mut self, key: &str, value: &str) -> bool {
        self.trace.as_ref().is_some_and(|trace| {
            trace.value(key, value);
            true
        })
    }

    pub fn value_int(&mut self, key: &str, value: rhai::INT) -> bool {
        self.value(key, &value.to_string())
    }

    pub fn value_float(&mut self, key: &str, value: rhai::FLOAT) -> bool {
        self.value(key, &value.to_string())
    }

    pub fn value_bool(&mut self, key: &str, value: bool) -> bool {
        self.value(key, if value { "true" } else { "false" })
    }

    pub fn end(&mut self) -> bool {
        self.trace.as_ref().is_some_and(|trace| trace.end())
    }

    pub fn clear(&mut self) -> bool {
        self.trace.as_ref().is_some_and(|trace| {
            trace.clear();
            true
        })
    }
}
