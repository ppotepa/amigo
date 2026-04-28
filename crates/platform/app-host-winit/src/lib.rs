use amigo_app_host_api::{HostControl, HostExitStrategy, HostHandler, HostLifecycleEvent};
use amigo_core::{AmigoError, AmigoResult};
use amigo_input_api::InputEvent;
use amigo_input_winit::{map_key_event, map_modifiers, map_mouse_button};
use amigo_window_api::{WindowEvent, WindowSurfaceHandles};
use amigo_window_winit::{map_window_event, to_winit_attributes};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent as WinitWindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

pub struct WinitAppHost;

impl WinitAppHost {
    pub fn run<H>(handler: H) -> AmigoResult<()>
    where
        H: HostHandler,
    {
        let event_loop =
            EventLoop::new().map_err(|error| AmigoError::Message(error.to_string()))?;
        let mut app = HostedApp::new(handler);
        event_loop
            .run_app(&mut app)
            .map_err(|error| AmigoError::Message(error.to_string()))?;

        if let Some(error) = app.fatal_error {
            return Err(error);
        }

        Ok(())
    }
}

struct HostedApp<H> {
    handler: H,
    window: Option<Window>,
    exit_after_redraw: bool,
    fatal_error: Option<AmigoError>,
}

impl<H> HostedApp<H>
where
    H: HostHandler,
{
    fn new(handler: H) -> Self {
        let exit_after_redraw = matches!(
            handler.config().exit_strategy,
            HostExitStrategy::AfterFirstRedraw
        );

        Self {
            handler,
            window: None,
            exit_after_redraw,
            fatal_error: None,
        }
    }

    fn apply_control(&mut self, event_loop: &ActiveEventLoop, outcome: AmigoResult<HostControl>) {
        match outcome {
            Ok(HostControl::Continue) => {}
            Ok(HostControl::Exit) => event_loop.exit(),
            Err(error) => {
                self.fatal_error = Some(error);
                event_loop.exit();
            }
        }
    }
}

impl<H> ApplicationHandler for HostedApp<H>
where
    H: HostHandler,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let outcome = self.handler.on_lifecycle(HostLifecycleEvent::Resumed);
        self.apply_control(event_loop, outcome);

        if self.window.is_some() {
            return;
        }

        let attributes = to_winit_attributes(&self.handler.config().window);
        match event_loop.create_window(attributes) {
            Ok(window) => {
                let handles = match extract_surface_handles(&window) {
                    Ok(handles) => handles,
                    Err(error) => {
                        self.fatal_error = Some(error);
                        event_loop.exit();
                        return;
                    }
                };
                self.window = Some(window);
                let outcome = self.handler.on_lifecycle(HostLifecycleEvent::WindowCreated);
                self.apply_control(event_loop, outcome);
                let outcome = self.handler.on_window_ready(handles);
                self.apply_control(event_loop, outcome);
            }
            Err(error) => {
                self.fatal_error = Some(AmigoError::Message(error.to_string()));
                event_loop.exit();
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
        let outcome = self.handler.on_lifecycle(HostLifecycleEvent::AboutToWait);
        self.apply_control(event_loop, outcome);

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let outcome = self.handler.on_lifecycle(HostLifecycleEvent::Exiting);
        self.apply_control(event_loop, outcome);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WinitWindowEvent,
    ) {
        event_loop.set_control_flow(ControlFlow::Wait);

        if let Some(mapped) = map_window_event(&event) {
            let outcome = self.handler.on_window_event(mapped);
            self.apply_control(event_loop, outcome);

            if matches!(mapped, WindowEvent::CloseRequested) {
                event_loop.exit();
                return;
            }
        }

        match event {
            WinitWindowEvent::KeyboardInput { event, .. } => {
                if let Some(mapped) = map_key_event(&event) {
                    let outcome = self.handler.on_input_event(mapped);
                    self.apply_control(event_loop, outcome);
                }
            }
            WinitWindowEvent::ModifiersChanged(modifiers) => {
                let outcome =
                    self.handler
                        .on_input_event(InputEvent::ModifiersChanged(map_modifiers(
                            modifiers.state(),
                        )));
                self.apply_control(event_loop, outcome);
            }
            WinitWindowEvent::CursorMoved { position, .. } => {
                let outcome = self.handler.on_input_event(InputEvent::CursorMoved {
                    x: position.x,
                    y: position.y,
                });
                self.apply_control(event_loop, outcome);
            }
            WinitWindowEvent::MouseInput { state, button, .. } => {
                let outcome = self.handler.on_input_event(InputEvent::MouseButton {
                    button: map_mouse_button(button),
                    pressed: matches!(state, winit::event::ElementState::Pressed),
                });
                self.apply_control(event_loop, outcome);
            }
            WinitWindowEvent::RedrawRequested => {
                let outcome = self.handler.on_redraw_requested();
                self.apply_control(event_loop, outcome);

                if self.exit_after_redraw {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

fn extract_surface_handles(window: &Window) -> AmigoResult<WindowSurfaceHandles> {
    let raw_window_handle = window
        .window_handle()
        .map_err(|error| AmigoError::Message(error.to_string()))?
        .as_raw();
    let raw_display_handle = window
        .display_handle()
        .map_err(|error| AmigoError::Message(error.to_string()))?
        .as_raw();
    let size = window.inner_size();

    Ok(WindowSurfaceHandles {
        raw_display_handle: Some(raw_display_handle),
        raw_window_handle,
        size: amigo_window_api::WindowSize {
            width: size.width.max(1),
            height: size.height.max(1),
        },
        scale_factor: window.scale_factor(),
    })
}
