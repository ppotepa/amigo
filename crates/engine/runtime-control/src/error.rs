use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeControlError {
    InvalidPath(String),
    UnknownTarget(String),
    UnknownComponent {
        target: String,
        component: String,
    },
    UnknownProperty {
        path: String,
    },
    NotWritable {
        path: String,
    },
    TypeMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    OutOfRange {
        path: String,
        value: String,
    },
    ProviderUnavailable {
        path: String,
    },
    Unsupported {
        path: String,
        reason: String,
    },
}

impl Display for RuntimeControlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath(path) => write!(f, "invalid runtime control path: {path}"),
            Self::UnknownTarget(path) => write!(f, "unknown target: {path}"),
            Self::UnknownComponent { target, component } => {
                write!(f, "unknown component `{component}` for target `{target}`")
            }
            Self::UnknownProperty { path } => write!(f, "unknown property: {path}"),
            Self::NotWritable { path } => write!(f, "property is readonly: {path}"),
            Self::TypeMismatch {
                path,
                expected,
                actual,
            } => write!(f, "expected {expected} for {path}, got {actual}"),
            Self::OutOfRange { path, value } => {
                write!(f, "value {value} is out of range for {path}")
            }
            Self::ProviderUnavailable { path } => {
                write!(f, "runtime control provider unavailable for {path}")
            }
            Self::Unsupported { path, reason } => write!(f, "{reason} for {path}"),
        }
    }
}

impl Error for RuntimeControlError {}

impl From<RuntimeControlError> for amigo_core::AmigoError {
    fn from(value: RuntimeControlError) -> Self {
        amigo_core::AmigoError::Message(value.to_string())
    }
}
