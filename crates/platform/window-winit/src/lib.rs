//! Winit-backed window service implementation.
//! It adapts the platform window API to desktop event loops and surface creation.

use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_window_api::{WindowBackend, WindowDescriptor, WindowEvent, WindowServiceInfo};

#[derive(Debug, Clone)]
pub struct WinitWindowBackend {
    descriptor: WindowDescriptor,
}

impl Default for WinitWindowBackend {
    fn default() -> Self {
        Self {
            descriptor: WindowDescriptor::default(),
        }
    }
}

impl WinitWindowBackend {
    pub fn logical_size(&self) -> winit::dpi::LogicalSize<u32> {
        winit::dpi::LogicalSize::new(self.descriptor.size.width, self.descriptor.size.height)
    }
}

pub fn to_winit_attributes(descriptor: &WindowDescriptor) -> winit::window::WindowAttributes {
    winit::window::Window::default_attributes()
        .with_title(descriptor.title.clone())
        .with_inner_size(winit::dpi::LogicalSize::new(
            descriptor.size.width,
            descriptor.size.height,
        ))
        .with_resizable(descriptor.resizable)
}

pub fn map_window_event(event: &winit::event::WindowEvent) -> Option<WindowEvent> {
    match event {
        winit::event::WindowEvent::Resized(size) => {
            Some(WindowEvent::Resized(amigo_window_api::WindowSize {
                width: size.width,
                height: size.height,
            }))
        }
        winit::event::WindowEvent::CloseRequested => Some(WindowEvent::CloseRequested),
        winit::event::WindowEvent::Focused(focused) => Some(WindowEvent::Focused(*focused)),
        winit::event::WindowEvent::RedrawRequested => Some(WindowEvent::RedrawRequested),
        winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            Some(WindowEvent::ScaleFactorChanged {
                scale_factor: *scale_factor,
            })
        }
        _ => None,
    }
}

impl WindowBackend for WinitWindowBackend {
    fn backend_name(&self) -> &'static str {
        "winit"
    }

    fn primary_window(&self) -> WindowDescriptor {
        self.descriptor.clone()
    }
}

pub struct WinitWindowPlugin {
    backend: WinitWindowBackend,
}

impl Default for WinitWindowPlugin {
    fn default() -> Self {
        Self {
            backend: WinitWindowBackend::default(),
        }
    }
}

impl RuntimePlugin for WinitWindowPlugin {
    fn name(&self) -> &'static str {
        "amigo-window-winit"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        let _ = self.backend.logical_size();

        registry.register(WindowServiceInfo {
            backend_name: self.backend.backend_name(),
            primary_window: self.backend.primary_window(),
        })?;
        registry.register(self.backend.clone())
    }
}
