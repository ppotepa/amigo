use crate::RuntimeControlError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlValue {
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    AssetRef(String),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Color([f32; 4]),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlValueType {
    Bool,
    I64,
    U64,
    F32,
    F64,
    String,
    AssetRef,
    Vec2,
    Vec3,
    Color,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl ControlValue {
    pub fn as_f32(&self) -> Option<f32> {
        self.as_f64().map(|value| value as f32)
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::I64(value) => Some(*value as f64),
            Self::U64(value) => Some(*value as f64),
            Self::F64(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(value) | Self::AssetRef(value) => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn value_type(&self) -> Option<ControlValueType> {
        match self {
            Self::Bool(_) => Some(ControlValueType::Bool),
            Self::I64(_) => Some(ControlValueType::I64),
            Self::U64(_) => Some(ControlValueType::U64),
            Self::F64(_) => Some(ControlValueType::F64),
            Self::String(_) => Some(ControlValueType::String),
            Self::AssetRef(_) => Some(ControlValueType::AssetRef),
            Self::Vec2(_) => Some(ControlValueType::Vec2),
            Self::Vec3(_) => Some(ControlValueType::Vec3),
            Self::Color(_) => Some(ControlValueType::Color),
            Self::Null => None,
        }
    }

    pub fn coerce_to(
        self,
        expected: ControlValueType,
    ) -> Result<ControlValue, RuntimeControlError> {
        match expected {
            ControlValueType::Vec2 | ControlValueType::Vec3 | ControlValueType::Color => {
                let finite = match &self {
                    Self::Vec2(v) => v.iter().all(|x| x.is_finite()),
                    Self::Vec3(v) => v.iter().all(|x| x.is_finite()),
                    Self::Color(v) => v.iter().all(|x| x.is_finite() && (0.0..=1.0).contains(x)),
                    _ => false,
                };
                if self.value_type() == Some(expected) && finite {
                    Ok(self)
                } else {
                    Err(RuntimeControlError::TypeMismatch {
                        path: String::new(),
                        expected: format!("{expected:?}"),
                        actual: format!("{:?}", self.value_type()),
                    })
                }
            }
            ControlValueType::Bool => self.as_bool().map(ControlValue::Bool).ok_or_else(|| {
                RuntimeControlError::TypeMismatch {
                    path: String::new(),
                    expected: "bool".to_owned(),
                    actual: value_type_name(self.value_type()).to_owned(),
                }
            }),
            ControlValueType::I64 => match self {
                Self::I64(value) => Ok(Self::I64(value)),
                Self::U64(value) => i64::try_from(value).map(Self::I64).map_err(|_| {
                    RuntimeControlError::TypeMismatch {
                        path: String::new(),
                        expected: "i64".to_owned(),
                        actual: "u64".to_owned(),
                    }
                }),
                _ => Err(RuntimeControlError::TypeMismatch {
                    path: String::new(),
                    expected: "i64".to_owned(),
                    actual: value_type_name(self.value_type()).to_owned(),
                }),
            },
            ControlValueType::U64 => match self {
                Self::U64(value) => Ok(Self::U64(value)),
                Self::I64(value) if value >= 0 => Ok(Self::U64(value as u64)),
                _ => Err(RuntimeControlError::TypeMismatch {
                    path: String::new(),
                    expected: "u64".to_owned(),
                    actual: value_type_name(self.value_type()).to_owned(),
                }),
            },
            ControlValueType::F32 | ControlValueType::F64 => self
                .as_f64()
                .map(ControlValue::F64)
                .ok_or_else(|| RuntimeControlError::TypeMismatch {
                    path: String::new(),
                    expected: if expected == ControlValueType::F32 {
                        "f32".to_owned()
                    } else {
                        "f64".to_owned()
                    },
                    actual: value_type_name(self.value_type()).to_owned(),
                }),
            ControlValueType::String => self
                .as_string()
                .map(|value| ControlValue::String(value.to_owned()))
                .ok_or_else(|| RuntimeControlError::TypeMismatch {
                    path: String::new(),
                    expected: "string".to_owned(),
                    actual: value_type_name(self.value_type()).to_owned(),
                }),
            ControlValueType::AssetRef => self
                .as_string()
                .map(|value| ControlValue::AssetRef(value.to_owned()))
                .ok_or_else(|| RuntimeControlError::TypeMismatch {
                    path: String::new(),
                    expected: "asset_ref".to_owned(),
                    actual: value_type_name(self.value_type()).to_owned(),
                }),
        }
    }
}

fn value_type_name(value_type: Option<ControlValueType>) -> &'static str {
    match value_type {
        Some(ControlValueType::Bool) => "bool",
        Some(ControlValueType::I64) => "i64",
        Some(ControlValueType::U64) => "u64",
        Some(ControlValueType::F32) => "f32",
        Some(ControlValueType::F64) => "f64",
        Some(ControlValueType::String) => "string",
        Some(ControlValueType::AssetRef) => "asset_ref",
        Some(ControlValueType::Vec2) => "vec2",
        Some(ControlValueType::Vec3) => "vec3",
        Some(ControlValueType::Color) => "color",
        None => "null",
    }
}
