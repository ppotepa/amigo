use amigo_core::TypedId;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowTag;
pub type WindowId = TypedId<WindowTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowDescriptor {
    pub title: String,
    pub size: WindowSize,
    pub resizable: bool,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            title: "Amigo".to_owned(),
            size: WindowSize {
                width: 1280,
                height: 720,
            },
            resizable: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowEvent {
    Resized(WindowSize),
    CloseRequested,
    Focused(bool),
    RedrawRequested,
    ScaleFactorChanged { scale_factor: f64 },
}

#[derive(Debug, Clone)]
pub struct WindowServiceInfo {
    pub backend_name: &'static str,
    pub primary_window: WindowDescriptor,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowSurfaceHandles {
    pub raw_display_handle: Option<RawDisplayHandle>,
    pub raw_window_handle: RawWindowHandle,
    pub size: WindowSize,
    pub scale_factor: f64,
}

pub trait WindowBackend: Send + Sync {
    fn backend_name(&self) -> &'static str;
    fn primary_window(&self) -> WindowDescriptor;
}
