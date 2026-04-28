use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;

pub type AmigoResult<T> = Result<T, AmigoError>;

#[derive(Debug, Clone)]
pub enum AmigoError {
    Message(String),
    Io(String),
    DuplicateService(&'static str),
    MissingService(&'static str),
}

impl Display for AmigoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => write!(f, "{message}"),
            Self::Io(message) => write!(f, "{message}"),
            Self::DuplicateService(service) => {
                write!(f, "service `{service}` was already registered")
            }
            Self::MissingService(service) => write!(f, "service `{service}` is not registered"),
        }
    }
}

impl Error for AmigoError {}

impl From<std::io::Error> for AmigoError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<String> for AmigoError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&str> for AmigoError {
    fn from(value: &str) -> Self {
        Self::Message(value.to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypedId<Tag> {
    raw: u64,
    marker: PhantomData<fn() -> Tag>,
}

impl<Tag> TypedId<Tag> {
    pub const fn new(raw: u64) -> Self {
        Self {
            raw,
            marker: PhantomData,
        }
    }

    pub const fn raw(&self) -> u64 {
        self.raw
    }
}

impl<Tag> Display for TypedId<Tag> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl EngineVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl Default for EngineVersion {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

impl Display for EngineVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticMessage {
    pub subsystem: &'static str,
    pub message: String,
}

impl DiagnosticMessage {
    pub fn new(subsystem: &'static str, message: impl Into<String>) -> Self {
        Self {
            subsystem,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LaunchSelection {
    pub startup_mod: Option<String>,
    pub startup_scene: Option<String>,
    pub requested_mods: Vec<String>,
    pub dev_mode: bool,
}

impl LaunchSelection {
    pub fn new(
        startup_mod: Option<String>,
        startup_scene: Option<String>,
        requested_mods: Vec<String>,
        dev_mode: bool,
    ) -> Self {
        Self {
            startup_mod,
            startup_scene,
            requested_mods,
            dev_mode,
        }
    }

    pub fn selected_mod(&self) -> &str {
        self.startup_mod.as_deref().unwrap_or("")
    }

    pub fn selected_scene(&self) -> &str {
        self.startup_scene.as_deref().unwrap_or("")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeDiagnostics {
    pub window_backend: String,
    pub input_backend: String,
    pub render_backend: String,
    pub script_backend: String,
    pub loaded_mods: Vec<String>,
    pub capabilities: Vec<String>,
    pub plugin_names: Vec<String>,
    pub service_names: Vec<String>,
}

impl RuntimeDiagnostics {
    pub fn new(
        window_backend: impl Into<String>,
        input_backend: impl Into<String>,
        render_backend: impl Into<String>,
        script_backend: impl Into<String>,
        loaded_mods: Vec<String>,
        capabilities: Vec<String>,
        plugin_names: Vec<String>,
        service_names: Vec<String>,
    ) -> Self {
        Self {
            window_backend: window_backend.into(),
            input_backend: input_backend.into(),
            render_backend: render_backend.into(),
            script_backend: script_backend.into(),
            loaded_mods,
            capabilities,
            plugin_names,
            service_names,
        }
    }
}
