use serde_yaml::Value;

use crate::refs::mapping_get;

pub fn prefab_id(value: &Value) -> Option<String> {
    let prefab = mapping_get(value, "prefab")?;

    if let Some(id) = mapping_get(prefab, "id").and_then(Value::as_str) {
        return Some(id.to_owned());
    }

    prefab.as_str().map(str::to_owned)
}
pub fn has_prefab_overrides(value: &Value) -> bool {
    mapping_get(value, "prefab_overrides").is_some()
}
