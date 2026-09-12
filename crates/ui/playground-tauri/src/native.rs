use amigo_playground_api::{PlaygroundFrameAck, PlaygroundViewportMode};
use amigo_playground_native::*;
use std::{
    cell::RefCell,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use windows::core::{Interface, PCWSTR};

pub struct NativeSession(pub Option<Arc<NativeLink>>);
struct SharedBuffers(Vec<ICoreWebView2SharedBuffer>);
impl Drop for SharedBuffers {
    fn drop(&mut self) {
        for buffer in &self.0 {
            unsafe {
                let _ = buffer.Close();
            }
        }
    }
}
#[derive(Default)]
struct Presentation {
    surface: Option<NativeSurface>,
    presenter: Option<GpuPresenter>,
    buffers: Option<SharedBuffers>,
    rect: Option<NativeViewportRect>,
    has_frame: bool,
}
thread_local! { static PRESENTATION: RefCell<Presentation> = RefCell::default(); }

#[tauri::command]
pub fn viewport_rect(window: tauri::WebviewWindow, rect: NativeViewportRect) -> Result<(), String> {
    window
        .with_webview(move |_| {
            PRESENTATION.with_borrow_mut(|state| {
                state.rect = Some(rect);
                let mut rect = rect;
                rect.visible &= state.has_frame;
                if let Some(surface) = &mut state.surface {
                    let _ = surface.place(rect);
                }
            });
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn frame_consumed(
    state: tauri::State<NativeSession>,
    ack: PlaygroundFrameAck,
) -> Result<(), String> {
    state
        .0
        .as_ref()
        .ok_or("Native channel unavailable")?
        .send(NativeMessage::Ack { ack })
}

#[tauri::command]
pub fn presentation_failed(
    state: tauri::State<NativeSession>,
    generation: u64,
    message: String,
) -> Result<(), String> {
    state
        .0
        .as_ref()
        .ok_or("Native channel unavailable")?
        .send(NativeMessage::Error {
            generation,
            message,
        })
}

pub fn attach(window: &tauri::WebviewWindow) -> Result<(), String> {
    let Some(link) = window.state::<NativeSession>().0.clone() else {
        return Ok(());
    };
    let (input, inputs) = mpsc::channel();
    let parent = window.hwnd().map_err(|e| e.to_string())?.0 as usize;
    window
        .with_webview(move |_| {
            PRESENTATION.with_borrow_mut(|state| {
                state.surface = NativeSurface::new(
                    windows_native::Win32::Foundation::HWND(parent as *mut _),
                    input,
                )
                .ok();
            });
        })
        .map_err(|e| e.to_string())?;
    let window = window.clone();
    std::thread::spawn(move || {
        while link.alive() {
            for input in inputs.try_iter() {
                let _ = window.emit("viewport-input", input);
            }
            for message in link.drain() {
                let target = window.clone();
                let connection = link.clone();
                let _ = window.with_webview(move |webview| {
                    let generation = match &message {
                        NativeMessage::RgbaConfigure { generation, .. } => *generation,
                        NativeMessage::GpuConfigure { resources } => resources.generation,
                        NativeMessage::Frame { header, .. } => header.generation,
                        _ => 0,
                    };
                    let result = PRESENTATION.with_borrow_mut(|state| {
                        handle(state, webview, &target, &connection, message)
                    });
                    if let Err(message) = result {
                        let _ = target.emit(
                            "viewport-error",
                            serde_json::json!({"generation":generation,"message":message}),
                        );
                        let _ = connection.send(NativeMessage::Error {
                            generation,
                            message,
                        });
                    }
                });
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    });
    Ok(())
}

fn handle(
    state: &mut Presentation,
    webview: tauri::webview::PlatformWebview,
    window: &tauri::WebviewWindow,
    link: &NativeLink,
    message: NativeMessage,
) -> Result<(), String> {
    match message {
        NativeMessage::RgbaConfigure { generation, size } => unsafe {
            if size.contains(&0) || size.iter().any(|axis| *axis > 16384) {
                return Err("Invalid shared buffer size".into());
            }
            let environment: ICoreWebView2Environment12 = webview
                .environment()
                .cast()
                .map_err(|e| format!("WebView2 SharedBuffer is unavailable: {e}"))?;
            let controller = webview.controller();
            let view: ICoreWebView2_17 = controller
                .CoreWebView2()
                .map_err(|e| e.to_string())?
                .cast()
                .map_err(|e| e.to_string())?;
            let mut buffers = SharedBuffers(Vec::new());
            let mut pending =
                PendingHandles::new(link.peer_pid.load(std::sync::atomic::Ordering::Acquire));
            let mut handles = [0u64; 3];
            for (slot, handle) in handles.iter_mut().enumerate() {
                let buffer = environment
                    .CreateSharedBuffer(u64::from(size[0]) * u64::from(size[1]) * 4)
                    .map_err(|e| e.to_string())?;
                let mut mapping = windows::Win32::Foundation::HANDLE::default();
                buffer
                    .FileMappingHandle(&mut mapping)
                    .map_err(|e| e.to_string())?;
                *handle =
                    pending.duplicate(windows_native::Win32::Foundation::HANDLE(mapping.0))?;
                let metadata = serde_json::json!({"generation":generation,"slot":slot,"size":size})
                    .to_string();
                let wide: Vec<_> = metadata.encode_utf16().chain(Some(0)).collect();
                view.PostSharedBufferToScript(
                    &buffer,
                    COREWEBVIEW2_SHARED_BUFFER_ACCESS_READ_ONLY,
                    PCWSTR(wide.as_ptr()),
                )
                .map_err(|e| e.to_string())?;
                buffers.0.push(buffer);
            }
            state.buffers = Some(buffers);
            link.send(NativeMessage::RgbaBuffers {
                generation,
                size,
                handles,
            })?;
            pending.transferred();
        },
        NativeMessage::GpuConfigure { resources } => {
            let surface = state
                .surface
                .as_ref()
                .ok_or("Native child window is unavailable")?;
            state.presenter = None; // Release the previous HWND swapchain before replacing it.
            state.has_frame = false;
            let presenter = GpuPresenter::new(surface.hwnd, resources)?;
            link.send(NativeMessage::GpuReady {
                generation: presenter.generation,
            })?;
            state.presenter = Some(presenter);
        }
        NativeMessage::Frame {
            slot,
            fence,
            header,
        } => {
            if header.mode == PlaygroundViewportMode::NativeGpu {
                let started = Instant::now();
                let presenter = state
                    .presenter
                    .as_mut()
                    .ok_or("Native presenter is not initialized")?;
                if presenter.generation != header.generation {
                    return Err("Obsolete native surface generation".into());
                }
                presenter.present(slot, fence)?;
                state.has_frame = true;
                if let (Some(surface), Some(rect)) = (&mut state.surface, state.rect) {
                    surface.place(rect)?;
                }
                let _ = window.emit("viewport-presented", &header);
                let ack = PlaygroundFrameAck {
                    sequence: header.sequence,
                    generation: header.generation,
                    presented: true,
                    decode_ms: 0.0,
                    present_ms: started.elapsed().as_secs_f64() * 1000.0,
                };
                let _ = window.emit("viewport-consumed", &ack);
                link.send(NativeMessage::Ack { ack })?;
            } else {
                let _ = window.emit(
                    "viewport-rgba-frame",
                    serde_json::json!({"slot":slot,"header":header}),
                );
            }
        }
        NativeMessage::Error {
            generation,
            message,
        } => {
            let _ = window.emit(
                "viewport-error",
                serde_json::json!({"generation":generation,"message":message}),
            );
        }
        NativeMessage::Close => {
            state.presenter = None;
            state.buffers = None;
            state.surface = None;
        }
        _ => return Err("Unexpected native host message".into()),
    }
    Ok(())
}
