use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaygroundViewportMode {
    #[default]
    NativeGpu,
    LocalRgba,
    Jpeg,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaygroundViewportAvailability {
    pub mode: PlaygroundViewportMode,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaygroundPresentationState {
    pub requested: PlaygroundViewportMode,
    pub active: Option<PlaygroundViewportMode>,
    pub switching: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaygroundPixelFormat {
    Rgba8Srgb,
}

/// All image paths use top-left origin, straight alpha and sRGB-encoded colour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaygroundFrameHeader {
    pub sequence: u64,
    pub revision: u64,
    pub generation: u64,
    pub size: [u32; 2],
    pub format: PlaygroundPixelFormat,
    pub mode: PlaygroundViewportMode,
    pub input_id: u64,
    pub stages: PlaygroundFrameStages,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaygroundFrameStages {
    pub extraction_ms: f64,
    pub submit_ms: f64,
    pub readback_ms: f64,
    pub encode_ms: f64,
    /// Number of older readbacks discarded before this frame was presented.
    pub stale_readback_drops: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaygroundFrameAck {
    pub sequence: u64,
    pub generation: u64,
    pub presented: bool,
    pub decode_ms: f64,
    pub present_ms: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlaygroundViewportRequest {
    pub mode: PlaygroundViewportMode,
    pub generation: u64,
    pub adaptive_resolution: bool,
    pub css_width: f64,
    pub css_height: f64,
    pub dpr: f64,
    pub interacting: bool,
    pub high_quality_capture: bool,
}

impl PlaygroundViewportRequest {
    pub fn effective_size(&self, limit: [u32; 2]) -> Result<[u32; 2], &'static str> {
        if [self.css_width, self.css_height, self.dpr]
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.0)
            || limit.contains(&0)
        {
            return Err("invalid viewport size");
        }
        let width = self.css_width * self.dpr;
        let height = self.css_height * self.dpr;
        if !width.is_finite() || !height.is_finite() {
            return Err("invalid viewport size");
        }
        let scale = (limit[0] as f64 / width)
            .min(limit[1] as f64 / height)
            .min(1.0);
        Ok([
            (width * scale).round().max(1.0) as u32,
            (height * scale).round().max(1.0) as u32,
        ])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaygroundViewportFrame {
    pub header: PlaygroundFrameHeader,
    pub payload: Vec<u8>,
}

impl PlaygroundViewportFrame {
    /// Versioned binary envelope: magic, JSON header length, header, image bytes.
    pub fn encode(&self) -> Vec<u8> {
        let header = serde_json::to_vec(&self.header).expect("finite frame metadata");
        let mut bytes = Vec::with_capacity(8 + header.len() + self.payload.len());
        bytes.extend_from_slice(b"APVF");
        bytes.extend_from_slice(&(header.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&header);
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

#[derive(Default)]
struct FrameQueueState {
    produced: bool,
    generation: u64,
    sequence: u64,
    occupied: BTreeMap<u64, (u64, bool)>,
    ready: VecDeque<PlaygroundViewportFrame>,
}

/// Two frame credits cover production, transport, decoding and consumption.
#[derive(Default)]
pub struct PlaygroundFrameQueue(Mutex<FrameQueueState>);

pub struct PlaygroundFramePermit {
    queue: Arc<PlaygroundFrameQueue>,
    pub sequence: u64,
    pub generation: u64,
    transferred: bool,
}
impl Drop for PlaygroundFramePermit {
    fn drop(&mut self) {
        if !self.transferred {
            self.queue.0.lock().unwrap().occupied.remove(&self.sequence);
        }
    }
}
impl PlaygroundFramePermit {
    /// Native transport has handed the slot to its consumer. GPU retirement is
    /// separately enforced by the backend even after this credit is returned.
    pub fn handoff(mut self) {
        let mut queue = self.queue.0.lock().unwrap();
        if let Some(entry) = queue.occupied.get_mut(&self.sequence) {
            entry.1 = true;
        }
        if queue.generation == self.generation {
            queue.produced = true;
        }
        self.transferred = true;
    }
    pub fn publish(mut self, frame: PlaygroundViewportFrame) {
        let mut queue = self.queue.0.lock().unwrap();
        if frame.header.sequence == self.sequence
            && frame.header.generation == self.generation
            && queue.generation == self.generation
        {
            queue.ready.push_back(frame);
            queue.produced = true;
            self.transferred = true;
        }
    }
}

impl PlaygroundFrameQueue {
    pub const CAPACITY: usize = 2;
    pub fn produced(&self) -> bool {
        self.0.lock().unwrap().produced
    }
    pub fn set_generation(&self, generation: u64) {
        let mut queue = self.0.lock().unwrap();
        if queue.generation != generation {
            queue.generation = generation;
            queue.produced = false;
            let obsolete = std::mem::take(&mut queue.ready);
            for frame in obsolete {
                queue.occupied.remove(&frame.header.sequence);
            }
        }
    }
    pub fn acquire(self: &Arc<Self>) -> Option<PlaygroundFramePermit> {
        let mut queue = self.0.lock().unwrap();
        if queue.occupied.len() >= Self::CAPACITY {
            return None;
        }
        queue.sequence += 1;
        let sequence = queue.sequence;
        let generation = queue.generation;
        queue.occupied.insert(sequence, (generation, false));
        Some(PlaygroundFramePermit {
            queue: self.clone(),
            sequence,
            generation,
            transferred: false,
        })
    }
    pub fn take_latest(&self) -> Option<PlaygroundViewportFrame> {
        let mut queue = self.0.lock().unwrap();
        let latest = queue.ready.pop_back()?;
        let obsolete = std::mem::take(&mut queue.ready);
        for frame in obsolete {
            queue.occupied.remove(&frame.header.sequence);
        }
        queue.occupied.get_mut(&latest.header.sequence).unwrap().1 = true;
        Some(latest)
    }
    pub fn acknowledge(&self, ack: &PlaygroundFrameAck) -> bool {
        let mut queue = self.0.lock().unwrap();
        if queue.occupied.get(&ack.sequence) != Some(&(ack.generation, true)) {
            return false;
        }
        queue.occupied.remove(&ack.sequence);
        true
    }
}

#[derive(Default)]
pub struct PlaygroundFramePolicy {
    last: Option<Instant>,
    interval: Duration,
}
impl PlaygroundFramePolicy {
    /// Call only when a readback slot is available. Committing a frame advances the clock.
    pub fn due(&mut self, now: Instant, dirty: bool, animated: bool, interacting: bool) -> bool {
        self.interval = Duration::from_secs_f64(1.0 / if interacting { 60.0 } else { 30.0 });
        (dirty || animated || interacting)
            && self
                .last
                .is_none_or(|last| now.saturating_duration_since(last) >= self.interval)
    }
    pub fn committed(&mut self, now: Instant) {
        // Keep the cadence when the host wakes a few milliseconds late. A slow
        // producer skips missed deadlines; it never accumulates catch-up jobs.
        self.last = Some(self.last.map_or(now, |last| {
            if self.interval.is_zero() {
                return now;
            }
            let elapsed = now.saturating_duration_since(last);
            now - Duration::from_nanos((elapsed.as_nanos() % self.interval.as_nanos()) as u64)
        }));
    }
}
