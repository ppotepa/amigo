use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use windows::{
    Win32::{
        Foundation::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::w,
};

#[derive(Clone, Serialize)]
pub struct NativeInput {
    pub kind: &'static str,
    pub x: f64,
    pub y: f64,
    pub button: u32,
    pub wheel: f64,
    pub key: u32,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    pub repeat: bool,
}
#[derive(Clone, Copy, Deserialize)]
pub struct NativeViewportRect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub dpr: f64,
    pub visible: bool,
}
struct InputState {
    sender: mpsc::Sender<NativeInput>,
    dpr: f64,
}

pub struct NativeSurface {
    pub hwnd: HWND,
    input: Box<InputState>,
}
impl NativeSurface {
    pub fn new(parent: HWND, sender: mpsc::Sender<NativeInput>) -> Result<Self, String> {
        unsafe {
            let instance = GetModuleHandleW(None).map_err(|e| e.to_string())?;
            let class = w!("AmigoPlaygroundViewport");
            RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: HINSTANCE(instance.0),
                lpszClassName: class,
                hCursor: LoadCursorW(None, IDC_ARROW).map_err(|e| e.to_string())?,
                ..Default::default()
            });
            let mut input = Box::new(InputState { sender, dpr: 1.0 });
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class,
                w!(""),
                WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
                0,
                0,
                1,
                1,
                Some(parent),
                None,
                Some(HINSTANCE(instance.0)),
                Some((&mut *input as *mut InputState).cast()),
            )
            .map_err(|e| e.to_string())?;
            Ok(Self { hwnd, input })
        }
    }
    pub fn place(&mut self, rect: NativeViewportRect) -> Result<(), String> {
        if [rect.left, rect.top, rect.width, rect.height, rect.dpr]
            .iter()
            .any(|x| !x.is_finite())
            || rect.dpr <= 0.0
            || rect.width < 0.0
            || rect.height < 0.0
        {
            return Err("invalid native viewport rectangle".into());
        }
        self.input.dpr = rect.dpr;
        unsafe {
            SetWindowPos(
                self.hwnd,
                Some(HWND_TOP),
                (rect.left * rect.dpr).round() as i32,
                (rect.top * rect.dpr).round() as i32,
                (rect.width * rect.dpr).round() as i32,
                (rect.height * rect.dpr).round() as i32,
                SWP_NOACTIVATE
                    | if rect.visible && rect.width > 0.0 && rect.height > 0.0 {
                        SWP_SHOWWINDOW
                    } else {
                        SWP_HIDEWINDOW
                    },
            )
        }
        .map_err(|e| e.to_string())
    }
}
impl Drop for NativeSurface {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if message == WM_NCCREATE {
            let create = &*(lparam.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
        }
        let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut InputState;
        if !state.is_null() {
            let state = &*state;
            let mut event = NativeInput {
                kind: "",
                x: (lparam.0 as i16) as f64 / state.dpr,
                y: ((lparam.0 >> 16) as i16) as f64 / state.dpr,
                button: 0,
                wheel: 0.0,
                key: 0,
                ctrl: false,
                shift: false,
                alt: false,
                meta: false,
                repeat: false,
            };
            match message {
                WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
                    let _ = SetFocus(Some(hwnd));
                    SetCapture(hwnd);
                    event.kind = "down";
                    event.button = if message == WM_RBUTTONDOWN {
                        2
                    } else if message == WM_MBUTTONDOWN {
                        1
                    } else {
                        0
                    };
                }
                WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => {
                    event.kind = "up";
                }
                WM_MOUSEMOVE => event.kind = "move",
                WM_MOUSEWHEEL => {
                    event.kind = "wheel";
                    event.wheel = ((wparam.0 >> 16) as i16) as f64 / 120.0 * 100.0;
                }
                WM_KEYDOWN => {
                    event.kind = "key";
                    event.key = wparam.0 as u32;
                    event.ctrl = GetKeyState(VK_CONTROL.0 as i32) < 0;
                    event.shift = GetKeyState(VK_SHIFT.0 as i32) < 0;
                    event.alt = GetKeyState(VK_MENU.0 as i32) < 0;
                    event.meta = GetKeyState(VK_LWIN.0 as i32) < 0
                        || GetKeyState(VK_RWIN.0 as i32) < 0;
                    event.repeat = lparam.0 & (1 << 30) != 0;
                }
                WM_KILLFOCUS | WM_CAPTURECHANGED => event.kind = "cancel",
                WM_SETFOCUS => event.kind = "focus",
                WM_ERASEBKGND => return LRESULT(1),
                _ => {}
            }
            if !event.kind.is_empty() {
                let _ = state.sender.send(event);
                if matches!(message, WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP) {
                    let _ = ReleaseCapture();
                }
                return LRESULT(0);
            }
        }
        DefWindowProcW(hwnd, message, wparam, lparam)
    }
}
