//! Host lifecycle API for driving an Amigo runtime.
//! It defines host events, control flow, and configuration used by app shells.

use amigo_core::AmigoResult;
use amigo_input_api::InputEvent;
use amigo_window_api::{WindowDescriptor, WindowEvent, WindowSurfaceHandles};

#[derive(Debug, Clone)]
pub struct HostConfig {
    pub window: WindowDescriptor,
    pub exit_strategy: HostExitStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostExitStrategy {
    Manual,
    AfterFirstRedraw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostLifecycleEvent {
    Resumed,
    WindowCreated,
    AboutToWait,
    Exiting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostControl {
    Continue,
    Exit,
}

pub trait HostHandler {
    fn config(&self) -> HostConfig;

    fn on_lifecycle(&mut self, _event: HostLifecycleEvent) -> AmigoResult<HostControl> {
        Ok(HostControl::Continue)
    }

    fn on_window_event(&mut self, _event: WindowEvent) -> AmigoResult<HostControl> {
        Ok(HostControl::Continue)
    }

    fn on_input_event(&mut self, _event: InputEvent) -> AmigoResult<HostControl> {
        Ok(HostControl::Continue)
    }

    fn on_window_ready(&mut self, _handles: WindowSurfaceHandles) -> AmigoResult<HostControl> {
        Ok(HostControl::Continue)
    }

    fn on_redraw_requested(&mut self) -> AmigoResult<HostControl> {
        Ok(HostControl::Continue)
    }
}
