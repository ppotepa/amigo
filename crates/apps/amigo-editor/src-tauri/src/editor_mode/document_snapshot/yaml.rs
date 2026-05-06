use serde_yaml::{Mapping, Value};

pub fn mapping(value: &Value) -> Option<&Mapping> {
    value.as_mapping()
}

pub fn string_field(mapping: &Mapping, key: &str) -> Option<String> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

pub fn bool_field(mapping: &Mapping, key: &str) -> Option<bool> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(Value::as_bool)
}

pub fn number_field(mapping: &Mapping, key: &str) -> Option<f32> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(number_value)
}

fn number_value(value: &Value) -> Option<f32> {
    value
        .as_f64()
        .map(|value| value as f32)
        .or_else(|| value.as_i64().map(|value| value as f32))
}
