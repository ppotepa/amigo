use windows::Win32::{
    Foundation::*,
    System::{Memory::*, Threading::*},
};

pub struct NativeHandle(pub HANDLE);
impl NativeHandle {
    /// Takes ownership of a handle duplicated into this process by the peer.
    pub unsafe fn from_raw(value: u64) -> Self {
        Self(HANDLE(value as usize as *mut _))
    }
}
unsafe impl Send for NativeHandle {}
impl Drop for NativeHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Duplicate only into the authenticated peer; the receiving Rust side owns it.
pub fn duplicate_for_process(handle: HANDLE, pid: u32) -> Result<u64, String> {
    unsafe {
        let process =
            NativeHandle(OpenProcess(PROCESS_DUP_HANDLE, false, pid).map_err(|e| e.to_string())?);
        let mut duplicated = HANDLE::default();
        DuplicateHandle(
            GetCurrentProcess(),
            handle,
            process.0,
            &mut duplicated,
            0,
            false,
            DUPLICATE_SAME_ACCESS,
        )
        .map_err(|e| e.to_string())?;
        Ok(duplicated.0 as usize as u64)
    }
}

/// Owns duplicates until a configuration is successfully queued for the peer.
/// Never revoke after transfer: the peer may already have closed/reused them.
pub struct PendingHandles {
    pid: u32,
    handles: Vec<u64>,
}
impl PendingHandles {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            handles: Vec::new(),
        }
    }
    pub fn duplicate(&mut self, source: HANDLE) -> Result<u64, String> {
        let handle = duplicate_for_process(source, self.pid)?;
        self.handles.push(handle);
        Ok(handle)
    }
    pub fn transferred(&mut self) {
        self.handles.clear();
    }
}
impl Drop for PendingHandles {
    fn drop(&mut self) {
        if self.handles.is_empty() {
            return;
        }
        unsafe {
            let Ok(process) = OpenProcess(PROCESS_DUP_HANDLE, false, self.pid) else {
                return;
            };
            let process = NativeHandle(process);
            for handle in self.handles.drain(..) {
                let mut local = HANDLE::default();
                if DuplicateHandle(
                    process.0,
                    HANDLE(handle as usize as *mut _),
                    GetCurrentProcess(),
                    &mut local,
                    0,
                    false,
                    DUPLICATE_CLOSE_SOURCE | DUPLICATE_SAME_ACCESS,
                )
                .is_ok()
                {
                    drop(NativeHandle(local));
                }
            }
        }
    }
}

pub struct RgbaMapping {
    view: MEMORY_MAPPED_VIEW_ADDRESS,
    length: usize,
    _handle: NativeHandle,
}
unsafe impl Send for RgbaMapping {}
impl RgbaMapping {
    /// Takes ownership of a handle duplicated by the authenticated WebView host.
    pub unsafe fn open(handle: u64, length: usize) -> Result<Self, String> {
        let handle = NativeHandle(HANDLE(handle as usize as *mut _));
        Self::from_handle(handle, length)
    }
    pub fn from_handle(handle: NativeHandle, length: usize) -> Result<Self, String> {
        let view = unsafe { MapViewOfFile(handle.0, FILE_MAP_WRITE, 0, 0, length) };
        if view.Value.is_null() {
            return Err(windows::core::Error::from_thread().to_string());
        }
        Ok(Self {
            view,
            length,
            _handle: handle,
        })
    }
    /// Caller must own the slot until the WebView acknowledges its consumption.
    pub fn write(&mut self, rgba: &[u8]) -> Result<(), String> {
        if rgba.len() != self.length {
            return Err("RGBA mapping size mismatch".into());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(rgba.as_ptr(), self.view.Value.cast(), rgba.len());
        }
        Ok(())
    }
}
impl Drop for RgbaMapping {
    fn drop(&mut self) {
        unsafe {
            let _ = UnmapViewOfFile(self.view);
        }
    }
}
