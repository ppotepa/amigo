//! Local companion transport and platform presentation, without domain policy.
use amigo_playground_api::{PlaygroundFrameAck, PlaygroundFrameHeader};
use serde::{Deserialize, Serialize};

#[cfg(windows)]
mod pipe;
#[cfg(windows)]
pub use pipe::*;
#[cfg(windows)]
mod memory;
#[cfg(windows)]
pub use memory::*;
#[cfg(windows)]
mod presenter;
#[cfg(windows)]
pub use presenter::*;
#[cfg(windows)]
mod surface;
#[cfg(windows)]
pub use surface::*;

/// Kept in Rust; this credential must never be serialized into the WebView.
#[derive(Clone, Serialize, Deserialize)]
pub struct NativeBootstrap {
    pub pipe: String,
    pub secret: String,
    pub server_pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeGpuResources {
    pub generation: u64,
    pub size: [u32; 2],
    pub adapter: [u32; 2],
    pub textures: [u64; 3],
    pub ready_fence: u64,
    pub done_fence: u64,
}

/// OS handles travel exclusively over the authenticated Rust pipe.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NativeMessage {
    Hello {
        secret: String,
    },
    RgbaConfigure {
        generation: u64,
        size: [u32; 2],
    },
    RgbaBuffers {
        generation: u64,
        size: [u32; 2],
        handles: [u64; 3],
    },
    GpuConfigure {
        resources: NativeGpuResources,
    },
    GpuReady {
        generation: u64,
    },
    Frame {
        slot: usize,
        fence: u64,
        header: PlaygroundFrameHeader,
    },
    Ack {
        ack: PlaygroundFrameAck,
    },
    Error {
        generation: u64,
        message: String,
    },
    Close,
}
