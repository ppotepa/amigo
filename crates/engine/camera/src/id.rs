use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CameraId(pub String);

impl CameraId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CameraBinding {
    Main,
    Named(CameraId),
}

impl CameraBinding {
    pub fn main() -> Self {
        Self::Main
    }

    pub fn none() -> Option<Self> {
        None
    }
}

impl Default for CameraBinding {
    fn default() -> Self {
        Self::main()
    }
}
